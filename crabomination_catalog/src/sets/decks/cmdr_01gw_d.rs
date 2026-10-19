//! Commander: 2026-10-19 (`01GwrDKP`) closers, fourth batch — Grub, Storied
//! Matriarch's Goblins; Noctis, Heir Apparent. Tests in
//! `tests/recent_b/cmdr_01gw_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardId, CardType, CreatureType, EventKind, EventScope, EventSpec,
    MayPlayDuration, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, generic, u, w, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CardType>, subtypes: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: types,
        subtypes: Subtypes { creature_types: subtypes, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn goblin() -> R {
    R::HasCreatureType(CreatureType::Goblin)
}

/// Champion of the Weird — {3}{B} 5/5 Goblin Berserker. As an additional cost,
/// behold a Goblin and exile it. Pay 1 life, blight 2: target opponent blights
/// 2 (sorcery speed). Leaving the battlefield returns the exiled card to its
/// owner's hand.
pub fn champion_of_the_weird() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::BeholdAndExile { filter: goblin() }],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            blight_cost: 2,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                crate::effect::shortcut::declare_target_opponent(),
                Effect::EachPlayerDoes { who: PlayerRef::Target(0), body: Box::new(Effect::Blight { n: Value::Const(2) }) },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::CardExiledWithSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::CardExiledWithSource))),
            },
        }],
        ..creature(
            "Champion of the Weird",
            cost(&[generic(3), b()]),
            vec![CardType::Creature],
            vec![CreatureType::Goblin, CreatureType::Berserker],
            5,
            5,
        )
    }
}

/// Taster of Wares — {2}{B} 3/2 Goblin Warlock. ETB, target opponent reveals X
/// cards from their hand (X = Goblins you control); you choose one and they
/// exile it. An instant or sorcery exiled this way you may cast while this is
/// on the battlefield, spending mana as though it were any type.
pub fn taster_of_wares() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            crate::effect::shortcut::declare_target_opponent(),
            Effect::RevealFromHandExileChosen {
                who: PlayerRef::Target(0),
                count: Value::count(Selector::EachPermanent(goblin().and(R::ControlledByYou))),
            },
            Effect::GrantMayPlay {
                what: Selector::MatchingAmong {
                    inner: Box::new(Selector::LastMoved),
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
                duration: MayPlayDuration::WhileSourceOnBattlefield { source: CardId(0), one_spell_a_turn: false },
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
        ]))],
        ..creature(
            "Taster of Wares",
            cost(&[generic(2), b()]),
            vec![CardType::Creature],
            vec![CreatureType::Goblin, CreatureType::Warlock],
            3,
            2,
        )
    }
}

/// Noctis, Heir Apparent — {W}{U}{B} 2/3 legendary Human Noble. Whenever a
/// creature you control enters during combat, you may attach target Equipment
/// you control to target creature you control. Warp-Strike — {3}: exile
/// Noctis; it returns under its owner's control tapped and attacking at the
/// beginning of that player's next declare attackers step, unblockable that
/// combat.
pub fn noctis_heir_apparent() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Keyword, Supertype};
    use crate::effect::{DelayedTriggerKind, Duration, Predicate};
    use crate::game::types::TurnStep;
    let during_combat = Predicate::Any(
        [TurnStep::BeginCombat, TurnStep::DeclareAttackers, TurnStep::DeclareBlockers, TurnStep::FirstStrikeDamage, TurnStep::CombatDamage, TurnStep::EndCombat]
            .into_iter()
            .map(Predicate::CurrentStepIs)
            .collect(),
    );
    CardDefinition {
        name: "Noctis, Heir Apparent",
        cost: cost(&[w(), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Noble], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                during_combat,
            ])),
            effect: Effect::MayDo {
                description: "Attach an Equipment?".into(),
                body: Box::new(Effect::Attach {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou),
                    },
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) },
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::YourNextDeclareAttackersStep,
                    capture: Selector::This,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::TargetFiltered { slot: 0, filter: R::InExile },
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                                tapped: true,
                            },
                        },
                        Effect::JoinCombatAttackingChosen { what: Selector::LastMoved, cleanup: Default::default() },
                        Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Unblockable, duration: Duration::EndOfCombat },
                    ])),
                },
                Effect::Move { what: Selector::This, to: ZoneDest::Exile },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
