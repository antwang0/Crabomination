//! Functionality tests for `catalog::sets::decks::recent110` — Fortify
//! (CR 702.71), the no-mana-cost suspend classics, and archetype staples.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

/// CR 702.71 — Fortify attaches to a land you control and the bonus applies.
#[test]
fn cr_702_71_fortify_attaches_and_grants_indestructible() {
    let mut g = two_player_game();
    let garrison = g.add_card_to_battlefield(0, catalog::darksteel_garrison());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: garrison, target: forest }).expect("fortify");
    assert_eq!(g.battlefield_find(garrison).unwrap().attached_to, Some(forest));
    let cp = g.computed_permanent(forest).unwrap();
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Indestructible));
}

/// CR 702.71c — fortify only targets lands; a creature is rejected.
#[test]
fn cr_702_71_fortify_rejects_creature() {
    let mut g = two_player_game();
    let garrison = g.add_card_to_battlefield(0, catalog::darksteel_garrison());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    let err = g.perform_action(GameAction::Equip { equipment: garrison, target: bear });
    assert!(matches!(err, Err(GameError::InvalidTarget)));
}

/// Fortified land becoming tapped pumps a target creature +1/+1 EOT.
#[test]
fn darksteel_garrison_tap_trigger_pumps() {
    let mut g = two_player_game();
    let garrison = g.add_card_to_battlefield(0, catalog::darksteel_garrison());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(garrison).unwrap().attached_to = Some(forest);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("tap forest for mana");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "tapped fortified land pumped the bear");
}

/// Saffi's sacrifice returns the watched creature when it dies this turn.
#[test]
fn saffi_returns_dying_creature() {
    let mut g = two_player_game();
    let saffi = g.add_card_to_battlefield(0, catalog::saffi_eriksdotter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(saffi);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: saffi, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    })
    .expect("sac Saffi");
    drain_stack(&mut g);
    assert!(g.battlefield_find(saffi).is_none(), "Saffi sacrificed");
    let events = g.resolve_effect(
        &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) },
        &crabomination::game::effects::EffectContext::for_spell(1, Some(Target::Permanent(bear)), 0, 0),
    )
    .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "bear returned to the battlefield");
}

/// Restore Balance levels lands, creatures, and hands down to the fewest.
#[test]
fn restore_balance_levels_everything() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    g.add_card_to_hand(0, catalog::lightning_bolt());
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::island());
    }
    g.add_card_to_hand(1, catalog::counterspell());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&crabomination::effect::Effect::Balance, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    let lands0 = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    let creatures0 = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count();
    assert_eq!(lands0, 2, "lands leveled to the fewest (2)");
    assert_eq!(creatures0, 0, "creatures leveled to the fewest (0)");
    assert_eq!(g.players[0].hand.len(), 1, "hand leveled to the smallest (1)");
    assert_eq!(g.players[1].hand.len(), 1, "smallest hand untouched");
}

/// Restore Balance can't be hard-cast (no mana cost) but resolves off suspend.
#[test]
fn restore_balance_is_suspend_only() {
    let mut g = two_player_game();
    let rb = g.add_card_to_hand(0, catalog::restore_balance());
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: rb, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(err, Err(GameError::NoManaCost)));
}

/// Wheel of Fate wheels every player into a fresh seven.
#[test]
fn wheel_of_fate_wheels() {
    let mut g = two_player_game();
    for p in 0..2 {
        for _ in 0..10 {
            g.add_card_to_library(p, catalog::island());
        }
        g.add_card_to_hand(p, catalog::lightning_bolt());
    }
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let eff = catalog::wheel_of_fate().effect.clone();
    let events = g.resolve_effect(&eff, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 7);
}

/// Hypergenesis dumps hand permanents onto the battlefield for every player.
#[test]
fn hypergenesis_dumps_permanents() {
    let mut g = two_player_game();
    let my_bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let my_bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let their_land = g.add_card_to_hand(1, catalog::island());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let eff = catalog::hypergenesis().effect.clone();
    let events = g.resolve_effect(&eff, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(my_bear).is_some(), "creature deployed");
    assert_eq!(g.battlefield_find(their_land).unwrap().controller, 1, "opponent keeps theirs");
    assert!(g.players[0].hand.iter().any(|c| c.id == my_bolt), "instant stays in hand");
}

