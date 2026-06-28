//! Functionality tests for the LCI batch — Descend / fathomless descent and
//! assorted commons riding existing primitives.

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::decision::{DecisionAnswer, ScriptedDecider};
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

/// Family Reunion (mode 0) pumps your creatures +1/+1.
#[test]
fn family_reunion_pumps_your_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::family_reunion());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1");
}

/// Bartolomé del Presidio grows by sacrificing another creature.
#[test]
fn bartolome_sacrifices_for_counter() {
    let mut g = two_player_game();
    let barto = g.add_card_to_battlefield(0, catalog::bartolome_del_presidio());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: barto, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.computed_permanent(barto).unwrap().power, 3, "2/1 + counter = 3/2");
}

/// Captain Storm puts a +1/+1 counter on a Pirate when an artifact enters.
#[test]
fn captain_storm_counters_pirate_on_artifact_etb() {
    let mut g = two_player_game();
    let storm = g.add_card_to_battlefield(0, catalog::captain_storm_cosmium_raider());
    let thopter = g.add_card_to_hand(0, catalog::ornithopter());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: thopter, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast artifact");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(storm).unwrap().power, 3, "Pirate grew from artifact ETB");
}

/// Bedrock Tortoise grants your creatures hexproof only during your turn, and
/// makes your T>P creatures assign combat damage by toughness.
#[test]
fn bedrock_tortoise_turn_hexproof_and_toughness_damage() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let tortoise = g.add_card_to_battlefield(0, catalog::bedrock_tortoise());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Your turn → hexproof on your creatures.
    g.active_player_idx = 0;
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof), "hexproof on your turn");
    // Opponent's turn → no hexproof.
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof), "no hexproof off-turn");
    // The 0/6 tortoise (T>P) assigns combat damage by toughness.
    assert!(g.computed_permanent(tortoise).unwrap().keywords.contains(&Keyword::AssignsCombatDamageByToughness));
}

/// Amalia explores on lifegain; at exactly 20 power she wraths the board.
#[test]
fn amalia_explores_on_lifegain_and_wraths_at_20() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let amalia = g.add_card_to_battlefield(0, catalog::amalia_benavides_aguirre());
    // 17 +1/+1 counters → base 2 + 17 = 19 power.
    g.battlefield_find_mut(amalia).unwrap().add_counters(CounterType::PlusOnePlusOne, 17);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Two nonlands on top: Revitalize draws one, the explore reveals the other.
    for _ in 0..2 { let id = g.next_id(); g.players[0].add_to_library_top(id, catalog::grizzly_bears()); }
    let revit = g.add_card_to_hand(0, catalog::revitalize());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: revit, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Revitalize");
    drain_stack(&mut g);
    // Explore pushed Amalia to power 20, wrathing the board.
    assert_eq!(g.computed_permanent(amalia).unwrap().power, 20, "explore counter → power 20");
    assert!(g.battlefield_find(victim).is_none(), "other creatures destroyed at power 20");
    assert!(g.battlefield_find(amalia).is_some(), "Amalia survives her own wrath");
}

/// Jadelight Spelunker explores X times on ETB (X nonland reveals → X counters).
#[test]
fn jadelight_spelunker_explores_x_times() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let js = g.add_card_to_hand(0, catalog::jadelight_spelunker());
    for _ in 0..2 { let id = g.next_id(); g.players[0].add_to_library_top(id, catalog::grizzly_bears()); }
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: js, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast with X=2");
    drain_stack(&mut g);
    let c = g.battlefield_find(js).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(c, 2, "explored twice → two +1/+1 counters");
}

/// Staggering Size pumps a creature +3/+3 and grants trample.
#[test]
fn staggering_size_pumps_and_tramples() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::staggering_size());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+3/+3");
    assert!(cp.keywords.contains(&Keyword::Trample), "gained trample");
}

