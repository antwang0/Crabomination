//! Functionality tests for the `catalog::sets::decks::recent100` Kamigawa: Neon
//! Dynasty batch 6.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;

fn pass_through_combat(g: &mut GameState) {
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(g);
}

/// Golden-Tail Trainer discounts an Equipment spell by its power (1).
#[test]
fn golden_tail_trainer_discounts_equipment() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::golden_tail_trainer());
    let scimitar = g.add_card_to_hand(0, catalog::leonin_scimitar()); // {1} Equipment
    g.priority.player_with_priority = 0;
    // {1} - power 1 = {0}: casts with no mana floated.
    cast(&mut g, scimitar);
    assert!(g.battlefield_find(scimitar).is_some(), "discounted Equipment resolved");
}

/// Golden-Tail Trainer's attack pumps other modified creatures by its power.
#[test]
fn golden_tail_trainer_attack_pumps_modified() {
    let mut g = two_player_game();
    let trainer = g.add_card_to_battlefield(0, catalog::golden_tail_trainer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // modified
    g.clear_sickness(trainer);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: trainer,
        target: AttackTarget::Player(1),
    }]))
    .expect("trainer attacks");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "2/2 base +1/+1 counter +1/+1 from trainer power");
}

/// Kami of Terrible Secrets draws + gains only when you control an artifact and
/// an enchantment.
#[test]
fn kami_of_terrible_secrets_conditional_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.players[0].life = 20;
    g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact
    g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_terrible_secrets());
    let hand = g.players[0].hand.len();
    g.fire_self_etb_triggers(kami, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.players[0].life, 21, "gained 1 life");
}

/// Sky-Blessed Samurai's Affinity for enchantments reduces its cost.
#[test]
fn sky_blessed_samurai_affinity_for_enchantments() {
    let mut g = two_player_game();
    // Two enchantments → {6}{W} becomes {4}{W}.
    for _ in 0..2 { g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); }
    let samurai = g.add_card_to_hand(0, catalog::sky_blessed_samurai());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    cast(&mut g, samurai);
    assert!(g.battlefield_find(samurai).is_some(), "cast for four-and-white via affinity");
}

/// Bamboo Grove Archer's Channel destroys a flyer from hand.
#[test]
fn bamboo_grove_archer_channel_kills_flyer() {
    let mut g = two_player_game();
    let archer = g.add_card_to_hand(0, catalog::bamboo_grove_archer());
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: archer,
        ability_index: 0,
        target: Some(Target::Permanent(flyer)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("channel Bamboo Grove Archer");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flyer).is_none(), "flyer destroyed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == archer), "archer discarded");
}

/// Walking Skyscraper is discounted per modified creature and has hexproof only
/// while untapped.
#[test]
fn walking_skyscraper_discount_and_hexproof() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // modified
    let tower = g.add_card_to_hand(0, catalog::walking_skyscraper());
    g.players[0].mana_pool.add_colorless(7); // {8} - 1 modified = {7}
    g.priority.player_with_priority = 0;
    cast(&mut g, tower);
    assert!(g.battlefield_find(tower).is_some(), "cast for seven");
    assert!(g.computed_permanent(tower).unwrap().keywords.contains(&Keyword::Hexproof), "hexproof untapped");
    g.battlefield_find_mut(tower).unwrap().tapped = true;
    assert!(!g.computed_permanent(tower).unwrap().keywords.contains(&Keyword::Hexproof), "no hexproof tapped");
}

/// Master's Rebuke bites: your creature deals its power to an opponent's creature.
#[test]
fn masters_rebuke_bites() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::masters_rebuke());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast Master's Rebuke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "2/2 took 4 damage and died");
}

/// Tempered in Solitude exiles the top card on a lone attack (impulse draw).
#[test]
fn tempered_in_solitude_impulse_on_solo_attack() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::tempered_in_solitude());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let exile_before = g.exile.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("bear attacks alone");
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), exile_before + 1, "top card exiled for impulse play");
}

/// Akki Ember-Keeper makes a Spirit when a nontoken modified creature dies (and
/// not on an unmodified one). Kills via the SBA path so the LKI snapshot keeps
/// the counter visible to the "modified" filter.
#[test]
fn akki_ember_keeper_makes_spirit_on_modified_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::akki_ember_keeper());
    // Unmodified creature dying makes nothing.
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(plain).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "no token for unmodified death");
    // Modified creature dying makes a Spirit.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // 3/3, modified
    g.battlefield_find_mut(bear).unwrap().damage = 3;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "made a Spirit token");
}

