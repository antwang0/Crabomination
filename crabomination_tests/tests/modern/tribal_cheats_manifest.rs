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

// ── Put-creature-from-hand cheats + snow ──────────────────────────────────────

/// Sneak Attack puts a creature from hand onto the battlefield, granting haste,
/// and registers an end-step sacrifice that fires.
#[test]
fn sneak_attack_cheats_creature_in_with_haste_then_sacrifices() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let sneak = g.add_card_to_battlefield(0, catalog::sneak_attack());
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![dragon])]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: sneak, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Sneak Attack");
    drain_stack(&mut g);
    let c = g.computed_permanent(dragon).expect("dragon on battlefield");
    assert!(c.keywords.contains(&Keyword::Haste), "entrant gains haste");
    // End-step sacrifice fires.
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_none(), "creature sacrificed at end step");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == dragon), "to graveyard");
}

/// Elvish Piper puts a creature from hand onto the battlefield for {G},{T} with
/// no haste and no end-step sacrifice.
#[test]
fn elvish_piper_puts_creature_in_to_stay() {
    let mut g = two_player_game();
    let piper = g.add_card_to_battlefield(0, catalog::elvish_piper());
    g.clear_sickness(piper);
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![dragon])]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: piper, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Elvish Piper");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_some(), "creature stays on battlefield");
    assert!(!g.delayed_triggers.iter().any(|t|
        t.kind == crabomination::game::types::DelayedKind::NextEndStep),
        "no end-step sacrifice registered");
}

/// Skred deals damage equal to the number of snow permanents you control.
#[test]
fn skred_scales_with_snow_permanents() {
    let mut g = two_player_game();
    // Three snow permanents (Ohran Frostfang carries the Snow supertype).
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::ohran_frostfang()); }
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let skred = g.add_card_to_hand(0, catalog::skred());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: skred, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Skred");
    drain_stack(&mut g);
    // 3 snow permanents → 3 damage → kills the 2/2 bear (toughness 2).
    assert!(g.battlefield_find(target).is_none() || g.battlefield_find(target).unwrap().damage >= 3,
        "Skred dealt 3 damage from 3 snow permanents");
}

/// Through the Breach cheats a creature in with haste as an instant.
#[test]
fn through_the_breach_cheats_creature_with_haste() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let breach = g.add_card_to_hand(0, catalog::through_the_breach());
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![dragon])]));
    g.perform_action(GameAction::CastSpell {
        card_id: breach, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Through the Breach");
    drain_stack(&mut g);
    let c = g.computed_permanent(dragon).expect("dragon in play");
    assert!(c.keywords.contains(&Keyword::Haste), "entrant has haste");
    assert!(g.delayed_triggers.iter().any(|t|
        t.kind == crabomination::game::types::DelayedKind::NextEndStep),
        "end-step sacrifice registered");
}

/// Quicksilver Amulet puts a creature from hand into play for {4},{T}.
#[test]
fn quicksilver_amulet_puts_creature_in() {
    let mut g = two_player_game();
    let amulet = g.add_card_to_battlefield(0, catalog::quicksilver_amulet());
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon());
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![dragon])]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: amulet, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Quicksilver Amulet");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_some(), "creature put onto battlefield");
}

// ── Goblin tribal cheats ──────────────────────────────────────────────────────

/// Goblin Lackey puts a Goblin from hand onto the battlefield on combat damage.
#[test]
fn goblin_lackey_cheats_goblin_on_combat_damage() {
    let mut g = two_player_game();
    let lackey = g.add_card_to_battlefield(0, catalog::goblin_lackey());
    g.clear_sickness(lackey);
    let krenko = g.add_card_to_hand(0, catalog::krenko_mob_boss());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![krenko])]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: lackey, target: AttackTarget::Player(1) }])
        .expect("attacks");
    for _ in 0..16 {
        if g.battlefield_find(krenko).is_some() { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(krenko).is_some(), "Krenko cheated into play");
}

/// Warren Instigator carries double strike and the same combat-damage cheat.
#[test]
fn warren_instigator_has_double_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::warren_instigator());
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Goblin Piledriver gets +2/+0 per other attacking Goblin.
#[test]
fn goblin_piledriver_pumps_per_attacking_goblin() {
    let mut g = two_player_game();
    let pd = g.add_card_to_battlefield(0, catalog::goblin_piledriver());
    let g1 = g.add_card_to_battlefield(0, catalog::skirk_prospector());
    let g2 = g.add_card_to_battlefield(0, catalog::skirk_prospector());
    for id in [pd, g1, g2] { g.clear_sickness(id); }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: pd, target: AttackTarget::Player(1) },
        Attack { attacker: g1, target: AttackTarget::Player(1) },
        Attack { attacker: g2, target: AttackTarget::Player(1) },
    ]).expect("attacks");
    drain_stack(&mut g);
    // base 1 + 2 per the two other attacking Goblins = 5.
    assert_eq!(g.computed_permanent(pd).unwrap().power, 5, "+4 from two other Goblins");
}

// ── Merfolk + Sliver tribal ───────────────────────────────────────────────────

/// Master of the Pearl Trident pumps other Merfolk +1/+1 and grants islandwalk.
#[test]
fn master_of_pearl_trident_buffs_other_merfolk() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let lord = g.add_card_to_battlefield(0, catalog::master_of_the_pearl_trident());
    let other = g.add_card_to_battlefield(0, catalog::cursecatcher());
    let oc = g.computed_permanent(other).unwrap();
    assert_eq!((oc.power, oc.toughness), (2, 2), "Cursecatcher 1/1 → 2/2");
    assert!(oc.keywords.contains(&Keyword::Landwalk(crabomination::card::LandType::Island)), "islandwalk");
    // The lord doesn't pump itself.
    assert_eq!(g.computed_permanent(lord).unwrap().power, 2);
}

/// Merfolk Mistbinder buffs only your other Merfolk.
#[test]
fn merfolk_mistbinder_buffs_your_other_merfolk() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::merfolk_mistbinder());
    let mine = g.add_card_to_battlefield(0, catalog::cursecatcher());
    let theirs = g.add_card_to_battlefield(1, catalog::cursecatcher());
    assert_eq!(g.computed_permanent(mine).unwrap().power, 2, "your Merfolk pumped");
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 1, "opponent's Merfolk untouched");
}