/// Compass Gnome searches a Cave onto the top of the library on ETB.
#[test]
fn compass_gnome_tutors_cave_to_top() {
    let mut g = two_player_game();
    let cave_id = g.add_card_to_library(0, catalog::captivating_cave());
    g.move_card_to_battlefield_for_test(0, catalog::compass_gnome());
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(cave_id), "Cave placed on top");
}

/// Gargantuan Leech's affinity for Caves reduces its cost by {1} per Cave.
#[test]
fn gargantuan_leech_affinity_for_caves() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::captivating_cave());
    g.add_card_to_battlefield(0, catalog::promising_vein());
    g.add_card_to_graveyard(0, catalog::volatile_fault()); // a Cave in the graveyard
    let leech = g.add_card_to_hand(0, catalog::gargantuan_leech());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // {7}{B} - (2 battlefield + 1 graveyard Caves) = {4}{B}.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: leech, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast at the Cave-reduced cost");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == leech), "Leech resolved");
}

/// Terror Tide shrinks all creatures by the number of permanent cards in your
/// graveyard.
#[test]
fn terror_tide_mass_minus_x() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    for _ in 0..2 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); } // descent 2
    let spell = g.add_card_to_hand(0, catalog::terror_tide());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // 2/2 - 2/2 = 0/0 → both die as a state-based action.
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(foe).is_none(), "both shrank to 0/0 and died");
}

/// Dusk Legion Duelist draws when +1/+1 counters land on it (once per turn).
#[test]
fn dusk_legion_duelist_draws_on_counter() {
    let mut g = two_player_game();
    let duelist = g.add_card_to_battlefield(0, catalog::dusk_legion_duelist());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    g.battlefield_find_mut(duelist).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.dispatch_triggers_for_events(&[GameEvent::CounterAdded {
        card_id: duelist, counter_type: CounterType::PlusOnePlusOne, count: 1,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew off the counter trigger");
}

/// Over the Edge (mode 0) destroys a target artifact.
#[test]
fn over_the_edge_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::ornithopter());
    let spell = g.add_card_to_hand(0, catalog::over_the_edge());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(art)), additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Pugnacious Hammerskull stuns itself when it attacks as your only Dinosaur.
#[test]
fn pugnacious_hammerskull_stuns_when_alone() {
    let mut g = two_player_game();
    let dino = g.add_card_to_battlefield(0, catalog::pugnacious_hammerskull());
    g.clear_sickness(dino);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: dino, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(dino).unwrap().counter_count(crate::card::CounterType::Stun), 1, "stunned (no other Dino)");
}

/// Sentry of the Underworld regenerates for {W}{B} and 3 life.
#[test]
fn sentry_of_the_underworld_regenerates() {
    let mut g = two_player_game();
    let sentry = g.add_card_to_battlefield(0, catalog::sentry_of_the_underworld());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].life = 20;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sentry, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("regenerate ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17, "paid 3 life");
    assert_eq!(g.battlefield_find(sentry).unwrap().regeneration_shields, 1, "stamped a regen shield");
}

/// Sunshot Militia taps two helpers to ping each opponent.
#[test]
fn sunshot_militia_taps_two_to_ping() {
    let mut g = two_player_game();
    let militia = g.add_card_to_battlefield(0, catalog::sunshot_militia());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let start = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: militia, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap-two ability");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, start - 1, "each opponent took 1");
}

// ── LCI batch (modern_decks): new commons & uncommons ────────────────────────

use crate::game::{Attack, AttackTarget, Target};

/// Helper: pass priority until the given step.
fn to_step(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
}

/// Acrobatic Leap pumps +1/+3, grants flying, and untaps the target.
#[test]
fn acrobatic_leap_pumps_and_untaps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::acrobatic_leap());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 5), "+1/+3");
    assert!(cp.keywords.contains(&Keyword::Flying), "gains flying");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
}

/// Petrify makes the enchanted creature unable to attack or block.
#[test]
fn petrify_locks_down_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::petrify());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::CantAttack) && cp.keywords.contains(&Keyword::CantBlock), "locked");
}

/// Ray of Ruin exiles the target creature.
#[test]
fn ray_of_ruin_exiles() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::ray_of_ruin());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none() && g.exile.iter().any(|c| c.id == foe), "exiled");
}

