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

// ── CR 603 — instance-granted triggers reach every hook ──

/// A trigger granted by a resolution (`Effect::GrantTriggeredAbility`) fires
/// off the two hooks that push their own kinds — the step walk and the cast
/// walk — exactly like a printed one, on a permanent with no printed trigger
/// at all (so the trigger member list would not have visited it). Neither
/// hook read the grant map before 2026-09-08; no catalog grant is of either
/// kind, so this is the pin, not a card.
#[test]
fn cr_603_2_instance_granted_step_and_cast_triggers_fire() {
    use crabomination::effect::shortcut::{gain_life, magecraft};
    use crabomination::effect::{Duration, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crabomination::game::effects::EffectContext;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_ability(bear, 0, None);
    for trigger in [
        magecraft(gain_life(3)),
        TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: gain_life(5),
        },
    ] {
        let grant = Effect::GrantTriggeredAbility {
            what: Selector::This,
            trigger: Box::new(trigger),
            duration: Duration::EndOfTurn,
        };
        g.resolve_effect(&grant, &ctx).expect("grant resolves");
    }
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "the granted magecraft fired off the cast walk");
    for _ in 0..20 {
        if g.step == TurnStep::End {
            break;
        }
        let _ = g.advance_step(Vec::new());
    }
    assert_eq!(g.step, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 8, "the granted end-step trigger fired off the step walk");
    let _ = g.advance_step(Vec::new());
    let _ = g.advance_step(Vec::new());
    assert!(g.granted_triggers_timed.is_empty(), "both grants ended at cleanup");
}

/// CR 603.2 — the three combat listener walks (`ControllerAttackedByOpponent`,
/// `YouAttack`, `ControllerDealtCombatDamage`) read instance-granted triggers
/// too, on a permanent with no printed trigger. Before 2026-09-08 all three
/// walked the printed member list only, so a grant of any of the kinds (none
/// in the catalog today) would have been dropped silently.
#[test]
fn cr_603_2_instance_granted_combat_listeners_fire() {
    use crabomination::effect::shortcut::gain_life;
    use crabomination::effect::{Duration, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{Attack, AttackTarget};
    let grant = |g: &mut GameState, host: CardId, kind: EventKind, scope: EventScope, life: i32| {
        let ctx = EffectContext::for_ability(host, 0, None);
        let grant = Effect::GrantTriggeredAbility {
            what: Selector::This,
            trigger: Box::new(TriggeredAbility { event: EventSpec::new(kind, scope), effect: gain_life(life) }),
            duration: Duration::EndOfTurn,
        };
        g.resolve_effect(&grant, &ctx).expect("grant resolves");
    };
    // The opponent attacks me / deals me combat damage: the listener is on my
    // bear (no printed trigger), the attacker theirs.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    grant(&mut g, mine, EventKind::Attacks, EventScope::ControllerAttackedByOpponent, 3);
    grant(&mut g, mine, EventKind::ControllerDealtCombatDamage, EventScope::SelfSource, 7);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(theirs);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let life = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: theirs,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "the granted attacked-by-opponent listener fired");
    g.step = TurnStep::CombatDamage;
    let evs = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3 - 2 + 7, "the granted combat-damage-to-me listener fired");
    // I attack: the "whenever you attack" listener is on a land of mine.
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    grant(&mut g, forest, EventKind::YouAttack, EventScope::SelfSource, 5);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    let life = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 5, "the granted you-attack listener fired");
}

// ── CR 602.5 — "Activate only during your turn" ──

/// CR 602.5 — an "activate only during your turn" ability is refused on the
/// opponent's turn and accepted at instant speed on yours; "before attackers
/// are declared" (Rag Man, Stern Marshal) is also refused from the declare-
/// attackers step on. Every card here shipped `sorcery_speed` as the
/// approximation until 2026-09-08 — a sorcery gate refuses your own upkeep
/// and beginning of combat, which the printed rider allows.
#[test]
fn cr_602_5_activate_only_during_your_turn_is_a_turn_gate_not_a_sorcery_gate() {
    /// (factory, ability index, prints "before attackers are declared").
    type Row = (fn() -> crabomination::card::CardDefinition, usize, bool);
    let cards: [Row; 8] = [
        (catalog::stern_marshal, 0, true),
        (catalog::rag_man, 0, true),
        (catalog::wishclaw_talisman, 0, false),
        (catalog::june_bounty_hunter, 0, false),
        (catalog::professor_zei_anthropologist, 1, false),
        (catalog::path_to_redemption, 0, false),
        (catalog::bitter_work, 0, false),
        (catalog::nebuchadnezzar, 0, false),
    ];
    for (def, idx, before_attackers) in cards {
        let name = def().name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def());
        g.clear_sickness(id);
        if let Some((kind, _)) = def().enters_with_counters {
            g.battlefield_find_mut(id).unwrap().counters.insert(kind, 3);
        }
        let gate = |g: &mut GameState, active: usize, step: TurnStep| {
            g.active_player_idx = active;
            g.step = step;
            g.priority.player_with_priority = 0;
            matches!(
                g.perform_action(GameAction::ActivateAbility {
                    card_id: id,
                    ability_index: idx,
                    target: None,
                    additional_targets: vec![],
                    x_value: None,
                    mode: None,
                }),
                Err(crabomination::game::GameError::AbilityConditionNotMet)
            )
        };
        assert!(gate(&mut g, 1, TurnStep::PreCombatMain), "{name}: refused on the opponent's turn");
        assert!(!gate(&mut g, 0, TurnStep::Upkeep), "{name}: your upkeep is not a sorcery window, and it is allowed");
        assert!(!gate(&mut g, 0, TurnStep::BeginCombat), "{name}: allowed at beginning of combat");
        assert_eq!(
            gate(&mut g, 0, TurnStep::DeclareBlockers),
            before_attackers,
            "{name}: the before-attackers rider decides declare blockers"
        );
    }
}

// ── CR 611.2 — a turn-gated live-filter grant ──

/// CR 611.2 — `WhileYourTurn` around a `GrantKeyword` whose filter is
/// stateful (`IsOutlaw` reads the computed type line) applies on your turn
/// and not on the opponent's. The stateful gather path matched the static
/// raw — no wrapper peel — so the grant emitted nothing on either path until
/// 2026-09-08 (found by At Knifepoint's re-shape; the card ships the
/// `only_your_turn` anthem, this pins the wrapper).
#[test]
fn cr_611_2_your_turn_wrapper_reaches_a_live_filter_keyword_grant() {
    use crabomination::card::{CreatureType, SelectionRequirement as R, StaticAbility, StaticEffect};
    let mut g = two_player_game();
    let mut lord = catalog::grizzly_bears();
    lord.static_abilities = vec![StaticAbility {
        description: "During your turn, outlaws you control have first strike.",
        effect: StaticEffect::WhileYourTurn {
            inner: Box::new(StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::IsOutlaw.and(R::ControlledByYou)),
                keyword: Keyword::FirstStrike,
            }),
        },
    }];
    g.add_card_to_battlefield(0, lord);
    let mut rogue = catalog::grizzly_bears();
    rogue.subtypes.creature_types = vec![CreatureType::Rogue];
    let outlaw = g.add_card_to_battlefield(0, rogue);
    let fs = |g: &GameState| g.computed_permanent(outlaw).unwrap().keywords().contains(&Keyword::FirstStrike);
    g.active_player_idx = 0;
    assert!(fs(&g), "granted on your turn");
    g.active_player_idx = 1;
    assert!(!fs(&g), "not on the opponent's");
}
