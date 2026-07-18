//! Functionality tests for `catalog::sets::decks::recent249` (suspect + artifact
//! Detectives, during-your-turn hexproof).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};

fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
}

/// Clandestine Meddler suspects another creature you control on ETB (not itself).
#[test]
fn clandestine_meddler_suspects_another_creature() {
    let mut g = two_player_game();
    let meddler = g.add_card_to_battlefield(0, catalog::clandestine_meddler());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::clandestine_meddler().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_trigger(meddler, 0, None, 0);
    ctx.targets = vec![Target::Permanent(ally)];
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(ally).unwrap().suspected, "the other creature is suspected");
    assert!(!g.battlefield_find(meddler).unwrap().suspected, "not itself");
}

/// Forensic Gadgeteer investigates whenever you cast an artifact spell.
#[test]
fn forensic_gadgeteer_investigates_on_artifact_cast() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let _gadgeteer = g.add_card_to_battlefield(0, catalog::forensic_gadgeteer());
    let artifact = g.add_card_to_hand(0, catalog::magnifying_glass()); // {3} artifact
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: artifact,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast an artifact spell");
    drain_stack(&mut g);
    assert_eq!(clues(&g, 0), 1, "casting an artifact investigated");
}

/// Pompous Gadabout has hexproof only during its controller's turn.
#[test]
fn pompous_gadabout_hexproof_your_turn_only() {
    let mut g = two_player_game();
    let gad = g.add_card_to_battlefield(0, catalog::pompous_gadabout());
    g.active_player_idx = 0;
    assert!(
        g.computed_permanent(gad).unwrap().keywords.contains(&Keyword::Hexproof),
        "hexproof on your turn"
    );
    g.active_player_idx = 1;
    assert!(
        !g.computed_permanent(gad).unwrap().keywords.contains(&Keyword::Hexproof),
        "no hexproof on the opponent's turn"
    );
}

/// Clandestine Meddler's second trigger fires off "you attack with a suspected
/// creature" — its filter reads the suspected attacker.
#[test]
fn clandestine_meddler_attack_trigger_gated_on_suspected() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::effect::Predicate;
    let def = catalog::clandestine_meddler();
    let attack_trig = &def.triggered_abilities[1];
    let ok = matches!(
        &attack_trig.event.filter,
        Some(Predicate::AttackedWithCreatureMatching { filter, .. }) if *filter == R::IsSuspected
    );
    assert!(ok, "the attack surveil is gated on attacking with a suspected creature");
}
