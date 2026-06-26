//! Phyrexia: All Will Be One — Incubate (CR 701.53). The Incubator token enters
//! with N +1/+1 counters; `{2}: Transform` flips it to a 0/0 Phyrexian creature
//! (so it becomes N/N).

use crate::card::{CardType, CounterType};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};

/// Resolve `effect` as though `player` were its controller.
fn resolve_for(g: &mut GameState, player: usize, effect: crate::effect::Effect) {
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_ability(src, player, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(g);
}

/// Incubate 3 mints an Incubator with three +1/+1 counters; transforming it
/// yields a 3/3 Phyrexian artifact creature (counters persist, CR 712).
#[test]
fn incubate_then_transform_to_n_over_n() {
    let mut g = two_player_game();
    resolve_for(&mut g, 0, crate::effect::Effect::Incubate {
        who: crate::effect::PlayerRef::You,
        amount: crate::effect::Value::Const(3),
    });
    let inc = g.battlefield.iter().find(|c| c.definition.name == "Incubator").expect("Incubator minted");
    let inc_id = inc.id;
    assert_eq!(inc.counter_count(CounterType::PlusOnePlusOne), 3, "three +1/+1 counters");
    let cp = g.computed_permanent(inc_id).unwrap();
    assert!(cp.card_types.contains(&CardType::Artifact) && !cp.card_types.contains(&CardType::Creature),
        "front is a noncreature artifact");
    // {2}: Transform.
    g.players[0].mana_pool.add_colorless(2);
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: inc_id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("transform the Incubator");
    drain_stack(&mut g);
    let back = g.computed_permanent(inc_id).unwrap();
    assert!(back.card_types.contains(&CardType::Creature), "back is a creature");
    assert_eq!((back.power, back.toughness), (3, 3), "0/0 base + three +1/+1 = 3/3");
}

/// Eyes of Gitaxias incubates 3 and draws a card.
#[test]
fn eyes_of_gitaxias_incubates_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    resolve_for(&mut g, 0, catalog::eyes_of_gitaxias().effect);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Incubator"), "Incubator minted");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

/// Injector Crocodile incubates 3 when it dies.
#[test]
fn injector_crocodile_incubates_on_death() {
    let mut g = two_player_game();
    let croc = g.add_card_to_battlefield(0, catalog::injector_crocodile());
    let ctx = crate::game::effects::EffectContext::for_ability(croc, 0, None);
    g.resolve_effect(
        &crate::effect::Effect::SacrificePermanent { what: crate::effect::Selector::Target(0) },
        &crate::game::effects::EffectContext { targets: vec![crate::game::types::Target::Permanent(croc)], ..ctx },
    ).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Incubator"), "death incubated 3");
}

/// Sunfall exiles all creatures and incubates X = the number exiled.
#[test]
fn sunfall_exiles_all_and_incubates_count() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    resolve_for(&mut g, 0, catalog::sunfall().effect); // resolve_for adds one more bear (4 total)
    assert!(!g.battlefield.iter().any(|c| c.definition.is_creature() && c.definition.name != "Incubator"),
        "all creatures exiled");
    let inc = g.battlefield.iter().find(|c| c.definition.name == "Incubator").expect("Incubator minted");
    assert_eq!(inc.counter_count(CounterType::PlusOnePlusOne), 4, "X = 4 creatures exiled");
}

/// Phyrexian Awakening's static grants vigilance to your Phyrexians.
#[test]
fn phyrexian_awakening_anthem_grants_vigilance() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::phyrexian_awakening());
    let croc = g.add_card_to_battlefield(0, catalog::injector_crocodile()); // a Phyrexian
    assert!(g.computed_permanent(croc).unwrap().keywords.contains(&Keyword::Vigilance));
}
