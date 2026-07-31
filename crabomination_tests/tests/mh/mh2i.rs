//! Functionality tests for `catalog::sets::decks::mh2i` — MH2 sweep batch 10.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

/// CR 702.26 — Out of Time phases all creatures out until it leaves.
#[test]
fn cr_702_26_out_of_time_linked_phase() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::mahamoti_djinn());
    let oot = g.add_card_to_hand(0, catalog::out_of_time());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, oot);
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "own creature phased out");
    assert!(g.battlefield_find(theirs).is_none(), "opponent creature phased out");
    assert_eq!(g.phased_out.len(), 2);
    assert_eq!(
        g.battlefield_find(oot).unwrap().counter_count(CounterType::Time),
        2,
        "a time counter per phased creature"
    );
    // Untap steps do NOT phase them in while Out of Time remains.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(mine).is_none(), "still phased out on untap");
    // Vanishing ticks Out of Time down; when it leaves, everything returns.
    g.active_player_idx = 0;
    let events = g.process_fading_vanishing();
    g.dispatch_triggers_for_events(&events);
    let events = g.process_fading_vanishing();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(oot).is_none(), "vanishing sacrificed it");
    assert!(g.battlefield_find(mine).is_some(), "phased back in");
    assert!(g.battlefield_find(theirs).is_some());
}

/// CR 601.3e — suspend-only cards can't be cast from hand.
#[test]
fn cr_601_3e_suspend_only_cast_rejected() {
    let mut g = two_player_game();
    let will = g.add_card_to_hand(0, catalog::gaeas_will());
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: will, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(err, Err(GameError::NoManaCost)));
}

/// Gaea's Will opens the graveyard for the turn and exiles graveyard-bound cards.
#[test]
fn gaeas_will_graveyard_turn() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::island());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::gaeas_will().effect, &ctx).unwrap();
    assert!(g.player_may_play_lands_from_graveyard(0));
    // The bear got a pay-own-cost permission for this turn.
    let bear_card = g.players[0].graveyard.iter().find(|c| c.id == bear).unwrap();
    assert!(bear_card.may_play_until.is_some());
    // A card bound for this player's graveyard is exiled instead.
    let milled = g.add_card_to_library(0, catalog::island());
    let card = g.players[0].library.remove(0);
    let mut events = vec![];
    assert!(g.route_to_graveyard(card, &mut events), "redirected to exile");
    assert!(g.exile.iter().any(|c| c.id == milled));
}

/// Inevitable Betrayal steals a creature from the opponent's library.
#[test]
fn inevitable_betrayal_steals() {
    let mut g = two_player_game();
    let djinn = g.add_card_to_library(1, catalog::mahamoti_djinn());
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1)];
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(djinn))]));
    let events = g.resolve_effect(&catalog::inevitable_betrayal().effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert!(
        g.battlefield.iter().any(|c| {
            c.definition.name == "Mahamoti Djinn" && c.controller == 0 && c.owner == 1
        }),
        "stolen onto my battlefield"
    );
}

/// Glimpse of Tomorrow rerolls the owner's board.
#[test]
fn glimpse_of_tomorrow_reroll() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::parcel_myr());
    let theirs = g.add_card_to_battlefield(1, catalog::mahamoti_djinn());
    // The library holds two permanents to flip into play.
    g.add_card_to_library(0, catalog::chrome_mox());
    g.add_card_to_library(0, catalog::steel_dromedary());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&Effect::GlimpseOfTomorrow, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    let mine = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(mine, 2, "revealed as many as were shuffled in");
    assert!(g.battlefield_find(theirs).is_some(), "opponent board untouched");
}

/// Garth One-Eye mints each classic once.
#[test]
fn garth_one_eye_classics() {
    let mut g = two_player_game();
    let garth = g.add_card_to_battlefield(0, catalog::garth_one_eye());
    g.clear_sickness(garth);
    // First activation: pick Black Lotus (index 5).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(5)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: garth, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("choose");
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Black Lotus"),
        "copy of Black Lotus created"
    );
    assert_eq!(g.battlefield_find(garth).unwrap().name_choices_used, 1 << 5);
    // Second activation: Lotus is spent; Mode(5) now maps into the remaining
    // five names, so Lotus can't be picked again.
    g.battlefield_find_mut(garth).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(5)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: garth, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("choose again");
    drain_stack(&mut g);
    let lotuses = g.players[0].hand.iter().filter(|c| c.definition.name == "Black Lotus").count();
    assert_eq!(lotuses, 1, "each name only once");
}

