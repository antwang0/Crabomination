//! Functionality tests for `catalog::sets::decks::spm` — Marvel's Spider-Man.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// Aunt May: another creature entering gains 1 life; a Spider also gets a
/// +1/+1 counter.
#[test]
fn aunt_may_gains_life_and_buffs_spiders() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aunt_may());
    let spider = g.add_card_to_hand(0, catalog::radioactive_spider());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].life;
    g.cast_spell(spider, None, vec![], None, None).expect("cast Spider");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 1, "gained 1 life on the enter");
    let cp = g.computed_permanent(spider).unwrap();
    assert_eq!(cp.toughness, 2, "Spider got a +1/+1 counter (1/1 -> 2/2)");
}

/// City Pigeon mints a Food token when it leaves the battlefield.
#[test]
fn city_pigeon_makes_food_on_leave() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::city_pigeon());
    g.active_player_idx = 0;
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Food"),
        "Food token created on leave",
    );
}

/// Gallant Citizen draws a card on enter.
#[test]
fn gallant_citizen_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::gallant_citizen());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card on enter");
}

/// Common Crook makes a Treasure when it dies.
#[test]
fn common_crook_dies_to_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::common_crook());
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure"),
        "Treasure token on death",
    );
}

/// Kraven's Cats pumps once, and the ability is once-per-turn.
#[test]
fn kravens_cats_once_per_turn_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kravens_cats());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).unwrap().power, 4, "2/2 -> 4/4");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let second = g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    });
    assert!(second.is_err(), "second activation blocked (once each turn)");
}

/// Lurking Lizards grows when you cast a mana value 4+ spell.
#[test]
fn lurking_lizards_grows_on_big_spell() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lurking_lizards());
    let spell = g.add_card_to_hand(0, catalog::explosive_vegetation());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let _ = g.cast_spell(spell, None, vec![], None, None);
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.toughness >= 4, "grew a +1/+1 counter from the big cast");
}

/// Angry Rabble pings each opponent when you cast a mana value 4+ spell.
#[test]
fn angry_rabble_pings_on_big_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::angry_rabble());
    let spell = g.add_card_to_hand(0, catalog::explosive_vegetation());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[1].life;
    let _ = g.cast_spell(spell, None, vec![], None, None);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "1 damage to the opponent");
}

/// Merciless Enforcers' activated ability pings each opponent.
#[test]
fn merciless_enforcers_pings() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::merciless_enforcers());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    let before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "1 damage to opponent");
}

/// Scorpion's Sting shrinks a creature by 3/3.
#[test]
fn scorpions_sting_shrinks() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cast = g.add_card_to_hand(0, catalog::scorpions_sting());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.cast_spell(cast, Some(Target::Permanent(bear)), vec![], None, None).expect("cast");
    drain_stack(&mut g);
    // 2/2 -3/-3 -> 0-toughness, dies as an SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear died to -3/-3");
}

/// Thwip! pumps and grants flying; a Spider target also gains 2 life.
#[test]
fn thwip_buffs_spider_and_gains_life() {
    let mut g = two_player_game();
    let spider = g.add_card_to_battlefield(0, catalog::radioactive_spider());
    let cast = g.add_card_to_hand(0, catalog::thwip());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].life;
    g.cast_spell(cast, Some(Target::Permanent(spider)), vec![], None, None).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(spider).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "gained flying");
    assert_eq!(cp.power, 3, "1/1 +2/+2 -> 3/3");
    assert_eq!(g.players[0].life, before + 2, "gained 2 life (Spider)");
}

/// Morlun enters with X counters and deals X to the opponent.
#[test]
fn morlun_x_counters_and_damage() {
    let mut g = two_player_game();
    let cast = g.add_card_to_hand(0, catalog::morlun_devourer_of_spiders());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[1].life;
    g.cast_spell(cast, None, vec![], None, Some(2)).expect("cast X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 2, "X=2 damage to opponent");
    let morlun = g.battlefield.iter().find(|c| c.definition.name.starts_with("Morlun")).unwrap();
    let cp = g.computed_permanent(morlun.id).unwrap();
    assert_eq!(cp.power, 4, "2/1 base + 2 counters -> 4 power");
}

/// Selfless Police Captain moves a counter to a target on leave.
#[test]
fn selfless_police_captain_hands_off_counter() {
    let mut g = two_player_game();
    let cap = g.move_card_to_battlefield_for_test(0, catalog::selfless_police_captain());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let _ = g.remove_to_graveyard_with_triggers(cap);
    // The LTB trigger auto-targets the bear (only creature you control).
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.toughness, 3, "bear gained a +1/+1 counter (2/2 -> 3/3)");
}

/// Spider-Bot searches a basic land onto the top of the library.
#[test]
fn spider_bot_tutors_basic_to_top() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let land = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(land))]));
    g.move_card_to_battlefield_for_test(0, catalog::spider_bot());
    drain_stack(&mut g);
    let top = g.players[0].library.first().map(|c| c.id);
    assert_eq!(top, Some(land), "Forest tutored to the top");
}

