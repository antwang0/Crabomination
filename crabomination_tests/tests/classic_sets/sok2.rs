//! Saviors of Kamigawa (SOK) wave 2 — Sweep, Channel, and hand-size matters.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Every SOK wave-2 factory is registered under its printed name.
#[test]
fn sok2_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::barrel_down_sokenzan as fn() -> crabomination::card::CardDefinition,
        catalog::charge_across_the_araba,
        catalog::plow_through_reito,
        catalog::sink_into_takenuma,
        catalog::shinen_of_fears_chill,
        catalog::shinen_of_flights_wings,
        catalog::shinen_of_furys_fire,
        catalog::shinen_of_lifes_roar,
        catalog::shinen_of_stars_light,
        catalog::jiwari_the_earth_aflame,
        catalog::kiyomaro_first_to_stand,
        catalog::okina_nightwatch,
        catalog::secretkeeper,
        catalog::descendant_of_kiyomaro,
        catalog::kitsune_loreweaver,
        catalog::kitsune_bonesetter,
        catalog::locust_miser,
        catalog::minamo_scrollkeeper,
        catalog::trusted_advisor,
        catalog::meishin_the_mind_cage,
        catalog::ivory_crane_netsuke,
        catalog::scroll_of_origins,
        catalog::presence_of_the_wise,
        catalog::spiraling_embers,
        catalog::inner_fire,
        catalog::one_with_nothing,
        catalog::oppressive_will,
        catalog::kagemaros_clutch,
        catalog::rending_vines,
        catalog::thoughts_of_ruin,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Sweep returns the Mountains and pays out twice their count.
#[test]
fn barrel_down_sokenzan_sweeps_mountains_for_double_damage() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::barrel_down_sokenzan());
    g.players[0].mana_pool.add(Color::Red, 3);
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert_eq!(g.players[0].hand.len(), 3, "all three Mountains bounced");
    assert!(g.battlefield_find(bear).is_none(), "6 damage killed the 2/2");
}

/// A Sweep with nothing to return still resolves for zero.
#[test]
fn sweep_with_no_lands_deals_nothing() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::barrel_down_sokenzan());
    g.players[0].mana_pool.add(Color::Red, 3);
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_some());
}

/// Sink into Takenuma strips one card per Swamp swept.
#[test]
fn sink_into_takenuma_discards_per_swamp() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::sink_into_takenuma());
    g.players[0].mana_pool.add(Color::Black, 4);
    cast(&mut g, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 2);
}

/// A Shinen channels its keyword onto another creature.
#[test]
fn shinen_of_flights_wings_channels_flying() {
    let mut g = two_player_game();
    let shinen = g.add_card_to_hand(0, catalog::shinen_of_flights_wings());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shinen,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("channel");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == shinen), "discarded as a cost");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

/// Kiyomaro is hand-sized, gains vigilance at four cards, and drains at seven.
#[test]
fn kiyomaro_tracks_your_hand() {
    let mut g = two_player_game();
    let kiyo = g.add_card_to_battlefield(0, catalog::kiyomaro_first_to_stand());
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let cp = g.computed_permanent(kiyo).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Vigilance));
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let life = g.players[0].life;
    let mut ev = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(1), 7, Some(kiyo), &mut ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 7, "the 7-card grip pays out");
}

/// Okina Nightwatch only grows while you're ahead on cards.
#[test]
fn okina_nightwatch_needs_hand_advantage() {
    let mut g = two_player_game();
    let watch = g.add_card_to_battlefield(0, catalog::okina_nightwatch());
    g.add_card_to_hand(1, catalog::forest());
    assert_eq!(g.computed_permanent(watch).unwrap().power, 4, "behind on cards");
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    assert_eq!(g.computed_permanent(watch).unwrap().power, 7);
}

