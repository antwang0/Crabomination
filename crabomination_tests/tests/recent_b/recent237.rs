//! Functionality tests for `catalog::sets::decks::recent237`.

use crabomination::card::{CardType, Keyword};
use crabomination::catalog;
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

fn door_effect(def: &crabomination::card::CardDefinition, right: bool) -> Effect {
    let room = def.room.as_ref().expect("room card");
    let door = if right { &room.right } else { &room.left };
    door.triggered_abilities[0].effect.clone()
}

/// Restricted Office's unlock destroys all creatures with power 3+.
#[test]
fn restricted_office_wraths_big_creatures() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — safe
    let bigger = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let def = catalog::restricted_office_lecture_hall();
    let src = g.add_card_to_battlefield(0, catalog::restricted_office_lecture_hall());
    let ctx = EffectContext::for_trigger(src, 0, None, 0);
    g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == big), "2/2 survives");
    assert!(!g.battlefield.iter().any(|c| c.id == bigger), "4/4 destroyed");
}

/// Tunnel of Hate grants double strike to a target attacker when you attack.
#[test]
fn tunnel_of_hate_grants_double_strike() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.set_attacking(vec![crabomination::game::types::Attack {
        attacker,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]);
    let def = catalog::ticket_booth_tunnel_of_hate();
    let src = g.add_card_to_battlefield(0, catalog::ticket_booth_tunnel_of_hate());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(attacker)],
        ..EffectContext::for_trigger(src, 0, None, 0)
    };
    g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
    assert!(
        g.computed_permanent(attacker).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "gained double strike",
    );
}

/// Peer Past the Veil discards the hand, then draws one per graveyard card type.
#[test]
fn peer_past_the_veil_draws_by_types() {
    let mut g = two_player_game();
    // Graveyard has a creature and an instant → 2 card types.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // Hand has three cards; library has plenty to draw.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand_before = g.players[0].hand.len();
    g.resolve_effect(&catalog::peer_past_the_veil().effect, &EffectContext::for_spell(0, None, 0, 0))
        .unwrap();
    // Discarded `hand_before` cards (they add types), then drew types-in-gy.
    // GY now holds creature + instant + the discarded creatures → still 2 types.
    assert_eq!(g.players[0].hand.len(), 2, "drew 2 (creature + instant card types)");
    assert!(g.players[0].hand.iter().all(|c| c.definition.is_land()), "drew from library");
}

/// The Swarmweaver makes two Insects and, with delirium, buffs them.
#[test]
fn swarmweaver_tokens_and_delirium_anthem() {
    let mut g = two_player_game();
    let sw = g.add_card_to_battlefield(0, catalog::the_swarmweaver());
    let effect = catalog::the_swarmweaver().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(sw, 0, None, 0)).unwrap();
    let insects: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Insect")
        .map(|c| c.id)
        .collect();
    assert_eq!(insects.len(), 2, "two Insect tokens");
    // No delirium yet → 1/1.
    assert_eq!(g.computed_permanent(insects[0]).unwrap().power, 1, "1/1 without delirium");
    // Seed 4 card types in graveyard → delirium.
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
    g.add_card_to_graveyard(0, catalog::forest()); // land
    g.add_card_to_graveyard(0, catalog::the_swarmweaver()); // artifact
    assert_eq!(g.computed_permanent(insects[0]).unwrap().power, 2, "2/2 with delirium");
    assert!(
        g.computed_permanent(insects[0]).unwrap().keywords.contains(&Keyword::Deathtouch),
        "deathtouch with delirium",
    );
}