/// Dermotaxi imprints a corpse and impersonates it for a turn.
#[test]
fn dermotaxi_imprint_copy() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(1, catalog::mahamoti_djinn());
    let taxi = g.add_card_to_hand(0, catalog::dermotaxi());
    let c1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: taxi, target: Some(Target::Permanent(corpse)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with imprint target");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == corpse), "imprinted in exile");
    g.perform_action(GameAction::ActivateAbility {
        card_id: taxi, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("crew-copy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(c1).unwrap().tapped && g.battlefield_find(c2).unwrap().tapped);
    assert_eq!(g.battlefield_find(taxi).unwrap().definition.name, "Mahamoti Djinn");
}

/// Chef's Kiss steals a burn spell and rethrows it at a random legal target.
#[test]
fn chefs_kiss_steal_and_copy() {
    let mut g = two_player_game();
    // Opponent aims a bolt at my bear; I kiss it.
    let my_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let their_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::slaying_fire());
    g.players[1].mana_pool.add(Color::Red, 3);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(my_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(bolt)];
    g.resolve_effect(&Effect::ChefsKiss, &ctx).unwrap();
    assert_eq!(g.stack.len(), 2, "original + copy");
    for si in &g.stack {
        let crabomination::game::types::StackItem::Spell { caster, target, .. } = si else {
            panic!("spell items")
        };
        assert_eq!(*caster, 0, "I control both");
        // Retargeted away from me: the opponent's bear or the opponent.
        assert!(
            matches!(target, Some(Target::Permanent(id)) if *id == their_bear)
                || matches!(target, Some(Target::Player(1))),
            "never me or mine: {target:?}"
        );
    }
    let _ = my_bear;
}

/// CR 604.3 — Grist is a creature everywhere but the battlefield.
#[test]
fn cr_604_3_grist_off_battlefield_creature() {
    let mut g = two_player_game();
    let in_gy = g.add_card_to_graveyard(0, catalog::grist_the_hunger_tide());
    // A graveyard filter sees it as a creature…
    assert!(g.evaluate_requirement_static(
        &crabomination::card::SelectionRequirement::Creature,
        &Target::Permanent(in_gy),
        0,
        None,
    ));
    // …but on the battlefield it's only a planeswalker.
    let on_bf = g.add_card_to_battlefield(0, catalog::grist_the_hunger_tide());
    assert!(!g.evaluate_requirement_static(
        &crabomination::card::SelectionRequirement::Creature,
        &Target::Permanent(on_bf),
        0,
        None,
    ));
}

/// Grist +1 mints Insects and snowballs off milled Insect cards.
#[test]
fn grist_plus_one_snowball() {
    let mut g = two_player_game();
    let grist = g.add_card_to_battlefield(0, catalog::grist_the_hunger_tide());
    g.battlefield_find_mut(grist).unwrap().counters.insert(CounterType::Loyalty, 3);
    // Top: another Grist card (an Insect off-battlefield!) then an Island.
    g.add_card_to_library(0, catalog::grist_the_hunger_tide());
    g.add_card_to_library(0, catalog::island());
    g.priority.player_with_priority = 0;
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: grist, ability_index: 0, target: None, x_value: None,
    }).expect("+1");
    drain_stack(&mut g);
    let insects = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Insect" && c.is_token)
        .count();
    assert_eq!(insects, 2, "repeat fired once off the milled Grist");
    // 3 (base) + 1 (cost) + 1 (milled Insect) = 5.
    assert_eq!(g.battlefield_find(grist).unwrap().counter_count(CounterType::Loyalty), 5);
}

/// Grist −5 drains for creature cards in your graveyard (Grist included).
#[test]
fn grist_minus_five() {
    let mut g = two_player_game();
    let grist = g.add_card_to_battlefield(0, catalog::grist_the_hunger_tide());
    g.battlefield_find_mut(grist).unwrap().counters.insert(CounterType::Loyalty, 6);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grist_the_hunger_tide()); // counts too
    g.add_card_to_graveyard(0, catalog::island()); // not a creature
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: grist, ability_index: 2, target: None, x_value: None,
    }).expect("-5");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "two creature cards in the graveyard");
}

/// Braingeyser draws X for the target player.
#[test]
fn braingeyser_draws_x() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 3);
    ctx.targets = vec![Target::Player(0)];
    g.resolve_effect(&catalog::braingeyser().effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), 3);
}
