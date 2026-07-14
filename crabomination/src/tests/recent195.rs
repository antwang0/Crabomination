//! Functionality tests for `catalog::sets::decks::recent195`.

use crate::card::Keyword;
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Malcolm investigates on your second spell each turn.
#[test]
fn malcolm_investigates_on_second_spell() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::malcolm_the_eyes());
    // Cast two cheap spells; the second should mint a Clue.
    for _ in 0..2 {
        let b = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: b, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bolt");
        drain_stack(&mut g);
    }
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "second spell investigated");
}

/// Reach for the Sky pumps and draws when it dies.
#[test]
fn reach_for_the_sky_pumps_and_draws_on_death() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::reach_for_the_sky());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reach for the Sky");
    drain_stack(&mut g);
    let c = g.compute_battlefield();
    let c = c.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 4), "+3/+2");
    assert!(c.keywords.contains(&Keyword::Reach), "granted reach");
    // Destroy the host → Aura goes to graveyard → draw.
    let ctx = crate::game::effects::EffectContext::for_ability(bear, 0, None);
    let evs = g.resolve_effect(&crate::effect::Effect::Destroy { what: crate::effect::Selector::This }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card when the Aura died");
}

/// Tomb Trawler tucks a graveyard card to the bottom of the library.
#[test]
fn tomb_trawler_tucks_graveyard_card() {
    let mut g = two_player_game();
    let trawler = g.add_card_to_battlefield(0, catalog::tomb_trawler());
    g.clear_sickness(trawler);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: trawler, ability_index: 0, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], x_value: None,
    }).expect("tuck");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bolt), "bolt on the bottom");
}

/// Steer Clear scales to 4 damage while you control a Mount.
#[test]
fn steer_clear_mount_scales_damage() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // An attacking 4-toughness creature; 2 wouldn't kill it, 4 does.
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.attacking.push(crate::game::types::Attack {
        attacker: victim,
        target: crate::game::types::AttackTarget::Player(0),
    });
    g.add_card_to_battlefield(0, catalog::drover_grizzly());
    let spell = g.add_card_to_hand(0, catalog::steer_clear());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Steer Clear");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), None, "4 damage with a Mount killed the 4/4");
}
