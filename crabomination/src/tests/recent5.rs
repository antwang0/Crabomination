//! Functionality tests for the `catalog::sets::decks::recent5` batch.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Plaguecrafter's ETB makes each player sacrifice a creature.
#[test]
fn plaguecrafter_etb_each_player_sacrifices() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::plaguecrafter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Plaguecrafter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "controller sacrificed (kept Plaguecrafter)");
    assert!(g.battlefield_find(theirs).is_none(), "opponent sacrificed their only creature");
}

/// Wither and Bloom kills a 2/2 with -3/-3, then its graveyard ability adds a
/// +1/+1 counter.
#[test]
fn wither_and_bloom_minus_then_graveyard_counter() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wither_and_bloom());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Wither and Bloom");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "-3/-3 kills the 2/2");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Wither and Bloom in graveyard");

    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(mine)), additional_targets: vec![], x_value: None,
    }).expect("graveyard ability");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.exile.iter().any(|c| c.id == id), "exiled as its own cost");
}

/// Sythis gains a life when you cast an enchantment spell.
#[test]
fn sythis_gains_life_on_enchantment_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sythis_harvests_hand());
    g.add_card_to_library(0, catalog::forest());
    let ench = g.add_card_to_hand(0, catalog::garruks_uprising());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast enchantment");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "Sythis gained 1 life for the enchantment cast");
}

/// Toski ships its keyword suite and draws when a creature deals combat damage
/// to a player.
#[test]
fn toski_keywords_and_combat_draw() {
    let t = catalog::toski_bearer_of_secrets();
    for kw in [Keyword::CantBeCountered, Keyword::Indestructible, Keyword::MustAttack] {
        assert!(t.keywords.contains(&kw), "Toski has {kw:?}");
    }
    let mut g = two_player_game();
    let toski = g.add_card_to_battlefield(0, catalog::toski_bearer_of_secrets());
    g.clear_sickness(toski);
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: toski, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[0].hand.len(), hand + 1, "combat damage to player drew a card");
}

/// Misdirection repoints a targeted spell at a new target.
#[test]
fn misdirection_changes_a_spells_target() {
    let mut g = two_player_game();
    // P1 bolts P0's bear; P0 misdirects it onto P1's bear.
    let my_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let their_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(my_bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("P1 casts bolt at my bear");
    let mis = g.add_card_to_hand(0, catalog::misdirection());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(their_bear))]));
    g.perform_action(GameAction::CastSpell {
        card_id: mis, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Misdirection at the bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(my_bear).is_some(), "my bear was spared");
    assert!(g.battlefield_find(their_bear).is_none(), "the bolt now killed their bear");
}

/// Flawless Maneuver grants the team indestructible.
#[test]
fn flawless_maneuver_grants_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::flawless_maneuver());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Flawless Maneuver");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible), "bear is indestructible");
}

/// Venser's ETB returns a permanent to its owner's hand.
#[test]
fn venser_bounces_a_permanent() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::venser_shaper_savant());
    assert!(catalog::venser_shaper_savant().keywords.contains(&Keyword::Flash), "Venser has flash");
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Venser");
    drain_stack(&mut g);
    g.perform_action(GameAction::PassPriority).ok();
    g.perform_action(GameAction::PassPriority).ok();
    // The ETB trigger needs a target; resolve via auto/scripted target.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(target))]));
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == target), "permanent returned to owner's hand");
}

/// Hullbreaker Horror ships flash + can't-be-countered and bounces a permanent
/// when you cast a spell.
#[test]
fn hullbreaker_horror_bounces_on_spell_cast() {
    let h = catalog::hullbreaker_horror();
    assert!(h.keywords.contains(&Keyword::Flash) && h.keywords.contains(&Keyword::CantBeCountered));
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hullbreaker_horror());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(victim)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a spell");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "Hullbreaker bounced the permanent");
}

/// Drown in Sorrow sweeps small creatures with -2/-2 and scries.
#[test]
fn drown_in_sorrow_minus_two_sweep() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::grave_titan()); // 6/6
    let id = g.add_card_to_hand(0, catalog::drown_in_sorrow());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Drown in Sorrow");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "2/2 dies to -2/-2");
    assert!(g.battlefield_find(big).is_some(), "6/6 survives");
}

/// Shamanic Revelation draws one per creature and gains 4 life per power-4.
#[test]
fn shamanic_revelation_draws_and_gains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_battlefield(0, catalog::grave_titan()); // 6/6, power >= 4
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::shamanic_revelation());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Shamanic Revelation");
    drain_stack(&mut g);
    // -1 hand for the cast, +2 drawn (two creatures).
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew one per creature");
    assert_eq!(g.players[0].life, life + 4, "ferocious gained 4 for the power-4 creature");
}

/// End-Raze Forerunners pumps the rest of the team on ETB.
#[test]
fn end_raze_forerunners_team_pump() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::end_raze_forerunners());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast End-Raze");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 4), "bear pumped +2/+2");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample), "bear gained trample");
}

/// Garruk's Uprising grants trample and draws when a power-4 creature enters.
#[test]
fn garruks_uprising_trample_and_power4_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::garruks_uprising());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample), "anthem grants trample");
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    let titan = g.add_card_to_hand(0, catalog::grave_titan()); // power 6
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: titan, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a power-4 creature");
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() > hand, "Garruk's Uprising drew for the power-4 ETB");
}

