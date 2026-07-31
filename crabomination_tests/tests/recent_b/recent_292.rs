//! Tests for the recent292 Ravnica batch 2 (guild commons/uncommons +
//! `Keyword::ProtectionFromMonocolored`).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

fn flood(g: &mut crabomination::game::GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 4);
    }
    g.players[0].mana_pool.add_colorless(8);
}

fn count_tokens(g: &crabomination::game::GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

#[test]
fn guardian_has_protection_from_monocolored() {
    let mut g = two_player_game();
    let guardian = g.add_card_to_battlefield(1, catalog::guardian_of_the_guildpact());
    // A monocolored (mono-red) spell can't target it.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    let res = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(guardian)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(res.is_err(), "monocolored source can't target protection-from-monocolored");
    assert!(g.battlefield_find(guardian).is_some(), "still alive");
    // A multicolored (B/R) spell gets through.
    let ball = g.add_card_to_hand(0, catalog::wrecking_ball());
    g.perform_action(GameAction::CastSpell {
        card_id: ball, target: Some(Target::Permanent(guardian)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("multicolored source targets fine");
    drain_stack(&mut g);
    assert!(g.battlefield_find(guardian).is_none(), "destroyed by a multicolored spell");
}

#[test]
fn ghost_warden_pumps_a_creature() {
    let mut g = two_player_game();
    let gw = g.add_card_to_battlefield(0, catalog::ghost_warden());
    g.clear_sickness(gw);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: gw, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("pump");
    drain_stack(&mut g);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (3, 3));
}

#[test]
fn selesnya_evangel_taps_a_creature_for_a_saproling() {
    let mut g = two_player_game();
    let ev = g.add_card_to_battlefield(0, catalog::selesnya_evangel());
    g.clear_sickness(ev);
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ev, ability_index: 0, target: None,
        additional_targets: vec![Target::Permanent(helper)], x_value: None, mode: None,
    }).expect("make saproling");
    drain_stack(&mut g);
    assert_eq!(count_tokens(&g, "Saproling"), 1);
    assert!(g.battlefield_find(helper).unwrap().tapped, "the helper creature was tapped as a cost");
}

#[test]
fn fists_of_the_anvil_gives_plus_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::fists_of_the_anvil());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (6, 2));
}

#[test]
fn fiery_conclusion_sacs_a_creature_and_deals_five() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let spell = g.add_card_to_hand(0, catalog::fiery_conclusion());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "a creature was sacrificed");
    assert!(g.battlefield_find(victim).is_none(), "5 damage killed the 4/4");
}

#[test]
fn gatherer_of_graces_scales_with_auras_and_regenerates() {
    let mut g = two_player_game();
    let g0 = g.add_card_to_battlefield(0, catalog::gatherer_of_graces());
    assert_eq!(g.computed_permanent(g0).unwrap().power, 1, "base 1/2");
    // Attach an Aura and it grows +1/+1.
    let aura = g.add_card_to_battlefield(0, catalog::fists_of_ironwood());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(g0);
    let p = g.computed_permanent(g0).unwrap();
    assert_eq!((p.power, p.toughness), (2, 3), "+1/+1 per attached Aura");
    // Sacrifice the Aura to set up a regeneration shield.
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: g0, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("regenerate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "the Aura was sacrificed");
    assert!(g.battlefield_find(g0).unwrap().regeneration_shields > 0, "a regen shield is up");
}

#[test]
fn bramble_elemental_mints_saprolings_when_enchanted() {
    let mut g = two_player_game();
    let bramble = g.add_card_to_battlefield(0, catalog::bramble_elemental());
    let aura = g.add_card_to_hand(0, catalog::fists_of_ironwood());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bramble)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    // Two from Fists' own ETB, plus two from Bramble's aura-attached trigger.
    assert_eq!(count_tokens(&g, "Saproling"), 4);
}

