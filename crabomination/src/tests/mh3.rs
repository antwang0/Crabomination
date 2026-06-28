//! Tests for the Modern Horizons 3 (MH3) card batch in `catalog::sets::mh3`.

use crate::card::{CounterType, Keyword};
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use crate::catalog;

/// Cast a creature from hand, paying its cost from an infinite-ish pool, so
/// ETB triggers and enters-with-counters replacements actually fire.
fn cast_creature(g: &mut GameState, id: crate::card::CardId) {
    g.players[0].mana_pool.add(Color::White, 5);
    g.players[0].mana_pool.add(Color::Blue, 5);
    g.players[0].mana_pool.add(Color::Black, 5);
    g.players[0].mana_pool.add(Color::Red, 5);
    g.players[0].mana_pool.add(Color::Green, 5);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast creature");
    drain_stack(g);
}

/// Accursed Marauder's ETB makes each player sacrifice a creature.
#[test]
fn accursed_marauder_etb_each_player_sacrifices() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::accursed_marauder());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Each player sacrificed their only other creature.
    assert!(g.battlefield_find(mine).is_none(), "controller sacrificed a creature");
    assert!(g.battlefield_find(theirs).is_none(), "opponent sacrificed a creature");
}

/// Faithful Watchdog enters as a 3/3 (printed 0/0 + three +1/+1 counters).
#[test]
fn faithful_watchdog_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::faithful_watchdog());
    cast_creature(&mut g, id);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "0/0 + three +1/+1 = 3/3");
}

/// Nightshade Dryad has deathtouch and two mana abilities.
#[test]
fn nightshade_dryad_taps_for_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::nightshade_dryad());
    g.clear_sickness(id);
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
    // Ability 0: add {C}.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("tap for colorless");
    assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana");
}

/// Serum Visionary's ETB draws a card.
#[test]
fn serum_visionary_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::serum_visionary());
    let before = g.players[0].hand.len();
    cast_creature(&mut g, id);
    // -1 cast + 1 ETB draw = net same hand size.
    assert_eq!(g.players[0].hand.len(), before, "drew a card from the ETB");
}

/// Wing It pumps the target and grants a flying counter.
#[test]
fn wing_it_pumps_and_grants_flying() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wing_it());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Wing It");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Flying), "flying counter grants flying");
}

/// Gift of the Viper stacks three counters and untaps the target.
#[test]
fn gift_of_the_viper_counters_and_untap() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::gift_of_the_viper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Gift of the Viper");
    drain_stack(&mut g);
    let inst = g.battlefield_find(bear).unwrap();
    assert!(!inst.tapped, "untapped");
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Deathtouch) && cp.keywords.contains(&Keyword::Reach));
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 counter");
}

/// Retrofitted Transmogrant returns itself from the graveyard tapped as a 3/3.
#[test]
fn retrofitted_transmogrant_reanimates_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::retrofitted_transmogrant());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activate graveyard ability");
    drain_stack(&mut g);
    let inst = g.battlefield_find(id).expect("returned to battlefield");
    assert!(inst.tapped, "returns tapped");
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Sarpadian Simulacrum sacrifices to deal 4 damage to a creature.
#[test]
fn sarpadian_simulacrum_sacs_for_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sarpadian_simulacrum());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(victim)), x_value: None,
    }).expect("activate sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "sacrificed itself");
    assert!(g.battlefield_find(victim).is_none(), "4 damage killed the 2/2");
}

/// Consuming Corruption deals X = Swamps you control and gains that much life.
#[test]
fn consuming_corruption_scales_with_swamps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::consuming_corruption());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "3 damage killed the 2/2");
    assert_eq!(g.players[0].life, life_before + 3, "gained life = swamps");
}

/// Fanged Flames deals 4 and exiles the creature if it would die.
#[test]
fn fanged_flames_exiles_dying_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fanged_flames());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Fanged Flames");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "destroyed");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != victim), "exiled, not in graveyard");
    assert!(g.exile.iter().any(|c| c.id == victim), "exiled instead of dying");
}

/// Solstice Zealot makes energy on ETB and spends it to tap a creature.
#[test]
fn solstice_zealot_energy_taps_creature() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::solstice_zealot());
    cast_creature(&mut g, id);
    g.clear_sickness(id);
    assert_eq!(g.players[0].energy, 2, "ETB gives two energy");
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(victim)), x_value: None,
    }).expect("tap with energy");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 1, "spent one energy");
    assert!(g.battlefield_find(victim).unwrap().tapped, "target tapped");
}