/// Guardian Project draws when a nontoken creature you control enters.
#[test]
fn guardian_project_draws_on_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::guardian_project());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature");
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() > hand, "Guardian Project drew for the creature ETB");
}

/// Neoform sacrifices a creature and tutors a creature with MV one higher onto
/// the battlefield with a +1/+1 counter.
#[test]
fn neoform_tutors_one_higher_with_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2 (the only creature; auto-sacrificed)
    g.add_card_to_library(0, catalog::grave_titan()); // MV 6 — not eligible
    let target = g.add_card_to_library(0, catalog::hypnotic_specter()); // MV 3 == 2+1
    let id = g.add_card_to_hand(0, catalog::neoform());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Neoform");
    drain_stack(&mut g);
    let tutored = g.battlefield_find(target).expect("tutored creature on battlefield");
    assert_eq!(tutored.counter_count(CounterType::PlusOnePlusOne), 1, "entered with a +1/+1 counter");
}

/// Eldritch Evolution tutors a creature with MV up to sacrificed+2 and exiles
/// itself.
#[test]
fn eldritch_evolution_tutors_up_to_plus_two() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2 → up to 4 (auto-sacrificed)
    let target = g.add_card_to_library(0, catalog::hypnotic_specter()); // MV 3 <= 4
    let id = g.add_card_to_hand(0, catalog::eldritch_evolution());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Eldritch Evolution");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_some(), "tutored creature on battlefield");
    assert!(g.exile.iter().any(|c| c.id == id), "Eldritch Evolution exiled itself");
}

/// Skrelv ships toxic + can't block and grants hexproof to another creature.
#[test]
fn skrelv_grants_hexproof() {
    let s = catalog::skrelv_defector_mite();
    assert!(s.keywords.contains(&Keyword::CantBlock) && s.keywords.contains(&Keyword::Toxic(1)));
    let mut g = two_player_game();
    let skrelv = g.add_card_to_battlefield(0, catalog::skrelv_defector_mite());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(skrelv);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skrelv, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None,
    }).expect("activate Skrelv");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof), "target gained hexproof");
}

/// Soul's Majesty draws cards equal to a creature's power.
#[test]
fn souls_majesty_draws_equal_to_power() {
    let mut g = two_player_game();
    let titan = g.add_card_to_battlefield(0, catalog::grave_titan()); // power 6
    for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::souls_majesty());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(titan)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Soul's Majesty");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "drew 6 (the titan's power)");
}

/// Momentous Fall sacrifices a creature to draw its power and gain its
/// toughness.
#[test]
fn momentous_fall_draw_and_gain() {
    let mut g = two_player_game();
    let titan = g.add_card_to_battlefield(0, catalog::grave_titan()); // 6/6
    for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::momentous_fall());
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![titan])]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Momentous Fall");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "drew 6 (sacrificed power)");
    assert_eq!(g.players[0].life, life + 6, "gained 6 (sacrificed toughness)");
}

/// Life's Legacy sacrifices a creature to draw cards equal to its power.
#[test]
fn lifes_legacy_draws_equal_to_power() {
    let mut g = two_player_game();
    let titan = g.add_card_to_battlefield(0, catalog::grave_titan()); // power 6
    for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::lifes_legacy());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![titan])]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Life's Legacy");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "drew 6 (sacrificed power)");
}

/// Return of the Wildspeaker — draw mode draws equal to greatest power.
#[test]
fn return_of_the_wildspeaker_draw_mode() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grave_titan()); // power 6
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::return_of_the_wildspeaker());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Return of the Wildspeaker");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "drew 6 (greatest power)");
}

/// Overrun pumps the whole team +3/+3 and grants trample.
#[test]
fn overrun_team_pump_and_trample() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::overrun());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Overrun");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (5, 5));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
}

/// Larger Than Life pumps a single creature +4/+4 with trample.
#[test]
fn larger_than_life_pumps_and_tramples() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::larger_than_life());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Larger Than Life");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (6, 6));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
}

/// Prey's Vengeance ships +2/+2 and the Rebound keyword.
#[test]
fn preys_vengeance_pumps_with_rebound() {
    assert!(catalog::preys_vengeance().keywords.contains(&Keyword::Rebound), "has Rebound");
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::preys_vengeance());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Prey's Vengeance");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 4));
}

/// Savage Smash pumps your creature then fights an opponent's.
#[test]
fn savage_smash_pumps_then_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 -> 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::savage_smash());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Savage Smash");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "4-power fights kill the 2/2");
    assert!(g.battlefield_find(mine).is_some(), "the 4/4 survives 2 damage");
}

/// Bite Down deals a creature's power to an opposing creature.
#[test]
fn bite_down_deals_power() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grave_titan()); // power 6
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::bite_down());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Bite Down");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "6 damage kills the 2/2");
}

/// Crushing Vines (mode 1) destroys a target artifact.
#[test]
fn crushing_vines_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::crushing_vines());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(art)), additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast Crushing Vines mode 1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Inspiring Call draws per +1/+1-countered creature and grants indestructible.
#[test]
fn inspiring_call_draws_and_protects() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no counter — not counted
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::inspiring_call());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Inspiring Call");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew 2 (the countered creatures)");
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::Indestructible));
}