/// Secretkeeper picks up flying alongside its pump.
#[test]
fn secretkeeper_flies_while_ahead() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::secretkeeper());
    g.add_card_to_hand(0, catalog::forest());
    let cp = g.computed_permanent(keeper).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Trusted Advisor widens your maximum hand size by two.
#[test]
fn trusted_advisor_raises_max_hand_size() {
    let mut g = two_player_game();
    let base = g.effective_max_hand_size(0).unwrap();
    g.add_card_to_battlefield(0, catalog::trusted_advisor());
    assert_eq!(g.effective_max_hand_size(0).unwrap(), base + 2);
    g.add_card_to_battlefield(0, catalog::minamo_scrollkeeper());
    assert_eq!(g.effective_max_hand_size(0).unwrap(), base + 3, "copies stack");
}

/// Meishin shrinks every creature's power by your hand size.
#[test]
fn meishin_shrinks_by_your_hand() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::meishin_the_mind_cage());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    assert_eq!(g.computed_permanent(mine).unwrap().power, 0);
    let cp = g.computed_permanent(theirs).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 2), "toughness is untouched");
}

/// Kagemaro's Clutch shrinks the host by the Aura controller's hand.
#[test]
fn kagemaros_clutch_shrinks_by_your_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::kagemaros_clutch());
    g.players[0].mana_pool.add(Color::Black, 4);
    g.add_card_to_hand(0, catalog::forest());
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    // One Forest left in hand after the Clutch itself leaves.
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
}

/// Ivory Crane Netsuke only pays out on a seven-card grip.
#[test]
fn ivory_crane_netsuke_needs_seven_cards() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ivory_crane_netsuke());
    for _ in 0..6 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "six cards is not enough");
    g.add_card_to_hand(0, catalog::forest());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4);
}

/// Oppressive Will taxes by your hand size.
#[test]
fn oppressive_will_taxes_by_your_hand() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bears");
    g.priority.player_with_priority = 0;
    let will = g.add_card_to_hand(0, catalog::oppressive_will());
    g.players[0].mana_pool.add(Color::Blue, 3);
    cast(&mut g, will, Some(Target::Permanent(bears)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears), "countered — no mana to pay 4");
}

/// Rending Vines only kills what your hand can pay for.
#[test]
fn rending_vines_is_gated_on_your_hand_size() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::wurmcoil_engine()); // MV 6
    let vines = g.add_card_to_hand(0, catalog::rending_vines());
    g.players[0].mana_pool.add(Color::Green, 3);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: vines,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(), "MV 6 is out of reach on a one-card hand");
}

/// Thoughts of Ruin costs each player a land per card in your hand.
#[test]
fn thoughts_of_ruin_scales_with_your_hand() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::thoughts_of_ruin());
    g.add_card_to_hand(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Red, 4);
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(8),
    ));
    cast(&mut g, spell, None);
    // One Forest is left in hand once Thoughts of Ruin is on the stack.
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 2);
}

/// Kitsune Bonesetter can only shield while you're ahead on cards.
#[test]
fn kitsune_bonesetter_needs_hand_advantage() {
    let mut g = two_player_game();
    let fox = g.add_card_to_battlefield(0, catalog::kitsune_bonesetter());
    g.battlefield_find_mut(fox).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    let activate = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: fox,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(activate(&mut g).is_err(), "behind on cards");
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    if let Some(c) = g.battlefield_find_mut(fox) {
        c.tapped = false;
    }
    assert!(activate(&mut g).is_ok());
}

/// Inner Fire converts your hand into red mana.
#[test]
fn inner_fire_adds_red_per_card() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::inner_fire());
    g.players[0].mana_pool.add(Color::Red, 4);
    cast(&mut g, spell, None);
    // Three Forests remain in hand once Inner Fire is on the stack.
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
}

/// One with Nothing empties your hand.
#[test]
fn one_with_nothing_discards_everything() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::one_with_nothing());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, spell, None);
    assert!(g.players[0].hand.is_empty());
}

