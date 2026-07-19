//! Functionality tests for `catalog::sets::decks::recent267`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game, CardId, GameState};
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Akki Scrapchomper sacrifices an artifact to draw.
#[test]
fn akki_scrapchomper_sacs_for_a_card() {
    let mut g = two_player_game();
    let akki = g.add_card_to_battlefield(0, catalog::akki_scrapchomper());
    g.clear_sickness(akki); // {T} cost needs no summoning sickness
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: akki,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone).is_none(), "artifact sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Argothian Opportunist makes a tapped Powerstone on ETB.
#[test]
fn argothian_opportunist_makes_tapped_powerstone() {
    let mut g = two_player_game();
    let opp = g.add_card_to_hand(0, catalog::argothian_opportunist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, opp, None);
    let ps = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Powerstone")
        .expect("Powerstone token");
    assert!(ps.tapped, "Powerstone enters tapped");
}

/// Ashnod's Intervention pumps and returns the creature to hand when it dies.
#[test]
fn ashnods_intervention_returns_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let effect = catalog::ashnods_intervention().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0");
    // Destroy it — the granted trigger returns it to its owner's hand.
    let dctx = EffectContext::for_ability(bear, 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &dctx).unwrap();
    g.dispatch_triggers_for_events(&[]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature left the battlefield");
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "returned to owner's hand"
    );
}

/// Gnawing Crescendo pumps the team and spawns a Rat when a nontoken dies.
#[test]
fn gnawing_crescendo_pumps_and_makes_rats() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let effect = catalog::gnawing_crescendo().effect;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "team +2/+0");
    // The nontoken bear dies (via SBA) → a Rat appears.
    g.battlefield_find_mut(bear).unwrap().damage = 100;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let rat = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Rat")
        .expect("Rat token");
    assert!(rat.definition.keywords.contains(&Keyword::CantBlock), "Rat can't block");
}

/// Angelic Intervention grants protection and a +1/+1 counter.
#[test]
fn angelic_intervention_protects_and_counters() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Black)]));
    let effect = catalog::angelic_intervention().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 counter");
    assert!(
        g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Protection(Color::Black)),
        "protected from black"
    );
}

/// Alabaster Host Intercessor exiles an opponent's creature until it leaves.
#[test]
fn alabaster_host_intercessor_exiles_until_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // opponent 4/4
    let inter = g.add_card_to_hand(0, catalog::alabaster_host_intercessor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, inter, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "opponent creature exiled");
    // Destroy the Intercessor → the exiled creature returns.
    let iid = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Alabaster Host Intercessor")
        .unwrap()
        .id;
    let dctx = EffectContext::for_ability(iid, 0, Some(Target::Permanent(iid)));
    g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &dctx).unwrap();
    g.dispatch_triggers_for_events(&[]);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Serra Angel"),
        "exiled creature returned"
    );
}