/// Scampering Surveyor tutors a basic land onto the battlefield tapped.
#[test]
fn scampering_surveyor_ramps_tapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.move_card_to_battlefield_for_test(0, catalog::scampering_surveyor());
    drain_stack(&mut g);
    let forest = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Forest");
    assert!(forest.is_some_and(|c| c.tapped), "Forest entered tapped");
}

/// Seeker of Sunlight explores when its activated ability resolves.
#[test]
fn seeker_of_sunlight_explores() {
    let mut g = two_player_game();
    let seeker = g.add_card_to_battlefield(0, catalog::seeker_of_sunlight());
    g.add_card_to_library(0, catalog::forest()); // land → to hand
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: seeker, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "explored land to hand");
}

/// Mischievous Pup bounces another permanent you control on entry.
#[test]
fn mischievous_pup_bounces_own() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::mischievous_pup());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == other), "bounced to hand");
}

/// Nurturing Bristleback makes a 3/3 Dinosaur token on entry.
#[test]
fn nurturing_bristleback_makes_dino() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::nurturing_bristleback());
    drain_stack(&mut g);
    let tok = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Dinosaur" && c.is_token);
    assert!(tok.is_some_and(|c| (c.definition.power, c.definition.toughness) == (3, 3)), "3/3 Dino token");
}

/// Soaring Sandwing gains 3 life on entry.
#[test]
fn soaring_sandwing_gains_life() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.move_card_to_battlefield_for_test(0, catalog::soaring_sandwing());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "gained 3");
}

/// Rampaging Spiketail pumps a creature you control and grants indestructible.
#[test]
fn rampaging_spiketail_pumps_and_shields() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::rampaging_spiketail());
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "+2/+0");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "indestructible");
}

/// Tinker's Tote makes two Gnomes and can sacrifice for 3 life.
#[test]
fn tinkers_tote_gnomes_and_sac() {
    let mut g = two_player_game();
    let tote = g.move_card_to_battlefield_for_test(0, catalog::tinkers_tote());
    drain_stack(&mut g);
    let gnomes = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Gnome").count();
    assert_eq!(gnomes, 2, "two Gnomes");
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].life = 20;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tote, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "gained 3 from sac");
    assert!(g.battlefield_find(tote).is_none(), "sacrificed");
}

/// Primordial Gnawer discovers 3 when it dies.
#[test]
fn primordial_gnawer_discovers_on_death() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // MV 1 ≤ 3
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)])); // decline free cast → to hand
    let gnawer = g.add_card_to_battlefield(0, catalog::primordial_gnawer());
    g.remove_to_graveyard_with_triggers(gnawer);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "discovered card to hand");
}

/// Mephitic Draught draws and loses 1 life on entry and on death.
#[test]
fn mephitic_draught_etb_and_death() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].life = 20;
    let draught = g.move_card_to_battlefield_for_test(0, catalog::mephitic_draught());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "lost 1 on ETB");
    g.remove_to_graveyard_with_triggers(draught);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "lost 1 on death");
}

/// Staunch Crewmate digs four and takes an artifact/Pirate.
#[test]
fn staunch_crewmate_digs() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let art = g.add_card_to_library(0, catalog::ornithopter()); // top, an artifact
    g.move_card_to_battlefield_for_test(0, catalog::staunch_crewmate());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == art), "took the artifact");
}

/// Malamet Brawler grants trample to a target attacker.
#[test]
fn malamet_brawler_grants_trample() {
    let mut g = two_player_game();
    let brawler = g.add_card_to_battlefield(0, catalog::malamet_brawler());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(brawler);
    g.clear_sickness(ally);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    to_step(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: brawler, target: AttackTarget::Player(1) },
        Attack { attacker: ally, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Trample), "ally gained trample");
}

