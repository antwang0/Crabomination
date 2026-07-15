//! Functionality tests for `catalog::sets::decks::recent54` — +1/+1 counters
//! matters (enter/cast payoffs, counter-doubling, counter-bearer anthems).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::*;
use crabomination::mana::Color;

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

fn cast_from_hand(g: &mut GameState, id: CardId, colors: &[(Color, u32)], generic: u32) {
    for (c, n) in colors {
        g.players[0].mana_pool.add(*c, *n);
    }
    if generic > 0 {
        g.players[0].mana_pool.add_colorless(generic);
    }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from hand");
    drain_stack(g);
}

#[test]
fn good_fortune_unicorn_counters_the_entrant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::good_fortune_unicorn());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
    assert_eq!(counters(&g, bear), 1, "entering creature got a +1/+1 counter");
}

#[test]
fn ivy_lane_denizen_counters_a_target_on_green_entry() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ivy_lane_denizen());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
    assert_eq!(counters(&g, ally), 1, "green entry put a counter on the chosen creature");
}

#[test]
fn managorger_hydra_grows_on_any_spell() {
    let mut g = two_player_game();
    let hydra = g.add_card_to_battlefield(0, catalog::managorger_hydra());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
    assert_eq!(counters(&g, hydra), 1, "cast a spell → +1/+1 on Managorger");
}

#[test]
fn herd_baloth_makes_a_beast_when_countered() {
    let mut g = two_player_game();
    let herd = g.add_card_to_battlefield(0, catalog::herd_baloth());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(herd)), 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let beasts = g.battlefield.iter().filter(|c| c.definition.name == "Beast").count();
    assert_eq!(beasts, 1, "putting a counter on Herd Baloth minted a 4/4 Beast");
}

#[test]
fn duskshell_crawler_grants_trample_to_counter_bearers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::duskshell_crawler());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(ally)), 0, 0);
    g.resolve_effect(
        &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        &ctx,
    )
    .unwrap();
    let cp = g.compute_battlefield();
    let a = cp.iter().find(|c| c.id == ally).unwrap();
    assert!(a.keywords.contains(&Keyword::Trample), "counter-bearer has trample");
}

#[test]
fn longshot_squad_grants_reach_to_counter_bearers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::longshot_squad());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(ally)), 0, 0);
    g.resolve_effect(
        &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        &ctx,
    )
    .unwrap();
    let cp = g.compute_battlefield();
    let a = cp.iter().find(|c| c.id == ally).unwrap();
    assert!(a.keywords.contains(&Keyword::Reach), "counter-bearer has reach");
    // A creature with no counter is unaffected.
    let bare = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.compute_battlefield();
    assert!(!cp.iter().find(|c| c.id == bare).unwrap().keywords.contains(&Keyword::Reach));
}

#[test]
fn kami_of_whispered_hopes_adds_an_extra_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kami_of_whispered_hopes());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(
        &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        &ctx,
    )
    .unwrap();
    assert_eq!(counters(&g, bear), 2, "one counter became two (that many plus one)");
}

#[test]
fn old_gnawbone_mints_treasure_per_combat_damage() {
    let mut g = two_player_game();
    let gnaw = g.add_card_to_battlefield(0, catalog::old_gnawbone());
    let effect = catalog::old_gnawbone().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { event_amount: 7, ..EffectContext::for_trigger(gnaw, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 7, "7 combat damage → 7 Treasure tokens");
}

#[test]
fn ulvenwald_tracker_fights() {
    let mut g = two_player_game();
    let tracker = g.add_card_to_battlefield(0, catalog::ulvenwald_tracker());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(tracker);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tracker,
        ability_index: 0,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(foe)],
        x_value: None,
    })
    .expect("fight");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(foe).is_none(), "2/2s trade in the fight");
}

#[test]
fn nissa_voice_pumps_the_team_and_makes_plants() {
    let mut g = two_player_game();
    let nissa = g.add_card_to_battlefield(0, catalog::nissa_voice_of_zendikar());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // -2: +1/+1 on each creature you control.
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: nissa, ability_index: 1, target: None, x_value: None })
        .expect("minus two");
    drain_stack(&mut g);
    assert_eq!(counters(&g, a), 1);
    assert_eq!(counters(&g, b), 1);
}

#[test]
fn nissa_voice_plus_one_makes_a_plant() {
    let mut g = two_player_game();
    let nissa = g.add_card_to_battlefield(0, catalog::nissa_voice_of_zendikar());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: nissa, ability_index: 0, target: None, x_value: None })
        .expect("plus one");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Plant").count(), 1, "made a 0/1 Plant");
}

#[test]
fn gyre_sage_taps_for_green_per_counter() {
    let mut g = two_player_game();
    let sage = g.add_card_to_battlefield(0, catalog::gyre_sage());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(sage)), 0, 0);
    g.resolve_effect(
        &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
        &ctx,
    )
    .unwrap();
    g.clear_sickness(sage);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sage, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("mana ability");
    assert_eq!(g.players[0].mana_pool.total(), 2, "{{T}}: two counters → two green mana");
}

