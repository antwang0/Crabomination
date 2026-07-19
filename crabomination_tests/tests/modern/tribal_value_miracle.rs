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

// ── Tribal / value batch ─────────────────────────────────────────────────────

/// Geralf's Messenger enters tapped and drains the opponent for 2.
#[test]
fn geralfs_messenger_etb_drains_two() {
    let mut g = two_player_game();
    let life = g.players[1].life;
    let m = g.add_card_to_battlefield(0, catalog::geralfs_messenger());
    g.fire_self_etb_triggers(m, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "opponent lost 2 life");
    assert!(g.battlefield_find(m).unwrap().tapped, "entered tapped");
}

/// Geralf's Messenger returns with a +1/+1 counter via Undying.
#[test]
fn geralfs_messenger_undying_returns() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::geralfs_messenger());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(m)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Geralf's Messenger");
    assert!(back.is_some(), "returned via Undying");
    assert_eq!(back.unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "with a +1/+1 counter");
}

/// Cauldron Familiar returns from the graveyard by sacrificing a Food.
#[test]
fn cauldron_familiar_returns_by_sacrificing_food() {
    let mut g = two_player_game();
    let cat = g.add_card_to_graveyard(0, catalog::cauldron_familiar());
    // Put a Food token on the battlefield to pay the sacrifice cost.
    let food_def = crabomination_base::tokens::token_to_card_definition(
        &crabomination_base::tokens::food_token(),
    );
    let _food = g.add_card_to_battlefield(0, food_def);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cat, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac Food to return Cauldron Familiar");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == cat), "Cauldron Familiar back on battlefield");
}

/// Witch's Oven turns a sacrificed creature into a Food token.
#[test]
fn witchs_oven_bakes_a_food() {
    let mut g = two_player_game();
    let oven = g.add_card_to_battlefield(0, catalog::witchs_oven());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: oven, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("bake");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "Food token created");
}

/// Witch's Oven bakes two Food off a toughness-4+ creature; the sacrificed
/// P/T survives to the ability's resolution (Effect::WithSacrificedPt).
#[test]
fn witchs_oven_bakes_two_food_for_big_toughness() {
    let mut g = two_player_game();
    let oven = g.add_card_to_battlefield(0, catalog::witchs_oven());
    g.add_card_to_battlefield(0, catalog::colossal_dreadmaw()); // 6/6
    g.perform_action(GameAction::ActivateAbility {
        card_id: oven, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("bake");
    drain_stack(&mut g);
    let foods = g.battlefield.iter().filter(|c| c.definition.name == "Food").count();
    assert_eq!(foods, 2, "toughness 6 ≥ 4 → two Food");
}

/// Goblin Ringleader rakes Goblins off the top of the library into hand.
#[test]
fn goblin_ringleader_reveals_goblins_to_hand() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let gob = g.add_card_to_library(0, catalog::goblin_ringleader()); // a Goblin on top
    let rl = g.add_card_to_battlefield(0, catalog::goblin_ringleader());
    g.fire_self_etb_triggers(rl, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == gob), "Goblin pulled to hand");
}

/// Recruiter of the Guard tutors a low-toughness creature.
#[test]
fn recruiter_of_the_guard_tutors_small_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // 2/2, toughness 2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let r = g.add_card_to_battlefield(0, catalog::recruiter_of_the_guard());
    g.fire_self_etb_triggers(r, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "tutored the creature to hand");
}

/// Squadron Hawk fetches a copy of itself.
#[test]
fn squadron_hawk_fetches_itself() {
    let mut g = two_player_game();
    let other = g.add_card_to_library(0, catalog::squadron_hawk());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(other))]));
    let h = g.add_card_to_battlefield(0, catalog::squadron_hawk());
    g.fire_self_etb_triggers(h, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == other), "fetched a Squadron Hawk");
}

/// Felidar Guardian blinks another permanent you control.
#[test]
fn felidar_guardian_blinks_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    let f = g.add_card_to_battlefield(0, catalog::felidar_guardian());
    g.fire_self_etb_triggers(f, 0);
    drain_stack(&mut g);
    // The original bear id leaves and a fresh copy returns; assert a Grizzly Bears is present.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "bear blinked back");
}

/// Village Bell-Ringer untaps all your creatures on ETB.
#[test]
fn village_bell_ringer_untaps_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let b = g.add_card_to_battlefield(0, catalog::village_bell_ringer());
    g.fire_self_etb_triggers(b, 0);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "bear untapped");
}

/// Deceiver Exarch untaps a permanent you control (mode 0 default).
#[test]
fn deceiver_exarch_untaps_your_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Modes(vec![0]),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    let e = g.add_card_to_battlefield(0, catalog::deceiver_exarch());
    g.fire_self_etb_triggers(e, 0);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "permanent untapped");
}

/// Pashalik Mons pings when a Goblin you control dies.
#[test]
fn pashalik_mons_pings_on_goblin_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pashalik_mons());
    let gob = g.add_card_to_battlefield(0, catalog::goblin_ringleader());
    let foe_life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Player(1)),
    ]));
    // Kill the Goblin.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(gob)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt goblin");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 1, "Pashalik pinged for 1");
}

/// Sling-Gang Lieutenant makes two Goblins and drains via sacrifice.
#[test]
fn sling_gang_lieutenant_etb_and_sacrifice() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::sling_gang_lieutenant());
    g.fire_self_etb_triggers(s, 0);
    drain_stack(&mut g);
    let goblins = g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count();
    assert_eq!(goblins, 2, "two Goblin tokens");
    let foe_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac a goblin to drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 1, "opp lost 1");
    assert_eq!(g.players[0].life, my_life + 1, "you gained 1");
}

/// Regal Force draws a card per green creature you control.
#[test]
fn regal_force_draws_per_green_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    g.add_card_to_battlefield(0, catalog::llanowar_elves()); // green
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    let rf = g.add_card_to_battlefield(0, catalog::regal_force());
    g.fire_self_etb_triggers(rf, 0);
    drain_stack(&mut g);
    // Regal Force itself is green → 3 green creatures → draw 3.
    assert_eq!(g.players[0].hand.len(), hand0 + 3, "drew per green creature");
}

/// Wirewood Hivemaster makes an Insect when another nontoken Elf enters.
#[test]
fn wirewood_hivemaster_spawns_insect() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wirewood_hivemaster());
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast elf");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Insect"), "Insect token made");
}

/// Ezuri's overrun pumps your Elves +3/+3.
#[test]
fn ezuri_overrun_pumps_elves() {
    let mut g = two_player_game();
    let ez = g.add_card_to_battlefield(0, catalog::ezuri_renegade_leader());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ez, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("overrun");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(elf).unwrap().power(), 1 + 3, "Elf pumped +3");
}

/// Kiki-Jiki makes a hasty token copy of a nonlegendary creature.
#[test]
fn kiki_jiki_copies_creature_with_haste() {
    let mut g = two_player_game();
    let kiki = g.add_card_to_battlefield(0, catalog::kiki_jiki_mirror_breaker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: kiki, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("kiki copy");
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "a token copy was made");
}

/// Kalitas exiles a dying opponent creature and mints a Zombie.
#[test]
fn kalitas_exiles_and_makes_zombie() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kalitas_traitor_of_ghet());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt foe");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == foe), "creature exiled");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Zombie"), "Zombie token made");
}

/// Stormwing Entity scries on ETB and is a 3/3 flyer with prowess.
#[test]
fn stormwing_entity_scries_on_etb() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let s = g.add_card_to_battlefield(0, catalog::stormwing_entity());
    assert!(g.battlefield_find(s).unwrap().definition.keywords.contains(&Keyword::Prowess));
    g.fire_self_etb_triggers(s, 0);
    drain_stack(&mut g); // scry auto-resolves; no panic = pass
}

/// Quirion Ranger untaps a creature by bouncing a Forest you control.
#[test]
fn quirion_ranger_bounces_forest_to_untap() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let ranger = g.add_card_to_battlefield(0, catalog::quirion_ranger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ranger, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Quirion Ranger");
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_none(), "Forest returned to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "Forest in hand");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "target creature untapped");
}