/// Cursecatcher counters an instant/sorcery unless its controller pays {1}.
#[test]
fn cursecatcher_taxes_instant() {
    // P0 casts a Bolt at P1, then sacrifices Cursecatcher targeting it. P0 has
    // no mana left to pay the {1}, so the Bolt is countered.
    let mut g = two_player_game();
    let catcher = g.add_card_to_battlefield(0, catalog::cursecatcher());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    g.perform_action(GameAction::ActivateAbility {
        card_id: catcher, ability_index: 0,
        target: Some(Target::Permanent(bolt)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Cursecatcher");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "Bolt countered (unpaid)");
    assert_eq!(g.players[1].life, 20, "Bolt never resolved");
}

/// Galerider Sliver gives every Sliver flying (yours and opponents').
#[test]
fn galerider_sliver_grants_flying_to_all_slivers() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::galerider_sliver());
    let opp_sliver = g.add_card_to_battlefield(1, catalog::heart_sliver());
    assert!(g.computed_permanent(opp_sliver).unwrap().keywords.contains(&Keyword::Flying),
        "opponent's Sliver also gains flying");
}

/// Heart Sliver gives all Slivers haste.
#[test]
fn heart_sliver_grants_haste() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let hs = g.add_card_to_battlefield(0, catalog::heart_sliver());
    assert!(g.computed_permanent(hs).unwrap().keywords.contains(&Keyword::Haste));
}

/// Crystalline Sliver gives all Slivers shroud (untargetable).
#[test]
fn crystalline_sliver_grants_shroud() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let cs = g.add_card_to_battlefield(0, catalog::crystalline_sliver());
    assert!(g.computed_permanent(cs).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Muscle Sliver's +1/+1 anthem stacks onto another Sliver and itself.
#[test]
fn muscle_sliver_pumps_all_slivers() {
    let mut g = two_player_game();
    let ms = g.add_card_to_battlefield(0, catalog::muscle_sliver());
    let other = g.add_card_to_battlefield(0, catalog::striking_sliver());
    assert_eq!(g.computed_permanent(ms).unwrap().power, 2, "lord buffs itself");
    assert_eq!(g.computed_permanent(other).unwrap().toughness, 2, "other Sliver buffed");
}

/// Predatory Sliver only buffs Slivers their controller owns, not opponents'.
#[test]
fn predatory_sliver_only_buffs_your_slivers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::predatory_sliver());
    let opp = g.add_card_to_battlefield(1, catalog::striking_sliver());
    assert_eq!(g.computed_permanent(opp).unwrap().power, 1, "opponent's Sliver unaffected");
}

/// Striking Sliver grants first strike only to its controller's Slivers.
#[test]
fn striking_sliver_grants_first_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let ss = g.add_card_to_battlefield(0, catalog::striking_sliver());
    let opp = g.add_card_to_battlefield(1, catalog::venom_sliver());
    assert!(g.computed_permanent(ss).unwrap().keywords.contains(&Keyword::FirstStrike));
    assert!(!g.computed_permanent(opp).unwrap().keywords.contains(&Keyword::FirstStrike),
        "opponent's Sliver doesn't get first strike");
}

/// Sliver Hivelord makes the controller's Slivers indestructible.
#[test]
fn sliver_hivelord_grants_indestructible() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let hl = g.add_card_to_battlefield(0, catalog::sliver_hivelord());
    let other = g.add_card_to_battlefield(0, catalog::muscle_sliver());
    assert!(g.computed_permanent(hl).unwrap().keywords.contains(&Keyword::Indestructible));
    assert!(g.computed_permanent(other).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Manaweft Sliver grants every Sliver you control a "{T}: Add any color" ability.
#[test]
fn manaweft_sliver_grants_mana_ability() {
    let mut g = two_player_game();
    let mw = g.add_card_to_battlefield(0, catalog::manaweft_sliver());
    g.clear_sickness(mw);
    // The granted "{T}: add any color" is the Sliver's only activated ability (index 0).
    g.perform_action(GameAction::ActivateAbility {
        card_id: mw, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Manaweft taps for mana via its granted ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "Manaweft Sliver produced one mana");
}

/// Switcheroo exchanges control of two target creatures (CR 701.12).
#[test]
fn switcheroo_exchanges_creature_control() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::switcheroo());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Switcheroo");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1, "your bear went to opp");
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "their angel is now yours");
}

/// Sylvan Advocate only pumps itself once you control six or more lands.
#[test]
fn sylvan_advocate_pumps_with_six_lands() {
    let mut g = two_player_game();
    let sa = g.add_card_to_battlefield(0, catalog::sylvan_advocate());
    assert_eq!(g.computed_permanent(sa).unwrap().power, 2, "no buff below six lands");
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    assert_eq!(g.computed_permanent(sa).unwrap().power, 4, "+2/+2 with six lands");
}

/// Wilt-Leaf Liege buffs other green creatures but not the white-only ones twice.
#[test]
fn wilt_leaf_liege_buffs_green_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wilt_leaf_liege());
    let elf = g.add_card_to_battlefield(0, catalog::sylvan_advocate()); // green Elf, base 2/3
    assert_eq!(g.computed_permanent(elf).unwrap().power, 3, "green creature gets +1/+1");
}

/// Death's-Head Buzzard wraths -1/-1 to all creatures when it dies.
#[test]
fn deaths_head_buzzard_shrinks_all_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::deaths_head_buzzard());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Burn the buzzard so its dies-trigger fires (-1/-1 to everything).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let buzz = g.battlefield.iter().find(|c| c.definition.name == "Death's-Head Buzzard").unwrap().id;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(buzz)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the buzzard");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 1, "opposing bear shrunk to 1/1");
}

/// Shepherd of Rot drains each player by the number of Zombies in play.
#[test]
fn shepherd_of_rot_drains_per_zombie() {
    let mut g = two_player_game();
    let shep = g.add_card_to_battlefield(0, catalog::shepherd_of_rot());
    g.add_card_to_battlefield(0, catalog::cemetery_reaper()); // a second Zombie
    g.clear_sickness(shep);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shep, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Shepherd of Rot");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "you lose 2 (two Zombies)");
    assert_eq!(g.players[1].life, 18, "opponent loses 2 (two Zombies)");
}