/// Every SOK wave-2 batch-2 factory is registered under its printed name.
#[test]
fn sok2_batch2_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::haru_onna as fn() -> crabomination::card::CardDefinition,
        catalog::kiri_onna,
        catalog::nikko_onna,
        catalog::yuki_onna,
        catalog::infernal_kirin,
        catalog::skyfire_kirin,
        catalog::inner_chamber_guard,
        catalog::kitsune_dawnblade,
        catalog::iizuka_the_ruthless,
        catalog::matsu_tribe_birdstalker,
        catalog::kashi_tribe_elite,
        catalog::oni_of_wild_places,
        catalog::stampeding_serow,
        catalog::skull_collector,
        catalog::oboro_breezecaller,
        catalog::oboro_envoy,
        catalog::moonbow_illusionist,
        catalog::oboro_palace_in_the_clouds,
        catalog::miren_the_moaning_well,
        catalog::manriki_gusari,
        catalog::soratami_cloud_chariot,
        catalog::wine_of_blood_and_iron,
        catalog::reverence,
        catalog::seed_the_land,
        catalog::molting_skin,
        catalog::razorjaw_oni,
        catalog::raving_oni_slave,
        catalog::reki_the_history_of_kamigawa,
        catalog::maga_traitor_to_mortals,
        catalog::torii_watchward,
        catalog::kami_of_the_tended_garden,
        catalog::moonwing_moth,
        catalog::path_of_angers_flame,
        catalog::sunder_from_within,
        catalog::ideas_unbound,
        catalog::overwhelming_intellect,
        catalog::twincast,
        catalog::endless_swarm,
        catalog::akuta_born_of_ash,
        catalog::exile_into_darkness,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Infernal Kirin strips every card sharing the spell's mana value.
#[test]
fn infernal_kirin_strips_the_matching_mana_value() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::infernal_kirin());
    g.add_card_to_hand(1, catalog::grizzly_bears()); // MV 2
    g.add_card_to_hand(1, catalog::memnite()); // MV 0
    let ray = g.add_card_to_hand(0, catalog::glacial_ray()); // MV 2 Arcane
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, ray, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 1, "only the MV-2 card was pitched");
}

/// Haru-Onna draws on entry and can hop home off spiritcraft.
#[test]
fn haru_onna_draws_then_bounces_on_spiritcraft() {
    let mut g = two_player_game();
    let haru = g.add_card_to_hand(0, catalog::haru_onna());
    g.players[0].mana_pool.add(Color::Green, 4);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    cast(&mut g, haru, None);
    assert_eq!(g.players[0].hand.len(), 1, "the ETB drew a card");
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(4),
    ));
    cast(&mut g, ray, Some(Target::Player(1)));
    assert!(g.players[0].hand.iter().any(|c| c.id == haru), "Haru-Onna bounced itself");
}

/// Reverence only stops small attackers.
#[test]
fn reverence_stops_small_attackers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::reverence());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    for id in [small, big] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    let attack = |a| vec![Attack { attacker: a, target: AttackTarget::Player(0) }];
    assert!(g.declare_attackers(attack(small)).is_err(), "power 2 is barred");
    g.priority.player_with_priority = 1;
    let big_ok = g.declare_attackers(attack(big));
    assert!(big_ok.is_ok(), "power 4 gets through: {big_ok:?}");
}