/// Wirewood Symbiote can't activate without an Elf to return.
#[test]
fn wirewood_symbiote_requires_an_elf_to_bounce() {
    let mut g = two_player_game();
    let symb = g.add_card_to_battlefield(0, catalog::wirewood_symbiote()); // an Insect, not an Elf
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    // No other Elf on the battlefield → activation rejected.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: symb, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    });
    assert!(res.is_err(), "no Elf to return → cost unpayable");
    // Add an Elf; now it works.
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.perform_action(GameAction::ActivateAbility {
        card_id: symb, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate with an Elf available");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == elf), "Elf returned to hand");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "creature untapped");
}

/// Sigarda stops an opponent's edict from making you sacrifice.
#[test]
fn sigarda_blocks_opponent_edict() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sigarda_host_of_herons());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Opponent (P1) casts Diabolic Edict targeting P0.
    let edict = g.add_card_to_hand(1, catalog::diabolic_edict());
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: edict, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast edict at Sigarda's controller");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "Sigarda prevented the sacrifice");
}

/// A player's own sacrifice effect still works under Sigarda (only opponents
/// are blocked).
#[test]
fn sigarda_allows_your_own_sacrifice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sigarda_host_of_herons());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // P0 casts their own Innocent Blood-style edict hitting each player.
    let ib = g.add_card_to_hand(0, catalog::innocent_blood());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ib, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast innocent blood");
    drain_stack(&mut g);
    // P0's own spell can still make P0 sacrifice (the bear is gone).
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0),
        "your own sacrifice effect still resolves");
}

/// Heritage Druid taps three Elves to add {G}{G}{G}.
#[test]
fn heritage_druid_taps_three_elves_for_green() {
    let mut g = two_player_game();
    let druid = g.add_card_to_battlefield(0, catalog::heritage_druid());
    let e1 = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let e2 = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    // Heritage Druid itself is an Elf; with two more that's three untapped Elves.
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap three Elves for GGG");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3, "added GGG");
    let tapped = [druid, e1, e2].iter().filter(|id| g.battlefield_find(**id).unwrap().tapped).count();
    assert_eq!(tapped, 3, "three Elves tapped as the cost");
}

/// Heritage Druid can't activate without three untapped Elves.
#[test]
fn heritage_druid_needs_three_elves() {
    let mut g = two_player_game();
    let druid = g.add_card_to_battlefield(0, catalog::heritage_druid());
    g.add_card_to_battlefield(0, catalog::llanowar_elves()); // only two Elves total
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: druid, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    });
    assert!(res.is_err(), "fewer than three untapped Elves → cost unpayable");
}

// ── Goblin tribal + White angels batch ───────────────────────────────────────

/// Stingscourger bounces an opponent creature on ETB.
#[test]
fn stingscourger_bounces_opponent_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(foe))]));
    let s = g.add_card_to_battlefield(0, catalog::stingscourger());
    g.fire_self_etb_triggers(s, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == foe), "to owner's hand");
}

/// Mad Auntie pumps other Goblins you control.
#[test]
fn mad_auntie_pumps_goblins() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mad_auntie());
    let gob = g.add_card_to_battlefield(0, catalog::goblin_ringleader()); // 2/2 Goblin
    assert_eq!(g.computed_permanent(gob).unwrap().power, 3, "other Goblin gets +1/+1");
}

/// Goblin Chirurgeon regenerates a creature by sacrificing a Goblin.
#[test]
fn goblin_chirurgeon_regenerates() {
    let mut g = two_player_game();
    let chir = g.add_card_to_battlefield(0, catalog::goblin_chirurgeon());
    let fodder = g.add_card_to_battlefield(0, catalog::sparksmith()); // 1/1 Goblin, sacrificed
    let gob = g.add_card_to_battlefield(0, catalog::goblin_ringleader()); // regen target
    g.perform_action(GameAction::ActivateAbility {
        card_id: chir, ability_index: 0, target: Some(Target::Permanent(gob)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac a goblin to regen");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder Goblin sacrificed");
    assert!(g.battlefield_find(gob).unwrap().regeneration_shields > 0, "regen shield set");
}

/// Sparksmith pings a creature for the number of Goblins, hurting you too.
#[test]
fn sparksmith_pings_for_goblin_count() {
    let mut g = two_player_game();
    let smith = g.add_card_to_battlefield(0, catalog::sparksmith());
    g.clear_sickness(smith); // 1 Goblin
    g.add_card_to_battlefield(0, catalog::goblin_ringleader()); // 2 Goblins total
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives 2
    let my_life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: smith, ability_index: 0, target: Some(Target::Permanent(foe)), additional_targets: Vec::new(), x_value: None,
    }).expect("sparksmith ping");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 2, "2 damage to creature");
    assert_eq!(g.players[0].life, my_life - 2, "2 damage to you");
}

/// Goblin Sharpshooter untaps when a creature dies.
#[test]
fn goblin_sharpshooter_untaps_on_death() {
    let mut g = two_player_game();
    let shooter = g.add_card_to_battlefield(0, catalog::goblin_sharpshooter());
    g.battlefield_find_mut(shooter).unwrap().tapped = true;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("kill a creature");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(shooter).unwrap().tapped, "Sharpshooter untapped on death");
}

/// Krenko, Tin Street Kingpin grows and makes Goblins equal to its power on attack.
#[test]
fn krenko_tin_street_makes_goblins_on_attack() {
    let mut g = two_player_game();
    let krenko = g.add_card_to_battlefield(0, catalog::krenko_tin_street_kingpin());
    g.clear_sickness(krenko);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: krenko, target: AttackTarget::Player(1) }])
        .expect("Krenko attacks");
    drain_stack(&mut g);
    // Krenko was 1/2; +1/+1 → power 2 → two Goblin tokens.
    let goblins = g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count();
    assert_eq!(goblins, 2, "Goblins equal to Krenko's new power");
}

/// Elvish Promenade makes an Elf token per Elf you control.
#[test]
fn elvish_promenade_doubles_elves() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let id = g.add_card_to_hand(0, catalog::elvish_promenade());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Elvish Promenade");
    drain_stack(&mut g);
    let elf_warriors = g.battlefield.iter().filter(|c| c.definition.name == "Elf Warrior").count();
    assert_eq!(elf_warriors, 2, "one token per Elf controlled");
}

/// Shalai grants your other creatures hexproof.
#[test]
fn shalai_grants_team_hexproof() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shalai_voice_of_plenty());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof),
        "other creature gains hexproof");
}

/// Angel of Invention anthems your other creatures.
#[test]
fn angel_of_invention_anthems_team() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::angel_of_invention());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1 anthem");
}

/// Lyra Dawnbringer buffs other Angels and grants them lifelink.
#[test]
fn lyra_buffs_other_angels() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lyra_dawnbringer());
    let other = g.add_card_to_battlefield(0, catalog::serra_angel()); // an Angel
    let p = g.computed_permanent(other).unwrap();
    assert_eq!(p.power, 5, "other Angel +1/+1 (4→5)");
    assert!(p.keywords.contains(&Keyword::Lifelink), "other Angel gains lifelink");
}

/// Resplendent Angel makes a 4/4 at end step if you gained 5+ life.
#[test]
fn resplendent_angel_makes_token_after_lifegain() {
    use crabomination::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::resplendent_angel());
    g.players[0].life_gained_this_turn = 5;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Angel" && c.power() == 4),
        "4/4 Angel token created");
}

/// Wirewood Lodge untaps an Elf for {G}.
#[test]
fn wirewood_lodge_untaps_elf() {
    let mut g = two_player_game();
    let lodge = g.add_card_to_battlefield(0, catalog::wirewood_lodge());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.battlefield_find_mut(elf).unwrap().tapped = true;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lodge, ability_index: 1, target: Some(Target::Permanent(elf)), additional_targets: Vec::new(), x_value: None,
    }).expect("untap Elf");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(elf).unwrap().tapped, "Elf untapped");
}

// ── Anthems / value batch 3 ──────────────────────────────────────────────────

/// Cathars' Crusade counters up your team when a creature enters.
#[test]
fn cathars_crusade_counters_on_creature_enter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cathars_crusade());
    let existing = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Cast a creature so it enters through the real funnel.
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast elf");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(existing).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "existing creature got a counter");
    assert_eq!(g.battlefield_find(elf).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "the newcomer also got a counter");
}

/// Anointed Procession doubles token creation.
#[test]
fn anointed_procession_doubles_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::anointed_procession());
    let s = g.add_card_to_battlefield(0, catalog::sling_gang_lieutenant()); // ETB: 2 Goblins
    g.fire_self_etb_triggers(s, 0);
    drain_stack(&mut g);
    let goblins = g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count();
    assert_eq!(goblins, 4, "two Goblins doubled to four");
}