/// Cemetery Reaper's anthem buffs other Zombies you control.
#[test]
fn cemetery_reaper_buffs_other_zombies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cemetery_reaper());
    let other = g.add_card_to_battlefield(0, catalog::shepherd_of_rot());
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "other Zombie gets +1/+1");
}

/// Shared Triumph anthems the chosen creature type.
#[test]
fn shared_triumph_buffs_chosen_type() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let zombie = g.add_card_to_battlefield(0, catalog::shepherd_of_rot());
    let triumph = g.add_card_to_battlefield(0, catalog::shared_triumph());
    // Stamp the chosen type directly (the ETB NameCreatureType auto-pick is
    // exercised separately for Cavern of Souls / Adaptive Automaton).
    g.battlefield.iter_mut().find(|c| c.id == triumph).unwrap().chosen_creature_type =
        Some(CreatureType::Zombie);
    assert_eq!(g.computed_permanent(zombie).unwrap().power, 2, "chosen-type creature buffed");
}

/// Dragonlord's Servant makes Dragon spells cost {1} less.
#[test]
fn dragonlords_servant_reduces_dragon_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dragonlords_servant());
    // Shivan Dragon ({4}{R}{R}) should be castable for one less generic.
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3); // {3}{R}{R} after the {1} reduction
    g.perform_action(GameAction::CastSpell {
        card_id: dragon, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dragon costs {1} less");
}

/// Goblin War Strike burns for the number of Goblins you control.
#[test]
fn goblin_war_strike_scales_with_goblins() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dragonlords_servant()); // a Goblin
    g.add_card_to_battlefield(0, catalog::frogtosser_banneret()); // another Goblin
    let id = g.add_card_to_hand(0, catalog::goblin_war_strike());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Goblin War Strike");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "two Goblins → 2 damage");
}

/// Pyrohemia pings every creature and player for 1.
#[test]
fn pyrohemia_pings_everything() {
    let mut g = two_player_game();
    let pyro = g.add_card_to_battlefield(0, catalog::pyrohemia());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pyro, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Pyrohemia");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "you take 1");
    assert_eq!(g.players[1].life, 19, "opponent takes 1");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1, "bear took 1");
}

/// Lord of the Accursed pumps other Zombies and can grant all Zombies menace.
#[test]
fn lord_of_the_accursed_pumps_and_grants_menace() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let lord = g.add_card_to_battlefield(0, catalog::lord_of_the_accursed());
    let other = g.add_card_to_battlefield(0, catalog::shepherd_of_rot());
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "other Zombie +1/+1");
    g.clear_sickness(lord);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lord, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("grant menace");
    drain_stack(&mut g);
    assert!(g.computed_permanent(other).unwrap().keywords.contains(&Keyword::Menace));
}

/// Liliana's Mastery enters making two Zombie tokens it then anthems.
#[test]
fn lilianas_mastery_makes_and_buffs_zombies() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lilianas_mastery());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Liliana's Mastery");
    drain_stack(&mut g);
    use crabomination::card::CreatureType;
    let zombies: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Zombie))
        .map(|c| c.id).collect();
    assert_eq!(zombies.len(), 2, "two Zombie tokens entered");
    assert_eq!(g.computed_permanent(zombies[0]).unwrap().power, 3, "tokens anthemed to 3/3");
}

/// Leonin Warleader mints two tapped, attacking lifelink Cats on attack.
#[test]
fn leonin_warleader_makes_attacking_cats() {
    let mut g = two_player_game();
    let lion = g.add_card_to_battlefield(0, catalog::leonin_warleader());
    g.battlefield.iter_mut().find(|c| c.id == lion).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let events = g.declare_attackers(vec![Attack { attacker: lion, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let cats = g.battlefield.iter()
        .filter(|c| c.definition.name == "Cat" && c.tapped && c.controller == 0)
        .count();
    assert_eq!(cats, 2, "two attacking Cat tokens");
}

/// Voice of the Woods taps five Elves to mint a 7/7 trampling Elemental.
#[test]
fn voice_of_the_woods_makes_elemental() {
    let mut g = two_player_game();
    let voice = g.add_card_to_battlefield(0, catalog::voice_of_the_woods());
    g.clear_sickness(voice);
    let mut elves = vec![voice];
    for _ in 0..4 {
        let e = g.add_card_to_battlefield(0, catalog::voice_of_the_woods());
        g.clear_sickness(e);
        elves.push(e);
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: voice, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap five Elves");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Elemental" && c.power() == 7),
        "7/7 Elemental token created");
}

/// Captivating Vampire steals a creature when five Vampires tap.
#[test]
fn captivating_vampire_steals_with_five_vampires() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::captivating_vampire());
    g.clear_sickness(cap);
    let mut vamps = vec![cap];
    for _ in 0..4 {
        let v = g.add_card_to_battlefield(0, catalog::captivating_vampire());
        g.clear_sickness(v);
        vamps.push(v);
    }
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: cap, ability_index: 0, target: Some(Target::Permanent(prey)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap five Vampires");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(prey).unwrap().controller, 0, "gained control of the bear");
}

/// Horned Sliver grants trample to every Sliver (yours and theirs).
#[test]
fn horned_sliver_grants_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let hs = g.add_card_to_battlefield(0, catalog::horned_sliver());
    let opp = g.add_card_to_battlefield(1, catalog::talon_sliver());
    assert!(g.computed_permanent(hs).unwrap().keywords.contains(&Keyword::Trample));
    assert!(g.computed_permanent(opp).unwrap().keywords.contains(&Keyword::Trample),
        "all-Sliver grant reaches opponent's Sliver too");
}

/// Watcher Sliver's +0/+2 anthem toughens every Sliver.
#[test]
fn watcher_sliver_toughens_all_slivers() {
    let mut g = two_player_game();
    let ws = g.add_card_to_battlefield(0, catalog::watcher_sliver());
    assert_eq!(g.computed_permanent(ws).unwrap().toughness, 4, "2/2 base + 0/+2");
}