/// Razorjaw Oni shuts down black blockers on both sides.
#[test]
fn razorjaw_oni_stops_black_blockers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::razorjaw_oni());
    let black = g.add_card_to_battlefield(1, catalog::gravedigger());
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.computed_permanent(black).unwrap().keywords.contains(&Keyword::CantBlock));
    assert!(!g.computed_permanent(green).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Kashi-Tribe Elite hands your legendary Snakes shroud.
#[test]
fn kashi_tribe_elite_shrouds_legendary_snakes() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kashi_tribe_elite());
    let sasuke = g.add_card_to_battlefield(0, catalog::seshiro_the_anointed());
    assert!(g.computed_permanent(sasuke).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Maga drains for the X it entered with.
#[test]
fn maga_drains_for_its_counters() {
    let mut g = two_player_game();
    let maga = g.add_card_to_hand(0, catalog::maga_traitor_to_mortals());
    g.players[0].mana_pool.add(Color::Black, 7);
    g.perform_action(GameAction::CastSpell {
        card_id: maga,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("cast Maga for X=4");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16);
    assert_eq!(g.computed_permanent(maga).unwrap().power, 4);
}

/// Seed the Land mints a Snake for whoever played the land.
#[test]
fn seed_the_land_mints_for_the_land_controller() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::seed_the_land());
    let land = g.add_card_to_hand(1, catalog::forest());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    let snakes = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Snake" && c.controller == 1)
        .count();
    assert_eq!(snakes, 1);
}

/// Miren eats a creature for its toughness in life.
#[test]
fn miren_pays_the_sacrificed_toughness() {
    let mut g = two_player_game();
    let miren = g.add_card_to_battlefield(0, catalog::miren_the_moaning_well());
    g.battlefield_find_mut(miren).unwrap().summoning_sick = false;
    g.add_card_to_battlefield(0, catalog::wall_of_omens()); // 0/4
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: miren,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("sac a creature");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4);
}

/// Overwhelming Intellect draws the countered spell's mana value.
#[test]
fn overwhelming_intellect_draws_the_countered_mana_value() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let angel = g.add_card_to_hand(1, catalog::serra_angel()); // MV 5
    g.players[1].mana_pool.add(Color::White, 5);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: angel,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast angel");
    g.priority.player_with_priority = 0;
    let intellect = g.add_card_to_hand(0, catalog::overwhelming_intellect());
    g.players[0].mana_pool.add(Color::Blue, 6);
    cast(&mut g, intellect, Some(Target::Permanent(angel)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == angel));
    assert_eq!(g.players[0].hand.len(), 5);
}

/// Akuta buys itself back out of the graveyard for a Swamp.
#[test]
fn akuta_returns_from_the_graveyard_for_a_swamp() {
    let mut g = two_player_game();
    let akuta = g.add_card_to_graveyard(0, catalog::akuta_born_of_ash());
    let swamp = g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(4),
    ));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(akuta).is_some(), "Akuta came back");
    assert!(g.battlefield_find(swamp).is_none(), "the Swamp paid for it");
}

/// Molting Skin bounces itself to regenerate a creature.
#[test]
fn molting_skin_returns_itself_to_regenerate() {
    let mut g = two_player_game();
    let skin = g.add_card_to_battlefield(0, catalog::molting_skin());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: skin,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bounce to regenerate");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == skin));
    assert!(g.battlefield_find(bear).unwrap().regeneration_shields > 0);
}

/// Reki draws off your legendary spells only.
#[test]
fn reki_draws_on_legendary_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::reki_the_history_of_kamigawa());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bears, None);
    assert_eq!(g.players[0].hand.len(), 0, "a nonlegendary spell draws nothing");
    let reki2 = g.add_card_to_hand(0, catalog::iizuka_the_ruthless());
    g.players[0].mana_pool.add(Color::Red, 5);
    cast(&mut g, reki2, None);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// The Ascendant flip cycle and the splice/Zubera batch are all registered.
#[test]
fn sok2_batch3_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::erayo_soratami_ascendant as fn() -> crabomination::card::CardDefinition,
        catalog::homura_human_ascendant,
        catalog::kuon_ogre_ascendant,
        catalog::rune_tail_kitsune_ascendant,
        catalog::sasaya_orochi_ascendant,
        catalog::into_the_fray,
        catalog::shifting_borders,
        catalog::rushing_tide_zubera,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Rune-Tail flips the moment you hit 30 life, then shields your board.