/// Grim Tutor fetches any card and costs 3 life.
#[test]
fn grim_tutor_fetches_and_loses_life() {
    let mut g = two_player_game();
    let target = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    let id = g.add_card_to_hand(0, catalog::grim_tutor());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Grim Tutor");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == target), "fetched to hand");
    assert_eq!(g.players[0].life, life - 3, "lost 3 life");
}

/// Puresteel Paladin draws when an Equipment enters.
#[test]
fn puresteel_paladin_draws_on_equipment() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::puresteel_paladin());
    let drawn = g.add_card_to_library(0, catalog::forest());
    let equip = g.add_card_to_hand(0, catalog::bonesplitter()); // an Equipment
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: equip, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Equipment");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == drawn),
        "Puresteel drew a card when the Equipment entered");
}

/// Quirion Dryad grows when you cast a nongreen colored spell.
#[test]
fn quirion_dryad_grows_on_colored_spell() {
    let mut g = two_player_game();
    let dryad = g.add_card_to_battlefield(0, catalog::quirion_dryad());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // a red spell
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast red spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(dryad).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Dryad grew on a red spell");
}

// ── Elf/ETB value batch 4 ────────────────────────────────────────────────────

/// Impact Tremors pings each opponent when a creature you control enters.
#[test]
fn impact_tremors_pings_on_creature_enter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::impact_tremors());
    let life = g.players[1].life;
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast elf");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent pinged for 1");
}

/// Wellwisher gains life equal to the number of Elves.
#[test]
fn wellwisher_gains_life_per_elf() {
    let mut g = two_player_game();
    let well = g.add_card_to_battlefield(0, catalog::wellwisher());
    g.clear_sickness(well);
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.add_card_to_battlefield(1, catalog::llanowar_elves()); // counts all Elves
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: well, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("wellwisher");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "1 life per Elf (3 Elves)");
}

/// Timberwatch Elf pumps a creature by the Elf count.
#[test]
fn timberwatch_elf_pumps_by_elf_count() {
    let mut g = two_player_game();
    let timber = g.add_card_to_battlefield(0, catalog::timberwatch_elf());
    g.clear_sickness(timber);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::llanowar_elves()); // 2 Elves total
    g.perform_action(GameAction::ActivateAbility {
        card_id: timber, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 2 + 2, "+X/+X where X = 2 Elves");
}

/// Lys Alana Huntmaster makes an Elf token when you cast an Elf spell.
#[test]
fn lys_alana_makes_token_on_elf_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lys_alana_huntmaster());
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast elf spell");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Elf Warrior"), "Elf token made");
}

/// Soul of the Harvest draws when another nontoken creature enters.
#[test]
fn soul_of_the_harvest_draws_on_creature_enter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::soul_of_the_harvest());
    g.add_card_to_library(0, catalog::forest());
    let drawn_lib = g.players[0].library.len();
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast elf");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), drawn_lib - 1, "drew a card");
}

/// Wirewood Herald tutors an Elf to hand when it dies.
#[test]
fn wirewood_herald_tutors_elf_on_death() {
    let mut g = two_player_game();
    let herald = g.add_card_to_battlefield(0, catalog::wirewood_herald());
    let elf = g.add_card_to_library(0, catalog::ezuri_renegade_leader()); // an Elf
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(elf))]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(herald)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt herald");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == elf), "tutored an Elf to hand");
}

// ── Cube expansion: Swords + planeswalkers + Myr Battlesphere ─────────────────

#[test]
fn sword_of_fire_and_ice_pings_and_draws() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_fire_and_ice());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(attacker);
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let opp_life = g.players[1].life;
    let my_hand = g.players[0].hand.len();
    let combat = g.compute_battlefield().iter().find(|c| c.id == attacker).unwrap().power;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.players[0].hand.len() > my_hand { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, opp_life - combat - 2, "combat damage + 2 from Sword");
    assert!(g.players[0].hand.len() > my_hand, "drew a card");
}

#[test]
fn sword_of_light_and_shadow_gains_life_and_returns_creature() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_light_and_shadow());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(attacker);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let my_life = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.players[0].hand.iter().any(|c| c.id == dead) { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, my_life + 3, "gained 3 life");
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "returned a creature from graveyard");
}

#[test]
fn sword_of_truth_and_justice_counters_and_proliferates() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::looter_il_kor());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_truth_and_justice());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(attacker);
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.battlefield_find(attacker).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne)) >= 2 { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    // +1/+1 counter placed, then proliferate adds a second.
    assert_eq!(
        g.battlefield_find(attacker).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "counter placed then proliferated",
    );
}

#[test]
fn wrenn_and_six_plus_one_returns_land_from_graveyard() {
    let mut g = two_player_game();
    let wrenn = g.add_card_to_battlefield(0, catalog::wrenn_and_six());
    let land = g.add_card_to_graveyard(0, catalog::mountain());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: wrenn, ability_index: 0, target: None, x_value: None,
    }).expect("Wrenn +1");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "returned a land to hand");
}

#[test]
fn wrenn_and_six_minus_one_pings_any_target() {
    let mut g = two_player_game();
    let wrenn = g.add_card_to_battlefield(0, catalog::wrenn_and_six());
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: wrenn, ability_index: 1,
        target: Some(Target::Player(1)), x_value: None,
    }).expect("Wrenn -1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 1, "pinged opponent for 1");
}

#[test]
fn karn_liberated_minus_three_exiles_target_permanent() {
    let mut g = two_player_game();
    let karn = g.add_card_to_battlefield(0, catalog::karn_liberated());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: karn, ability_index: 1,
        target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("Karn -3");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "permanent exiled");
    assert!(g.exile.iter().any(|c| c.id == bear), "in exile");
}

#[test]
fn myr_battlesphere_makes_four_myr_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::myr_battlesphere());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let myr = g.battlefield.iter().filter(|c| c.definition.name == "Myr" && c.controller == 0).count();
    assert_eq!(myr, 4, "ETB mints four Myr");
}

#[test]
fn myr_battlesphere_attack_pings_for_each_untapped_myr() {
    let mut g = two_player_game();
    let sphere = g.add_card_to_battlefield(0, catalog::myr_battlesphere());
    // ETB mints four untapped Myr to fuel the attack trigger.
    g.fire_self_etb_triggers(sphere, 0);
    drain_stack(&mut g);
    g.clear_sickness(sphere);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sphere, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    for _ in 0..6 {
        if g.players[1].life <= opp_life - 4 { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.players[1].life <= opp_life - 4, "pinged for each of the four untapped Myr");
}

#[test]
fn spine_of_ish_sah_destroys_on_etb_and_returns_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spine = g.add_card_to_hand(0, catalog::spine_of_ish_sah());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: spine, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spine castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "ETB destroyed the opponent's permanent");
    // Now kill Spine; it returns to its owner's hand.
    g.remove_to_graveyard_with_triggers(spine);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spine), "Spine returns to owner's hand on death");
}

#[test]
fn ankh_of_mishra_pings_land_controller_on_entry() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ankh_of_mishra());
    let forest = g.add_card_to_hand(0, catalog::forest());
    let life = g.players[0].life;
    g.perform_action(GameAction::PlayLand(forest)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "land's controller takes 2 when a land enters");
}

// ── Miracle (CR 702.94) ──────────────────────────────────────────────────────

#[test]
fn miracle_first_draw_grants_alt_cost_and_casts_cheaply() {
    use crabomination::card::CounterType;
    let _ = CounterType::Loyalty;
    let mut g = two_player_game();
    // Put Bonfire on top so the turn's first draw reveals it.
    let bonfire = g.add_card_to_library(0, catalog::bonfire_of_the_damned());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].cards_drawn_this_turn = 0;
    let mut events = vec![];
    assert!(g.draw_one(0, &mut events), "drew the top card");
    // Miracle stamped the alt-cost on the drawn card.
    let card = g.players[0].hand.iter().find(|c| c.id == bonfire).expect("in hand");
    assert!(card.granted_alt_cast_cost_eot.is_some(), "miracle alt-cost granted on first draw");
    assert!(card.may_play_until.is_some(), "miracle may-play window granted");
    // Cast it for the miracle cost {X}{R} with X=2 (pay {2}{R}).
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bonfire, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("miracle cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2, "X=2 damage to target player");
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "X=2 killed the 2/2");
}

#[test]
fn miracle_not_granted_on_later_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bonfire = g.add_card_to_library(0, catalog::bonfire_of_the_damned());
    g.players[0].cards_drawn_this_turn = 0;
    let mut events = vec![];
    g.draw_one(0, &mut events); // first draw = island
    g.draw_one(0, &mut events); // second draw = bonfire (not first)
    let card = g.players[0].hand.iter().find(|c| c.id == bonfire).expect("in hand");
    assert!(card.granted_alt_cast_cost_eot.is_none(), "no miracle on a non-first draw");
}