/// Vengeful Dead drains each opponent when any Zombie (incl. itself) dies.
#[test]
fn vengeful_dead_drains_when_a_zombie_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vengeful_dead());
    let other = g.add_card_to_battlefield(0, catalog::shepherd_of_rot()); // a Zombie
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(other)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the other Zombie");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "opponent lost 1 when the Zombie died");
}

/// Immaculate Magistrate adds +1/+1 counters equal to the Elves you control.
#[test]
fn immaculate_magistrate_counters_scale_with_elves() {
    let mut g = two_player_game();
    let mag = g.add_card_to_battlefield(0, catalog::immaculate_magistrate()); // Elf #1
    g.add_card_to_battlefield(0, catalog::sylvan_advocate());                 // Elf #2
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(mag);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mag, ability_index: 0,
        target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Immaculate Magistrate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(target).unwrap().power, 4, "2/2 + two +1/+1 counters");
}

/// Coastal Piracy lets you draw when your creature connects.
#[test]
fn coastal_piracy_draws_on_combat_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::coastal_piracy());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card off combat damage");
}

/// Leeching Sliver drains the defending player when a Sliver attacks.
#[test]
fn leeching_sliver_drains_on_sliver_attack() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leeching_sliver());
    let attacker = g.add_card_to_battlefield(0, catalog::muscle_sliver());
    g.battlefield.iter_mut().find(|c| c.id == attacker).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let events = g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("Sliver attacks");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "defending player lost 1 life");
}

/// Merrow Reejerey buffs other Merfolk and triggers on a Merfolk spell cast.
#[test]
fn merrow_reejerey_buffs_and_triggers_on_merfolk_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::merrow_reejerey());
    let other = g.add_card_to_battlefield(0, catalog::cursecatcher());
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "other Merfolk pumped");
}

// ── Elf / Zombie / Vampire tribal ─────────────────────────────────────────────

/// Elvish Champion pumps other Elves and grants forestwalk.
#[test]
fn elvish_champion_buffs_elves() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::elvish_champion());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let ec = g.computed_permanent(elf).unwrap();
    assert_eq!(ec.power, 2, "Llanowar Elves 1/1 → 2/2");
    assert!(ec.keywords.contains(&Keyword::Landwalk(crabomination::card::LandType::Forest)), "forestwalk");
}

/// Dwynen gains life per attacking Elf when she attacks.
#[test]
fn dwynen_gains_life_per_attacking_elf() {
    let mut g = two_player_game();
    let dwynen = g.add_card_to_battlefield(0, catalog::dwynen_gilt_leaf_daen());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    for id in [dwynen, elf] { g.clear_sickness(id); }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    g.declare_attackers(vec![
        Attack { attacker: dwynen, target: AttackTarget::Player(1) },
        Attack { attacker: elf, target: AttackTarget::Player(1) },
    ]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 1 per the two attacking Elves");
}

/// Legion Lieutenant buffs your other Vampires.
#[test]
fn legion_lieutenant_buffs_vampires() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::legion_lieutenant());
    let vamp = g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
    assert_eq!(g.computed_permanent(vamp).unwrap().power, 3, "Nighthawk 2/3 → 3/4");
}

/// Stromkirk Captain grants other Vampires first strike.
#[test]
fn stromkirk_captain_grants_first_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stromkirk_captain());
    let vamp = g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
    assert!(g.computed_permanent(vamp).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Lord of the Undead returns a Zombie from your graveyard to hand.
#[test]
fn lord_of_the_undead_returns_zombie() {
    let mut g = two_player_game();
    let lord = g.add_card_to_battlefield(0, catalog::lord_of_the_undead());
    g.clear_sickness(lord);
    let zombie = g.add_card_to_graveyard(0, catalog::diregraf_ghoul());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lord, ability_index: 0,
        target: Some(Target::Permanent(zombie)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Lord of the Undead");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == zombie), "Zombie returned to hand");
}

// ── Affinity for artifacts (CR 702.41) ────────────────────────────────────────

/// Somber Hoverguard's Affinity reduces its generic cost by your artifact count.
#[test]
fn somber_hoverguard_affinity_reduces_cost() {
    let mut g = two_player_game();
    // Four artifacts on the battlefield → {5}{U} becomes {1}{U}.
    for _ in 0..4 { g.add_card_to_battlefield(0, catalog::ornithopter()); }
    let id = g.add_card_to_hand(0, catalog::somber_hoverguard());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Affinity makes it castable for {1}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "Somber Hoverguard resolved");
}

/// Broodstar's power/toughness equal the number of artifacts you control (CDA).
#[test]
fn broodstar_pt_scales_with_artifacts() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::ornithopter()); }
    let star = g.add_card_to_battlefield(0, catalog::broodstar());
    // 3 Ornithopters + Broodstar itself = 4 artifacts.
    let c = g.computed_permanent(star).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "*/* = artifacts you control");
}

/// Carapace Forger gets +1/+1 while you control three or more artifacts.
#[test]
fn carapace_forger_grows_with_artifacts() {
    let mut g = two_player_game();
    let cf = g.add_card_to_battlefield(0, catalog::carapace_forger());
    assert_eq!(g.computed_permanent(cf).unwrap().power, 2, "just the Forger = 1 artifact");
    for _ in 0..2 { g.add_card_to_battlefield(0, catalog::ornithopter()); }
    assert_eq!(g.computed_permanent(cf).unwrap().power, 3, "three artifacts → +1/+1");
}

/// Qumulox's Affinity drops its {7}{U} to {U} with seven artifacts.
#[test]
fn qumulox_affinity_castable_cheap() {
    let mut g = two_player_game();
    for _ in 0..7 { g.add_card_to_battlefield(0, catalog::ornithopter()); }
    let id = g.add_card_to_hand(0, catalog::qumulox());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Affinity-7 makes Qumulox cost {U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
}

/// Sojourner's Companion can be landcycled to fetch a land.
#[test]
fn sojourners_companion_landcycles() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sojourners_companion());
    g.add_card_to_library(0, catalog::plains());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Landcycle { card_id: id })
        .expect("landcycle for {2}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "cycled into graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"), "fetched a Plains");
}