/// Radioactive Spider tutors a Spider Hero to hand when sacrificed.
#[test]
fn radioactive_spider_tutors_spider_hero() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::radioactive_spider());
    let hero = g.add_card_to_library(0, catalog::spider_girl_legacy_hero());
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(hero))]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("sac to tutor");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == hero), "Spider Hero to hand");
}

/// Spider-Suit grants +2/+2 and the Spider Hero types while equipped.
#[test]
fn spider_suit_equips_and_types() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let suit = g.add_card_to_battlefield(0, catalog::spider_suit());
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: suit, target: bear })
    .expect("equip");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "2/2 +2/+2 -> 4/4");
    assert!(
        cp.subtypes.creature_types.contains(&crate::card::CreatureType::Spider),
        "gained Spider type",
    );
}

/// Mary Jane Watson draws when a Spider enters, once each turn.
#[test]
fn mary_jane_draws_once_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mary_jane_watson());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let s1 = g.add_card_to_hand(0, catalog::radioactive_spider());
    let s2 = g.add_card_to_hand(0, catalog::radioactive_spider());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let lib_before = g.players[0].library.len();
    g.cast_spell(s1, None, vec![], None, None).expect("cast Spider 1");
    drain_stack(&mut g);
    g.cast_spell(s2, None, vec![], None, None).expect("cast Spider 2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1, "exactly one draw this turn");
}

/// Spider-Girl has flying on your turn only.
#[test]
fn spider_girl_flies_on_your_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spider_girl_legacy_hero());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying), "flies on your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying), "grounded off-turn");
}

/// Vibrant Cityscape fetches a basic onto the battlefield tapped.
#[test]
fn vibrant_cityscape_ramps_tapped() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::vibrant_cityscape());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("fetch");
    drain_stack(&mut g);
    let f = g.battlefield.iter().find(|c| c.id == forest).expect("Forest on battlefield");
    assert!(f.tapped, "entered tapped");
}

/// Cosmogoyf's power scales with cards you own in exile (real EOE card).
#[test]
fn cosmogoyf_scales_with_exiled_cards() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cosmogoyf());
    assert_eq!(g.computed_permanent(id).unwrap().power, 0, "0 exiled -> 0/1");
    assert_eq!(g.computed_permanent(id).unwrap().toughness, 1);
    g.exile.push(crate::card::CardInstance::new(
        crate::card::CardId(9101), catalog::grizzly_bears(), 0,
    ));
    g.exile.push(crate::card::CardInstance::new(
        crate::card::CardId(9102), catalog::lightning_bolt(), 0,
    ));
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!(cp.power, 2, "2 exiled -> 2 power");
    assert_eq!(cp.toughness, 3, "toughness = power + 1");
}

/// Flying Octobot grows when another Villain you control enters (once/turn).
#[test]
fn flying_octobot_grows_on_villain() {
    let mut g = two_player_game();
    let octo = g.add_card_to_battlefield(0, catalog::flying_octobot());
    let villain = g.add_card_to_hand(0, catalog::common_crook());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.cast_spell(villain, None, vec![], None, None).expect("cast Villain");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(octo).unwrap().power, 2, "1/1 -> 2/2 on Villain enter");
}

/// Hobgoblin pumps +2/+0 whenever you discard a card.
#[test]
fn hobgoblin_pumps_on_discard() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::hobgoblin_mantled_marauder());
    g.add_card_to_hand(0, catalog::forest());
    let cast = g.add_card_to_hand(0, catalog::romantic_rendezvous());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Romantic Rendezvous discards a card as it resolves → Hobgoblin triggers.
    g.cast_spell(cast, None, vec![], None, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "1/2 +2/+0 -> 3 power");
}

/// Skyward Spider flies only while modified (a +1/+1 counter counts).
#[test]
fn skyward_spider_flies_while_modified() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::skyward_spider());
    assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying), "grounded unmodified");
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying), "flies while modified");
}

/// Costume Closet enters with two counters and hands one to a creature.
#[test]
fn costume_closet_moves_counter() {
    let mut g = two_player_game();
    let closet = g.move_card_to_battlefield_for_test(0, catalog::costume_closet());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: closet, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("move counter");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 3, "bear 2/2 -> 3/3");
}

/// Eerie Gravestone draws on enter.
#[test]
fn eerie_gravestone_draws_on_enter() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::eerie_gravestone());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card on enter");
}

/// Spectacular Tactics' destroy mode kills a power-4+ creature.
#[test]
fn spectacular_tactics_destroys_big_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
    let cast = g.add_card_to_hand(0, catalog::spectacular_tactics());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Choose mode 1 (destroy) targeting the 4/4.
    g.cast_spell(cast, Some(Target::Permanent(big)), vec![], Some(1), None).expect("cast destroy mode");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == big), "the 4/4 was destroyed");
}

/// Spectacular Spider-Man's sac ability blankets your team in hexproof +
/// indestructible.
#[test]
fn spectacular_spider_man_shields_team() {
    let mut g = two_player_game();
    let spidey = g.add_card_to_battlefield(0, catalog::spectacular_spider_man());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: spidey, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("sac for team shield");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Indestructible), "bear gained indestructible");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "bear gained hexproof");
}