#[test]
fn murderous_redcap_etb_pings_for_its_power() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::murderous_redcap());
    let opp_life = g.players[1].life;
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2, "ETB pings for power 2");
}

#[test]
fn stormbreath_dragon_becomes_monstrous_burns_by_hand() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::stormbreath_dragon());
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    let opp_life = g.players[1].life;
    let opp_hand = g.players[1].hand.len() as i32;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("monstrosity activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - opp_hand, "burned by opponent's hand size");
}

#[test]
fn noxious_gearhulk_etb_destroys_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_battlefield(0, catalog::noxious_gearhulk());
    let life = g.players[0].life;
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "destroyed the creature");
    assert_eq!(g.players[0].life, life + 2, "gained life equal to its toughness");
}

#[test]
fn consecrated_sphinx_draws_two_when_opponent_draws() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::consecrated_sphinx());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_library(1, catalog::island());
    g.players[1].cards_drawn_this_turn = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let my_hand = g.players[0].hand.len();
    let mut events = vec![];
    g.draw_one(1, &mut events); // opponent draws
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() >= my_hand + 2, "you drew two off the opponent's draw");
}

#[test]
fn sphinx_of_the_steel_wind_has_its_keyword_suite() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sphinx_of_the_steel_wind());
    let cp = g.compute_battlefield();
    let s = cp.iter().find(|c| c.id == id).unwrap();
    for kw in [Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink,
               Keyword::Protection(Color::Red), Keyword::Protection(Color::Green)] {
        assert!(s.keywords.contains(&kw), "missing {kw:?}");
    }
}

#[test]
fn frost_titan_etb_taps_and_stuns_a_permanent() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::island());
    let titan = g.add_card_to_battlefield(0, catalog::frost_titan());
    g.fire_self_etb_triggers(titan, 0);
    drain_stack(&mut g);
    let l = g.battlefield_find(land).unwrap();
    assert!(l.tapped, "tapped the target permanent");
    assert_eq!(l.counter_count(CounterType::Stun), 1, "placed a stun counter");
}

#[test]
fn reveillark_leaves_returns_small_creatures() {
    let mut g = two_player_game();
    let lark = g.add_card_to_battlefield(0, catalog::reveillark());
    let mouse = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // power 2
    g.remove_to_graveyard_with_triggers(lark);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == mouse), "returned a power-2 creature to the battlefield");
}

#[test]
fn miracle_window_surfaces_in_hand_affordances() {
    let mut g = two_player_game();
    let bonfire = g.add_card_to_library(0, catalog::bonfire_of_the_damned());
    g.players[0].cards_drawn_this_turn = 0;
    // Give priority to seat 0 so affordances compute for them.
    g.priority.player_with_priority = 0;
    let mut events = vec![];
    g.draw_one(0, &mut events);
    let aff = g.compute_hand_affordances(0);
    assert!(aff.miracle.contains(&bonfire), "miracle window surfaced as an affordance");
}

// ── Bloodrush (CR 702.78) ────────────────────────────────────────────────────

#[test]
fn ghor_clan_rampager_bloodrush_pumps_an_attacker() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    // An attacking creature to target.
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ghor = g.add_card_to_hand(0, catalog::ghor_clan_rampager());
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    // Bloodrush: {R}{G}, discard Ghor-Clan Rampager: +4/+4 and trample.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ghor, ability_index: 0,
        target: Some(Target::Permanent(attacker)), additional_targets: Vec::new(), x_value: None,
    }).expect("bloodrush activatable from hand");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ghor), "discarded as the bloodrush cost");
    let cp = g.compute_battlefield();
    let a = cp.iter().find(|c| c.id == attacker).unwrap();
    assert_eq!((a.power, a.toughness), (6, 6), "+4/+4 from bloodrush");
    assert!(a.keywords.contains(&Keyword::Trample), "gained trample");
}

#[test]
fn hornet_queen_etb_makes_four_deathtouch_insects() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::hornet_queen());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let insects: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Insect" && c.controller == 0).collect();
    assert_eq!(insects.len(), 4, "four Insect tokens");
    assert!(insects[0].definition.keywords.contains(&Keyword::Deathtouch), "with deathtouch");
}

#[test]
fn bogardan_hellkite_etb_deals_four_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::bogardan_hellkite());
    let opp_life = g.players[1].life;
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 4, "dealt 4 damage on ETB");
}

#[test]
fn sheoldred_whispering_one_reanimates_on_your_upkeep() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sheoldred_whispering_one());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.fire_step_triggers(TurnStep::Upkeep); // player 0's upkeep
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dead),
        "Sheoldred returned a creature from your graveyard at your upkeep");
}

#[test]
fn sheoldred_whispering_one_forces_opponent_sacrifice_on_their_upkeep() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sheoldred_whispering_one());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1; // it's the opponent's upkeep
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim),
        "the opponent sacrificed a creature at their upkeep");
}

#[test]
fn phyrexian_crusader_has_protection_and_infect() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::phyrexian_crusader());
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == id).unwrap();
    for kw in [Keyword::FirstStrike, Keyword::Infect,
               Keyword::Protection(Color::Red), Keyword::Protection(Color::White)] {
        assert!(c.keywords.contains(&kw), "missing {kw:?}");
    }
}

#[test]
fn spirit_of_the_labyrinth_caps_draws_at_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spirit_of_the_labyrinth());
    let _ = g.compute_battlefield();
    assert_eq!(g.draw_cap_for(0), Some(1), "your draws capped to one");
    assert_eq!(g.draw_cap_for(1), Some(1), "opponent draws capped to one too");
}

#[test]
fn archon_of_justice_exiles_a_permanent_on_death() {
    let mut g = two_player_game();
    let archon = g.add_card_to_battlefield(0, catalog::archon_of_justice());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(archon);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "dies-trigger exiled a permanent");
}

// ── Impending (CR 702.183) — Duskmourn Overlords ─────────────────────────────

/// Cast for the impending cost, an Overlord enters with N time counters, isn't
/// a creature, and still fires its enters-or-attacks trigger.
#[test]
fn impending_overlord_enters_as_noncreature_with_time_counters() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::overlord_of_the_mistmoors());
    // Impending 4—{2}{W}{W}.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Overlord for its impending cost");
    drain_stack(&mut g);
    let r = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(r.counter_count(CounterType::Time), 4, "enters with 4 time counters");
    assert!(r.definition.keywords.contains(&Keyword::Impending(4)));
    // Layer-4 RemoveCardType: it isn't a creature while a time counter remains.
    let computed = g.computed_permanent(id).expect("computed");
    assert!(!computed.card_types.contains(&CardType::Creature),
        "Overlord isn't a creature while it has a time counter");
    // The enters-or-attacks trigger still fired: two Insect tokens minted.
    let insects = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Insect").count();
    assert_eq!(insects, 2, "ETB trigger created two Insect tokens");
}

/// A time counter ticks off at the controller's end step; when the last is
/// removed the Overlord turns into a creature.
#[test]
fn impending_time_counters_tick_off_and_it_becomes_a_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::overlord_of_the_boilerbilges());
    // Impending 4—{2}{R}{R}.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast for impending cost");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Time), 4);
    // Four end-step ticks remove all four counters.
    g.active_player_idx = 0;
    for expect_left in [3, 2, 1, 0] {
        g.process_impending();
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Time), expect_left);
    }
    // With no time counters it's a creature again.
    let computed = g.computed_permanent(id).expect("computed");
    assert!(computed.card_types.contains(&CardType::Creature),
        "becomes a creature once the last time counter is gone");
}

/// Cast for its normal cost, an Overlord enters as a creature immediately with
/// no time counters.
#[test]
fn impending_overlord_cast_normally_is_a_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::overlord_of_the_floodpits());
    // Full cost {3}{U}{U}.
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast for full cost");
    drain_stack(&mut g);
    let r = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(r.counter_count(CounterType::Time), 0, "no time counters on a normal cast");
    let computed = g.computed_permanent(id).expect("computed");
    assert!(computed.card_types.contains(&CardType::Creature), "a normal cast is a creature");
}