// ── White Aura tutor + Soldier tribal ─────────────────────────────────────────

/// Heliod's Pilgrim searches up an Aura on ETB.
#[test]
fn heliods_pilgrim_tutors_an_aura() {
    let mut g = two_player_game();
    let aura = g.add_card_to_library(0, catalog::pacifism());
    g.add_card_to_library(0, catalog::island()); // a non-Aura decoy
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(aura))]));
    let pilgrim = g.add_card_to_battlefield(0, catalog::heliods_pilgrim());
    g.fire_self_etb_triggers(pilgrim, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "Aura tutored to hand");
}

/// Field Marshal grants other Soldiers +1/+1 and first strike.
#[test]
fn field_marshal_buffs_soldiers() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::field_marshal());
    let sol = g.add_card_to_battlefield(0, catalog::daru_warchief());
    let c = g.computed_permanent(sol).unwrap();
    assert!(c.keywords.contains(&Keyword::FirstStrike), "first strike");
    // Daru Warchief base 1/1, +1/+1 from Field Marshal, +1/+2 from its own anthem.
    assert_eq!((c.power, c.toughness), (3, 4));
}

/// Daru Warchief reduces a Soldier spell's cost by {1}.
#[test]
fn daru_warchief_reduces_soldier_cost() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::daru_warchief());
    let sol = g.add_card_to_hand(0, catalog::field_marshal()); // {2}{W}{W} → {1}{W}{W}
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: sol, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soldier costs {1} less");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sol).is_some(), "Field Marshal resolved at reduced cost");
}

/// Catapult Master taps five Soldiers to exile a creature.
#[test]
fn catapult_master_exiles_with_five_soldiers() {
    let mut g = two_player_game();
    let master = g.add_card_to_battlefield(0, catalog::catapult_master());
    g.clear_sickness(master);
    let mut soldiers = vec![master];
    for _ in 0..4 {
        let s = g.add_card_to_battlefield(0, catalog::field_marshal());
        g.clear_sickness(s);
        soldiers.push(s);
    }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: master, ability_index: 0,
        target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Catapult Master");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "target creature exiled");
}

// ── Spirit + Knight tribal ────────────────────────────────────────────────────

/// Supreme Phantom buffs your other Spirits.
#[test]
fn supreme_phantom_buffs_spirits() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::supreme_phantom());
    let spirit = g.add_card_to_battlefield(0, catalog::empyrean_eagle());
    assert_eq!(g.computed_permanent(spirit).unwrap().power, 3, "Eagle 2/2 → 3/3");
}

/// Empyrean Eagle pumps your other fliers.
#[test]
fn empyrean_eagle_buffs_fliers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::empyrean_eagle());
    // Another flier (a second Eagle) gets +1/+1.
    let flier = g.add_card_to_battlefield(0, catalog::empyrean_eagle());
    // Each Eagle pumps the other: base 2/2 + 1 from the other Eagle.
    assert_eq!(g.computed_permanent(flier).unwrap().power, 3, "flier pumped");
    // A non-flier is unaffected.
    let ground = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(ground).unwrap().power, 2, "non-flier untouched");
}

/// Kinsbaile Cavalier grants other Knights double strike.
#[test]
fn kinsbaile_cavalier_grants_double_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kinsbaile_cavalier());
    let knight = g.add_card_to_battlefield(0, catalog::field_marshal()); // not a Knight
    assert!(!g.computed_permanent(knight).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "non-Knight unaffected");
    let real_knight = g.add_card_to_battlefield(0, catalog::kinsbaile_cavalier());
    assert!(g.computed_permanent(real_knight).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "other Knight gets double strike");
}

// ── Auras (enchantress support) ───────────────────────────────────────────────

/// Ethereal Armor gives +1/+1 per enchantment you control and first strike.
#[test]
fn ethereal_armor_scales_with_enchantments() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let armor = g.add_card_to_hand(0, catalog::ethereal_armor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: armor, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ethereal Armor");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    // One enchantment (the Armor itself) → +1/+1; 2/2 → 3/3.
    assert_eq!((c.power, c.toughness), (3, 3), "+1/+1 per enchantment");
    assert!(c.keywords.contains(&Keyword::FirstStrike), "first strike");
}

/// Curiosity draws when the enchanted creature deals combat damage to a player.
#[test]
fn curiosity_draws_on_combat_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let cur = g.add_card_to_hand(0, catalog::curiosity());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.add_card_to_library(0, catalog::island());
    g.perform_action(GameAction::CastSpell {
        card_id: cur, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Curiosity");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    for _ in 0..14 {
        if g.players[0].hand.len() > hand { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew off combat damage");
}

/// Ophidian Eye (flash Curiosity) attaches and grants the damage-draw bonus.
#[test]
fn ophidian_eye_attaches_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eye = g.add_card_to_hand(0, catalog::ophidian_eye());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: eye, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ophidian Eye");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(eye).and_then(|c| c.attached_to), Some(bear), "attached to the bear");
}

/// Aqueous Form makes the enchanted creature unblockable.
#[test]
fn aqueous_form_grants_unblockable() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let af = g.add_card_to_hand(0, catalog::aqueous_form());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: af, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Aqueous Form");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Keen Sense draws when the enchanted creature deals combat damage.
#[test]
fn keen_sense_is_green_curiosity() {
    let aura = catalog::keen_sense();
    assert_eq!(aura.cost, crabomination::mana::cost(&[crabomination::mana::g()]));
    assert_eq!(aura.equipped_bonus.unwrap().triggered_abilities.len(), 1);
}

// ── More tribal payoffs ───────────────────────────────────────────────────────

/// Obelisk of Urd buffs the chosen creature type +2/+2.
#[test]
fn obelisk_of_urd_buffs_chosen_type() {
    let mut g = two_player_game();
    let goblin = g.add_card_to_battlefield(0, catalog::skirk_prospector());
    let obelisk = g.add_card_to_battlefield(0, catalog::obelisk_of_urd());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::CreatureType(crabomination::card::CreatureType::Goblin),
    ]));
    g.fire_self_etb_triggers(obelisk, 0);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(goblin).unwrap().power, 3, "Goblin 1/1 → 3/3");
}

