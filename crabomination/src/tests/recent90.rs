//! Functionality tests for `catalog::sets::decks::recent90` (Izzet
//! spells-matter batch + the `DoubleControllerTriggersOfType` static).

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::game::effects::EntityRef;
use crate::game::two_player_game;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// Cast a Lightning Bolt from P0 at P1's face (a red instant/sorcery cast).
fn p0_bolt_face(g: &mut GameState) {
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(g);
}

#[test]
fn adeliz_pumps_wizards_on_instant_cast() {
    let mut g = two_player_game();
    let adeliz = g.add_card_to_battlefield(0, catalog::adeliz_the_cinder_wind());
    p0_bolt_face(&mut g);
    // Adeliz is a Wizard, so it pumps itself +1/+1 → 3/3.
    let cp = g.computed_permanent(adeliz).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "Adeliz pumps Wizards +1/+1 on I/S cast");
}

#[test]
fn balmor_pumps_team_and_grants_trample() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::balmor_battlemage_captain());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    p0_bolt_face(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "team gets +1/+0");
    assert!(cp.keywords.contains(&Keyword::Trample), "team gains trample");
}

#[test]
fn bloodwater_entity_bottoms_gy_spell_to_library_top() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let ent = g.add_card_to_battlefield(0, catalog::bloodwater_entity());
    // Opt into the optional "may put target …".
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.fire_self_etb_triggers(ent, 0);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bolt), "bolt left the graveyard");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bolt), "bolt is on top of library");
}

#[test]
fn improbable_alliance_mints_faerie_on_second_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::improbable_alliance());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].cards_drawn_this_turn = 0;
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Faerie").count(), 0);
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2);
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Faerie").count(), 1,
        "second draw mints a Faerie");
}

#[test]
fn runaway_steam_kin_gains_counters_capped_at_three_then_taps_for_mana() {
    let mut g = two_player_game();
    let kin = g.add_card_to_battlefield(0, catalog::runaway_steam_kin());
    // Four red spells; the counter clause stops at three.
    for _ in 0..4 { p0_bolt_face(&mut g); }
    assert_eq!(
        g.battlefield_find(kin).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        3, "capped at three +1/+1 counters");
    // Remove three counters → add {R}{R}{R}.
    g.perform_action(GameAction::ActivateAbility {
        card_id: kin, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activation");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "activation adds RRR");
    assert_eq!(
        g.battlefield_find(kin).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        0, "three counters removed as a cost");
}

#[test]
fn harmonic_prodigy_doubles_a_shamans_trigger() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::young_pyromancer()); // Human Shaman
    g.add_card_to_battlefield(0, catalog::harmonic_prodigy());
    p0_bolt_face(&mut g);
    // Young Pyromancer's Shaman trigger fires twice → two Elementals.
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Elemental").count(), 2,
        "Harmonic Prodigy doubles the Shaman's token trigger");
}

/// CR 603.x — the subtype trigger doubler fires a matching Wizard's *non-cast*
/// trigger an additional time (the general dispatch path, not the Magecraft
/// one). Niv-Mizzet (a Wizard) pings on each draw; doubled → 2 damage per draw.
#[test]
fn cr_603_x_subtype_doubler_doubles_a_wizard_draw_trigger() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::harmonic_prodigy());
    g.add_card_to_battlefield(0, catalog::niv_mizzet_the_firemind());
    g.add_card_to_library(0, catalog::forest());
    let life = g.players[1].life;
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "Niv-Mizzet's draw trigger fires twice");
}

#[test]
fn spellheart_chimera_power_scales_with_instants_in_graveyard() {
    let mut g = two_player_game();
    let chimera = g.add_card_to_battlefield(0, catalog::spellheart_chimera());
    assert_eq!(g.computed_permanent(chimera).unwrap().power, 0, "empty gy → 0 power");
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let cp = g.computed_permanent(chimera).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "power = I/S in gy, toughness fixed 3");
}