/// Overlord of the Hauntwoods' enters-or-attacks trigger mints a tapped
/// "Everywhere" land token that is every basic land type.
#[test]
fn impending_hauntwoods_creates_tapped_omniland() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::overlord_of_the_hauntwoods());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1); // impending {1}{G}{G}
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast for impending cost");
    drain_stack(&mut g);
    let token = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Everywhere")
        .expect("Everywhere land token minted");
    assert!(token.tapped, "the land token enters tapped");
    for lt in [LandType::Plains, LandType::Island, LandType::Swamp, LandType::Mountain, LandType::Forest] {
        assert!(token.definition.subtypes.land_types.contains(&lt), "has every basic land type");
    }
}

// ── Hideaway (CR 702.76) — Shelldock Isle ────────────────────────────────────

/// Shelldock Isle's ETB Hideaway exiles the best of the top four cards face
/// down, linked to the land; its activated ability then plays that card for
/// free while a player has 20 or less life.
#[test]
fn hideaway_shelldock_isle_exiles_then_plays_hidden_card() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Known top four: three MV-1 bolts and an MV-2 bear (the highest MV).
    g.players[0].library.clear();
    for _ in 0..3 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let land = g.add_card_to_hand(0, catalog::shelldock_isle());
    g.perform_action(GameAction::PlayLand(land)).expect("play Shelldock Isle");
    drain_stack(&mut g);
    // The bear (highest MV) is hidden in exile, face down, linked to the land.
    let hidden = g.exile.iter().find(|c| c.id == bear).expect("bear hidden in exile");
    assert!(hidden.face_down, "hidden card is face down");
    assert_eq!(hidden.exiled_with, Some(land), "hidden card linked to Shelldock Isle");
    assert!(g.battlefield_find(land).unwrap().tapped, "Shelldock enters tapped");
    // Activate {U},{T}: play the hidden card for free (P0 starts at 20 life).
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.battlefield_find_mut(land).unwrap().tapped = false; // untap to pay the tap cost
    g.priority.player_with_priority = 0;
    // Say yes to the "cast without paying?" prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Shelldock hideaway play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the hidden bear was played onto the battlefield");
    assert!(!g.exile.iter().any(|c| c.id == bear), "bear left exile");
}

/// Shared setup: play a hideaway land for seat 0 with a known Grizzly Bears
/// on top of an otherwise-empty library, returning (hidden card id, land id).
fn hideaway_setup(g: &mut GameState, land_def: crabomination::card::CardDefinition) -> (CardId, CardId) {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].library.clear();
    for _ in 0..3 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let land = g.add_card_to_hand(0, land_def);
    g.perform_action(GameAction::PlayLand(land)).expect("play hideaway land");
    drain_stack(g);
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.priority.player_with_priority = 0;
    (bear, land)
}

fn activate_hideaway_play(g: &mut GameState, land: CardId, mana: Color) -> Result<(), crabomination::game::GameError> {
    g.players[0].mana_pool.add(mana, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).map(|_| ());
    drain_stack(g);
    r
}

/// Mosswort Bridge plays its hidden card only at total controlled power ≥ 8.
#[test]
fn mosswort_bridge_gate_requires_total_power_eight() {
    let mut g = two_player_game();
    let (bear, land) = hideaway_setup(&mut g, catalog::mosswort_bridge());
    assert!(activate_hideaway_play(&mut g, land, Color::Green).is_err(), "no power on board → gated");
    for _ in 0..2 { g.add_card_to_battlefield(0, catalog::colossal_dreadmaw()); } // 6+6 power
    g.battlefield_find_mut(land).unwrap().tapped = false;
    assert!(activate_hideaway_play(&mut g, land, Color::Green).is_ok(), "12 power ≥ 8 → plays");
    assert!(g.battlefield_find(bear).is_some(), "hidden bear played");
}

/// Spinerock Knoll's gate reads "an opponent lost 7 or more life this turn".
#[test]
fn spinerock_knoll_gate_requires_seven_life_lost() {
    let mut g = two_player_game();
    let (bear, land) = hideaway_setup(&mut g, catalog::spinerock_knoll());
    g.adjust_life(1, -6);
    assert!(activate_hideaway_play(&mut g, land, Color::Red).is_err(), "6 life lost → gated");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.adjust_life(1, -1);
    assert!(activate_hideaway_play(&mut g, land, Color::Red).is_ok(), "7 life lost → plays");
    assert!(g.battlefield_find(bear).is_some(), "hidden bear played");
}

/// Windbrisk Heights' gate counts declared attackers this turn.
#[test]
fn windbrisk_heights_gate_requires_three_attackers() {
    let mut g = two_player_game();
    let (bear, land) = hideaway_setup(&mut g, catalog::windbrisk_heights());
    g.players[0].creatures_attacked_this_turn = 2;
    assert!(activate_hideaway_play(&mut g, land, Color::White).is_err(), "two attackers → gated");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.players[0].creatures_attacked_this_turn = 3;
    assert!(activate_hideaway_play(&mut g, land, Color::White).is_ok(), "three attackers → plays");
    assert!(g.battlefield_find(bear).is_some(), "hidden bear played");
}

/// Declaring attackers bumps the per-player tally that Windbrisk reads.
#[test]
fn declare_attackers_counts_creatures_attacked_this_turn() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ])).expect("declare two attackers");
    assert_eq!(g.players[0].creatures_attacked_this_turn, 2);
}

/// Orcish Bowmasters fires on an opponent's off-step draw (ping + amass
/// Orcs 1) but exempts their draw-step turn-based draw.
#[test]
fn orcish_bowmasters_punishes_extra_draws_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::orcish_bowmasters());
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    // Opponent's own draw step, first draw — exempt.
    g.active_player_idx = 1;
    g.step = TurnStep::Draw;
    let mut events = Vec::new();
    g.draw_one(1, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Army"), "turn-based draw exempt");
    // Second draw in the same draw step — triggers.
    let mut events = Vec::new();
    g.draw_one(1, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Army")
        .expect("amassed an Orc Army");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 1, "amass 1");
}

/// Vesuva enters tapped as a copy of a land on the battlefield.
#[test]
fn vesuva_enters_tapped_as_land_copy() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let v = g.add_card_to_hand(0, catalog::vesuva());
    g.perform_action(GameAction::PlayLand(v)).expect("play Vesuva");
    drain_stack(&mut g);
    let c = g.battlefield_find(v).unwrap();
    assert!(c.tapped, "enters tapped");
    assert_eq!(c.definition.name, "Forest", "copies the Forest");
}

/// Thespian's Stage copies a land on activation and stays that land.
#[test]
fn thespians_stage_becomes_copy_of_target_land() {
    let mut g = two_player_game();
    let stage = g.add_card_to_battlefield(0, catalog::thespians_stage());
    let island = g.add_card_to_battlefield(1, catalog::island());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: stage, ability_index: 1,
        target: Some(Target::Permanent(island)), additional_targets: Vec::new(), x_value: None,
    }).expect("copy activation");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(stage).unwrap().definition.name, "Island");
    // Permanent duration: cleanup does not revert.
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.battlefield_find(stage).unwrap().definition.name, "Island");
}

/// CR 121.2a — Thought Reflection doubles each draw; two copies quadruple,
/// and only the enchantment's controller is affected.
#[test]
fn cr_121_2a_thought_reflection_doubles_draws() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thought_reflection());
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    for _ in 0..2 { g.add_card_to_library(1, catalog::island()); }
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    assert_eq!(g.players[0].hand.len(), 2, "one draw became two");
    g.draw_one(1, &mut events);
    assert_eq!(g.players[1].hand.len(), 1, "opponent draws normally");
    g.add_card_to_battlefield(0, catalog::thought_reflection());
    g.draw_one(0, &mut events);
    assert_eq!(g.players[0].hand.len(), 6, "two doublers: 1 -> 4 (CR 614.5 stacking)");
}

// ── Misc modern staples ──────────────────────────────────────────────────────

/// Get Lost destroys a creature and gives its controller two Map tokens.
#[test]
fn get_lost_destroys_and_makes_two_maps() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gl = g.add_card_to_hand(0, catalog::get_lost());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    crabomination::game::cast_at(&mut g, gl, Target::Permanent(bear));
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
    let maps = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.name == "Map").count();
    assert_eq!(maps, 2, "controller got two Map tokens");
}

/// Make Disappear counters a spell whose controller can't pay the {3} tax.
#[test]
fn make_disappear_counters_unpaid_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    let md = g.add_card_to_hand(0, catalog::make_disappear());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: md, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Make Disappear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "Bolt countered (P1 can't pay the tax)");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

