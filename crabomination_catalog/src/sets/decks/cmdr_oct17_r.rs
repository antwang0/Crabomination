//! Commander: 2026-10-17 (`3chree`) closers, second batch — each the last
//! missing card of one or more unseated decks. Tests in
//! `tests/recent_b/cmdr_oct17_r.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, r, u, w};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// Chronomantic Escape — {4}{W}{W} sorcery, suspend 3—{2}{W}. Until your next
/// turn, creatures can't attack you; then it exiles itself with three time
/// counters (suspended again, CR 702.62a).
pub fn chronomantic_escape() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(3, cost(&[generic(2), w()]))],
        ..spell(
            "Chronomantic Escape",
            cost(&[generic(4), w(), w()]),
            CardType::Sorcery,
            Effect::Seq(vec![Effect::CreaturesCantAttackYouUntilYourNextTurn, Effect::ExileSelfSuspended]),
        )
    }
}

/// Aurora Awakener — {6}{G} 7/7 trample Giant Druid. Vivid — ETB: reveal until
/// X permanent cards (X = colors among permanents you control); put them onto
/// the battlefield ("any number" — all of them), the rest on the bottom.
pub fn aurora_awakener() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::RevealUntilMatchingToBattlefield {
            filter: R::Permanent,
            count: Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::ControlledByYou))),
            rest_bottom: true,
        })],
        ..creature("Aurora Awakener", cost(&[generic(6), g()]), vec![CreatureType::Giant, CreatureType::Druid], 7, 7)
    }
}

/// Back for More — {4}{B}{G} instant. Return target creature card from your
/// graveyard to the battlefield; when you do, it fights up to one target
/// creature you don't control (a reflexive trigger, CR 603.12).
pub fn back_for_more() -> CardDefinition {
    spell(
        "Back for More",
        cost(&[generic(4), b(), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::ForEach {
                selector: Selector::LastMoved,
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::Fight {
                            attacker: Selector::TriggerSource,
                            defender: target_filtered(R::Creature.and(R::ControlledByYou.negate())),
                        }),
                    }),
                }),
            },
        ]),
    )
}

/// Sword of Once and Future — {3} Equipment, equip {2}. +2/+2, protection from
/// blue and from black. Combat damage to a player: surveil 2, then you may cast
/// an instant or sorcery with mana value 2 or less from your graveyard free;
/// it's exiled instead of going to the graveyard.
pub fn sword_of_once_and_future() -> CardDefinition {
    CardDefinition {
        name: "Sword of Once and Future",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Protection(Color::Blue), Keyword::Protection(Color::Black)],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
                    Effect::CastUpToFreeFromHandAndGraveyard {
                        count: 1,
                        filter: R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery))
                            .and(R::InGraveyard)
                            .and(R::ManaValueAtMost(2)),
                        total_mana_value: 2,
                    },
                ]),
            }],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Gnarlroot Trapper — {B} 1/1 Elf Druid. {T}, pay 1 life: add {G} for Elf
/// creature spells only. {T}: target attacking Elf you control gains
/// deathtouch until end of turn.
pub fn gnarlroot_trapper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Green])),
                        SpendRestriction::CreatureSpellOfType(CreatureType::Elf),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(
                        R::Creature.and(R::HasCreatureType(CreatureType::Elf)).and(R::ControlledByYou).and(R::IsAttacking),
                    ),
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature("Gnarlroot Trapper", cost(&[b()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 1)
    }
}

/// Rowdy Crew — {2}{R}{R} 3/3 trample Human Pirate. ETB: draw three, discard
/// two at random; if the two share a card type, two +1/+1 counters on it.
pub fn rowdy_crew() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: true },
            Effect::If {
                cond: Predicate::TwoShareACardType(Selector::DiscardedThisResolution { filter: R::Any }),
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Rowdy Crew", cost(&[generic(2), r(), r()]), vec![CreatureType::Human, CreatureType::Pirate], 3, 3)
    }
}

/// Complete the Circuit — {5}{U} instant, convoke. Sorceries have flash for
/// you this turn; your next instant or sorcery spell this turn is copied
/// twice (new targets allowed).
pub fn complete_the_circuit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..spell(
            "Complete the Circuit",
            cost(&[generic(5), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::GrantSorcerySpellsFlashThisTurn,
                Effect::OnYourNextInstantSorceryThisTurn {
                    body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::Const(2) }),
                },
            ]),
        )
    }
}

/// Starlight Spectacular — {2}{W}{W} enchantment. Parade! — at the beginning
/// of combat on your turn, line your creatures up one at a time; each gets
/// +1/+1 until end of turn per creature chosen before it.
pub fn starlight_spectacular() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Parade,
        }],
        ..spell("Starlight Spectacular", cost(&[generic(2), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Soul Partition — {1}{W} instant. Exile target nonland permanent; its owner
/// may play it for as long as it remains exiled, a spell an opponent casts
/// this way costing {2} more.
pub fn soul_partition() -> CardDefinition {
    spell(
        "Soul Partition",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Permanent.and(R::Land.negate())) },
            Effect::GrantMayPlay {
                what: Selector::Target(0),
                duration: MayPlayDuration::WhileExiled,
                to_owner: true,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::OwnedByYou.negate() },
                then: Box::new(Effect::StampMayPlaySurcharge {
                    what: Selector::Target(0),
                    cost: cost(&[generic(2)]),
                    filter: R::Any.negate(),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}
