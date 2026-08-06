//! Stronghold (STH) — `catalog::sets::sth`.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn activate(
    g: &mut GameState,
    id: CardId,
    index: usize,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

/// Flowstone Mauler's {R} trades toughness for power, repeatedly.
#[test]
fn flowstone_mauler_pumps_plus_one_minus_one() {
    let mut g = two_player_game();
    let mauler = g.add_card_to_battlefield(0, catalog::flowstone_mauler());
    g.clear_sickness(mauler);
    g.step = TurnStep::PreCombatMain;
    for _ in 0..2 {
        g.players[0].mana_pool.add(Color::Red, 1);
        activate(&mut g, mauler, 0, None).expect("pump");
        drain_stack(&mut g);
    }
    let cp = g.computed_permanent(mauler).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 3));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Revenant's power and toughness track the creature cards in your graveyard.
#[test]
fn revenant_scales_with_the_graveyard() {
    let mut g = two_player_game();
    let rev = g.add_card_to_battlefield(0, catalog::revenant());
    assert_eq!(g.computed_permanent(rev).unwrap().power, 0);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // not a creature
    let cp = g.computed_permanent(rev).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Stronghold Taskmaster shrinks other black creatures but not itself.
#[test]
fn stronghold_taskmaster_spares_itself() {
    let mut g = two_player_game();
    let boss = g.add_card_to_battlefield(0, catalog::stronghold_taskmaster());
    let other = g.add_card_to_battlefield(1, catalog::dungeon_shade()); // 1/1 black
    let green = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(boss).unwrap().power, 4);
    assert_eq!(g.computed_permanent(other).unwrap().toughness, 0);
    assert_eq!(g.computed_permanent(green).unwrap().power, 2);
}

/// A Spike moves its counters onto another creature.
#[test]
fn spike_soldier_transfers_a_counter() {
    let mut g = two_player_game();
    let spike = g.move_card_to_battlefield_for_test(0, catalog::spike_soldier());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(spike).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3
    );
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(spike);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, spike, 0, Some(Target::Permanent(bear))).expect("transfer");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(spike).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Mogg Maniac throws the damage it takes back at an opponent.
#[test]
fn mogg_maniac_reflects_damage() {
    let mut g = two_player_game();
    let maniac = g.add_card_to_battlefield(0, catalog::mogg_maniac());
    let mut events = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(maniac),
        3,
        None,
        &mut events,
    );
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Wall of Souls mirrors the combat damage it takes; Wall of Essence banks
/// it as life.
#[test]
fn the_walls_convert_the_combat_damage_they_take() {
    let mut g = two_player_game();
    let souls = g.add_card_to_battlefield(0, catalog::wall_of_souls());
    let essence = g.add_card_to_battlefield(0, catalog::wall_of_essence());
    let attackers: Vec<CardId> = (0..2)
        .map(|_| {
            let id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
            g.clear_sickness(id);
            id
        })
        .collect();
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(
        attackers
            .iter()
            .map(|&a| Attack { attacker: a, target: AttackTarget::Player(0) })
            .collect(),
    ))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![
        (souls, attackers[0]),
        (essence, attackers[1]),
    ]))
    .expect("block");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "Wall of Souls sent 2 back");
    assert_eq!(g.players[0].life, 22, "Wall of Essence gained 2");
}

/// Ruination leaves basics alone.
#[test]
fn ruination_spares_basic_lands() {
    let mut g = two_player_game();
    let basic = g.add_card_to_battlefield(0, catalog::forest());
    let nonbasic = g.add_card_to_battlefield(1, catalog::volcanic_island());
    let ctx = EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(&catalog::ruination().effect, &ctx).expect("ruination");
    assert!(g.battlefield_find(basic).is_some());
    assert!(g.battlefield_find(nonbasic).is_none());
}

/// Constant Mists fogs, and its buyback is a land sacrifice rather than mana.
#[test]
fn constant_mists_buys_back_for_a_land() {
    let mut g = two_player_game();
    let mists = g.add_card_to_hand(0, catalog::constant_mists());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let lands = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellBuyback {
        card_id: mists,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("buyback cast");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == mists), "bought back");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.is_land()).count(),
        lands - 1,
        "a land paid the buyback"
    );
}

/// Horn of Greed cantrips off every player's land drop.
#[test]
fn horn_of_greed_draws_for_whoever_played_the_land() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::horn_of_greed());
    let land = g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let hand = g.players[1].hand.len();
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(land)).expect("land drop");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand, "played one, drew one");
}

/// Tortured Existence swaps a creature card in hand for one in the graveyard.
#[test]
fn tortured_existence_swaps_creature_cards() {
    let mut g = two_player_game();
    let engine = g.add_card_to_battlefield(0, catalog::tortured_existence());
    let pitched = g.add_card_to_hand(0, catalog::grizzly_bears());
    let buried = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, engine, 0, Some(Target::Permanent(buried))).expect("swap");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == buried));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitched));
}

/// Rolling Stones lets a Wall attack.
#[test]
fn rolling_stones_lifts_defender_off_walls() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_razors());
    assert!(g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender));
    g.add_card_to_battlefield(0, catalog::rolling_stones());
    assert!(!g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender));
}

/// Mortuary sends your dead creatures back to the top of your library.
#[test]
fn mortuary_recycles_the_dead_onto_your_library() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mortuary());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = vec![];
    g.destroy_permanent(bear, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bear));
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear));
}

/// Contemplation gains a life per spell you cast.
#[test]
fn contemplation_gains_life_per_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::contemplation());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
}