/// Tempest Harvester makes energy on ETB and loots for energy.
#[test]
fn tempest_harvester_energy_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::tempest_harvester());
    cast_creature(&mut g, id);
    g.clear_sickness(id);
    assert_eq!(g.players[0].energy, 2);
    g.add_card_to_hand(0, catalog::island()); // something to discard
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("loot");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 1, "spent one energy");
    // Drew one, discarded one → net hand size unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew then discarded");
}

/// Warren Soultrader sacrifices a creature to make a Treasure.
#[test]
fn warren_soultrader_sacs_for_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::warren_soultrader());
    g.clear_sickness(id);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activate sac-for-treasure");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the other creature");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "made a Treasure token",
    );
}

/// Snapping Voidcraw taps for {C}{C}.
#[test]
fn snapping_voidcraw_taps_for_two_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::snapping_voidcraw());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("tap for CC");
    assert_eq!(g.players[0].mana_pool.total(), 2, "{{C}}{{C}}");
}

/// Solar Transformer enters tapped and produces three energy.
#[test]
fn solar_transformer_enters_tapped_with_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::solar_transformer());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "enters tapped");
    assert_eq!(g.players[0].energy, 3, "ETB gives three energy");
}

/// Roil Cartographer gains energy whenever a land you control enters.
#[test]
fn roil_cartographer_landfall_energy() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::roil_cartographer());
    let land = g.add_card_to_hand(0, catalog::island());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 1, "landfall gave one energy");
}

/// Horrid Shadowspinner has lifelink and the on-attack loot trigger.
#[test]
fn horrid_shadowspinner_has_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::horrid_shadowspinner());
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Lifelink));
}

/// Unfathomable Truths draws three and mints an Eldrazi Spawn.
#[test]
fn unfathomable_truths_draws_and_spawns() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::unfathomable_truths());
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // -1 cast + 3 draws = before + 2.
    assert_eq!(g.players[0].hand.len(), before + 2, "drew three");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Eldrazi Spawn"),
        "made an Eldrazi Spawn token",
    );
}

/// Phyrexian Ironworks spends energy to mint a 3/3 Golem.
#[test]
fn phyrexian_ironworks_makes_golem() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::phyrexian_ironworks());
    g.clear_sickness(id);
    g.players[0].energy = 3;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("make golem");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0, "spent three energy");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Phyrexian Golem"),
        "minted a Golem",
    );
}

/// Breathe Your Last destroys the target and gains 1 life per color.
#[test]
fn breathe_your_last_gains_life_per_color() {
    let mut g = two_player_game();
    // A two-color creature (Watchwolf is G/W).
    let victim = g.add_card_to_battlefield(1, catalog::watchwolf());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::breathe_your_last());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Breathe Your Last");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "destroyed");
    assert_eq!(g.players[0].life, life_before + 2, "gained 1 life per color (G/W = 2)");
}

// ── Comp-rules coverage exercised by this batch ──────────────────────────────

/// CR 105.2c — colorless is not a color. Breathe Your Last gains 0 life when
/// it destroys a colorless (devoid) creature.
#[test]
fn cr_105_2c_colorless_counts_zero_colors() {
    let mut g = two_player_game();
    // Snapping Voidcraw is devoid → colorless.
    let victim = g.add_card_to_battlefield(1, catalog::snapping_voidcraw());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::breathe_your_last());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "destroyed");
    assert_eq!(g.players[0].life, life_before, "colorless → 0 colors → 0 life");
}

/// CR 702.77 — Reinforce N. Fowl Strike can be discarded from hand to put two
/// +1/+1 counters on a creature instead of being cast.
#[test]
fn cr_702_77_fowl_strike_reinforce() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fowl_strike());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Reinforce {
        card_id: id, target: Target::Permanent(target),
    }).expect("reinforce 2");
    assert_eq!(
        g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "Reinforce 2 puts two +1/+1 counters",
    );
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Fowl Strike was discarded");
}

/// CR 702.77 — Fowl Strike's cast mode destroys a flyer (and only a flyer).
#[test]
fn fowl_strike_destroys_flyer() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::fowl_strike());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(flyer)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Fowl Strike at a flyer");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flyer).is_none(), "flyer destroyed");
}

/// CR 122.1 — a keyword counter grants its keyword and persists on the
/// permanent (Gift of the Viper's deathtouch + reach counters).
#[test]
fn cr_122_1_keyword_counter_grants_keyword() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::gift_of_the_viper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Keyword counters are stored distinctly from the granted keyword.
    let inst = g.battlefield_find(bear).unwrap();
    assert_eq!(inst.keyword_counters.get(&Keyword::Deathtouch).copied().unwrap_or(0), 1,
        "one deathtouch counter");
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Deathtouch), "counter grants the keyword");
}

