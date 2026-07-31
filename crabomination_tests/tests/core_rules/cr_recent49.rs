//! CR conformance for this run's sweep:
//! - CR 114 — emblems.
//! - CR 201 — names (the 201.4a restricted namespace).
//! - CR 304 — instants.

use crabomination::card::{CardType, Keyword};
use crabomination::catalog;
use crabomination::decision::{Decision, DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;

// ── CR 114 — Emblems ──

/// CR 114.2 — an emblem is owned and controlled by the player the effect names,
/// and only that player.
#[test]
fn cr_114_2_emblem_goes_to_its_own_players_command_zone() {
    let mut g = two_player_game();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::CreateEmblem {
            who: PlayerRef::You,
            name: "Test Emblem".into(),
            triggered: vec![],
            statics: vec![],
        },
        &ctx,
    )
    .expect("emblem");
    assert_eq!(g.players[0].emblems.len(), 1);
    assert!(g.players[1].emblems.is_empty(), "the opponent gets nothing");
}

/// CR 114.4 / 114.5 — an emblem's abilities function from the command zone,
/// and the emblem is not a permanent (nothing joins the battlefield).
#[test]
fn cr_114_4_emblem_abilities_function_in_the_command_zone() {
    let mut g = two_player_game();
    let before = g.battlefield.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::CreateEmblem {
            who: PlayerRef::You,
            name: "Upkeep Emblem".into(),
            triggered: vec![crabomination::card::TriggeredAbility {
                event: crabomination::card::EventSpec::new(
                    crabomination::card::EventKind::StepBegins(TurnStep::Upkeep),
                    crabomination::card::EventScope::YourControl,
                ),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: crabomination::card::Value::Const(2),
                },
            }],
            statics: vec![],
        },
        &ctx,
    )
    .expect("emblem");
    assert_eq!(g.battlefield.len(), before, "an emblem is not a permanent");
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22, "the emblem's trigger fired from the command zone");
}

// ── CR 201 — Name ──

/// CR 201.4a — "choose a land card name" only accepts land names; an
/// off-namespace answer names nothing.
#[test]
fn cr_201_4a_name_choice_honors_its_namespace() {
    let named = |answer: &str| {
        let mut g = two_player_game();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard(
            answer.to_string(),
        )]));
        let hamlet = g.add_card_to_hand(0, catalog::petrified_hamlet());
        g.perform_action(GameAction::PlayLand(hamlet)).expect("play");
        drain_stack(&mut g);
        g.battlefield_find(hamlet).unwrap().named_card.clone()
    };
    assert_eq!(named("Island"), Some("Island".to_string()), "a land name sticks");
    assert_eq!(named("Lightning Bolt"), None, "a nonland name isn't a legal choice");
}

/// CR 201.4a — the suggestion feed offered to the chooser is filtered to the
/// allowed namespace too, so an auto-decider can't name outside it.
#[test]
fn cr_201_4a_suggestions_are_filtered_to_the_namespace() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::lightning_bolt());
    }
    g.add_card_to_battlefield(1, catalog::petrified_hamlet());
    let hamlet = g.add_card_to_hand(0, catalog::petrified_hamlet());
    g.perform_action(GameAction::PlayLand(hamlet)).expect("play");
    drain_stack(&mut g);
    // The Bolts outnumber the land, but only land names are on offer.
    assert_eq!(
        g.battlefield_find(hamlet).unwrap().named_card.as_deref(),
        Some("Petrified Hamlet"),
    );
}

/// CR 201.2a — objects share a name when their names match; the name-keyed
/// grant reaches every same-named land and nothing else.
#[test]
fn cr_201_2a_same_name_objects_share_the_named_grant() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard(
        "Island".to_string(),
    )]));
    let island = g.add_card_to_battlefield(0, catalog::island());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let hamlet = g.add_card_to_hand(0, catalog::petrified_hamlet());
    g.perform_action(GameAction::PlayLand(hamlet)).expect("play");
    drain_stack(&mut g);
    assert_eq!(g.granted_abilities_for(island).len(), 1, "the Island picks up '{{T}}: Add {{C}}'");
    assert!(g.granted_abilities_for(forest).is_empty(), "the Forest doesn't share the name");
}

// ── CR 304 — Instants ──

/// CR 304.4 — an instant can't enter the battlefield; it stays where it was.
#[test]
fn cr_304_4_instants_cant_enter_the_battlefield() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crabomination::card::Zone::Hand,
                filter: crabomination::card::SelectionRequirement::HasCardType(CardType::Instant),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        &ctx,
    )
    .expect("move");
    assert!(g.battlefield_find(bolt).is_none(), "the instant never lands");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "it stays in hand");
}

/// CR 304.2 — a resolved instant goes to its owner's graveyard.
#[test]
fn cr_304_2_resolved_instant_goes_to_its_owners_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "owner's graveyard");
}

/// CR 304.1 — an instant is castable whenever its controller has priority,
/// including on an opponent's turn; a sorcery in the same window is not.
#[test]
fn cr_304_1_instants_ignore_the_sorcery_speed_window() {
    let try_cast = |def: crabomination::card::CardDefinition| {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareBlockers;
        let id = g.add_card_to_hand(1, def);
        g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[1].mana_pool.add_colorless(4);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok()
    };
    assert!(try_cast(catalog::lightning_bolt()), "an instant may be cast on their turn");
    assert!(!try_cast(catalog::spire_barrage()), "a sorcery may not");
}

/// CR 304.5 — a Flash permanent uses the same "any time you could cast an
/// instant" window without being an instant.
#[test]
fn cr_304_5_flash_uses_the_instant_window() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareBlockers;
    let mut def = catalog::grizzly_bears();
    def.keywords.push(Keyword::Flash);
    let bear = g.add_card_to_hand(1, def);
    g.players[1].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok(),
        "flash creature castable in the blocker step"
    );
}

/// The `Decision::NameCard` prompt carries its namespace noun so a UI seat can
/// say what it's allowed to name (CR 201.4a).
#[test]
fn cr_201_4a_prompt_carries_the_namespace_noun() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let hamlet = g.add_card_to_hand(0, catalog::petrified_hamlet());
    g.perform_action(GameAction::PlayLand(hamlet)).expect("play");
    drain_stack(&mut g);
    let pd = g.pending_decision.as_ref().expect("the name prompt is pending");
    match &pd.decision {
        Decision::NameCard { restriction, .. } => {
            assert_eq!(restriction.as_deref(), Some("land"))
        }
        other => panic!("expected NameCard, got {other:?}"),
    }
}