#[test]
fn rune_tail_flips_at_thirty_life() {
    let mut g = two_player_game();
    let fox = g.add_card_to_battlefield(0, catalog::rune_tail_kitsune_ascendant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.check_state_based_actions();
    assert!(!g.battlefield_find(fox).unwrap().flipped, "still a creature at 20 life");
    g.players[0].life = 30;
    g.check_state_based_actions();
    assert!(g.battlefield_find(fox).unwrap().flipped, "flipped at 30");
    let mut ev = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(bear), 2, None, &mut ev);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "the Essence prevented it");
}

/// Erayo flips on the fourth spell of a turn.
#[test]
fn erayo_flips_on_the_fourth_spell() {
    let mut g = two_player_game();
    let erayo = g.add_card_to_battlefield(0, catalog::erayo_soratami_ascendant());
    for i in 0..4 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Player(1)));
        let flipped = g.battlefield_find(erayo).unwrap().flipped;
        assert_eq!(flipped, i == 3, "flips on exactly the fourth spell");
    }
}

/// Homura comes back as its Essence and anthems the team.
#[test]
fn homura_returns_flipped_and_anthems() {
    let mut g = two_player_game();
    let homura = g.add_card_to_battlefield(0, catalog::homura_human_ascendant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ev = vec![];
    g.destroy_permanent(homura, false, &mut ev);
    drain_stack(&mut g);
    let back = g.battlefield_find(homura).expect("Homura returned");
    assert!(back.flipped);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Kuon flips at end step once three creatures have died.
#[test]
fn kuon_flips_after_three_deaths() {
    let mut g = two_player_game();
    let kuon = g.add_card_to_battlefield(0, catalog::kuon_ogre_ascendant());
    for _ in 0..3 {
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mut ev = vec![];
        g.destroy_permanent(bear, false, &mut ev);
        drain_stack(&mut g);
    }
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(kuon).unwrap().flipped);
}

/// Sasaya flips on a hand stuffed with lands.
#[test]
fn sasaya_flips_on_seven_lands_in_hand() {
    let mut g = two_player_game();
    let sasaya = g.add_card_to_battlefield(0, catalog::sasaya_orochi_ascendant());
    g.battlefield_find_mut(sasaya).unwrap().summoning_sick = false;
    let flip = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: sasaya,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    for _ in 0..6 {
        g.add_card_to_hand(0, catalog::forest());
    }
    assert!(flip(&mut g).is_err(), "six lands is not enough");
    g.add_card_to_hand(0, catalog::forest());
    assert!(flip(&mut g).is_ok());
    drain_stack(&mut g);
    assert!(g.battlefield_find(sasaya).unwrap().flipped);
}

/// Rushing-Tide Zubera only draws when four damage killed it.
#[test]
fn rushing_tide_zubera_needs_four_damage() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let zubera = g.add_card_to_battlefield(0, catalog::rushing_tide_zubera());
    let mut ev = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(zubera), 4, None, &mut ev);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 3);
}

/// The Epic batch and the last SOK stragglers are all registered.
#[test]
fn sok2_batch4_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::eternal_dominion as fn() -> crabomination::card::CardDefinition,
        catalog::neverending_torment,
        catalog::undying_flames,
        catalog::curtain_of_light,
        catalog::michiko_konda_truth_seeker,
        catalog::measure_of_wickedness,
        catalog::iname_as_one,
        catalog::sakashima_the_impostor,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Undying Flames digs past lands and burns for the first nonland's mana value.
#[test]
fn undying_flames_burns_for_the_first_nonland() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::serra_angel()); // MV 5 — deepest
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest()); // top
    let spell = g.add_card_to_hand(0, catalog::undying_flames());
    g.players[0].mana_pool.add(Color::Red, 6);
    cast(&mut g, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 15);
    assert_eq!(g.exile.iter().filter(|c| c.owner == 0).count(), 3);
}