/// Kumena's Speaker is 1/1 alone, 2/2 with an Island.
#[test]
fn kumenas_speaker_pumps_with_island() {
    let mut g = two_player_game();
    let speaker = g.add_card_to_battlefield(0, catalog::kumenas_speaker());
    let cp = g.computed_permanent(speaker).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    g.add_card_to_battlefield(0, catalog::island());
    let cp = g.computed_permanent(speaker).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Shriekhorn enters with three charges and mills two per activation.
#[test]
fn shriekhorn_mills_two() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let horn = g.add_card_to_hand(0, catalog::shriekhorn());
    g.players[0].mana_pool.add_colorless(1);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: horn, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Shriekhorn");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(horn).unwrap().counter_count(CounterType::Charge), 3);
    let gy_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: horn, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    })
    .expect("mill two");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 2);
    assert_eq!(g.battlefield_find(horn).unwrap().counter_count(CounterType::Charge), 2);
}

/// Emrakul costs {1} less per card type in your graveyard.
#[test]
fn emrakul_promised_end_cost_reduction() {
    let mut g = two_player_game();
    // Graveyard: creature + instant + land = 3 card types → {10}.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::forest());
    let emmy = g.add_card_to_hand(0, catalog::emrakul_the_promised_end());
    g.players[0].mana_pool.add_colorless(10);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: emmy, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {10} with 3 card types in graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(emmy).is_some());
}

/// Worldspine Wurm leaves three 5/5 Wurms and shuffles itself back.
#[test]
fn worldspine_wurm_dies_tokens_and_shuffles_back() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::worldspine_wurm());
    let events = g.remove_to_graveyard_with_triggers(wurm);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let tokens = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Wurm" && c.is_token)
        .count();
    assert_eq!(tokens, 3, "three 5/5 Wurm tokens");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == wurm), "not in graveyard");
    assert!(g.players[0].library.iter().any(|c| c.id == wurm), "shuffled into library");
}

/// Martyr of Sands gains 3 life per white card in hand.
#[test]
fn martyr_of_sands_gains_life() {
    let mut g = two_player_game();
    let martyr = g.add_card_to_battlefield(0, catalog::martyr_of_sands());
    g.add_card_to_hand(0, catalog::savannah_lions());
    g.add_card_to_hand(0, catalog::savannah_lions());
    g.players[0].mana_pool.add_colorless(1);
    g.clear_sickness(martyr);
    let life = g.players[0].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: martyr, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac Martyr");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 6, "3 × 2 white cards");
    assert!(g.battlefield_find(martyr).is_none());
}

/// Proclamation of Rebirth mass-reanimates up to three MV≤1 creatures.
#[test]
fn proclamation_of_rebirth_returns_three() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::savannah_lions());
    }
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 — stays
    let proc = g.add_card_to_hand(0, catalog::proclamation_of_rebirth());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: proc, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let lions = g.battlefield.iter().filter(|c| c.definition.name == "Savannah Lions").count();
    assert_eq!(lions, 3);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.is_creature()).count(), 1);
}

/// Prismatic Omen makes your lands every basic type (a Forest taps for {U}).
#[test]
fn prismatic_omen_grants_all_basic_types() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismatic_omen());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let cp = g.computed_permanent(forest).unwrap();
    for lt in [
        crabomination::card::LandType::Plains,
        crabomination::card::LandType::Island,
        crabomination::card::LandType::Swamp,
        crabomination::card::LandType::Mountain,
        crabomination::card::LandType::Forest,
    ] {
        assert!(cp.subtypes.land_types.contains(&lt), "missing {lt:?}");
    }
}

/// Norin dodges any spell cast and returns at the next end step.
#[test]
fn norin_dodges_spells() {
    let mut g = two_player_game();
    let norin = g.add_card_to_battlefield(0, catalog::norin_the_wary());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(norin).is_none(), "Norin exiled on the cast");
    assert!(g.exile.iter().any(|c| c.id == norin));
    g.fire_step_triggers(crabomination::game::TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(norin).is_some(), "returns at the next end step");
}

/// Genesis Chamber mints a Myr for the entering creature's controller, only
/// while untapped.
#[test]
fn genesis_chamber_mints_myr() {
    let mut g = two_player_game();
    let chamber = g.add_card_to_battlefield(0, catalog::genesis_chamber());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: theirs }]);
    drain_stack(&mut g);
    let myr1 = g.battlefield.iter().filter(|c| c.definition.name == "Myr" && c.controller == 1).count();
    assert_eq!(myr1, 1, "the entering creature's controller gets the Myr");
    g.battlefield_find_mut(chamber).unwrap().tapped = true;
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: mine }]);
    drain_stack(&mut g);
    let myr0 = g.battlefield.iter().filter(|c| c.definition.name == "Myr" && c.controller == 0).count();
    assert_eq!(myr0, 0, "tapped Chamber stays silent");
}