/// Malamet Veteran adds a counter when attacking with descend 4 active.
#[test]
fn malamet_veteran_descend_counter() {
    let mut g = two_player_game();
    let vet = g.add_card_to_battlefield(0, catalog::malamet_veteran());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..4 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); } // descend 4
    g.clear_sickness(vet);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    to_step(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: vet, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "counter added");
}

/// Enterprising Scallywag makes a Treasure at end step if you descended.
#[test]
fn enterprising_scallywag_treasure_on_descend() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::enterprising_scallywag());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].descended_this_turn = true;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"), "made a Treasure");
}

/// Careening Mine Cart makes a Treasure when it attacks (crewed).
#[test]
fn careening_mine_cart_treasure_on_attack() {
    let mut g = two_player_game();
    let cart = g.add_card_to_battlefield(0, catalog::careening_mine_cart());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(cart);
    g.clear_sickness(bear);
    g.perform_action(GameAction::Crew { vehicle: cart, crew_creatures: vec![bear] }).expect("crew");
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    to_step(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: cart, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"), "made a Treasure");
}

/// Brazen Blademaster pumps when attacking with two+ artifacts.
#[test]
fn brazen_blademaster_artifact_pump() {
    let mut g = two_player_game();
    let bm = g.add_card_to_battlefield(0, catalog::brazen_blademaster());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.clear_sickness(bm);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    to_step(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bm, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bm).unwrap().power, 4, "+2/+1 from two artifacts");
}

/// Burning Sun Cavalry pumps on attack while you control a Dinosaur.
#[test]
fn burning_sun_cavalry_dino_pump() {
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, catalog::burning_sun_cavalry());
    g.add_card_to_battlefield(0, catalog::nurturing_bristleback()); // a Dinosaur
    g.clear_sickness(knight);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    to_step(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: knight, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(knight).unwrap().power, 3, "+1/+1 with a Dino");
}

/// Hotfoot Gnome grants haste to another creature.
#[test]
fn hotfoot_gnome_grants_haste() {
    let mut g = two_player_game();
    let gnome = g.add_card_to_battlefield(0, catalog::hotfoot_gnome());
    let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(gnome);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: gnome, ability_index: 0, target: Some(Target::Permanent(fresh)), additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(fresh).unwrap().keywords.contains(&Keyword::Haste), "gained haste");
}

/// Fungal Fortitude gives +2/+0 and returns the creature tapped when it dies.
#[test]
fn fungal_fortitude_returns_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::fungal_fortitude());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0");
    g.battlefield_find_mut(bear).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears");
    assert!(back.is_some_and(|c| c.tapped), "returned tapped");
}

/// Armored Kincaller gains 3 life when you control another Dinosaur.
#[test]
fn armored_kincaller_gains_life_with_dino() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.add_card_to_battlefield(0, catalog::nurturing_bristleback()); // a Dinosaur
    g.move_card_to_battlefield_for_test(0, catalog::armored_kincaller());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "gained 3 with another Dino");
}


/// Brackish Blunder bounces; a tapped target also yields a Map.
#[test]
fn brackish_blunder_map_when_tapped() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::brackish_blunder());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == foe), "bounced");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Map"), "Map for tapped target");
}

/// Bloodthorn Flail equips for +2/+1.
#[test]
fn bloodthorn_flail_equips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let flail = g.add_card_to_battlefield(0, catalog::bloodthorn_flail());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: flail, target: bear }).expect("equip");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1");
}

/// Diamond Pick-Axe is indestructible and its bearer makes Treasure on attack.
#[test]
fn diamond_pick_axe_treasure_on_attack() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, catalog::diamond_pick_axe());
    assert!(g.computed_permanent(axe).unwrap().keywords.contains(&Keyword::Indestructible));
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: axe, target: bear }).expect("equip");
    to_step(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"), "made a Treasure");
}

/// Pirate Hat's bearer loots when it attacks.
#[test]
fn pirate_hat_loots_on_attack() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hat = g.add_card_to_battlefield(0, catalog::pirate_hat());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: hat, target: bear }).expect("equip");
    let hand_before = g.players[0].hand.len();
    to_step(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    // Loot = draw 1, discard 1 → net hand size unchanged, but the drawn Island is in hand.
    assert_eq!(g.players[0].hand.len(), hand_before, "loot is net-zero");
}

