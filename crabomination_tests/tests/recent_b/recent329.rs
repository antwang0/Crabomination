//! The one-primitive backlog batch (`decks::recent329`).

use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood_mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Resolve an enchantment ETB, which is what Eerie watches.
fn cast_enchantment(g: &mut GameState) {
    let e = g.add_card_to_hand(0, catalog::goblin_bombardment());
    g.perform_action(GameAction::CastSpell {
        card_id: e,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchantment");
    drain_stack(g);
}

/// Victor's Eerie escalates across the turn: surveil, then a discard, then a
/// reanimation.
#[test]
fn victor_eerie_escalates_through_three_branches() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::victor_valgavoths_seneschal());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());

    flood_mana(&mut g, 0);

    // 1st resolution: surveil 2 only.
    cast_enchantment(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "no discard yet");

    // 2nd: the opponent discards.
    cast_enchantment(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "second resolution discards");

    // 3rd: a creature card comes back under Victor's controller.
    cast_enchantment(&mut g);
    assert!(
        g.battlefield
            .iter()
            .any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0),
        "third resolution reanimates"
    );
}

/// Two escalating sources keep independent tallies (CR 603 — "this ability").
#[test]
fn nth_resolution_tally_is_per_source() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::victor_valgavoths_seneschal());
    let second = g.add_card_to_battlefield(0, catalog::victor_valgavoths_seneschal());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(1, catalog::island());
    flood_mana(&mut g, 0);
    // One enchantment ETB fires both Victors' first branch (surveil), so
    // neither reaches the discard branch.
    cast_enchantment(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "both sources are still on branch 1");
    assert!(g.battlefield_find(second).is_some());
}

/// Alania copies the turn's first sorcery and only that one.
#[test]
fn alania_copies_only_the_first_sorcery() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::alania_divergent_storm());
    let a = g.add_card_to_hand(0, catalog::divination());
    let b = g.add_card_to_hand(0, catalog::divination());
    for _ in 0..12 {
        g.add_card_to_library(0, catalog::island());
    }
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::island());
    }
    flood_mana(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let opp_before = g.players[1].hand.len();
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: a,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("first sorcery");
    drain_stack(&mut g);
    // Divination draws 2; the copy draws 2 more, and the opponent drew one.
    assert_eq!(g.players[1].hand.len(), opp_before + 1, "the opponent was gifted a card");
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 4, "the sorcery was copied");

    let opp_after = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: b,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("second sorcery");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_after, "the second sorcery doesn't trigger");
}

/// Heirloom Epic's {4} is payable entirely by tapping creatures (CR 702.51 on
/// an activated ability).
#[test]
fn heirloom_epic_convokes_its_activation() {
    let mut g = main_phase();
    let epic = g.add_card_to_battlefield(0, catalog::heirloom_epic());
    let helpers: Vec<_> = (0..4)
        .map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears()))
        .collect();
    for c in &helpers {
        g.battlefield_find_mut(*c).unwrap().summoning_sick = false;
    }
    g.add_card_to_library(0, catalog::island());
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbilityWaterbend {
        card_id: epic,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        helpers: helpers.clone(),
    })
    .expect("convoked activation");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "the draw resolved with no mana");
    assert!(helpers.iter().all(|c| g.battlefield_find(*c).unwrap().tapped), "helpers tapped");
}

/// Eriette steals the enchanted permanent, and gives it back when the Aura
/// leaves.
#[test]
fn eriette_steals_while_the_aura_stays_attached() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::eriette_the_beguiler());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("aura");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "stolen");

    let mut evs = Vec::new();
    g.destroy_permanent(aura, false, &mut evs);
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "returned when the Aura left");
}

/// A pricier host is out of Eriette's reach.
#[test]
fn eriette_ignores_a_host_above_the_auras_mana_value() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::eriette_the_beguiler());
    let victim = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    assert!(
        catalog::colossal_dreadmaw().card_types.contains(&CardType::Creature),
        "the host is a nonland permanent"
    );
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("aura");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "too expensive to steal");
}


/// Sacrificing to Rottenmouth Viper's additional cost discounts it {1} apiece.
#[test]
fn rottenmouth_viper_costs_less_per_sacrifice() {
    let mut g = main_phase();
    let viper = g.add_card_to_hand(0, catalog::rottenmouth_viper());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    // {5}{B} minus three sacrifices = {2}{B}.
    for c in [Color::Black] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: viper,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast at the discounted price");
    drain_stack(&mut g);
    assert!(g.battlefield_find(viper).is_some(), "the Viper resolved");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count(),
        0,
        "all three were sacrificed"
    );
}

/// Each blight counter squeezes the opponent once; with no permanents and no
/// hand they pay the life.
#[test]
fn rottenmouth_viper_squeezes_once_per_blight_counter() {
    let mut g = main_phase();
    let viper = g.add_card_to_battlefield(0, catalog::rottenmouth_viper());
    g.battlefield_find_mut(viper).unwrap().summoning_sick = false;
    g.battlefield_find_mut(viper).unwrap().add_counters(CounterType::Blight, 1);
    g.players[1].hand.clear();
    g.players[1].life = 20;
    // The attack trigger adds a second counter, so two squeezes land.
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: viper,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 12, "two blight counters, 4 life each");
}