#[test]
fn elusive_krasis_is_unblockable_and_evolves() {
    let mut g = two_player_game();
    let krasis = g.add_card_to_battlefield(0, catalog::elusive_krasis());
    assert!(g.battlefield_find(krasis).unwrap().definition.keywords.contains(&Keyword::Unblockable));
    // A bigger creature entering evolves the 0/4 (bear power 2 > 0).
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
    assert_eq!(counters(&g, krasis), 1, "evolve triggered on the entering creature");
}

#[test]
fn corpsejack_menace_doubles_counters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::corpsejack_menace());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(
        &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        &ctx,
    )
    .unwrap();
    assert_eq!(counters(&g, bear), 2, "one counter doubled to two");
}

#[test]
fn prime_speaker_zegana_enters_scaled_and_draws() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let hand = g.players[0].hand.len();
    let zeg = g.move_card_to_battlefield_for_test(0, catalog::prime_speaker_zegana());
    drain_stack(&mut g);
    assert_eq!(counters(&g, zeg), 4, "entered with counters = greatest other power");
    assert_eq!(g.players[0].hand.len(), hand + 5, "drew cards equal to its power (1+4)");
}

#[test]
fn cold_eyed_selkie_has_islandwalk_and_draws_on_damage() {
    let mut g = two_player_game();
    let selkie = g.add_card_to_battlefield(0, catalog::cold_eyed_selkie());
    assert!(g
        .battlefield_find(selkie)
        .unwrap()
        .definition
        .keywords
        .iter()
        .any(|k| matches!(k, Keyword::Landwalk(_))));
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let effect = catalog::cold_eyed_selkie().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { event_amount: 3, ..EffectContext::for_trigger(selkie, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew that many cards");
}

#[test]
fn bioshift_moves_counters() {
    let mut g = two_player_game();
    let from = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let to = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let seed = EffectContext::for_spell(0, Some(Target::Permanent(from)), 0, 0);
    g.resolve_effect(
        &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(3) },
        &seed,
    )
    .unwrap();
    let shift = g.add_card_to_hand(0, catalog::bioshift());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: shift,
        target: Some(Target::Permanent(from)),
        additional_targets: vec![Target::Permanent(to)],
        mode: None,
        x_value: None,
    })
    .expect("cast bioshift");
    drain_stack(&mut g);
    assert_eq!(counters(&g, from), 0);
    assert_eq!(counters(&g, to), 3, "counters moved to the second creature");
}

#[test]
fn woodland_champion_grows_per_token() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::woodland_champion());
    // Resolve a token-minting effect; the token's entry triggers the Champion.
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::treasure_token(),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(counters(&g, champ), 1, "a token entering grew the Champion");
}

#[test]
fn feat_of_resistance_counters_and_protects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let feat = g.add_card_to_hand(0, catalog::feat_of_resistance());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: feat, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast feat");
    drain_stack(&mut g);
    assert_eq!(counters(&g, bear), 1, "got a +1/+1 counter");
    let cp = g.compute_battlefield();
    assert!(cp.iter().find(|c| c.id == bear).unwrap().keywords.contains(&Keyword::Protection(Color::Red)));
}

#[test]
fn travel_preparations_counters_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tp = g.add_card_to_hand(0, catalog::travel_preparations());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(a)),
        DecisionAnswer::Target(Target::Permanent(b)),
    ]));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tp, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    })
    .expect("cast travel preparations");
    drain_stack(&mut g);
    assert_eq!(counters(&g, a), 1);
    assert_eq!(counters(&g, b), 1);
    assert!(catalog::travel_preparations().keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))));
}

#[test]
fn graft_moves_a_counter_to_a_new_creature() {
    // CR 702.58 — Plaxcaster Frogling enters with 3 counters and grafts one
    // onto each creature that enters afterward.
    let mut g = two_player_game();
    let frog = g.move_card_to_battlefield_for_test(0, catalog::plaxcaster_frogling());
    drain_stack(&mut g);
    assert_eq!(counters(&g, frog), 3, "graft enters with 3 +1/+1 counters");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
    assert_eq!(counters(&g, bear), 1, "grafted a counter onto the entrant");
    assert_eq!(counters(&g, frog), 2, "graft source lost the moved counter");
}

#[test]
fn renown_triggers_only_once() {
    // CR 702.111 — renowns once on combat damage; a second trigger is inert.
    let mut g = two_player_game();
    let aven = g.add_card_to_battlefield(0, catalog::stalwart_aven());
    let effect = catalog::stalwart_aven().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(aven, 0, None, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(counters(&g, aven), 1, "renowned with one +1/+1 counter");
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(counters(&g, aven), 1, "already renowned → no second counter");
}

#[test]
fn master_biomancer_scales_entrants_by_its_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::master_biomancer()); // 2/4
    let bear = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    assert_eq!(counters(&g, bear), 2, "entered with counters equal to Biomancer's power");
}

#[test]
fn managorger_also_grows_on_opponent_spells() {
    let mut g = two_player_game();
    let hydra = g.add_card_to_battlefield(0, catalog::managorger_hydra());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("opponent casts");
    drain_stack(&mut g);
    assert_eq!(counters(&g, hydra), 1, "opponent's spell also grows Managorger");
}
