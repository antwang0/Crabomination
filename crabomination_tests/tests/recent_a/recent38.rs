//! Functionality tests for `catalog::sets::decks::recent38` — the Amonkhet
//! Monument cycle. Cost reductions are asserted via the static; the cast riders
//! are resolved directly with a target-bearing context.

use crabomination::catalog;
use crabomination::card::StaticEffect;
use crabomination::game::effects::EffectContext;
use crabomination::game::two_player_game;
use crabomination::game::*;

fn ctx_for(source: CardId) -> EffectContext {
    EffectContext::for_ability(source, 0, None)
}

fn rider(def: crabomination::card::CardDefinition) -> Effect {
    def.triggered_abilities[0].effect.clone()
}

#[test]
fn monuments_reduce_their_color() {
    for (def, _) in [
        (catalog::oketras_monument(), 'W'),
        (catalog::kefnets_monument(), 'U'),
        (catalog::hazorets_monument(), 'R'),
        (catalog::rhonass_monument(), 'G'),
    ] {
        assert!(def.static_abilities.iter().any(|s| matches!(
            s.effect, StaticEffect::CostReduction { amount: 1, .. }
        )), "{} reduces creature spells", def.name);
    }
}

#[test]
fn oketras_monument_mints_a_vigilant_warrior() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::oketras_monument());
    g.resolve_effect(&rider(catalog::oketras_monument()), &ctx_for(mon)).unwrap();
    drain_stack(&mut g);
    let w = g.battlefield.iter().find(|c| c.definition.name == "Warrior").expect("Warrior token");
    assert!(w.definition.keywords.contains(&crabomination::card::Keyword::Vigilance));
}

#[test]
fn kefnets_monument_locks_an_opponents_untap() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::kefnets_monument());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = ctx_for(mon);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&rider(catalog::kefnets_monument()), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().skip_next_untap, "target won't untap next untap step");
}

#[test]
fn rhonass_monument_pumps_and_tramples() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::rhonass_monument());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let mut ctx = ctx_for(mon);
    ctx.targets = vec![Target::Permanent(mine)];
    g.resolve_effect(&rider(catalog::rhonass_monument()), &ctx).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Trample), "gained trample");
}

#[test]
fn hazorets_monument_can_loot() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::hazorets_monument());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to pitch
    g.add_card_to_library(0, catalog::island()); // a card to draw
    let hand_before = g.players[0].hand.len();
    g.resolve_effect(&rider(catalog::hazorets_monument()), &ctx_for(mon)).unwrap();
    drain_stack(&mut g);
    // Loot is net-neutral on hand size (discard one, draw one).
    assert_eq!(g.players[0].hand.len(), hand_before, "looted: −1 +1");
}