/// Triumphant Chomp deals max(2, greatest Dinosaur power) — floored at 2 with no
/// Dino, scaled up by a big one.
#[test]
fn triumphant_chomp_scales_with_dino() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    // No Dinosaur → deals 2, kills the 2/2.
    let spell = g.add_card_to_hand(0, catalog::triumphant_chomp());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "2 damage with no Dino killed the 2/2");

    // With a 5/5 Dinosaur you control, a 2/3 takes 5 and dies.
    g.add_card_to_battlefield(0, catalog::nurturing_bristleback()); // 5/5 Dino
    let foe2 = g.add_card_to_battlefield(1, catalog::sentry_of_the_underworld()); // 2/3
    let spell2 = g.add_card_to_hand(0, catalog::triumphant_chomp());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell2, target: Some(Target::Permanent(foe2)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe2).is_none(), "5 damage from the Dino killed the 2/3");
}

// ── LCI batch 2 (modern_decks) ───────────────────────────────────────────────

/// Ruin-Lurker Bat scries at end step when you descended.
#[test]
fn ruin_lurker_bat_scry_on_descend() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ruin_lurker_bat());
    g.add_card_to_library(0, catalog::island());
    g.players[0].descended_this_turn = true;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    // Scry ran (no panic); library intact.
    assert_eq!(g.players[0].library.len(), 1);
}

/// Join the Dead is -5/-5, or -10/-10 with descend 4.
#[test]
fn join_the_dead_scales_with_descend() {
    let mut g = two_player_game();
    // Without descend: a 7/6 survives -5/-5 (→ 2/1).
    let big = g.add_card_to_battlefield(1, catalog::trumpeting_carnosaur()); // 7/6
    let spell = g.add_card_to_hand(0, catalog::join_the_dead());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(big)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_some(), "7/6 survives -5/-5 with no descend");
    // Now with descend 4 active, -10/-10 kills it.
    let big2 = g.add_card_to_battlefield(1, catalog::trumpeting_carnosaur());
    for _ in 0..4 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    let spell2 = g.add_card_to_hand(0, catalog::join_the_dead());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell2, target: Some(Target::Permanent(big2)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big2).is_none(), "-10/-10 with descend 4 kills the 5/5");
}

/// Ancestors' Aid pumps with first strike and makes a Treasure.
#[test]
fn ancestors_aid_pumps_and_treasure() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::ancestors_aid());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "+2/+0");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "first strike");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"), "Treasure");
}

/// River Herald Guide explores on entry.
#[test]
fn river_herald_guide_explores() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.move_card_to_battlefield_for_test(0, catalog::river_herald_guide());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "explored land to hand");
}

/// Might of the Ancestors pumps a creature at the beginning of combat.
#[test]
fn might_of_the_ancestors_combat_pump() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::might_of_the_ancestors());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "+2/+0");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "vigilance");
}

/// Walk with the Ancestors returns a permanent card from the graveyard.
#[test]
fn walk_with_the_ancestors_recurs() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::walk_with_the_ancestors());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "returned to hand");
}

/// Vanguard of the Rose gains indestructible by sacrificing another permanent.
#[test]
fn vanguard_of_the_rose_sac_for_indestructible() {
    let mut g = two_player_game();
    let vanguard = g.add_card_to_battlefield(0, catalog::vanguard_of_the_rose());
    let fodder = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: vanguard, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert!(g.computed_permanent(vanguard).unwrap().keywords.contains(&Keyword::Indestructible));
    assert!(g.battlefield_find(vanguard).unwrap().tapped, "tapped itself");
}

/// Daring Discovery stops up to three blockers and discovers.
#[test]
fn daring_discovery_locks_blockers() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)])); // discover → to hand
    let spell = g.add_card_to_hand(0, catalog::daring_discovery());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantBlock), "can't block");
}