#[test]
fn roil_eruption_deals_three_unkicked() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::roil_eruption());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "unkicked Roil Eruption deals 3");
}

#[test]
fn naru_meha_anthems_other_wizards() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::naru_meha_master_wizard());
    let other = g.add_card_to_battlefield(0, catalog::adeliz_the_cinder_wind()); // Wizard
    let cp = g.computed_permanent(other).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "other Wizard gets +1/+1 from Naru Meha");
}

#[test]
fn docent_transforms_at_three_wizards() {
    let mut g = two_player_game();
    // Two Wizards already down (Dualcaster Mage is a Human Wizard).
    g.add_card_to_battlefield(0, catalog::dualcaster_mage());
    g.add_card_to_battlefield(0, catalog::dualcaster_mage());
    let docent = g.add_card_to_battlefield(0, catalog::docent_of_perfection());
    p0_bolt_face(&mut g);
    // Cast made a 3rd Wizard token → Docent transforms to Final Iteration.
    assert_eq!(g.battlefield_find(docent).unwrap().definition.name, "Final Iteration",
        "three Wizards flips Docent");
}

#[test]
fn beacon_bolt_scales_with_instants_in_graveyard_and_exile() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_exile(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::beacon_bolt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // 1 in gy + 1 in exile = 2 damage on a 4/4 → survives with 2 damage marked.
    let cp = g.battlefield_find(victim).expect("survives 2 damage");
    assert_eq!(cp.damage, 2, "Beacon Bolt deals gy+exile I/S count");
}

#[test]
fn archaeomancer_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let arch = g.add_card_to_battlefield(0, catalog::archaeomancer());
    g.fire_self_etb_triggers(arch, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "bolt returned to hand");
}

#[test]
fn magmatic_insight_requires_a_land_pitch_and_draws_two() {
    let mut g = two_player_game();
    // A land in hand to pay the additional cost; a nonland must NOT be pitched.
    let land = g.add_card_to_hand(0, catalog::mountain());
    let keep = g.add_card_to_hand(0, catalog::lightning_bolt());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::magmatic_insight());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable with a land to pitch");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == land), "the land was discarded");
    assert!(g.players[0].hand.iter().any(|c| c.id == keep), "the nonland was kept");
    // -spell -land +2 draws = net +0 from the starting hand count.
    assert_eq!(g.players[0].hand.len(), hand_before - 2 + 2, "drew two");
}

#[test]
fn magmatic_insight_uncastable_without_a_land() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::lightning_bolt()); // nonland only
    let id = g.add_card_to_hand(0, catalog::magmatic_insight());
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no land to pitch → not castable");
}

#[test]
fn niv_mizzet_pings_when_you_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::niv_mizzet_the_firemind());
    g.add_card_to_library(0, catalog::forest());
    let life = g.players[1].life;
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "drawing pings the opponent for 1");
}


#[test]
fn cloud_sprite_can_block_only_flyers() {
    let mut g = two_player_game();
    let cs = g.add_card_to_battlefield(0, catalog::cloud_sprite());
    let kw = &g.computed_permanent(cs).unwrap().keywords;
    assert!(kw.contains(&Keyword::Flying) && kw.contains(&Keyword::CanBlockOnlyFlying));
}

#[test]
fn cinder_elemental_sacrifices_for_x_damage() {
    let mut g = two_player_game();
    let ce = g.add_card_to_battlefield(0, catalog::cinder_elemental());
    g.clear_sickness(ce);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ce, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: Some(3),
    }).expect("activate for X=3");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "X=3 damage to face");
    assert!(g.battlefield_find(ce).is_none(), "sacrificed as a cost");
}

#[test]
fn living_lightning_returns_instant_on_death() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let ll = g.add_card_to_battlefield(0, catalog::living_lightning());
    g.remove_to_graveyard_with_triggers(ll);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "dying returns an I/S from gy");
}

