//! Functionality tests for `catalog::sets::decks::recent233` (OTJ Spree).

use crabomination::catalog;
use crabomination::effect::{Effect, SpreeMode};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

fn spree_modes(def: &crabomination::card::CardDefinition) -> Vec<SpreeMode> {
    match &def.effect {
        Effect::Spree { modes } => modes.clone(),
        _ => panic!("not a spree card"),
    }
}

/// Metamorphic Blast's shrink mode turns a creature into a 0/1 Rabbit.
#[test]
fn metamorphic_blast_shrinks_to_rabbit() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let modes = spree_modes(&catalog::metamorphic_blast());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(bear)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&modes[0].effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 1), "becomes a 0/1");
    assert!(cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Rabbit), "a Rabbit");
}

/// Metamorphic Blast's draw mode draws two for the targeted player.
#[test]
fn metamorphic_blast_draws_two() {
    let mut g = two_player_game();
    for i in 0..3 {
        g.players[0].add_to_library_top(crabomination::card::CardId(9200 + i), catalog::mountain());
    }
    let modes = spree_modes(&catalog::metamorphic_blast());
    let before = g.players[0].hand.len();
    let ctx = EffectContext {
        targets: vec![Target::Player(0)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&modes[1].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 2, "target player draws two");
}

/// Return the Favor's copy mode duplicates an instant/sorcery spell on the
/// stack (the copy lands on the stack above the original).
#[test]
fn return_the_favor_copies_a_spell() {
    let mut g = two_player_game();
    // Put a Lightning Bolt on the stack targeting a creature.
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(crabomination::game::GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    let stack_before = g.stack.len();
    let modes = spree_modes(&catalog::return_the_favor());
    // The spell on the stack is targeted by its own card id.
    let ctx = EffectContext {
        targets: vec![Target::Permanent(bolt)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&modes[0].effect, &ctx).unwrap();
    assert!(g.stack.len() > stack_before, "a copy was added to the stack");
}