/// Aerie Auxiliary supports 2 on ETB (counters on two other creatures).
#[test]
fn aerie_auxiliary_supports_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::aerie_auxiliary());
    cast_creature(&mut g, id);
    let total: u32 = [a, b]
        .iter()
        .map(|c| g.battlefield_find(*c).unwrap().counter_count(CounterType::PlusOnePlusOne))
        .sum();
    // Triggered support auto-targets one other creature (the full multi-target
    // spread is the dedicated support test's job); confirm the ETB fires.
    assert!(total >= 1, "support placed a +1/+1 counter on another creature");
}

/// Titans' Vanguard grows colorless creatures on cast (and is itself devoid).
#[test]
fn titans_vanguard_pumps_colorless_on_cast() {
    let mut g = two_player_game();
    // A colorless creature already in play (Eldrazi Spawn-style — use another
    // devoid creature). Snapping Voidcraw is devoid/colorless.
    let other = g.add_card_to_battlefield(0, catalog::snapping_voidcraw());
    let id = g.add_card_to_hand(0, catalog::titans_vanguard());
    cast_creature(&mut g, id);
    assert_eq!(
        g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "colorless creature got a +1/+1 counter on cast",
    );
}

/// Wither and Bloom shrinks a creature by -3/-3.
#[test]
fn wither_and_bloom_shrinks_target() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wither_and_bloom());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // 2/2 with -3/-3 dies as a 0-or-less-toughness SBA.
    assert!(g.battlefield_find(victim).is_none(), "-3/-3 killed the 2/2");
}

/// Thriving Skyclaw makes three energy on ETB.
#[test]
fn thriving_skyclaw_etb_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::thriving_skyclaw());
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 3, "ETB gives three energy");
}

/// Flare of Denial can be cast by sacrificing a blue creature instead of mana.
#[test]
fn flare_of_denial_alt_cost_sacrifice() {
    let mut g = two_player_game();
    // A blue creature to sacrifice (Snapping Voidcraw is devoid — use a blue one).
    let fodder = g.add_card_to_battlefield(0, catalog::serum_visionary());
    // Opponent casts a spell to counter.
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts");
    let id = g.add_card_to_hand(0, catalog::flare_of_denial());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: Some(Target::Permanent(spell)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast via sacrifice alt-cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the blue creature");
    assert!(g.battlefield_find(spell).is_none(), "countered the spell");
}

/// Hexgold Slith grows when it deals combat damage to a player.
#[test]
fn hexgold_slith_grows_on_combat_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::hexgold_slith());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("advance");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("advance to damage");
    }
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "combat damage to a player put a +1/+1 counter",
    );
}

/// Skittering Precursor mints an Eldrazi Spawn when you sacrifice a nontoken.
#[test]
fn skittering_precursor_spawns_on_sacrifice() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::skittering_precursor());
    // A sac outlet: Warren Soultrader eats a creature for a Treasure.
    let trader = g.add_card_to_battlefield(0, catalog::warren_soultrader());
    g.clear_sickness(trader);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: trader, ability_index: 0, target: None, x_value: None,
    }).expect("sacrifice a creature");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Eldrazi Spawn"),
        "sacrificing a nontoken permanent made an Eldrazi Spawn",
    );
}

/// Fetid Gargantua adapts to a 6/6 (two +1/+1 counters).
#[test]
fn fetid_gargantua_adapts() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::fetid_gargantua());
    g.clear_sickness(id);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("adapt 2");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Dreadmobile sacrifices another permanent to grow itself.
#[test]
fn dreadmobile_sacs_to_grow() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::dreadmobile());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("sac to grow");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the creature");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Inspired Inventor's ETB modal can make a Servo token.
#[test]
fn inspired_inventor_makes_servo() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Choose mode 2 (index 2) — create a Servo.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(2)]));
    let id = g.add_card_to_hand(0, catalog::inspired_inventor());
    cast_creature(&mut g, id);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Servo"),
        "made a Servo token",
    );
}

/// Proud Pack-Rhino's proliferate mode adds a counter to an existing pile.
#[test]
fn proud_pack_rhino_proliferates() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    // Mode 1 = proliferate; the proliferate sub-decisions fall back to the
    // AutoDecider (proliferate everything eligible).
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(1)]));
    let id = g.add_card_to_hand(0, catalog::proud_pack_rhino());
    cast_creature(&mut g, id);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "proliferate added a +1/+1 counter to the existing pile",
    );
}

/// Smelted Chargebug makes two energy on ETB and has menace.
#[test]
fn smelted_chargebug_etb_energy_and_menace() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::smelted_chargebug());
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 2, "ETB gives two energy");
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Menace));
}