#[test]
fn needle_drop_only_hits_already_damaged_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    {
        let c = g.battlefield_find_mut(bear).unwrap();
        c.damage = 1;
        c.dealt_damage_this_turn = true; // legal target for Needle Drop
    }
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::needle_drop());
    g.players[0].mana_pool.add(Color::Red, 1);
    for _ in 0..1 { g.add_card_to_library(0, catalog::forest()); }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable at a damaged creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 with 1 marked dies to +1");
    // hand_before excluded Needle Drop; casting it and drawing 1 nets +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

#[test]
fn rise_from_the_tides_mints_a_zombie_per_instant_in_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // not I/S — ignored
    let id = g.add_card_to_hand(0, catalog::rise_from_the_tides());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let zs: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Zombie").collect();
    assert_eq!(zs.len(), 2, "one tapped Zombie per I/S in gy");
    assert!(zs.iter().all(|z| z.tapped), "the Zombies enter tapped");
}

#[test]
fn storm_fleet_aerialist_raid_counter() {
    // Enters with a +1/+1 counter only after you've attacked this turn.
    let mut g = two_player_game();
    g.players[0].attacked_this_turn = true;
    let a = g.add_card_to_battlefield(0, catalog::storm_fleet_aerialist());
    g.fire_self_etb_triggers(a, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "Raid → enters 2/3");
}

/// CR 510/119 — a player dealt *noncombat* damage fires the new
/// `PlayerDealtNoncombatDamage` trigger; combat damage does not.
#[test]
fn chandras_spitfire_pumps_on_opponent_noncombat_damage() {
    let mut g = two_player_game();
    let spitfire = g.add_card_to_battlefield(0, catalog::chandras_spitfire());
    // A Lightning Bolt (noncombat) to the opponent's face.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(spitfire).unwrap().power, 4, "+3/+0 on opponent noncombat damage");
}

/// Combat damage to a player must NOT fire Chandra's Spitfire (CR 510).
#[test]
fn chandras_spitfire_ignores_combat_damage() {
    let mut g = two_player_game();
    let spitfire = g.add_card_to_battlefield(0, catalog::chandras_spitfire());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(spitfire).unwrap().power, 1, "combat damage doesn't pump");
}

#[test]
fn cinder_pyromancer_pings_and_untaps_on_red_spell() {
    let mut g = two_player_game();
    let cp = g.add_card_to_battlefield(0, catalog::cinder_pyromancer());
    g.clear_sickness(cp);
    // {T}: 1 damage to a player.
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cp, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("tap for 1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
    assert!(g.battlefield_find(cp).unwrap().tapped, "tapped by its own cost");
    // Casting a red spell may untap it.
    g.decider = Box::new(crate::decision::ScriptedDecider::new(
        vec![crate::decision::DecisionAnswer::Bool(true)],
    ));
    p0_bolt_face(&mut g);
    assert!(!g.battlefield_find(cp).unwrap().tapped, "untapped after a red spell");
}

#[test]
fn mystic_retrieval_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::mystic_retrieval());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "bolt back in hand");
}

#[test]
fn deprive_counters_and_bounces_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::island());
    // A spell on the stack to counter (P0's own bolt, left unresolved).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt on stack");
    let dep = g.add_card_to_hand(0, catalog::deprive());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: dep, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Deprive castable with a land to bounce");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "the land was bounced");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "the bolt was countered");
}

#[test]
fn cerebral_vortex_draws_then_burns_by_draw_count() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
    g.players[1].cards_drawn_this_turn = 0;
    let id = g.add_card_to_hand(0, catalog::cerebral_vortex());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Target drew 2 this turn → takes 2 damage.
    assert_eq!(g.players[1].life, life - 2, "damage = cards drawn this turn (2)");
}

