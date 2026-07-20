//! Tests for the recent292 Ravnica batch 2 (guild commons/uncommons +
//! `Keyword::ProtectionFromMonocolored`).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Target, TurnStep};
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
fn ravnica2_stats_and_keywords() {
    // One table-driven check for the vanilla / keyword-only bodies.
    let cases: &[(fn() -> crabomination::card::CardDefinition, (i32, i32), &[Keyword])] = &[
        (catalog::watchwolf, (3, 3), &[]),
        (catalog::skyknight_legionnaire, (2, 2), &[Keyword::Flying, Keyword::Haste]),
        (catalog::siege_wurm, (5, 5), &[Keyword::Convoke, Keyword::Trample]),
        (catalog::nightguard_patrol, (2, 1), &[Keyword::FirstStrike, Keyword::Vigilance]),
    ];
    let mut g = two_player_game();
    for (factory, (p, t), kws) in cases {
        let id = g.add_card_to_battlefield(0, factory());
        let comp = g.computed_permanent(id).unwrap();
        assert_eq!((comp.power, comp.toughness), (*p, *t));
        for kw in *kws {
            assert!(comp.keywords.contains(kw), "{:?} missing", kw);
        }
    }
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
fn silhana_ledgewalker_has_evasion_and_hexproof() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::silhana_ledgewalker());
    let comp = g.computed_permanent(s).unwrap();
    assert!(comp.keywords.contains(&Keyword::Hexproof));
    assert!(comp.keywords.iter().any(|k| matches!(k, Keyword::CantBeBlockedExceptBy(_))));
}

#[test]
fn ghost_warden_pumps_a_creature() {
    let mut g = two_player_game();
    let gw = g.add_card_to_battlefield(0, catalog::ghost_warden());
    g.clear_sickness(gw);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: gw, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
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
        additional_targets: vec![Target::Permanent(helper)], x_value: None,
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
fn vinelasher_kudzu_grows_on_landfall() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    let kudzu = g.add_card_to_battlefield(0, catalog::vinelasher_kudzu());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kudzu).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
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
        card_id: g0, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
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
        card_id: n, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
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
        additional_targets: vec![], x_value: None,
    }).expect("tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).unwrap().tapped);
}

#[test]
fn douse_in_gloom_burns_and_gains_life() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives 2
    let spell = g.add_card_to_hand(0, catalog::douse_in_gloom());
    flood(&mut g);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(angel).unwrap().damage, 2, "2 damage marked");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
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
        additional_targets: vec![], x_value: None,
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