/// Entreat the Angels mints X Angels off the {X}{X} cost.
#[test]
fn entreat_the_angels_mints_x() {
    let mut g = two_player_game();
    let entreat = g.add_card_to_hand(0, catalog::entreat_the_angels());
    // X = 2: {2}{2}{W}{W}{W}.
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: entreat, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("cast for X=2");
    drain_stack(&mut g);
    let angels = g.battlefield.iter().filter(|c| c.definition.name == "Angel").count();
    assert_eq!(angels, 2);
}

/// Fracturing Gust sweeps artifacts/enchantments and gains 2 per kill.
#[test]
fn fracturing_gust_sweeps_and_gains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismatic_omen());
    g.add_card_to_battlefield(1, catalog::chrome_mox());
    g.add_card_to_battlefield(1, catalog::chrome_mox());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let eff = catalog::fracturing_gust().effect.clone();
    let events = g.resolve_effect(&eff, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(bear).is_some(), "creatures survive");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.is_artifact() || c.definition.is_enchantment()).count(),
        0
    );
    assert_eq!(g.players[0].life, life + 6, "2 × 3 destroyed");
}

/// Hurkyl's Recall bounces all of the target player's artifacts.
#[test]
fn hurkyls_recall_bounces_artifacts() {
    let mut g = two_player_game();
    let mox1 = g.add_card_to_battlefield(1, catalog::chrome_mox());
    let mox2 = g.add_card_to_battlefield(1, catalog::chrome_mox());
    let mine = g.add_card_to_battlefield(0, catalog::chrome_mox());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    let eff = catalog::hurkyls_recall().effect.clone();
    let events = g.resolve_effect(&eff, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(mox1).is_none() && g.battlefield_find(mox2).is_none());
    assert_eq!(g.players[1].hand.len(), 2);
    assert!(g.battlefield_find(mine).is_some(), "your artifacts stay");
}

/// Slippery Scoundrel gains hexproof + unblockable with the city's blessing.
#[test]
fn slippery_scoundrel_with_blessing() {
    let mut g = two_player_game();
    let rogue = g.add_card_to_battlefield(0, catalog::slippery_scoundrel());
    let cp = g.computed_permanent(rogue).unwrap();
    assert!(!cp.keywords.contains(&crabomination::card::Keyword::Hexproof), "no blessing yet");
    g.players[0].city_blessing = true;
    let cp = g.computed_permanent(rogue).unwrap();
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Hexproof));
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Unblockable));
}

/// Tempest Djinn grows +1/+0 per Island you control.
#[test]
fn tempest_djinn_scales_with_islands() {
    let mut g = two_player_game();
    let djinn = g.add_card_to_battlefield(0, catalog::tempest_djinn());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.add_card_to_battlefield(1, catalog::island()); // theirs doesn't count
    let cp = g.computed_permanent(djinn).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4));
}

/// Undercity Informer mills the target down to their first land.
#[test]
fn undercity_informer_mills_until_land() {
    let mut g = two_player_game();
    let informer = g.add_card_to_battlefield(0, catalog::undercity_informer());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::counterspell());
    g.add_card_to_library(1, catalog::island()); // 3rd from top
    g.players[0].mana_pool.add_colorless(1);
    g.clear_sickness(informer);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: informer, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "a creature was sacrificed");
    assert_eq!(g.players[1].graveyard.len(), 3, "milled through the first land");
}

/// Runeflare Trap: {R} when an opponent drew 3+, damage = their hand size.
#[test]
fn runeflare_trap_alt_cost_and_damage() {
    let mut g = two_player_game();
    let trap = g.add_card_to_hand(0, catalog::runeflare_trap());
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::island());
    }
    g.players[1].cards_drawn_this_turn = 3;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: trap, target: Some(Target::Player(1)), additional_targets: vec![], mode: None,
        x_value: None, pitch_card: None,
    })
    .expect("trap cost");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4, "damage equals their hand size");
}

