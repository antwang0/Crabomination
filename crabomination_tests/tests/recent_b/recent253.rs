//! Functionality tests for `catalog::sets::decks::recent253` (Ravnica legends).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Trostani, Three Whispers grants deathtouch to a target creature.
#[test]
fn trostani_grants_deathtouch() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let trostani = g.add_card_to_battlefield(0, catalog::trostani_three_whispers());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: trostani,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate the deathtouch ability");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch),
        "bear gained deathtouch",
    );
}

/// Ezrim investigates twice on ETB and gains a chosen keyword by sacrificing an
/// artifact.
#[test]
fn ezrim_investigates_and_grants_chosen_keyword() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let ezrim = g.add_card_to_battlefield(0, catalog::ezrim_agency_chief());
    g.fire_self_etb_triggers(ezrim, 0);
    drain_stack(&mut g);
    let clues = g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count();
    assert_eq!(clues, 2, "investigated twice");
    // The keyword-grant ability carries a "sacrifice an artifact" cost.
    let ability = &catalog::ezrim_agency_chief().activated_abilities[0];
    assert!(ability.sac_other_filter.is_some(), "sacrifice-an-artifact cost present");
    // Resolve the modal grant, choosing lifelink (mode 1).
    let ctx = EffectContext::for_trigger(ezrim, 0, None, 1);
    g.resolve_effect(&ability.effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(ezrim).unwrap().keywords.contains(&Keyword::Lifelink),
        "Ezrim gained the chosen keyword (lifelink)",
    );
}

/// Agrus Kos suspects a clean creature, then exiles it once it's suspected.
#[test]
fn agrus_kos_suspects_then_exiles() {
    let mut g = two_player_game();
    let agrus = g.add_card_to_battlefield(0, catalog::agrus_kos_spirit_of_justice());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let interrogate = catalog::agrus_kos_spirit_of_justice().triggered_abilities[0].effect.clone();
    // First interrogation: not suspected → suspect it.
    let ctx = EffectContext::for_trigger(agrus, 0, Some(Target::Permanent(foe)), 0);
    g.resolve_effect(&interrogate, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().suspected, "creature suspected");
    // Second interrogation: already suspected → exile it.
    let ctx = EffectContext::for_trigger(agrus, 0, Some(Target::Permanent(foe)), 0);
    g.resolve_effect(&interrogate, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "suspected creature exiled");
    assert!(g.exile.iter().any(|c| c.id == foe), "moved to exile");
}