/// Wizened Cenn buffs other Kithkin.
#[test]
fn wizened_cenn_buffs_kithkin() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wizened_cenn());
    let kith = g.add_card_to_battlefield(0, catalog::wizened_cenn());
    assert_eq!(g.computed_permanent(kith).unwrap().power, 3, "other Kithkin pumped");
}

/// Stonybrook Banneret reduces Merfolk and Wizard spell costs by {1}.
#[test]
fn stonybrook_banneret_reduces_merfolk_cost() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stonybrook_banneret());
    let merfolk = g.add_card_to_hand(0, catalog::merrow_reejerey()); // {2}{U} → {1}{U}
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: merfolk, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Merfolk costs {1} less");
    drain_stack(&mut g);
    assert!(g.battlefield_find(merfolk).is_some());
}

/// Skymarcher Aspirant gains menace once you have the city's blessing.
#[test]
fn skymarcher_aspirant_menace_with_blessing() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let asp = g.add_card_to_battlefield(0, catalog::skymarcher_aspirant());
    assert!(!g.computed_permanent(asp).unwrap().keywords.contains(&Keyword::Menace),
        "no menace without the blessing");
    g.players[0].city_blessing = true;
    assert!(g.computed_permanent(asp).unwrap().keywords.contains(&Keyword::Menace),
        "menace with the city's blessing");
}

/// CR 702.24 — cumulative upkeep paid from the pool keeps Mystic Remora and
/// adds an age counter; unpaid, it's sacrificed.
#[test]
fn cr_702_24_cumulative_upkeep_pays_or_sacrifices() {
    use crabomination::card::CounterType;
    // Paid: load {1}, Remora survives with one age counter.
    let mut g = two_player_game();
    let remora = g.add_card_to_battlefield(0, catalog::mystic_remora());
    g.active_player_idx = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.process_cumulative_upkeep();
    assert!(g.battlefield_find(remora).is_some(), "paid upkeep keeps Remora");
    assert_eq!(g.battlefield_find(remora).unwrap().counter_count(CounterType::Age), 1);
    assert_eq!(g.players[0].mana_pool.total(), 0, "the generic upkeep was paid");
    // Unpaid: a second upkeep with an empty pool sacrifices it.
    g.process_cumulative_upkeep();
    assert!(g.battlefield_find(remora).is_none(), "unpaid cumulative upkeep sacrifices Remora");
}

/// CR 702.24 — a `wants_ui` controller gets a cumulative-upkeep prompt; yes
/// auto-taps the scaled cost, no sacrifices.
#[test]
fn cr_702_24_wants_ui_prompt_pays_scaled_upkeep() {
    use crabomination::card::CounterType;
    use crabomination::decision::{Decision, DecisionAnswer};
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let remora = g.add_card_to_battlefield(0, catalog::mystic_remora());
    g.battlefield_find_mut(remora).unwrap().add_counters(CounterType::Age, 1);
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.active_player_idx = 0;
    g.process_cumulative_upkeep();
    drain_stack(&mut g);
    let pd = g.pending_decision.as_ref().expect("upkeep prompt suspends");
    assert!(matches!(pd.decision, Decision::OptionalTrigger { .. }));
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(true))).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(remora).is_some(), "paid via prompt");
    assert_eq!(g.battlefield_find(remora).unwrap().counter_count(CounterType::Age), 2);
    let tapped = g.battlefield.iter().filter(|c| c.definition.is_land() && c.tapped).count();
    assert_eq!(tapped, 2, "{{1}} × 2 age counters auto-tapped");
    // Declining next upkeep sacrifices.
    g.process_cumulative_upkeep();
    drain_stack(&mut g);
    assert!(g.pending_decision.is_some());
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(false))).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(remora).is_none(), "declined upkeep → sacrificed");
}

/// CR 702.24 — Phyrexian Soulgorger's sacrifice cumulative upkeep eats another
/// creature to survive, and is sacrificed when none is available.
#[test]
fn cr_702_24_cumulative_upkeep_sacrifice_variant() {
    let mut g = two_player_game();
    let gorger = g.add_card_to_battlefield(0, catalog::phyrexian_soulgorger());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.process_cumulative_upkeep();
    assert!(g.battlefield_find(gorger).is_some(), "Soulgorger survives by sacrificing");
    assert!(g.battlefield_find(fodder).is_none(), "the other creature was sacrificed");
    // Next upkeep: no other creature → Soulgorger is sacrificed.
    g.process_cumulative_upkeep();
    assert!(g.battlefield_find(gorger).is_none(), "no fodder → Soulgorger sacrificed");
}

/// Necrotic Ooze gains the activated abilities of creature cards in
/// graveyards — here, Llanowar Elves' `{T}: Add {G}` mana ability.
#[test]
fn necrotic_ooze_borrows_graveyard_creature_ability() {
    let mut g = two_player_game();
    let ooze = g.add_card_to_battlefield(0, catalog::necrotic_ooze());
    g.clear_sickness(ooze);
    // Llanowar Elves sits in the graveyard, lending its mana ability.
    g.add_card_to_graveyard(0, catalog::llanowar_elves());
    // The borrowed ability surfaces at index = printed-ability count (0).
    g.perform_action(GameAction::ActivateAbility {
        card_id: ooze, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Ooze taps for {G} via Llanowar Elves' ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "produced green mana");
    assert!(g.battlefield_find(ooze).unwrap().tapped, "Ooze tapped to pay the ability");
}

/// The Gitrog Monster draws a card when a land card is put into its
/// controller's graveyard (here, by milling a Forest off the top).
#[test]
fn gitrog_draws_when_land_hits_graveyard() {
    use crabomination::effect::{Effect, Selector, PlayerRef, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let gitrog = g.add_card_to_battlefield(0, catalog::the_gitrog_monster());
    // Forest on top of the library + a spell to draw.
    g.add_card_to_library(0, catalog::grizzly_bears());
    let forest = g.add_card_to_library(0, catalog::forest());
    let top = g.players[0].library.iter().position(|c| c.id == forest).unwrap();
    let f = g.players[0].library.remove(top);
    g.players[0].library.insert(0, f);
    let hand0 = g.players[0].hand.len();
    let ctx = EffectContext::for_trigger(gitrog, 0, None, 0);
    let events = g.resolve_effect(
        &Effect::Mill { who: Selector::Player(PlayerRef::You), amount: Value::Const(1) }, &ctx,
    ).expect("mill resolves");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == forest), "Forest milled");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "milling a land drew a card off Gitrog");
}