/// Thundering Raiju deals damage equal to the count of other modified creatures.
#[test]
fn thundering_raiju_pings_per_modified() {
    let mut g = two_player_game();
    let raiju = g.add_card_to_battlefield(0, catalog::thundering_raiju());
    // Two other modified creatures.
    for _ in 0..2 {
        let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(c).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(raiju);
    g.players[1].life = 20;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(target))]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: raiju,
        target: AttackTarget::Player(1),
    }]))
    .expect("raiju attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "2 other modified creatures → 2 damage");
}

/// Scrapyard Steelbreaker pumps itself by sacrificing another artifact.
#[test]
fn scrapyard_steelbreaker_sac_pump() {
    let mut g = two_player_game();
    let breaker = g.add_card_to_battlefield(0, catalog::scrapyard_steelbreaker());
    g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact to sac
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: breaker,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Scrapyard Steelbreaker");
    drain_stack(&mut g);
    let cp = g.computed_permanent(breaker).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "3/4 +2/+1");
}

/// Atsushi's death (default mode: impulse) exiles the top two cards of the library.
#[test]
fn atsushi_death_impulse_exiles_two() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let atsushi = g.add_card_to_battlefield(0, catalog::atsushi_the_blazing_sky());
    let exile_before = g.exile.len();
    g.remove_to_graveyard_with_triggers(atsushi);
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), exile_before + 2, "impulse-exiled the top two cards");
}

/// Junji's death (default mode: drain) makes each opponent discard two and lose 2.
#[test]
fn junji_death_discard_and_drain() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    g.players[1].life = 20;
    let junji = g.add_card_to_battlefield(0, catalog::junji_the_midnight_sky());
    g.remove_to_graveyard_with_triggers(junji);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "opponent discarded two");
    assert_eq!(g.players[1].life, 18, "opponent lost 2 life");
}

/// Chishiro makes a Spirit when an Equipment you control enters.
#[test]
fn chishiro_spirit_on_equipment_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::chishiro_the_shattered_blade());
    let eq = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: eq }]);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "made a Spirit token");
}

/// Risona gains an indestructible counter when it deals combat damage.
#[test]
fn risona_gains_indestructible_counter() {
    let mut g = two_player_game();
    let risona = g.add_card_to_battlefield(0, catalog::risona_asari_commander());
    g.clear_sickness(risona);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: risona,
        target: AttackTarget::Player(1),
    }]))
    .expect("risona attacks");
    drain_stack(&mut g);
    pass_through_combat(&mut g);
    assert_eq!(
        g.battlefield_find(risona).unwrap().counter_count(CounterType::Indestructible),
        1,
        "got an indestructible counter"
    );
}

/// Risona sheds an indestructible counter when its controller is dealt combat
/// damage.
#[test]
fn risona_loses_counter_when_you_take_damage() {
    let mut g = two_player_game();
    let risona = g.add_card_to_battlefield(0, catalog::risona_asari_commander());
    g.battlefield_find_mut(risona).unwrap().add_counters(CounterType::Indestructible, 1);
    // Opponent's attacker connects with player 0.
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.fire_combat_damage_to_player_triggers(attacker, 0, 2);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(risona).unwrap().counter_count(CounterType::Indestructible),
        0,
        "indestructible counter removed"
    );
}

/// Traproot Kami's toughness equals the number of Forests in play.
#[test]
fn traproot_kami_toughness_tracks_forests() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::traproot_kami());
    assert_eq!(g.computed_permanent(kami).unwrap().toughness, 0, "no Forests yet");
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::forest());
    assert_eq!(g.computed_permanent(kami).unwrap().toughness, 2, "two Forests in play");
}

/// Unstoppable Ogre's ETB stops a creature from blocking.
#[test]
fn unstoppable_ogre_stops_blocker() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ogre = g.add_card_to_battlefield(0, catalog::unstoppable_ogre());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    g.fire_self_etb_triggers(ogre, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// You Are Already Dead destroys a damaged creature and draws.
#[test]
fn you_are_already_dead_kills_damaged() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().damage = 1; // dealt damage this turn
    g.battlefield_find_mut(bear).unwrap().dealt_damage_this_turn = true;
    let spell = g.add_card_to_hand(0, catalog::you_are_already_dead());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast You Are Already Dead");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "damaged creature destroyed");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "spell left hand, drew one");
}