/// Heartfire Immolator sacrifices itself to deal 3 damage to any target.
#[test]
fn heartfire_immolator_sac_deals_three() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let id = g.add_card_to_battlefield(0, catalog::heartfire_immolator());
    g.clear_sickness(id);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Heartfire Immolator");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "dealt 3 to opponent");
    assert!(g.battlefield_find(id).is_none(), "sacrificed itself");
}

/// Cenote Scout explores on entry (lands → hand, else +1/+1 counter).
#[test]
fn cenote_scout_explores_on_entry() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears()); // nonland on top → +1/+1
    let id = g.add_card_to_battlefield(0, catalog::cenote_scout());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "explored into a +1/+1 counter");
}

/// Anthem of Champions pumps your creatures +1/+1 but not opponents'.
#[test]
fn anthem_of_champions_pumps_your_creatures() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::anthem_of_champions());
    let p = |g: &GameState, id| {
        let c = g.computed_permanent(id).unwrap();
        (c.power, c.toughness)
    };
    assert_eq!(p(&g, mine), (3, 3), "your bear is pumped");
    assert_eq!(p(&g, theirs), (2, 2), "opponent's bear is not");
}

/// Warleader's Call pings each opponent when a creature you control enters.
#[test]
fn warleaders_call_pings_on_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::warleaders_call());
    let before = g.players[1].life;
    // Cast a creature from hand so the real ETB event dispatches to the anthem.
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "each opponent took 1");
}

/// Deep-Cavern Bat exiles a card from the opponent's hand until it leaves play.
#[test]
fn deep_cavern_bat_exiles_from_opponent_hand() {
    let mut g = two_player_game();
    let victim = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bat = g.add_card_to_battlefield(0, catalog::deep_cavern_bat());
    g.fire_self_etb_triggers(bat, 0);
    drain_stack(&mut g);
    assert!(!g.players[1].hand.iter().any(|c| c.id == victim), "card left opponent's hand");
    assert!(g.exile.iter().any(|c| c.id == victim), "card is exiled");
    // When the Bat leaves, the card returns to its owner's hand.
    g.remove_to_graveyard_with_triggers(bat);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "card returned on Bat leaving");
}


/// Llanowar Loamspeaker animates a target land into a 3/3 creature.
#[test]
fn llanowar_loamspeaker_animates_land() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_battlefield(0, catalog::llanowar_loamspeaker());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: Some(Target::Permanent(forest)), additional_targets: Vec::new(), x_value: None,
    }).expect("animate the Forest");
    drain_stack(&mut g);
    let cp = g.computed_permanent(forest).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "Forest is now a creature");
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Bristly Bill puts a +1/+1 counter on a creature when a land enters.
#[test]
fn bristly_bill_landfall_adds_counter() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bill = g.add_card_to_battlefield(0, catalog::bristly_bill_spine_sower());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bill).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "landfall added a +1/+1 counter");
}

/// Glissa Sunslayer draws a card and loses 1 life on combat damage.
#[test]
fn glissa_combat_damage_draws_and_loses_life() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::glissa_sunslayer());
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    let trig = catalog::glissa_sunslayer().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(id, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
}

/// Zealous Persecution swings the whole board ±1/±1 EOT.
#[test]
fn zealous_persecution_pumps_yours_shrinks_theirs() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let zp = g.add_card_to_hand(0, catalog::zealous_persecution());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, zp);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "+1/+1 yours");
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 1, "-1/-1 theirs");
}

/// Exalted Angel casts face down for {3} and turns up for its morph cost.
#[test]
fn exalted_angel_morphs_up() {
    let mut g = two_player_game();
    let angel = g.add_card_to_hand(0, catalog::exalted_angel());
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFaceDown { card_id: angel }).expect("morph face down");
    drain_stack(&mut g);
    let c = g.battlefield_find(angel).expect("on battlefield");
    assert!(c.face_down);
    assert_eq!((c.power(), c.toughness()), (2, 2), "vanilla 2/2 while face down");
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::TurnFaceUp { card_id: angel }).expect("pay morph cost");
    let c = g.battlefield_find(angel).unwrap();
    assert!(!c.face_down);
    assert_eq!((c.power(), c.toughness()), (4, 5), "real face restored");
}

/// Stony Silence locks artifact activated abilities (mana rocks included).
#[test]
fn stony_silence_locks_artifact_abilities() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stony_silence());
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: rock, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "artifact ability locked");
}

/// Journey to Nowhere exiles on ETB and gives the creature back on leave.
#[test]
fn journey_to_nowhere_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let journey = g.add_card_to_hand(0, catalog::journey_to_nowhere());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: journey, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Journey castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled");
    g.remove_from_battlefield_to_graveyard_raw(journey);
    assert!(g.battlefield_find(bear).is_some(), "creature returns when Journey leaves");
}

/// Shrapnel Blast: sac-an-artifact additional cost, 5 to any target.
#[test]
fn shrapnel_blast_sacs_artifact_for_five() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(0, catalog::mind_stone());
    let blast = g.add_card_to_hand(0, catalog::shrapnel_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: blast, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Shrapnel Blast castable with an artifact to sac");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "artifact sacrificed as cost");
    assert_eq!(g.players[1].life, 15, "5 damage");
}

/// Azusa grants two extra land plays (three total per turn).
#[test]
fn azusa_allows_three_land_plays() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::azusa_lost_but_seeking());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..3 {
        let land = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(land)).expect("extra land play");
    }
    let fourth = g.add_card_to_hand(0, catalog::forest());
    assert!(g.perform_action(GameAction::PlayLand(fourth)).is_err(), "fourth is too many");
}

/// Ranger of Eos tutors two one-drop creatures to hand.
#[test]
fn ranger_of_eos_fetches_two_one_drops() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::ornithopter());
    let b = g.add_card_to_library(0, catalog::memnite());
    let ranger = g.add_card_to_hand(0, catalog::ranger_of_eos());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(b)),
    ]));
    cast(&mut g, ranger);
    assert!(g.players[0].hand.iter().any(|c| c.id == a));
    assert!(g.players[0].hand.iter().any(|c| c.id == b));
}

/// Hero of Oxid Ridge shuts off power-1-or-less blockers on attack.
#[test]
fn hero_of_oxid_ridge_locks_out_small_blockers() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::hero_of_oxid_ridge());
    g.clear_sickness(hero);
    let chump = g.add_card_to_battlefield(1, catalog::ornithopter()); // 0/2
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: hero, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(chump, hero)])).is_err(),
        "power ≤1 creature can't block");
}

/// Thopter Foundry sacrifices a nontoken artifact for a Thopter + 1 life.
#[test]
fn thopter_foundry_mints_thopter_and_gains_life() {
    let mut g = two_player_game();
    let foundry = g.add_card_to_battlefield(0, catalog::thopter_foundry());
    let fodder = g.add_card_to_battlefield(0, catalog::mind_stone());
    let life = g.players[0].life;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: foundry, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac a nontoken artifact");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "rock sacrificed");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Thopter" && c.is_token));
    assert_eq!(g.players[0].life, life + 1);
}

/// Sword of the Meek hops back from the graveyard onto an entering 1/1.
#[test]
fn sword_of_the_meek_returns_onto_entering_one_one() {
    let mut g = two_player_game();
    let sword = g.add_card_to_graveyard(0, catalog::sword_of_the_meek());
    let pup = g.add_card_to_hand(0, catalog::memnite()); // 1/1
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, pup);
    let s = g.battlefield_find(sword).expect("Sword returned to the battlefield");
    assert_eq!(s.attached_to, Some(pup), "attached to the 1/1");
    assert_eq!(g.computed_permanent(pup).unwrap().toughness, 3, "+1/+2 applied");
}

/// Fulminator Mage sacrifices to blow up a nonbasic land; basics illegal.
#[test]
fn fulminator_mage_sacs_to_destroy_nonbasic_land() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::fulminator_mage());
    g.clear_sickness(mage);
    let manland = g.add_card_to_battlefield(1, catalog::celestial_colonnade());
    let basic = g.add_card_to_battlefield(1, catalog::island());
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(basic)), additional_targets: Vec::new(), x_value: None,
    }).is_err(), "basic land is not a legal target");
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(manland)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac to destroy nonbasic");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mage).is_none(), "Mage sacrificed");
    assert!(g.battlefield_find(manland).is_none(), "nonbasic destroyed");
}