/// Curtain of Light blanks an unblocked attacker's damage.
#[test]
fn curtain_of_light_blocks_an_unblocked_attacker() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(atk).unwrap().summoning_sick = false;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(0) }])
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    let spell = g.add_card_to_hand(0, catalog::curtain_of_light());
    g.players[0].mana_pool.add(Color::White, 2);
    g.priority.player_with_priority = 0;
    cast(&mut g, spell, Some(Target::Permanent(atk)));
    let life = g.players[0].life;
    for _ in 0..8 {
        if g.step == TurnStep::EndCombat || g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
    assert_eq!(g.players[0].life, life, "the attacker is blocked by nothing");
}

/// Michiko Konda punishes any damage an opponent's source deals you.
#[test]
fn michiko_konda_taxes_damage_dealt_to_you() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::michiko_konda_truth_seeker());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Player(0)));
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 1).count(),
        0,
        "the burn cost them their board"
    );
}

/// Measure of Wickedness passes to an opponent when a card hits your graveyard.
#[test]
fn measure_of_wickedness_changes_hands() {
    let mut g = two_player_game();
    let measure = g.add_card_to_battlefield(0, catalog::measure_of_wickedness());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ev = vec![];
    g.destroy_permanent(bear, false, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(measure).unwrap().controller, 1);
}

/// Sakashima enters as a copy but keeps its own name.
#[test]
fn sakashima_copies_but_keeps_its_name() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let saka = g.add_card_to_hand(0, catalog::sakashima_the_impostor());
    g.players[0].mana_pool.add(Color::Blue, 4);
    cast(&mut g, saka, Some(Target::Permanent(angel)));
    let cp = g.computed_permanent(saka).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert_eq!(g.battlefield_find(saka).unwrap().definition.name, "Sakashima the Impostor");
}

/// Shape Stealer takes on its blocker's body.
#[test]
fn shape_stealer_copies_its_blocker() {
    let mut g = two_player_game();
    let thief = g.add_card_to_battlefield(0, catalog::shape_stealer());
    g.battlefield_find_mut(thief).unwrap().summoning_sick = false;
    let blocker = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: thief, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, thief)])).expect("block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(thief).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Sokenzan Renegade defects to whoever is holding the most cards.
#[test]
fn sokenzan_renegade_defects_to_the_fullest_hand() {
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::sokenzan_renegade());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
    }
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ogre).unwrap().controller, 1);
}

/// Tomb of Urami eats your mana base for a 5/5 flier.
#[test]
fn tomb_of_urami_trades_your_lands_for_a_demon() {
    let mut g = two_player_game();
    let tomb = g.add_card_to_battlefield(0, catalog::tomb_of_urami());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    g.players[0].mana_pool.add(Color::Black, 4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tomb,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("make Urami");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 0);
    let urami = g.battlefield.iter().find(|c| c.definition.name == "Urami").expect("token");
    assert_eq!((urami.power(), urami.toughness()), (5, 5));
}

/// The last SOK batch is registered.
#[test]
fn sok2_batch5_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::shape_stealer as fn() -> crabomination::card::CardDefinition,
        catalog::sokenzan_renegade,
        catalog::tomb_of_urami,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Cowed by Wisdom taxes the host by the Aura controller's hand size.
#[test]
fn cowed_by_wisdom_taxes_by_your_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    let aura = g.add_card_to_hand(0, catalog::cowed_by_wisdom());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let attack = vec![Attack { attacker: bear, target: AttackTarget::Player(0) }];
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    assert!(g.declare_attackers(attack.clone()).is_err(), "no mana for the two-card tax");
    g.players[1].mana_pool.add(Color::Green, 2);
    g.priority.player_with_priority = 1;
    assert!(g.declare_attackers(attack).is_ok());
}

/// Cowed by Wisdom is registered.
#[test]
fn cowed_by_wisdom_is_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    assert!(names.contains(&catalog::cowed_by_wisdom().name));
}
