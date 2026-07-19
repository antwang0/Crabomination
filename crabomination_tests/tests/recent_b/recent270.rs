//! Functionality tests for `catalog::sets::decks::recent270`.

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game};

/// Black Market Tycoon mints a Treasure, and its upkeep bites for 2 per Treasure.
#[test]
fn black_market_tycoon_treasure_and_bite() {
    let mut g = two_player_game();
    let tycoon = g.add_card_to_battlefield(0, catalog::black_market_tycoon());
    g.clear_sickness(tycoon);
    // Tap for a Treasure.
    g.perform_action(GameAction::ActivateAbility {
        card_id: tycoon,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "made a Treasure"
    );
    // The upkeep trigger deals 2 per Treasure (one here → 2 damage).
    let effect = catalog::black_market_tycoon().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(tycoon, 0, None);
    let life = g.players[0].life;
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].life, life - 2, "2 damage for one Treasure");
}

/// Balduvian Atrocity reanimates a small creature only when kicked.
#[test]
fn balduvian_atrocity_kicked_reanimates() {
    let mut g = two_player_game();
    // A bear in the graveyard to reanimate.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let effect = catalog::balduvian_atrocity().triggered_abilities[0].effect.clone();
    // Unkicked: nothing returns.
    let ctx = EffectContext::for_ability(
        crabomination::game::CardId(999),
        0,
        None,
    );
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(
        !g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "unkicked: no reanimation"
    );
    // Kicked: the bear returns with haste.
    let target = g.players[0].graveyard.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    let mut kctx = EffectContext::for_ability(crabomination::game::CardId(999), 0, Some(Target::Permanent(target)));
    kctx.kicked = true;
    g.resolve_effect(&effect, &kctx).unwrap();
    drain_stack(&mut g);
    let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("reanimated");
    assert!(bear.has_keyword(&crabomination::card::Keyword::Haste), "gained haste");
}