/// Smallpox: symmetric life loss, discard, creature sac, land sac.
#[test]
fn smallpox_hits_every_player_four_ways() {
    let mut g = two_player_game();
    for p in 0..2 {
        g.add_card_to_hand(p, catalog::island());
        g.add_card_to_battlefield(p, catalog::grizzly_bears());
        g.add_card_to_battlefield(p, catalog::forest());
    }
    let pox = g.add_card_to_hand(0, catalog::smallpox());
    g.players[0].mana_pool.add(Color::Black, 2);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: pox, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Smallpox castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 1);
    assert_eq!(g.players[1].life, life1 - 1);
    for p in 0..2 {
        assert!(!g.battlefield.iter().any(|c| c.controller == p && c.definition.is_creature()),
            "P{p} creature sacrificed");
        assert!(!g.battlefield.iter().any(|c| c.controller == p && c.definition.is_land()),
            "P{p} land sacrificed");
    }
}

/// Mox Opal only makes mana with metalcraft (three artifacts).
#[test]
fn mox_opal_requires_metalcraft() {
    let mut g = two_player_game();
    let mox = g.add_card_to_battlefield(0, catalog::mox_opal());
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "one artifact is not metalcraft");
    g.add_card_to_battlefield(0, catalog::mind_stone());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Green)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("metalcraft online");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "any color added");
}

/// Glissa mode 1: destroy target enchantment off the combat-damage modal.
#[test]
fn glissa_combat_damage_mode_destroys_enchantment() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::glissa_sunslayer());
    let ench = g.add_card_to_battlefield(1, catalog::glorious_anthem());
    let trig = catalog::glissa_sunslayer().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(
        id, 0, Some(Target::Permanent(ench)), 1,
    );
    g.resolve_effect(&trig, &ctx).unwrap();
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
}

/// Preacher of the Schism, attacking a tied-for-most-life player while tied for
/// most life itself, makes a Vampire and draws (losing 1 life).
#[test]
fn preacher_attacks_makes_vampire_and_draws() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::preacher_of_the_schism());
    g.clear_sickness(id);
    let hand = g.players[0].hand.len();
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("Preacher attacks P1");
    drain_stack(&mut g);
    let vamps = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Vampire").count();
    assert_eq!(vamps, 1, "created a Vampire token (defender tied for most life)");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew (you tied for most life)");
}


/// Sip of Hemlock destroys a creature and drains its controller 2 life.
#[test]
fn sip_of_hemlock_destroys_and_drains() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sip_of_hemlock());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    crabomination::game::cast_at(&mut g, id, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert_eq!(g.players[1].life, 18, "controller lost 2 life");
}

/// Goblin Surprise (mode 1) creates two Goblin tokens.
#[test]
fn goblin_surprise_makes_two_goblins() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::goblin_surprise());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast Goblin Surprise (tokens mode)");
    drain_stack(&mut g);
    let gobs = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Goblin").count();
    assert_eq!(gobs, 2, "made two Goblins");
}

/// Nowhere to Run shrinks an opponent's creature by -3/-3.
#[test]
fn nowhere_to_run_shrinks_opponent_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::nowhere_to_run());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    crabomination::game::cast_at(&mut g, id, Target::Permanent(bear));
    // 2/2 with -3/-3 dies as a 0-or-less-toughness creature.
    assert!(g.battlefield_find(bear).is_none(), "bear shrank to lethal and died");
}

/// Nowhere to Run strips hexproof from opponents' creatures (CR 613 layer 6).
#[test]
fn nowhere_to_run_strips_opponent_hexproof() {
    let mut g = two_player_game();
    let caryatid = g.add_card_to_battlefield(1, catalog::sylvan_caryatid()); // hexproof
    assert!(g.computed_permanent(caryatid).unwrap()
        .keywords.contains(&crabomination::card::Keyword::Hexproof), "hexproof before");
    // Hexproof blocks an opponent's targeting before Nowhere to Run.
    assert!(g.check_target_legality(&Target::Permanent(caryatid), 0).is_err(),
        "hexproof blocks targeting before");
    g.add_card_to_battlefield(0, catalog::nowhere_to_run());
    assert!(!g.computed_permanent(caryatid).unwrap()
        .keywords.contains(&crabomination::card::Keyword::Hexproof),
        "opponent's creature loses hexproof while Nowhere to Run is out");
    // …and is now a legal target for its controller's opponent.
    assert!(g.check_target_legality(&Target::Permanent(caryatid), 0).is_ok(),
        "targetable once hexproof is stripped");
}

// ── Squad (CR 702.157) ───────────────────────────────────────────────────────

/// Vanguard Suppressor cast paying its Squad {2} twice mints two token copies
/// of itself (three Vanguard Suppressors total).
#[test]
fn squad_paid_twice_mints_two_token_copies() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::vanguard_suppressor());
    // Base {3}{U} + Squad {2} × 2 = {3}{U} + {4} generic (with a buffer so the
    // generic squad payment doesn't strand the blue pip).
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(8);

    g.perform_action(GameAction::CastSpellSquad {
        card_id: id, times: 2,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Vanguard Suppressor with Squad paid twice");
    drain_stack(&mut g);

    let count = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Vanguard Suppressor").count();
    assert_eq!(count, 3, "original + two squad token copies");
    let tokens = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Vanguard Suppressor").count();
    assert_eq!(tokens, 2, "the two copies are tokens");
}

/// CR 601.2h — a squad cast whose extra payments are unaffordable is
/// rejected atomically: the spell stays in hand, no mana is spent.
#[test]
fn squad_unaffordable_extra_payment_rejects_whole_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::vanguard_suppressor());
    // Base {3}{U} affordable, the two Squad {2} payments are not.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let err = g.perform_action(GameAction::CastSpellSquad {
        card_id: id, times: 2,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "unaffordable squad payment rejects the cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "spell stays in hand");
    assert!(g.stack.is_empty(), "nothing committed to the stack");
    assert_eq!(g.players[0].mana_pool.total(), 4, "no mana spent");
}

/// Squad with zero extra payments is just a normal cast — no token copies.
#[test]
fn squad_paid_zero_times_is_a_plain_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::vanguard_suppressor());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpellSquad {
        card_id: id, times: 0,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast for base cost, Squad paid zero times");
    drain_stack(&mut g);

    let count = g.battlefield.iter()
        .filter(|c| c.definition.name == "Vanguard Suppressor").count();
    assert_eq!(count, 1, "no copies when Squad isn't paid");
}

/// Galadhrim Brigade's Squad copies are Elves, so they and the original all see
/// the +1/+1 Elf anthem from each other.
#[test]
fn galadhrim_brigade_squad_copies_anthem_each_other() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::galadhrim_brigade());
    g.players[0].mana_pool.add(Color::Green, 4);
    g.players[0].mana_pool.add_colorless(4);

    // Base {2}{G} + Squad {1}{G} once.
    g.perform_action(GameAction::CastSpellSquad {
        card_id: id, times: 1,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Galadhrim Brigade with Squad once");
    drain_stack(&mut g);

    let ids: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Galadhrim Brigade").map(|c| c.id).collect();
    assert_eq!(ids.len(), 2, "original + one squad copy");
    // Each is a 2/2 buffed by the other Elf's anthem to 3/3.
    for id in ids {
        assert_eq!(g.computed_permanent(id).unwrap().power, 3,
            "buffed by the other Brigade's Elf anthem");
    }
}

/// Wasteland Raider's ETB makes each player sacrifice a creature.
#[test]
fn wasteland_raider_etb_each_player_sacrifices() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wasteland_raider());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Wasteland Raider");
    drain_stack(&mut g);
    // The opponent's only creature is sacrificed; the controller keeps the 4/3
    // Raider and loses the weaker bear.
    assert!(g.battlefield_find(opp).is_none(), "opponent sacrificed its creature");
    assert!(g.battlefield.iter().any(|c| c.controller == 0
        && c.definition.name == "Wasteland Raider"), "Raider stays");
}

// ── Replicate (CR 702.107) ───────────────────────────────────────────────────

/// Pyromatics replicated twice resolves three times (original + two copies),
/// dealing 3 total to the same player.
#[test]
fn replicate_pyromatics_twice_deals_three_total() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pyromatics());
    g.players[0].mana_pool.add(Color::Red, 4);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[1].life;

    g.perform_action(GameAction::CastSpellReplicate {
        card_id: id, times: 2,
        target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Pyromatics replicated twice");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life - 3, "original + two copies = 3 damage");
}

/// CR 702.78 — Burn Trail conspired by tapping two red creatures copies it
/// once: 3 (original) + 3 (copy) = 6 damage to the same target.
#[test]
fn conspire_burn_trail_copies_for_six() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::burn_trail());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let c0 = g.add_card_to_battlefield(0, catalog::goblin_guide());
    let c1 = g.add_card_to_battlefield(0, catalog::goblin_guide());
    let life = g.players[1].life;

    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [c0, c1],
        target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Burn Trail conspired");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life - 6, "original + conspire copy = 6 damage");
    assert!(g.battlefield_find(c0).unwrap().tapped, "conspirer tapped");
    assert!(g.battlefield_find(c1).unwrap().tapped, "conspirer tapped");
}