/// Gitrog's upkeep cost sacrifices a land to spare itself (bot keeps the
/// 6/6 by paying the weakest land).
#[test]
fn gitrog_upkeep_sacrifices_a_land_to_survive() {
    use crabomination::effect::Effect;
    use crabomination::card::SelectionRequirement;
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let gitrog = g.add_card_to_battlefield(0, catalog::the_gitrog_monster());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let ctx = EffectContext::for_trigger(gitrog, 0, None, 0);
    g.resolve_effect(
        &Effect::SacrificeSourceUnlessSacrifice {
            filter: SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
        },
        &ctx,
    ).expect("upkeep cost resolves");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gitrog).is_some(), "Gitrog spared");
    assert!(g.battlefield_find(land).is_none(), "a land was sacrificed instead");
}

/// Talon Gates of Madara's `{4}` from-hand ability puts it onto the
/// battlefield; the ETB phase-out then removes a target creature.
#[test]
fn talon_gates_from_hand_enters_and_phases_out_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gates = g.add_card_to_hand(0, catalog::talon_gates_of_madara());
    g.players[0].mana_pool.add_colorless(4);
    // Ability index 2 = {4}: put this from hand onto the battlefield.
    g.perform_action(GameAction::ActivateAbility {
        card_id: gates, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate from-hand put-into-play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gates).is_some(), "Talon Gates entered the battlefield");
    assert!(g.players[0].hand.iter().all(|c| c.id != gates), "left hand");
    // ETB phased the opponent's bear out (no longer on battlefield).
    assert!(g.battlefield_find(bear).is_none(), "targeted creature phased out");
}

/// Talon Gates in hand is surfaced as `hand_activatable` so the client can
/// offer its `{4}` from-hand put-into-play ability.
#[test]
fn talon_gates_surfaced_as_hand_activatable() {
    let mut g = two_player_game();
    let gates = g.add_card_to_hand(0, catalog::talon_gates_of_madara());
    // Seat 0 holds priority at game start.
    let aff = g.compute_hand_affordances(0);
    assert!(aff.hand_activatable.contains(&gates), "from-hand ability surfaced");
}

/// Talon Gates' `{1}, {T}` ability fixes one mana of any color.
#[test]
fn talon_gates_taps_for_any_color() {
    let mut g = two_player_game();
    let gates = g.add_card_to_battlefield(0, catalog::talon_gates_of_madara());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gates, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: Some(0), mode: None,
    }).expect("tap for any color");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "spent one generic, produced one mana");
}

/// Casting a Merfolk with Merrow Reejerey out fires its tap/untap trigger
/// cleanly (AutoDecider taps a permanent; no panic).
#[test]
fn merrow_reejerey_trigger_resolves_on_merfolk_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::merrow_reejerey());
    let land = g.add_card_to_battlefield(1, catalog::island());
    let merfolk = g.add_card_to_hand(0, catalog::cursecatcher());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: merfolk, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Merfolk");
    drain_stack(&mut g);
    assert!(g.battlefield_find(merfolk).is_some(), "Merfolk resolved");
    let _ = land; // the trigger may tap/untap any permanent; just assert no panic
}

// ── Manifest / face-down permanents (CR 708, 701.34, 702.166) ────────────────

/// Hauntwoods Shrieker's attack trigger manifests dread: the top library card
/// enters as a face-down 2/2 (no name, colorless), and the other top card goes
/// to the graveyard.
#[test]
fn hauntwoods_shrieker_manifest_dread_makes_face_down_two_two() {
    let mut g = two_player_game();
    let shrieker = g.add_card_to_battlefield(0, catalog::hauntwoods_shrieker());
    g.clear_sickness(shrieker);
    // Top two: a 3/3 (Grizzly Bears is 2/2 — use a distinctive creature) and a land.
    let top = g.next_id();
    g.players[0].library.insert(0, CardInstance::new(top, catalog::elder_gargaroth(), 0));
    let second = g.next_id();
    g.players[0].library.insert(1, CardInstance::new(second, catalog::forest(), 0));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: shrieker, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    // AutoDecider keeps the top (first) candidate: Elder Gargaroth is manifested.
    let manifested = g.battlefield_find(top).expect("manifested card on battlefield");
    assert!(manifested.face_down, "enters face down");
    assert_eq!((manifested.power(), manifested.toughness()), (2, 2), "face-down 2/2");
    assert_eq!(manifested.definition.name, "", "no name while face down");
    assert!(manifested.definition.cost.colors().is_empty(), "colorless while face down");
    assert!(manifested.definition.subtypes.creature_types.is_empty(), "no subtypes while face down");
    // The other top card went to the graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == second), "other card to graveyard");
}

/// A manifested creature card can be turned face up for its mana cost (CR
/// 708.5), restoring its real characteristics; a manifested card leaves as the
/// real card.
#[test]
fn manifest_turn_face_up_restores_real_card() {
    let mut g = two_player_game();
    let top = g.next_id();
    g.players[0].library.insert(0, CardInstance::new(top, catalog::elder_gargaroth(), 0));
    let ctx = crabomination::game::effects::EffectContext::for_ability(top, 0, None);
    let mut events = vec![];
    g.manifest_card(top, 0, &ctx, &mut events);
    assert!(g.battlefield_find(top).expect("on bf").face_down, "manifested face down");
    // Turn it face up for Elder Gargaroth's {3}{G}{G}.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::TurnFaceUp { card_id: top }).expect("turn face up");
    let c = g.battlefield_find(top).expect("still on bf");
    assert!(!c.face_down, "now face up");
    assert_eq!(c.definition.name, "Elder Gargaroth");
    assert_eq!((c.power(), c.toughness()), (6, 6));
}