#[test]
fn skarrgan_pit_skulk_bloodthirst_and_evasion() {
    let mut g = two_player_game();
    g.players[1].was_dealt_damage_this_turn = true; // Bloodthirst active
    let s = g.add_card_to_hand(0, catalog::skarrgan_pit_skulk());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Skarrgan Pit-Skulk").unwrap().id;
    let comp = g.computed_permanent(id).unwrap();
    assert_eq!((comp.power, comp.toughness), (2, 2), "entered with a bloodthirst counter");
    assert!(comp.keywords.contains(&Keyword::CantBeBlockedByPowerLess));
}

#[test]
fn gruul_nodorog_grants_itself_menace() {
    let mut g = two_player_game();
    let n = g.add_card_to_battlefield(0, catalog::gruul_nodorog());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: n, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("menace");
    drain_stack(&mut g);
    assert!(g.computed_permanent(n).unwrap().keywords.contains(&Keyword::Menace));
}

#[test]
fn ostiary_thrull_taps_a_creature() {
    let mut g = two_player_game();
    let thrull = g.add_card_to_battlefield(0, catalog::ostiary_thrull());
    g.clear_sickness(thrull);
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: thrull, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).unwrap().tapped);
}

#[test]
fn rakdos_ickspitter_pings_and_drains_controller() {
    let mut g = two_player_game();
    let ick = g.add_card_to_battlefield(0, catalog::rakdos_ickspitter());
    g.clear_sickness(ick);
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ick, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().damage, 1, "1 damage to the creature");
    assert_eq!(g.players[1].life, life - 1, "its controller lost 1 life");
}

#[test]
fn galvanic_arc_burns_on_enter_and_grants_first_strike() {
    // Opponent has no creatures, so the ETB's auto-chosen "any target" is the
    // opponent's face.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::galvanic_arc());
    flood(&mut g);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(mine)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&Keyword::FirstStrike),
        "enchanted creature has first strike");
    assert_eq!(g.players[1].life, foe_life - 3, "ETB dealt 3 to the opponent");
}

#[test]
fn ghor_clan_bloodscale_pumps_once_per_turn() {
    let mut g = two_player_game();
    let b = g.add_card_to_battlefield(0, catalog::ghor_clan_bloodscale());
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::FirstStrike));
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: b, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("pump");
    drain_stack(&mut g);
    let p = g.computed_permanent(b).unwrap();
    assert_eq!((p.power, p.toughness), (4, 3), "+2/+2");
    // Once each turn — a second activation is rejected.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: b, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "the ability is once-per-turn");
}

#[test]
fn sandsower_taps_three_to_tap_a_creature() {
    let mut g = two_player_game();
    let sower = g.add_card_to_battlefield(0, catalog::sandsower());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Cost taps three creatures you control (the sower itself + two bears).
    g.perform_action(GameAction::ActivateAbility {
        card_id: sower, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).unwrap().tapped, "the target creature is tapped");
    let tapped_own = [sower, a, b].iter().filter(|id| g.battlefield_find(**id).unwrap().tapped).count();
    assert_eq!(tapped_own, 3, "three of your creatures tapped as the cost");
}

#[test]
fn gruul_scrapper_gains_haste_only_when_red_was_spent() {
    // {3}{G}; pay the {3} with red → {R} was spent → haste.
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::gruul_scrapper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(s).unwrap().keywords.contains(&Keyword::Haste), "red spent → haste");

    // No red spent ({G} + colorless {3}) → no haste.
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::gruul_scrapper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(s).unwrap().keywords.contains(&Keyword::Haste), "no red → no haste");
}

#[test]
fn steamcore_weird_burns_only_when_red_was_spent() {
    // Opponent has no creatures → the ETB burn auto-targets their face.
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::steamcore_weird());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 3);
    let foe = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 2, "red spent → 2 damage");

    // No red spent → no burn.
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::steamcore_weird());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let foe = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe, "no red → no burn");
}

#[test]
fn torch_drake_flies_and_firebreathes() {
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::torch_drake());
    assert!(g.computed_permanent(drake).unwrap().keywords.contains(&Keyword::Flying));
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: drake, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("pump");
    drain_stack(&mut g);
    let p = g.computed_permanent(drake).unwrap();
    assert_eq!((p.power, p.toughness), (3, 2), "+1/+0");
}