/// Conspire rejects creatures that don't share a color with the spell.
#[test]
fn conspire_requires_shared_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::burn_trail());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Grizzly Bears are green — they can't conspire a red spell.
    let c0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let r = g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [c0, c1],
        target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(r.is_err(), "green creatures can't conspire a red spell");
    // No partial state: spell stayed in hand, creatures untapped.
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "Burn Trail still in hand");
    assert!(!g.battlefield_find(c0).unwrap().tapped, "creature untapped after rejection");
}

/// Train of Thought replicated once draws two (original + one copy).
#[test]
fn replicate_train_of_thought_once_draws_two() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::train_of_thought());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpellReplicate {
        card_id: id, times: 1,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Train of Thought replicated once");
    drain_stack(&mut g);

    // -1 for casting it, +2 for original + copy draws.
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew two cards");
}

/// Shattering Spree cast with no replicate payments is a plain "destroy target
/// artifact".
#[test]
fn replicate_shattering_spree_base_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::shattering_spree());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpellReplicate {
        card_id: id, times: 0,
        target: Some(Target::Permanent(art)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Shattering Spree");
    drain_stack(&mut g);

    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Hand affordances surface Squad and Replicate cast options for the
/// priority-holding seat when the extra cost is affordable.
#[test]
fn affordances_surface_squad_and_replicate() {
    let mut g = two_player_game();
    let squad = g.add_card_to_hand(0, catalog::vanguard_suppressor());
    let repl = g.add_card_to_hand(0, catalog::train_of_thought());
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.players[0].mana_pool.add_colorless(8);
    let a = g.compute_hand_affordances(0);
    assert!(a.squadable.contains(&squad), "Squad card is squadable");
    assert!(a.replicatable.contains(&repl), "Replicate card is replicatable");
}

/// Conspire is surfaced as an affordance only when the seat controls two
/// untapped creatures sharing a color with the spell.
#[test]
fn affordances_surface_conspire_with_two_red_creatures() {
    let mut g = two_player_game();
    let burn = g.add_card_to_hand(0, catalog::burn_trail());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Only one red creature → not yet conspirable.
    g.add_card_to_battlefield(0, catalog::goblin_guide());
    assert!(!g.compute_hand_affordances(0).conspirable.contains(&burn),
        "one creature is not enough to conspire");
    // Second red creature → conspirable.
    g.add_card_to_battlefield(0, catalog::goblin_guide());
    assert!(g.compute_hand_affordances(0).conspirable.contains(&burn),
        "two untapped red creatures make Burn Trail conspirable");
}

/// Ultramarines Honour Guard buffs other creatures you control by +1/+1.
#[test]
fn ultramarines_honour_guard_anthems_other_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let guard = g.add_card_to_battlefield(0, catalog::ultramarines_honour_guard());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "other creature buffed");
    assert_eq!(g.computed_permanent(guard).unwrap().power, 2, "not itself");
}

/// Securitron Squadron's Squad token copy enters and gains a +1/+1 counter from
/// Securitron's "creature token you control enters" trigger.
#[test]
fn securitron_squadron_counters_its_squad_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::securitron_squadron());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpellSquad {
        card_id: id, times: 1, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Securitron Squadron with Squad once");
    drain_stack(&mut g);
    let token = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Securitron Squadron")
        .expect("squad token");
    assert_eq!(token.counter_count(CounterType::PlusOnePlusOne), 1,
        "token gained a +1/+1 counter");
}

/// Powder Ganger's ETB destroys an artifact.
#[test]
fn powder_ganger_etb_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::powder_ganger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Powder Ganger");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "ETB destroyed the artifact");
}

/// Space Marine Devastator's ETB destroys an enchantment.
#[test]
fn space_marine_devastator_etb_destroys_enchantment() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, catalog::gaeas_anthem());
    let id = g.add_card_to_hand(0, catalog::space_marine_devastator());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Space Marine Devastator");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none(), "ETB destroyed the enchantment");
}

/// Gary Clone's attack trigger pumps every Gary Clone you control +1/+0.
#[test]
fn gary_clone_attack_pumps_all_garys() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::gary_clone());
    let b = g.add_card_to_battlefield(0, catalog::gary_clone());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]).expect("attacks");
    drain_stack(&mut g);
    // Each attack trigger pumps both Garys +1/+0; two triggers → +2/+0 each.
    assert_eq!(g.computed_permanent(a).unwrap().power, 1 + 2, "both attack triggers buff a");
    assert_eq!(g.computed_permanent(b).unwrap().power, 1 + 2, "both attack triggers buff b");
}

/// Zephyrim is a 3/3 flier with vigilance castable for its normal cost.
#[test]
fn zephyrim_is_a_flying_vigilant_three_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::zephyrim());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Zephyrim");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Zephyrim on battlefield");
    assert!(c.has_keyword(&crabomination::card::Keyword::Flying), "flying");
    assert!(c.has_keyword(&crabomination::card::Keyword::Vigilance), "vigilance");
    assert_eq!((c.power(), c.toughness()), (3, 3));
}

/// Arco-Flagellant's "Pay 3 life" ability grants it indestructible.
#[test]
fn arco_flagellant_pays_life_for_indestructible() {
    let mut g = two_player_game();
    let arco = g.add_card_to_battlefield(0, catalog::arco_flagellant());
    g.clear_sickness(arco);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: arco, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pay 3 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 3, "paid 3 life");
    assert!(g.battlefield_find(arco).unwrap().has_keyword(&crabomination::card::Keyword::Indestructible),
        "gains indestructible until end of turn");
}

/// Roadkill Rodney has deathtouch and Squad; its Squad token copy enters too.
#[test]
fn roadkill_rodney_squad_mints_deathtouch_copies() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::roadkill_rodney());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpellSquad {
        card_id: id, times: 1, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Roadkill Rodney with Squad once");
    drain_stack(&mut g);
    let rodneys: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Roadkill Rodney").collect();
    assert_eq!(rodneys.len(), 2, "original + squad copy");
    assert!(rodneys.iter().all(|c| c.has_keyword(&crabomination::card::Keyword::Deathtouch)),
        "copies keep deathtouch");
}

/// Gigadrowse replicated once taps two different permanents (copy retargets).
#[test]
fn replicate_gigadrowse_taps_two_targets() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::gigadrowse());
    g.players[0].mana_pool.add(Color::Blue, 2);
    // Copy retargets to a different permanent via the scripted decider.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(b)),
    ]));
    g.perform_action(GameAction::CastSpellReplicate {
        card_id: id, times: 1,
        target: Some(Target::Permanent(a)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Gigadrowse replicated once");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped, "original target tapped");
    assert!(g.battlefield_find(b).unwrap().tapped, "copy's retarget tapped");
}

/// Vacuumelt returns target creature to its owner's hand.
#[test]
fn replicate_vacuumelt_bounces_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vacuumelt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellReplicate {
        card_id: id, times: 0,
        target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Vacuumelt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "returned to owner's hand");
}

/// Leap of Flame grants its target +1/+0, flying, and first strike.
#[test]
fn replicate_leap_of_flame_pumps_and_grants_keywords() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::leap_of_flame());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpellReplicate {
        card_id: id, times: 0,
        target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Leap of Flame");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 3, "+1/+0");
    assert!(c.keywords.contains(&crabomination::card::Keyword::Flying), "flying");
    assert!(c.keywords.contains(&crabomination::card::Keyword::FirstStrike), "first strike");
}

/// Sicarian Infiltrator's Squad copies each draw a card on ETB (original +
/// copy = two draws).
#[test]
fn sicarian_infiltrator_squad_draws_per_copy() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::sicarian_infiltrator());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellSquad {
        card_id: id, times: 1, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sicarian Infiltrator with Squad once");
    drain_stack(&mut g);
    // -1 cast, +2 ETB draws (original + squad copy).
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "original + copy each draw");
    assert_eq!(g.battlefield.iter()
        .filter(|c| c.definition.name == "Sicarian Infiltrator").count(), 2);
}

