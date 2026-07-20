//! Comprehensive-Rules conformance for behaviours touched this batch:
//! CR 702.16 (protection from monocolored — Guardian of the Guildpact),
//! CR 303.4 ("whenever an Aura becomes attached to this creature" self-trigger
//! via `EventScope::SelfSource` — Bramble Elemental), and CR 122.5 (moving
//! counters is a transfer, not a "put", so `Doubling Season` does NOT double
//! the moved amount — Simic Guildmage).

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game, Target};
use crabomination::game::GameAction;

/// CR 702.16 — protection from monocolored: a source of exactly one color can't
/// target the protected permanent; a multicolored source can.
#[test]
fn cr_702_16_protection_from_monocolored_gates_targeting() {
    let mut g = two_player_game();
    let guardian = g.add_card_to_battlefield(1, catalog::guardian_of_the_guildpact());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // mono-red
    for c in [crabomination::mana::Color::Black, crabomination::mana::Color::Red] {
        g.players[0].mana_pool.add(c, 3);
    }
    let mono = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(guardian)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(mono.is_err(), "monocolored spell can't target it");
    let ball = g.add_card_to_hand(0, catalog::wrecking_ball()); // {2}{B}{R}
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ball, target: Some(Target::Permanent(guardian)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("multicolored spell targets fine");
    drain_stack(&mut g);
    assert!(g.battlefield_find(guardian).is_none(), "multicolored removal resolved");
}

/// CR 303.4 — a "whenever an Aura becomes attached to this creature" ability is
/// a SelfSource trigger: it fires only for the *enchanted* creature, not for
/// every copy on the battlefield.
#[test]
fn cr_303_4_aura_attached_self_trigger_is_source_specific() {
    let mut g = two_player_game();
    let enchanted = g.add_card_to_battlefield(0, catalog::bramble_elemental());
    let _other = g.add_card_to_battlefield(0, catalog::bramble_elemental());
    let aura = g.add_card_to_hand(0, catalog::fists_of_ironwood());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(enchanted)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant one Bramble");
    drain_stack(&mut g);
    let saprolings = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Saproling")
        .count();
    // 2 from Fists' own ETB + 2 from only the enchanted Bramble's trigger. If
    // SelfSource leaked, the second Bramble would add 2 more (6 total).
    assert_eq!(saprolings, 4, "only the enchanted Bramble's attach trigger fired");
}

/// CR 122.5 — moving counters preserves counter identity (it's not "putting"
/// counters), so `Doubling Season` does not double the moved amount.
#[test]
fn cr_122_5_moving_counters_is_not_doubled_by_doubling_season() {
    let mut g = two_player_game();
    let _ds = g.add_card_to_battlefield(0, catalog::doubling_season());
    let gm = g.add_card_to_battlefield(0, catalog::simic_guildmage());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dst = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(src).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 0, target: Some(Target::Permanent(src)),
        additional_targets: vec![Target::Permanent(dst)], x_value: None,
    }).expect("move counter");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(dst).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "exactly one counter moved — Doubling Season doesn't apply to a move (CR 122.5)"
    );
}

/// "When this card is put into your hand from your graveyard" (Golgari
/// Brownscale) fires off the card now in hand — via any graveyard→hand return
/// (`EventKind::PutIntoHandFromGraveyard`), here a Recollect.
#[test]
fn put_into_hand_from_graveyard_trigger_fires_on_return() {
    let mut g = two_player_game();
    let brownscale = g.add_card_to_graveyard(0, catalog::golgari_brownscale());
    let recollect = g.add_card_to_hand(0, catalog::recollect()); // return a gy card to hand
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: recollect, target: Some(Target::Permanent(brownscale)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Recollect on Brownscale");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == brownscale), "Brownscale returned to hand");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life from the return trigger");
}