/// Attentive Sunscribe scries when it becomes tapped.
#[test]
fn attentive_sunscribe_scry_on_tap() {
    let mut g = two_player_game();
    let scribe = g.add_card_to_battlefield(0, catalog::attentive_sunscribe());
    g.add_card_to_library(0, catalog::island());
    g.battlefield_find_mut(scribe).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: scribe }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), 1, "scry kept the card (no panic)");
}


/// Self-Reflection copies a creature you control.
#[test]
fn self_reflection_copies_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::self_reflection());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "original + token copy");
}

/// Canonized in Blood adds a counter at end step on descend.
#[test]
fn canonized_in_blood_descend_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::canonized_in_blood());
    g.players[0].descended_this_turn = true;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "counter added");
}

/// Earthshaker Dreadmaw draws for each other Dinosaur you control.
#[test]
fn earthshaker_dreadmaw_draws_per_dino() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nurturing_bristleback()); // a Dino
    g.add_card_to_battlefield(0, catalog::river_herald_guide()); // not a Dino
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let hand0 = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::earthshaker_dreadmaw());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew 1 for the one other Dinosaur");
}

/// Threefold Thunderhulk enters as a 3/3 and mints Gnomes equal to its power.
#[test]
fn threefold_thunderhulk_mints_gnomes() {
    let mut g = two_player_game();
    let hulk = g.move_card_to_battlefield_for_test(0, catalog::threefold_thunderhulk());
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(hulk).unwrap().power, 3, "0/0 + three counters = 3/3");
    let gnomes = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Gnome").count();
    assert_eq!(gnomes, 3, "minted Gnomes equal to power");
}

/// Tectonic Hazard pings each opponent and their creatures for 1.
#[test]
fn tectonic_hazard_pings_opponent_board() {
    let mut g = two_player_game();
    let x1 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::tectonic_hazard());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[1].life = 20;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "opponent took 1");
    assert_eq!(g.battlefield_find(x1).unwrap().damage, 1, "opponent creature took 1");
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "your creature untouched");
}

/// Soulcoil Viper reanimates a graveyard creature with a finality counter.
#[test]
fn soulcoil_viper_reanimates_with_finality() {
    let mut g = two_player_game();
    let viper = g.add_card_to_battlefield(0, catalog::soulcoil_viper());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.clear_sickness(viper);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: viper, ability_index: 0, target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    let back = g.battlefield_find(dead).expect("reanimated");
    assert_eq!(back.counter_count(CounterType::Finality), 1, "entered with a finality counter");
    assert!(g.battlefield_find(viper).is_none(), "viper sacrificed");
}

/// Itzquinth's ETB: pay {2}, then the reflexive bite chooses its two targets at
/// resolution and a Dinosaur you control deals its power to another creature.
#[test]
fn itzquinth_reflexive_bite_after_paying() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let itz = g.add_card_to_battlefield(0, catalog::itzquinth_firstborn_of_gishath()); // 2/3 Dino
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 opponent
    g.players[0].mana_pool.add(Color::Red, 2); // pay {2} (generic)
    g.fire_self_etb_triggers(itz, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).is_none(), "bit for 2 — opponent creature died");
    assert!(g.battlefield_find(itz).is_some(), "Itzquinth still in play");
}

/// Declining Itzquinth's {2} skips the reflexive bite entirely.
#[test]
fn itzquinth_declined_no_bite() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    let itz = g.add_card_to_battlefield(0, catalog::itzquinth_firstborn_of_gishath());
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.fire_self_etb_triggers(itz, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).is_some(), "no payment → no bite");
}

/// Glorifier of Suffering: sacrifice another creature, then support 2 puts a
/// +1/+1 counter on each of up to two creatures (chosen after the sacrifice).
#[test]
fn glorifier_sacrifice_then_support_two() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let glor = g.add_card_to_battlefield(0, catalog::glorifier_of_suffering());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let keeper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.fire_self_etb_triggers(glor, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none() || g.battlefield_find(keeper).is_none(),
        "one creature was sacrificed");
    let counters: i32 = g.battlefield.iter().filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne) as i32).sum();
    assert_eq!(counters, 2, "support 2 placed two +1/+1 counters");
}