/// Dream Prowler is unblockable only while it attacks alone.
#[test]
fn dream_prowler_is_unblockable_alone() {
    let mut g = two_player_game();
    let prowler = g.add_card_to_battlefield(0, catalog::dream_prowler());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(prowler).unwrap().keywords.contains(&Keyword::Unblockable));

    g.clear_sickness(prowler);
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: prowler,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack alone");
    drain_stack(&mut g);
    assert!(g.computed_permanent(prowler).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Mogg Bombers goes off the moment another creature lands.
#[test]
fn mogg_bombers_detonates_on_the_next_creature() {
    let mut g = two_player_game();
    let bombers = g.add_card_to_battlefield(0, catalog::mogg_bombers());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast a creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bombers).is_none(), "it sacrificed itself");
    assert_eq!(g.players[1].life, 17);
}

/// Hermit Druid digs to a basic and bins everything above it.
#[test]
fn hermit_druid_mills_to_the_first_basic() {
    let mut g = two_player_game();
    let druid = g.add_card_to_battlefield(0, catalog::hermit_druid());
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let basic = g.add_card_to_library(0, catalog::forest());
    g.clear_sickness(druid);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, druid, 0, None).expect("dig");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == basic));
    assert_eq!(g.players[0].graveyard.len(), 3);
}

/// An en-Kor shifts the damage aimed at it onto another of your creatures.
#[test]
fn nomads_en_kor_shifts_damage_to_another_creature() {
    let mut g = two_player_game();
    let kor = g.add_card_to_battlefield(0, catalog::nomads_en_kor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, kor, 0, Some(Target::Permanent(bear))).expect("shield");
    drain_stack(&mut g);

    let mut events = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(kor),
        1,
        None,
        &mut events,
    );
    assert_eq!(g.battlefield_find(kor).unwrap().damage, 0, "the Kor took none");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1, "the Bear took it");
}

/// Crovax eats a creature to grow, and shrinks on the turns he doesn't.
#[test]
fn crovax_grows_or_shrinks_each_upkeep() {
    let mut g = two_player_game();
    let crovax = g.move_card_to_battlefield_for_test(0, catalog::crovax_the_cursed());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(crovax).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4
    );
    let def = catalog::crovax_the_cursed();
    let ctx = EffectContext::for_ability(crovax, 0, None);
    // Nothing else to eat: he wastes away.
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("upkeep");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(crovax).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3
    );
    // With a snack he'll take, he grows.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("upkeep");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(crovax).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4
    );
}

/// Endangered Armodon bails as soon as you control anything fragile.
#[test]
fn endangered_armodon_flees_a_fragile_board() {
    let mut g = two_player_game();
    let armodon = g.add_card_to_battlefield(0, catalog::endangered_armodon());
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(armodon).is_some(), "4/5 alone is fine");

    g.add_card_to_battlefield(0, catalog::nomads_en_kor()); // 1/1
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(armodon).is_none());
}

/// Walking Dream stays tapped while an opponent has two creatures.
#[test]
fn walking_dream_stays_tapped_against_a_real_board() {
    let mut g = two_player_game();
    let dream = g.add_card_to_battlefield(0, catalog::walking_dream());
    g.battlefield_find_mut(dream).unwrap().tapped = true;
    assert!(!g.untap_prevented_by_static(dream));
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    assert!(g.untap_prevented_by_static(dream));
}

/// Intruder Alarm locks every seat's untap step, not just its controller's.
#[test]
fn intruder_alarm_locks_both_seats() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::intruder_alarm());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.untap_prevented_by_static(theirs));
    assert!(g.untap_prevented_by_static(mine));
}

/// Volrath's Laboratory names a colour and a type on entry, then prints them.
#[test]
fn volraths_laboratory_prints_the_chosen_type() {
    let mut g = two_player_game();
    let lab = g.add_card_to_battlefield(0, catalog::volraths_laboratory());
    let ctx = EffectContext::for_ability(lab, 0, None);
    g.resolve_effect(&catalog::volraths_laboratory().triggered_abilities[0].effect, &ctx)
        .expect("etb");
    let chosen = g.battlefield_find(lab).unwrap().chosen_creature_type;
    assert!(chosen.is_some(), "a creature type was named");

    g.clear_sickness(lab);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(5);
    activate(&mut g, lab, 0, None).expect("mint");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.is_token).expect("token");
    assert_eq!((token.definition.power, token.definition.toughness), (2, 2));
    assert_eq!(token.definition.subtypes.creature_types.first().copied(), chosen);
}

/// Portcullis swallows the third creature and gives it back when it leaves.
#[test]
fn portcullis_holds_the_third_creature() {
    let mut g = two_player_game();
    let gate = g.add_card_to_battlefield(0, catalog::portcullis());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let third = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: third }]);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == third), "the third is held");

    let mut events = vec![];
    g.destroy_permanent(gate, false, &mut events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(third).is_some(), "and released");
}

/// Hesitation eats itself to counter the next spell anyone casts.
#[test]
fn hesitation_counters_the_next_spell() {
    let mut g = two_player_game();
    let hes = g.add_card_to_battlefield(0, catalog::hesitation());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hes).is_none(), "it sacrificed itself");
    assert_eq!(g.players[0].life, 20, "the Bolt was countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

/// Shard Phoenix sweeps the ground and buys itself back from the graveyard.
#[test]
fn shard_phoenix_sweeps_then_returns() {
    let mut g = two_player_game();
    let phoenix = g.add_card_to_battlefield(0, catalog::shard_phoenix());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, phoenix, 0, None).expect("sweep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ground).is_none());
    assert!(g.battlefield_find(flier).is_some(), "fliers are spared");

    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.players[0].mana_pool.add(Color::Red, 3);
    activate(&mut g, phoenix, 1, None).expect("return");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == phoenix));
}
