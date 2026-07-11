//! Functionality tests for `catalog::sets::decks::recent146` (WOE deferred).

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::game::two_player_game;
use crate::mana::Color;

/// Resolve a standalone effect controlled by `player` (test helper).
fn resolve_for(g: &mut GameState, player: usize, effect: crate::effect::Effect) {
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_ability(src, player, None);
    let events = g.resolve_effect(&effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Bitter Chill's enchanted creature doesn't untap during its controller's
/// untap step.
#[test]
fn bitter_chill_locks_untap() {
    let mut g = two_player_game();
    let chill = g.add_card_to_battlefield(0, catalog::bitter_chill());
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(chill).unwrap().attached_to = Some(creature);
    g.battlefield_find_mut(creature).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(creature).unwrap().tapped, "locked creature stays tapped");
}

/// Syr Ginger gains trample/hexproof/haste only while an opponent controls a
/// planeswalker.
#[test]
fn syr_ginger_planeswalker_keywords() {
    let mut g = two_player_game();
    let ginger = g.add_card_to_battlefield(0, catalog::syr_ginger_the_meal_ender());
    assert!(
        !g.computed_permanent(ginger).unwrap().keywords.contains(&Keyword::Trample),
        "no keywords without an opposing planeswalker",
    );
    g.add_card_to_battlefield(1, catalog::karn_scion_of_urza());
    let kws = g.computed_permanent(ginger).unwrap().keywords;
    assert!(kws.contains(&Keyword::Trample), "trample once opponent has a planeswalker");
    assert!(kws.contains(&Keyword::Hexproof), "hexproof too");
    assert!(kws.contains(&Keyword::Haste), "haste too");
}

/// A friendly artifact dying grows Syr Ginger; a friendly creature dying does
/// not (the "another artifact" filter).
#[test]
fn syr_ginger_artifact_death_counter() {
    let mut g = two_player_game();
    let ginger = g.add_card_to_battlefield(0, catalog::syr_ginger_the_meal_ender());
    g.add_card_to_battlefield(0, catalog::mind_stone());
    // Destroy only the noncreature artifact (Syr Ginger is an artifact creature).
    resolve_for(&mut g, 0, crate::effect::Effect::Destroy {
        what: crate::effect::Selector::EachPermanent(
            crate::card::SelectionRequirement::Artifact
                .and(crate::card::SelectionRequirement::Noncreature),
        ),
    });
    assert_eq!(
        g.battlefield_find(ginger).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "an artifact dying added a +1/+1 counter",
    );
}

/// Archon of the Wild Rose sets your other enchanted creatures to base 4/4 with
/// flying; an unenchanted creature is untouched.
#[test]
fn archon_buffs_enchanted_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::archon_of_the_wild_rose());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Enchant only `bear` with an Aura.
    let aura = g.add_card_to_battlefield(0, catalog::pacifism());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let buffed = g.computed_permanent(bear).unwrap();
    assert_eq!((buffed.power, buffed.toughness), (4, 4), "enchanted creature is base 4/4");
    assert!(buffed.keywords.contains(&Keyword::Flying), "and has flying");
    let plain_c = g.computed_permanent(plain).unwrap();
    assert_eq!((plain_c.power, plain_c.toughness), (2, 2), "unenchanted creature unchanged");
}

/// Back for Seconds returns two creatures to hand when not bargained.
#[test]
fn back_for_seconds_unbargained_returns_two() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::back_for_seconds());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Back for Seconds");
    drain_stack(&mut g);
    let creatures_in_hand =
        g.players[0].hand.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(creatures_in_hand, 2, "both creatures returned to hand");
}

/// When bargained, Back for Seconds may reanimate a MV≤4 creature.
#[test]
fn back_for_seconds_bargained_reanimates() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let fodder = g.add_token_to_battlefield(0, &crate::game::effects::treasure_token());
    let spell = g.add_card_to_hand(0, catalog::back_for_seconds());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true), // accept the reanimation
    ]));
    g.perform_action(GameAction::CastSpellBargain {
        card_id: spell,
        sacrifice: Some(fodder),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bargained Back for Seconds");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.id == dead),
        "the MV≤4 creature returned to the battlefield",
    );
}

/// Faunsbane Troll saccs its Monster Role to fight; the fought creature is
/// exiled instead of dying.
#[test]
fn faunsbane_troll_fight_exiles() {
    let mut g = two_player_game();
    // The ETB mints a Monster Role attached to the Troll.
    let troll = g.move_card_to_battlefield_for_test(0, catalog::faunsbane_troll());
    drain_stack(&mut g);
    g.clear_sickness(troll);
    let role = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Monster" && c.attached_to == Some(troll))
        .expect("Monster Role attached")
        .id;
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: troll,
        ability_index: 0,
        target: Some(Target::Permanent(prey)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Faunsbane fight");
    drain_stack(&mut g);
    assert!(g.battlefield_find(role).is_none(), "Monster Role sacrificed as a cost");
    assert!(g.exile.iter().any(|c| c.id == prey), "fought creature exiled, not in graveyard");
    assert!(g.battlefield_find(troll).is_some(), "Troll survives the 2-power back-swing");
}

/// Horned Loch-Whale enters tapped on an opponent's turn, untapped on yours.
#[test]
fn horned_loch_whale_conditional_enters_tapped() {
    let mut g = two_player_game();
    g.active_player_idx = 1; // opponent's turn
    let whale = g.move_card_to_battlefield_for_test(0, catalog::horned_loch_whale());
    assert!(g.battlefield_find(whale).unwrap().tapped, "enters tapped on opponent's turn");

    g.active_player_idx = 0; // your turn
    let whale2 = g.move_card_to_battlefield_for_test(0, catalog::horned_loch_whale());
    assert!(!g.battlefield_find(whale2).unwrap().tapped, "enters untapped on your turn");
}