#[test]
fn flamewave_invoker_burns_a_player_for_five() {
    let mut g = two_player_game();
    let inv = g.add_card_to_battlefield(0, catalog::flamewave_invoker());
    g.clear_sickness(inv);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(7);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: inv, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 5, "5 damage to the player");
}

#[test]
fn goblin_taskmaster_pumps_a_goblin() {
    let mut g = two_player_game();
    let tm = g.add_card_to_battlefield(0, catalog::goblin_taskmaster());
    let other = g.add_card_to_battlefield(0, catalog::goblin_taskmaster());
    g.clear_sickness(tm);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tm, ability_index: 0, target: Some(Target::Permanent(other)),
        additional_targets: vec![], x_value: None,
    }).expect("pump a Goblin");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "+1/+0 to a Goblin");
}

#[test]
fn fireslinger_pings_target_and_self() {
    let mut g = two_player_game();
    let fs = g.add_card_to_battlefield(0, catalog::fireslinger());
    g.clear_sickness(fs);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fs, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "1 to target");
    assert_eq!(g.players[0].life, l0 - 1, "1 to you");
}

#[test]
fn jackal_pup_reflects_damage_to_you() {
    let mut g = two_player_game();
    let pup = g.add_card_to_battlefield(0, catalog::jackal_pup());
    let life = g.players[0].life;
    let mut ev = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(pup), 1, None, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "reflects the damage to its controller");
}

#[test]
fn rummaging_goblin_loots() {
    let mut g = two_player_game();
    let rg = g.add_card_to_battlefield(0, catalog::rummaging_goblin());
    g.clear_sickness(rg);
    let pitch = g.add_card_to_hand(0, catalog::mountain());
    g.add_card_to_library(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: rg, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("loot");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "discarded a card");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "drew a card");
}

#[test]
fn peel_from_reality_bounces_one_of_each() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::peel_from_reality());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(),
        "both creatures returned to hand");
}

#[test]
fn consume_spirit_drains_for_x() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::consume_spirit());
    g.players[0].mana_pool.add(Color::Black, 3); // {2}{1}{B} for X=2
    g.players[0].mana_pool.add_colorless(1);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: Some(2),
    }).expect("cast X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 2, "X=2 damage");
    assert_eq!(g.players[0].life, l0 + 2, "gain X life");
}

#[test]
fn vessel_of_nascency_digs_four_and_fills_graveyard() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::vessel_of_nascency());
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: v, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac to dig");
    drain_stack(&mut g);
    // Took one of four to hand; the other three (+ the sacrificed Vessel) hit gy.
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "kept one");
    assert!(g.players[0].graveyard.len() >= gy_before + 3, "rest milled");
}

#[test]
fn skywinder_drake_and_ridgetop_raptor_have_their_keywords() {
    let mut g = two_player_game();
    let d = g.add_card_to_battlefield(0, catalog::skywinder_drake());
    let r = g.add_card_to_battlefield(0, catalog::ridgetop_raptor());
    let dk = &g.computed_permanent(d).unwrap().keywords;
    assert!(dk.contains(&Keyword::Flying) && dk.contains(&Keyword::CanBlockOnlyFlying));
    assert!(g.computed_permanent(r).unwrap().keywords.contains(&Keyword::DoubleStrike));
    let p = g.add_card_to_battlefield(0, catalog::cloud_pirates());
    let pk = &g.computed_permanent(p).unwrap().keywords;
    assert!(pk.contains(&Keyword::Flying) && pk.contains(&Keyword::CanBlockOnlyFlying));
}

#[test]
fn warden_of_evos_isle_discounts_flying_creatures() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::warden_of_evos_isle());
    // A flying creature (Serra Angel) is discounted {1}; a nonflyer isn't.
    let angel = crate::card::CardInstance::new(g.next_id(), catalog::serra_angel(), 0);
    let bears = crate::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &angel, None), 1, "flying creature −1");
    assert_eq!(cost_reduction_for_spell(&g, 0, &bears, None), 0, "nonflyer unaffected");
}