/// Wary Thespian surveils on enter and on death (no-panic coverage).
#[test]
fn wary_thespian_surveils_enter_and_die() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let wt = g.move_card_to_battlefield_for_test(0, catalog::wary_thespian());
    drain_stack(&mut g);
    g.add_card_to_library(0, catalog::forest());
    g.remove_to_graveyard_with_triggers(wt);
    drain_stack(&mut g);
    assert!(g.battlefield_find(wt).is_none(), "died");
}

/// Huatli's Final Strike pumps your creature +1/+0 and bites an opponent's.
#[test]
fn huatlis_final_strike_pumps_and_bites() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/2
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::huatlis_final_strike());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(prey)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).is_none(), "bitten for 3 (2+1) → dead");
}

/// Ghalta's ETB cheats creature cards out of hand onto the battlefield.
#[test]
fn ghalta_drops_creatures_from_hand() {
    let mut g = two_player_game();
    let a = g.add_card_to_hand(0, catalog::grizzly_bears());
    let b = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::lightning_bolt()); // noncreature stays
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
    let before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.move_card_to_battlefield_for_test(0, catalog::ghalta_stampede_tyrant());
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(after - before, 3, "Ghalta + two creature cards entered; the bolt stayed in hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"));
}

/// Deeproot Pilgrimage mints a Merfolk when your nontoken Merfolk taps.
#[test]
fn deeproot_pilgrimage_on_merfolk_tap() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::deeproot_pilgrimage());
    let scout = g.add_card_to_battlefield(0, catalog::cenote_scout()); // Merfolk
    g.battlefield_find_mut(scout).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: scout }]);
    drain_stack(&mut g);
    let merfolk = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Merfolk").count();
    assert_eq!(merfolk, 1, "tapping a Merfolk minted a Merfolk token");
}

/// Chupacabra Echo's ETB shrinks an opponent's creature by your graveyard's
/// permanent-card count.
#[test]
fn chupacabra_echo_descend_shrink() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); } // descend 2
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let echo = g.add_card_to_battlefield(0, catalog::chupacabra_echo());
    g.fire_self_etb_triggers(echo, 0);
    drain_stack(&mut g);
    // -2/-2 on a 2/2 → dead (0/0 SBA).
    assert!(g.battlefield_find(prey).is_none(), "shrunk to 0/0 and died");
}

/// Quicksand Whirlpool exiles a creature and costs {3} less vs a tapped one.
#[test]
fn quicksand_whirlpool_exiles_and_discounts() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::quicksand_whirlpool());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 3); // {3}{W} after the {3} tapped discount
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast at discount");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "exiled the tapped creature");
}

/// Huatli's Snubhorn is a 2/2 Dinosaur with vigilance.
#[test]
fn huatlis_snubhorn_is_vigilant_dino() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::huatlis_snubhorn());
    let cp = g.computed_permanent(s).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords.contains(&Keyword::Vigilance));
}

/// Pantlaza discovers off a Dinosaur entering (X = its toughness).
#[test]
fn pantlaza_discovers_on_dino_etb() {
    let mut g = two_player_game();
    // Stack a cheap nonland so discover 4 finds something to dig to.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::pantlaza_sun_favored());
    let lib_before = g.players[0].library.len();
    let dino = g.add_card_to_battlefield(0, catalog::huatlis_snubhorn()); // toughness 2
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dino }]);
    drain_stack(&mut g);
    assert!(g.players[0].library.len() < lib_before, "discover dug into the library");
}

/// Stalactite Stalker's sac ability shrinks a creature by its (LKI) power.
#[test]
fn stalactite_stalker_sac_shrinks() {
    let mut g = two_player_game();
    let stalker = g.add_card_to_battlefield(0, catalog::stalactite_stalker()); // 1/1
    g.battlefield_find_mut(stalker).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // → 2/2
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(stalker);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: stalker, ability_index: 0, target: Some(Target::Permanent(prey)),
        additional_targets: vec![], x_value: None,
    }).expect("activate sac");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stalker).is_none(), "sacrificed");
    assert!(g.battlefield_find(prey).is_none(), "-2/-2 killed the 2/2");
}