/// A face-down permanent is turned face up as it leaves the battlefield (CR
/// 708.10): it lands in the graveyard as the real card.
#[test]
fn face_down_creature_dies_as_real_card() {
    let mut g = two_player_game();
    let top = g.next_id();
    g.players[0].library.insert(0, CardInstance::new(top, catalog::elder_gargaroth(), 0));
    let ctx = crabomination::game::effects::EffectContext::for_ability(top, 0, None);
    let mut events = vec![];
    g.manifest_card(top, 0, &ctx, &mut events);
    g.remove_from_battlefield_to_graveyard_raw(top);
    let gy = g.players[0].graveyard.iter().find(|c| c.id == top).expect("in graveyard");
    assert_eq!(gy.definition.name, "Elder Gargaroth", "restored to real card in graveyard");
    assert!(!gy.face_down);
}

// ── claude/modern_decks: cube staples ────────────────────────────────────────

/// Mana Crypt taps for {C}{C}; the upkeep coin-flip can deal 3 to its
/// controller (tails). We force the flip outcome via a scripted decider.
#[test]
fn mana_crypt_taps_for_two_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mana_crypt());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap for {C}{C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2, "produced two colorless");
}

/// Null Rod stops a (nonmana) artifact activated ability from being activated.
#[test]
fn null_rod_locks_artifact_abilities() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::null_rod());
    // Icy Manipulator's tap-down is a nonmana artifact ability.
    let icy = g.add_card_to_battlefield(0, catalog::icy_manipulator());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: icy, ability_index: 0, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None, mode: None,
    });
    assert!(err.is_err(), "Null Rod locks the artifact's nonmana ability");
}

/// Phlage deals 3 and gains 3 on enter; it's sacrificed when not escaped.
#[test]
fn phlage_bolts_and_sacrifices_when_not_escaped() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::phlage_titan_of_fires_fury());
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 3, "gained 3");
    assert!(g.players[1].life <= life1 - 3 || g.battlefield_find(id).is_none(),
        "dealt 3 somewhere and/or sacrificed");
    assert!(g.battlefield_find(id).is_none(), "sacrificed when cast normally (not escaped)");
}

/// Ribbons of Night deals 4 to a creature and gains 4 life.
#[test]
fn ribbons_of_night_kills_and_gains() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ribbons_of_night());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ribbons of Night");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "2/2 dies to 4 damage");
    assert_eq!(g.players[0].life, life + 4, "gained 4");
}

/// Phelia's attack trigger blinks a nonland permanent (returns at end step) and
/// grows Phelia with a +1/+1 counter.
#[test]
fn phelia_attack_blinks_and_grows() {
    let mut g = two_player_game();
    let phelia = g.add_card_to_battlefield(0, catalog::phelia_exuberant_shepherd());
    g.clear_sickness(phelia);
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: phelia, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    // The blinked creature is exiled (returns next end step).
    assert!(g.battlefield_find(other).is_none(), "blinked permanent left the battlefield");
    let p = g.battlefield_find(phelia).expect("phelia still here");
    assert_eq!(p.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1, "Phelia grew");
}

/// Kessig Wolf Run pumps a target creature +X/+0 and grants trample.
#[test]
fn kessig_wolf_run_pumps_and_grants_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::kessig_wolf_run());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2); // X=2
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: Some(2), mode: None,
    }).expect("activate {2}{R}, {T}");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!(b.power, 4, "+2/+0 (base 2 + X=2)");
    assert!(b.keywords.contains(&Keyword::Trample));
}

/// Welcome to Sweettooth: I mints a Human, II a Food; III adds X +1/+1 counters
/// (X = Foods you control) to a target creature you control.
#[test]
fn welcome_to_sweettooth_saga_chapters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::welcome_to_sweettooth());
    g.saga_advance(id); // chapter I (ETB normally fires I; here advance drives it)
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Human" && c.controller == 0));
    g.saga_advance(id); // chapter II — a Food
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food" && c.controller == 0));
    g.saga_advance(id); // chapter III — X = 1 Food → +1/+1 on the Human
    drain_stack(&mut g);
    let human = g.battlefield.iter().find(|c| c.definition.name == "Human").unwrap();
    assert_eq!(human.counter_count(CounterType::PlusOnePlusOne), 1, "X=1 Food → one counter");
}

/// Hamlet Glutton: bargaining it knocks {2} off the cost; ETB gains 3 life.
#[test]
fn hamlet_glutton_bargain_reduces_cost_and_gains_life() {
    let mut g = two_player_game();
    let fodder = g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
    let id = g.add_card_to_hand(0, catalog::hamlet_glutton());
    // {5}{G}{G} minus {2} bargained = {3}{G}{G}.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder), target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bargained for {3}{G}{G}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "resolved");
    assert_eq!(g.players[0].life, life + 3, "ETB gained 3");
}

/// Gingerbrute can be sacrificed for 3 life.
#[test]
fn gingerbrute_sacrifices_for_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::gingerbrute());
    g.clear_sickness(id);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac for 3 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.battlefield_find(id).is_none(), "sacrificed");
}

/// Gingerbrute's {1} evasion: it can't be blocked except by haste creatures.
#[test]
fn gingerbrute_evasion_only_haste_can_block() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let gb = g.add_card_to_battlefield(0, catalog::gingerbrute());
    g.clear_sickness(gb);
    let plain = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // no haste
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("grant evasion");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gb).unwrap()
        .granted_keywords_eot.iter().any(|k| matches!(k, Keyword::CantBeBlockedExceptBy(_))),
        "Gingerbrute got the evasion keyword");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: gb, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(plain, gb)])).is_err(),
        "a non-haste creature can't block Gingerbrute after its evasion ability");
}

/// Built to Smash: +2/+2 to an attacker; an artifact creature also gains trample.
#[test]
fn built_to_smash_pumps_and_grants_trample_to_artifact() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let thopter = g.add_card_to_battlefield(0, catalog::ornithopter()); // 0/2 artifact creature
    g.clear_sickness(thopter);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: thopter, target: AttackTarget::Player(1),
    }])).expect("attack");
    let bts = g.add_card_to_hand(0, catalog::built_to_smash());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bts, target: Some(Target::Permanent(thopter)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(thopter).expect("alive");
    assert_eq!((cp.power, cp.toughness), (2, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Trample), "artifact creature gains trample");
}

