//! Saviors of Kamigawa (SOK) wave 2 — Sweep, Channel, and hand-size matters.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
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