/// Molten Psyche wheels hands into libraries; metalcraft burns per draw.
#[test]
fn molten_psyche_wheels_and_burns() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::chrome_mox()); // metalcraft
        g.add_card_to_hand(1, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let life = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let eff = catalog::molten_psyche().effect.clone();
    let events = g.resolve_effect(&eff, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert_eq!(g.players[1].hand.len(), 3, "same count drawn back");
    assert_eq!(g.players[1].life, life - 3, "metalcraft burn = their draws");
}

/// Master of the Feast makes each opponent draw at your upkeep.
#[test]
fn master_of_the_feast_upkeep_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::master_of_the_feast());
    g.add_card_to_library(1, catalog::island());
    let hand = g.players[1].hand.len();
    g.active_player_idx = 0;
    g.fire_step_triggers(crabomination::game::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand + 1);
}

/// Spiteful Visions pings every drawer for 1.
#[test]
fn spiteful_visions_pings_drawers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spiteful_visions());
    g.add_card_to_library(1, catalog::island());
    let life = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
    let events = g
        .resolve_effect(
            &crabomination::effect::Effect::Draw {
                who: crabomination::effect::Selector::Player(crabomination::effect::PlayerRef::You),
                amount: crabomination::effect::Value::Const(1),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "drawer pinged for 1");
}

/// Tolaria West transmutes for an MV-0 card.
#[test]
fn tolaria_west_transmutes() {
    let mut g = two_player_game();
    let target = g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let tw = g.add_card_to_hand(0, catalog::tolaria_west());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tw, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("transmute");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == target), "MV-0 card fetched");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == tw), "Tolaria West discarded");
}

/// Boseiju mana makes the instant it funds uncounterable (and costs 2 life).
#[test]
fn boseiju_funds_uncounterable_instant() {
    let mut g = two_player_game();
    let boseiju = g.add_card_to_battlefield(0, catalog::boseiju_who_shelters_all());
    g.battlefield_find_mut(boseiju).unwrap().tapped = false;
    let life = g.players[0].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: boseiju, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("tap Boseiju");
    assert_eq!(g.players[0].life, life - 2);
    // Psionic Blast {2}{U}: the Boseiju {C} pays part of the generic.
    let blast = g.add_card_to_hand(0, catalog::psionic_blast());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: blast, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Psionic Blast off Boseiju mana");
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(blast)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("counterspell castable");
    let life1 = g.players[1].life;
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 4, "blast resolved despite the counter");
}

/// Pendelhaven pumps a 1/1 (and only a 1/1).
#[test]
fn pendelhaven_pumps_only_one_ones() {
    let mut g = two_player_game();
    let haven = g.add_card_to_battlefield(0, catalog::pendelhaven());
    let lion = g.add_card_to_battlefield(0, catalog::savannah_lions()); // 2/1
    let speaker = g.add_card_to_battlefield(0, catalog::kumenas_speaker()); // 1/1
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: haven, ability_index: 1, target: Some(Target::Permanent(lion)),
        additional_targets: vec![], x_value: None,
    });
    assert!(err.is_err(), "a 2/1 is not a legal target");
    g.battlefield_find_mut(haven).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: haven, ability_index: 1, target: Some(Target::Permanent(speaker)),
        additional_targets: vec![], x_value: None,
    })
    .expect("pump the 1/1");
    drain_stack(&mut g);
    let cp = g.computed_permanent(speaker).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
}

/// Wanderwine Hub enters untapped only when revealing a Merfolk.
#[test]
fn wanderwine_hub_reveal_gate() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::kumenas_speaker());
    let hub = g.add_card_to_battlefield(0, catalog::wanderwine_hub());
    g.fire_self_etb_triggers(hub, 0);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(hub).unwrap().tapped, "revealed a Merfolk → untapped");
    g.players[0].hand.clear();
    let hub2 = g.add_card_to_battlefield(0, catalog::wanderwine_hub());
    g.fire_self_etb_triggers(hub2, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(hub2).unwrap().tapped, "no Merfolk to reveal → tapped");
}

/// Fevered Visions draws for the active player and burns a full-handed
/// opponent at their end step.
#[test]
fn fevered_visions_end_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fevered_visions());
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::island());
    }
    g.add_card_to_library(1, catalog::island());
    let life = g.players[1].life;
    let hand = g.players[1].hand.len();
    g.active_player_idx = 1;
    g.fire_step_triggers(crabomination::game::TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand + 1, "opponent drew at their end step");
    assert_eq!(g.players[1].life, life - 2, "4+ cards in hand → 2 damage");
}
