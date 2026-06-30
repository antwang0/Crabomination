//! Functionality tests for `catalog::sets::decks::recent32` — aristocrats /
//! sacrifice-matters payoffs and the `CantAttackOrBlockUnlessCreatureDiedThisTurn`
//! combat gate (Bontu the Glorified).

use crate::card::Keyword;
use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::two_player_game;
use crate::game::*;

fn ctx_for(source: CardId) -> EffectContext {
    EffectContext::for_ability(source, 0, None)
}

fn ability0_effect(def: crate::card::CardDefinition) -> crate::card::Effect {
    def.activated_abilities.into_iter().next().unwrap().effect
}

#[test]
fn bloodflow_connoisseur_grows_on_sacrifice() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::bloodflow_connoisseur());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder
    g.resolve_effect(&ability0_effect(catalog::bloodflow_connoisseur()), &ctx_for(id)).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "a +1/+1 counter from the sacrifice");
}

#[test]
fn vampire_aristocrat_pumps_on_sacrifice() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::vampire_aristocrat());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.resolve_effect(&ability0_effect(catalog::vampire_aristocrat()), &ctx_for(id)).unwrap();
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

#[test]
fn cartel_aristocrat_gains_protection() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cartel_aristocrat());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.resolve_effect(&ability0_effect(catalog::cartel_aristocrat()), &ctx_for(id)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "another creature was sacrificed");
    assert!(g.computed_permanent(id).unwrap().keywords.iter()
        .any(|k| matches!(k, Keyword::Protection(_))), "gained protection from a color");
}

#[test]
fn yahenni_grows_when_opponent_creature_dies() {
    let mut g = two_player_game();
    let yah = g.add_card_to_battlefield(0, catalog::yahenni_undying_partisan());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().damage = 2; // lethal → CreatureDied
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(yah).unwrap().power, 3, "+1/+1 counter from the opponent's death");
}

#[test]
fn bontu_gate_opens_after_a_creature_dies() {
    let mut g = two_player_game();
    let bontu = g.add_card_to_battlefield(0, catalog::bontu_the_glorified());
    assert!(g.computed_permanent(bontu).unwrap().keywords
        .contains(&Keyword::CantAttackOrBlockUnlessCreatureDiedThisTurn));
    // Make Bontu an otherwise-legal attacker.
    g.active_player_idx = 0;
    g.step = crate::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.battlefield_find_mut(bontu).unwrap().summoning_sick = false;
    assert!(!g.legal_attackers(0).contains(&bontu), "gate shut with no death this turn");
    g.players[0].creatures_died_this_turn = 1;
    assert!(g.legal_attackers(0).contains(&bontu), "gate opens once a creature died under your control");
}

#[test]
fn bontu_ability_drains_each_opponent() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    let bontu = g.add_card_to_battlefield(0, catalog::bontu_the_glorified());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    let before = g.players[1].life;
    // Bontu's ability is the second activated ability slot here (the only one).
    g.resolve_effect(&ability0_effect(catalog::bontu_the_glorified()), &ctx_for(bontu)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "each opponent loses 1");
    assert_eq!(g.players[0].life, 21, "you gain 1");
}

#[test]
fn smothering_abomination_draws_when_you_sacrifice() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::smothering_abomination());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    // The upkeep trigger sacrifices a creature; that sacrifice fires the draw.
    let upkeep = catalog::smothering_abomination().triggered_abilities[0].effect.clone();
    let evs = g.resolve_effect(&upkeep, &EffectContext::for_ability(crate::card::CardId(0), 0, None)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.is_creature()),
        "a creature was sacrificed at upkeep");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "the sacrifice drew a card");
}

#[test]
fn butcher_ghoul_returns_via_undying() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::butcher_ghoul());
    let mut ctx = ctx_for(id);
    ctx.targets = vec![Target::Permanent(id)];
    g.resolve_effect(&crate::card::Effect::Destroy { what: crate::effect::Selector::Target(0) }, &ctx).unwrap();
    drain_stack(&mut g);
    let ghoul = g.battlefield.iter().find(|c| c.definition.name == "Butcher Ghoul")
        .expect("Undying returned it to the battlefield");
    assert_eq!(ghoul.counter_count(crate::card::CounterType::PlusOnePlusOne), 1, "returns with a +1/+1 counter");
}

#[test]
fn elas_il_kor_gains_life_on_creature_etb() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.add_card_to_battlefield(0, catalog::elas_il_kor_sadistic_pilgrim());
    // Cast another creature so its ETB event dispatches to Elas's trigger.
    g.active_player_idx = 0;
    g.step = crate::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Grizzly Bears castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21, "gained 1 from another creature entering");
}

#[test]
fn mahadi_makes_treasures_for_the_dead() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mahadi_emporium_master());
    g.players[0].creatures_died_this_turn = 2; // two creatures died this turn
    let end_step = catalog::mahadi_emporium_master().triggered_abilities[0].effect.clone();
    g.resolve_effect(&end_step, &ctx_for(id)).unwrap();
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure").count();
    assert_eq!(treasures, 2, "one Treasure per creature that died this turn");
}

#[test]
fn heartless_summoning_shrinks_your_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::heartless_summoning());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "-1/-1 to your creatures");
}
