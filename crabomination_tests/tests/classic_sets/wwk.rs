//! Worldwake — Allies, landfall, multikicker and the Zendikon animations.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn play_land(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(id)).expect("play land");
    drain_stack(g);
    id
}

/// Hada Freeblade and the other "put a +1/+1 counter" Allies grow on each Ally
/// entering, including their own.
#[test]
fn wwk_allies_grow_on_each_ally_entering() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // The printed counters are all "you may".
    g.decider = Box::new(ScriptedDecider::new(std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(4)));
    let freeblade = g.add_card_to_battlefield(0, catalog::hada_freeblade());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: freeblade }]);
    drain_stack(&mut g);
    let hunter = g.add_card_to_battlefield(0, catalog::graypelt_hunter());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: hunter }]);
    drain_stack(&mut g);
    let counters = |id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters(freeblade), 2, "its own ETB plus the Hunter's");
    assert_eq!(counters(hunter), 1, "the Hunter saw only its own");
}

/// Harabaz Druid taps for one mana per Ally you control.
#[test]
fn harabaz_druid_scales_with_allies() {
    let mut g = two_player_game();
    let druid = g.add_card_to_battlefield(0, catalog::harabaz_druid());
    g.clear_sickness(druid);
    g.add_card_to_battlefield(0, catalog::hada_freeblade());
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2, "the Druid counts itself and the Freeblade");
}

/// Halimar Excavator mills one per Ally on each Ally entering.
#[test]
fn halimar_excavator_mills_per_ally() {
    let mut g = two_player_game();
    for _ in 0..10 {
        g.add_card_to_library(1, catalog::lightning_bolt());
    }
    let exc = g.add_card_to_battlefield(0, catalog::halimar_excavator());
    g.add_card_to_battlefield(0, catalog::hada_freeblade());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: exc }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 2, "two Allies, two cards milled");
}

/// Calcite Snapper's landfall switches its power and toughness.
#[test]
fn calcite_snapper_switches_pt_on_landfall() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let snapper = g.add_card_to_battlefield(0, catalog::calcite_snapper());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    play_land(&mut g, catalog::island());
    let cp = g.computed_permanent(snapper).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 1), "1/4 becomes 4/1");
}

/// Guardian Zendikon animates the enchanted land into a 2/6 Wall.
#[test]
fn guardian_zendikon_animates_the_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::plains());
    let aura = g.add_card_to_hand(0, catalog::guardian_zendikon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).expect("the land is still there");
    assert_eq!((cp.power, cp.toughness), (2, 6));
    assert!(cp.keywords.contains(&Keyword::Defender), "and it has defender");
    assert!(
        cp.card_types.contains(&crabomination::card::CardType::Land),
        "it's still a land"
    );
}

/// Enclave Elite enters with a +1/+1 counter per multikick.
#[test]
fn enclave_elite_counts_its_kicks() {
    let mut g = two_player_game();
    let elite = g.add_card_to_hand(0, catalog::enclave_elite());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: elite,
        times: 2,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked twice");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(elite).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "two kicks, two counters"
    );
}

/// Khalni Garden enters tapped and mints a Plant.
#[test]
fn khalni_garden_enters_tapped_with_a_plant() {
    let mut g = two_player_game();
    let garden = play_land(&mut g, catalog::khalni_garden());
    assert!(g.battlefield_find(garden).unwrap().tapped);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Plant"), "a 0/1 Plant");
}

/// Dread Statuary animates itself for the turn and stays a land.
#[test]
fn dread_statuary_animates_itself() {
    let mut g = two_player_game();
    let statuary = g.add_card_to_battlefield(0, catalog::dread_statuary());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: statuary, ability_index: 1, target: None, additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(statuary).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land));
}

/// Sejiri Merfolk gains first strike and lifelink only while you control a
/// Plains.
#[test]
fn sejiri_merfolk_needs_a_plains() {
    let mut g = two_player_game();
    let merfolk = g.add_card_to_battlefield(0, catalog::sejiri_merfolk());
    assert!(!g.computed_permanent(merfolk).unwrap().keywords.contains(&Keyword::Lifelink));
    g.add_card_to_battlefield(0, catalog::plains());
    let kws = g.computed_permanent(merfolk).unwrap().keywords.clone();
    assert!(kws.contains(&Keyword::Lifelink) && kws.contains(&Keyword::FirstStrike));
}

/// Rest for the Weary gains 8 with a land drop this turn, 4 without.
#[test]
fn rest_for_the_weary_doubles_on_landfall() {
    let cast = |landfall: bool| {
        let mut g = two_player_game();
        if landfall {
            play_land(&mut g, catalog::plains());
        }
        let rest = g.add_card_to_hand(0, catalog::rest_for_the_weary());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        let before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: rest,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        g.players[0].life - before
    };
    assert_eq!(cast(false), 4, "no land this turn");
    assert_eq!(cast(true), 8, "landfall");
}

/// Novablast Wurm wipes every other creature when it attacks.
#[test]
fn novablast_wurm_wipes_on_attack() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::novablast_wurm());
    g.clear_sickness(wurm);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: wurm,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some(), "the Wurm survives");
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
}

/// Ruin Ghost blinks a land you control.
#[test]
fn ruin_ghost_blinks_a_land() {
    let mut g = two_player_game();
    let ghost = g.add_card_to_battlefield(0, catalog::ruin_ghost());
    g.clear_sickness(ghost);
    let land = g.add_card_to_battlefield(0, catalog::khalni_garden());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ghost,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("blink");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some(), "it came back");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Plant"), "re-triggering its ETB");
}