/// Portent of Calamity exiles at most one card per card type and hands the
/// rest back.
#[test]
fn portent_of_calamity_exiles_one_per_card_type() {
    let mut g = main_phase();
    let portent = g.add_card_to_hand(0, catalog::portent_of_calamity());
    // Two creatures, an instant and a land on top.
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::colossal_dreadmaw());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: portent,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("cast");
    drain_stack(&mut g);
    // Creature + instant + land = three types, so one Bears-or-Dreadmaw goes
    // to the graveyard.
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.is_creature()).count(), 1);
    assert_eq!(g.players[0].hand.len(), 3, "the exiled cards end up in hand");
}

/// Niko blinks a creature and every Shard becomes a copy of it.
#[test]
fn niko_turns_shards_into_copies_of_the_blinked_creature() {
    let mut g = main_phase();
    let niko = g.add_card_to_hand(0, catalog::niko_light_of_hope());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: niko,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Niko");
    drain_stack(&mut g);
    g.battlefield_find_mut(niko).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: niko,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("blink");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the creature is exiled");
    assert_eq!(
        g.battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
            .count(),
        2,
        "both Shards copied it"
    );
}

/// Wishing Well's coin counter sets the mana value it can free-cast.
#[test]
fn wishing_well_free_casts_at_the_coin_count() {
    let mut g = main_phase();
    let well = g.add_card_to_battlefield(0, catalog::wishing_well());
    g.battlefield_find_mut(well).unwrap().summoning_sick = false;
    // Lightning Bolt is mana value 1 — exactly one coin counter.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: well,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(well).unwrap().counter_count(CounterType::Coin),
        1,
        "a coin counter went on"
    );
    assert_eq!(g.players[1].life, life - 3, "the free Bolt resolved");
    assert!(
        g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "it was exiled instead of returning to the graveyard"
    );
}


/// Valgavoth eats an opponent's dying creature and lets you cast it for life.
#[test]
fn valgavoth_exiles_then_sells_back_for_life() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::valgavoth_terror_eater());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.destroy_permanent(victim, false, &mut evs);
    g.check_state_based_actions();
    assert!(
        g.exile.iter().any(|c| c.id == victim),
        "it was exiled instead of hitting the graveyard"
    );
    let life = g.players[0].life;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: victim,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast off Valgavoth");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "paid the Bears' mana value in life");
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "it's yours now");
}

/// Osteomancer Adept turns the graveyard's creatures into forage-cast spells
/// that arrive with a finality counter.
#[test]
fn osteomancer_adept_casts_creatures_by_foraging() {
    let mut g = main_phase();
    let adept = g.add_card_to_battlefield(0, catalog::osteomancer_adept());
    g.battlefield_find_mut(adept).unwrap().summoning_sick = false;
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: adept,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("grant");
    drain_stack(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("graveyard cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::Finality),
        1,
        "it entered with a finality counter"
    );
    assert_eq!(g.exile.len(), 3, "three graveyard cards were foraged away");
}

/// The Tale of Tamiyo's chapter mills two and repeats (drawing) while the pair
/// shares a card type.
#[test]
fn tale_of_tamiyo_repeats_on_a_shared_card_type() {
    let mut g = main_phase();
    // Two Islands share a type → repeat and draw; the next pair is a land and
    // an instant, which stops it.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let saga = g.add_card_to_hand(0, catalog::the_tale_of_tamiyo());
    flood_mana(&mut g, 0);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: saga,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("saga");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 4, "two pairs milled");
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "one repeat drew one card");
}

/// Kaito animates into a hexproof 3/4 Ninja on your turn while he has loyalty.
#[test]
fn kaito_is_a_ninja_on_your_turn() {
    use crabomination::card::Keyword;
    let mut g = main_phase();
    let kaito = g.add_card_to_battlefield(0, catalog::kaito_bane_of_nightmares());
    g.battlefield_find_mut(kaito).unwrap().add_counters(CounterType::Loyalty, 4);
    let cp = g.computed_permanent(kaito).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "animated");
    assert_eq!((cp.power, cp.toughness), (3, 4));
    assert!(cp.keywords.contains(&Keyword::Hexproof));

    g.active_player_idx = 1;
    let cp = g.computed_permanent(kaito).unwrap();
    assert!(!cp.card_types.contains(&CardType::Creature), "inert on the opponent's turn");
}

/// Ninjutsu puts Kaito in tapped and attacking off an unblocked attacker.
#[test]
fn kaito_ninjutsus_in_as_a_planeswalker() {
    let mut g = main_phase();
    let sneaker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(sneaker).unwrap().summoning_sick = false;
    let kaito = g.add_card_to_hand(0, catalog::kaito_bane_of_nightmares());
    flood_mana(&mut g, 0);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sneaker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::Ninjutsu { ninja: kaito, returning: sneaker })
        .expect("ninjutsu");
    assert!(g.battlefield_find(kaito).unwrap().tapped, "entered tapped");
    assert!(g.attacking.iter().any(|a| a.attacker == kaito), "and attacking");
    assert!(g.players[0].hand.iter().any(|c| c.id == sneaker), "the attacker bounced");
}

/// The exile view tells the client what Valgavoth's pile costs to play.
#[test]
fn valgavoth_exile_view_shows_the_life_price() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::valgavoth_terror_eater());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.destroy_permanent(victim, false, &mut evs);
    g.check_state_based_actions();
    let view = crabomination::server::view::project(&g, 0);
    let entry = view.exile.iter().find(|c| c.id == victim).expect("in exile");
    assert_eq!(entry.play_for_life, Some(2), "Bears cost 2 life to play");
}