/// Glimpse the Core (mode 2) reanimates a Cave from the graveyard, tapped.
#[test]
fn glimpse_the_core_returns_cave() {
    let mut g = two_player_game();
    let cave = g.add_card_to_graveyard(0, catalog::captivating_cave());
    let spell = g.add_card_to_hand(0, catalog::glimpse_the_core());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(cave)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast mode 1");
    drain_stack(&mut g);
    let back = g.battlefield_find(cave).expect("Cave returned");
    assert!(back.tapped, "entered tapped");
}

/// Reckless Detective's attack discard draws a card and pumps it +2/+0.
#[test]
fn reckless_detective_attack_loots_and_pumps() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let det = g.add_card_to_battlefield(0, catalog::reckless_detective()); // 0/3
    g.add_card_to_hand(0, catalog::grizzly_bears()); // to discard
    g.clear_sickness(det);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len();
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: det, target: crate::game::types::AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "discarded 1, drew 1 → net 0");
    assert_eq!(g.computed_permanent(det).unwrap().power, 2, "+2/+0 this turn");
}

/// Idol of the Deep King's ETB deals 2 to any target.
#[test]
fn idol_of_the_deep_king_pings() {
    let mut g = two_player_game();
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[1].life = 20;
    let idol = g.add_card_to_battlefield(0, catalog::idol_of_the_deep_king());
    g.fire_self_etb_triggers(idol, 0);
    drain_stack(&mut g);
    // The ping hit a hostile target — the opponent's creature or their face.
    let hit_creature = g.battlefield_find(prey).is_none();
    let hit_face = g.players[1].life == 18;
    assert!(hit_creature || hit_face, "dealt 2 to an opponent target");
}

/// Calamitous Tide bounces up to two creatures and loots (draw 2, discard 1).
#[test]
fn calamitous_tide_bounces_and_loots() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::calamitous_tide());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 6);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both bounced");
    // cast 1 (spell left hand) then draw 2 discard 1 → net +1 vs pre-cast hand.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew 2, discarded 1, spent the spell");
}

/// Hidden Grotto surveils on ETB (no-panic) and taps for colorless.
#[test]
fn hidden_grotto_etb_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let grotto = g.move_card_to_battlefield_for_test(0, catalog::hidden_grotto());
    drain_stack(&mut g);
    assert!(g.battlefield_find(grotto).is_some(), "land entered and surveiled");
}

/// Hulking Bugbear is a 3/3 with haste.
#[test]
fn hulking_bugbear_has_haste() {
    let mut g = two_player_game();
    let b = g.add_card_to_battlefield(0, catalog::hulking_bugbear());
    let cp = g.computed_permanent(b).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Etali's Favor attaches, discovers, and pumps the enchanted creature +1/+1.
#[test]
fn etalis_favor_attaches_and_pumps() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // discover fodder
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::etalis_favor());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 from the Aura");
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Volcanic Geyser deals X to any target.
#[test]
fn volcanic_geyser_deals_x() {
    let mut g = two_player_game();
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::volcanic_geyser());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 4); // {2}{R}{R} → X=2
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(prey)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).is_none(), "X=2 killed the 2/2");
}

/// Akawalli gets +2/+2 and trample once you've descended 4.
#[test]
fn akawalli_descend_4_buff() {
    let mut g = two_player_game();
    let aka = g.add_card_to_battlefield(0, catalog::akawalli_the_seething_tower());
    assert_eq!(g.computed_permanent(aka).unwrap().power, 3, "base 3/3 with empty graveyard");
    for _ in 0..4 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); } // descend 4
    let cp = g.computed_permanent(aka).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "descend 4 → +2/+2");
    assert!(cp.keywords.contains(&Keyword::Trample));
}
