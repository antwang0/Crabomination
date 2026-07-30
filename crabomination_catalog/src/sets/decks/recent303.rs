//! Dissension batch 2: the multicolored-matters Eidolon cycle (a graveyard
//! self-return on `EventScope::FromYourGraveyard`) plus Hellbent/Defender
//! utility. Tests in `recent_b/recent_303`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};

/// "Whenever you cast a multicolored spell, you may return this from your
/// graveyard to your hand" — the shared Eidolon trigger (CR
/// `EventScope::FromYourGraveyard`).
fn eidolon_recur() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Multicolored,
            },
        ),
        effect: Effect::MayDo {
            description: "return this from your graveyard to your hand".into(),
            body: Box::new(Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
    }
}

/// A 2/2 Spirit Eidolon with `activated` as its sac ability plus the shared
/// multicolored graveyard-recur trigger.
fn eidolon(
    name: &'static str,
    color_cost: crate::mana::ManaCost,
    activated: ActivatedAbility,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: color_cost,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![activated],
        triggered_abilities: vec![eidolon_recur()],
        ..Default::default()
    }
}

/// Enigma Eidolon — {3}{U}. {U}, Sacrifice: target player mills three.
pub fn enigma_eidolon() -> CardDefinition {
    eidolon(
        "Enigma Eidolon",
        cost(&[generic(3), u()]),
        ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::Const(3),
            },
            ..Default::default()
        },
    )
}

/// Sandstorm Eidolon — {3}{R}. {R}, Sacrifice: target creature can't block this turn.
pub fn sandstorm_eidolon() -> CardDefinition {
    eidolon(
        "Sandstorm Eidolon",
        cost(&[generic(3), r()]),
        ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        },
    )
}

/// Verdant Eidolon — {3}{G}. {G}, Sacrifice: add three mana of any one color.
pub fn verdant_eidolon() -> CardDefinition {
    eidolon(
        "Verdant Eidolon",
        cost(&[generic(3), g()]),
        ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(3)),
            },
            ..Default::default()
        },
    )
}

/// Entropic Eidolon — {3}{B}. {B}, Sacrifice: target player loses 1, you gain 1.
pub fn entropic_eidolon() -> CardDefinition {
    eidolon(
        "Entropic Eidolon",
        cost(&[generic(3), b()]),
        ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: target_filtered(R::Player),
                    amount: Value::ONE,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        },
    )
}

/// Aurora Eidolon — {3}{W}. {W}, Sacrifice: prevent the next 3 damage to any target.
pub fn aurora_eidolon() -> CardDefinition {
    eidolon(
        "Aurora Eidolon",
        cost(&[generic(3), w()]),
        ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_cost: true,
            effect: Effect::PreventNextDamage {
                target: crate::effect::shortcut::target_any(),
                amount: Value::Const(3),
            },
            ..Default::default()
        },
    )
}

/// Ragamuffyn — {2}{B} 2/2 Zombie Cleric. Hellbent — {T}, Sacrifice a creature
/// or land: Draw a card. Activate only if you have no cards in hand.
pub fn ragamuffyn() -> CardDefinition {
    CardDefinition {
        name: "Ragamuffyn",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature.or(R::Land), 1)),
            condition: Some(Predicate::HellbentActive {
                who: PlayerRef::You,
            }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Soulsworn Jury — {2}{W} 1/4 Spirit. Defender; {1}{U}, Sacrifice this
/// creature: Counter target creature spell.
pub fn soulsworn_jury() -> CardDefinition {
    CardDefinition {
        name: "Soulsworn Jury",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            sac_cost: true,
            effect: Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Creature))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stoic Ephemera — {2}{W} 5/5 Spirit. Defender, flying. When it blocks,
/// sacrifice it at end of combat.
pub fn stoic_ephemera() -> CardDefinition {
    CardDefinition {
        name: "Stoic Ephemera",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Defender, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::DelayUntil {
                kind: DelayedTriggerKind::EndOfCombat,
                body: Box::new(Effect::SacrificePermanent {
                    what: Selector::This,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Demon's Jester — {3}{B} 2/2 Imp. Flying; Hellbent — gets +2/+1 as long as
/// you have no cards in hand.
pub fn demons_jester() -> CardDefinition {
    CardDefinition {
        name: "Demon's Jester",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Hellbent — gets +2/+1 while you have no cards in hand.",
            effect: crate::card::StaticEffect::PumpSelfIf {
                condition: Predicate::HellbentActive {
                    who: PlayerRef::You,
                },
                power: 2,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Minister of Impediments — {2}{W/U} 1/1 Human Advisor. {T}: Tap target creature.
pub fn minister_of_impediments() -> CardDefinition {
    CardDefinition {
        name: "Minister of Impediments",
        cost: cost(&[generic(2), hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Flame-Kin War Scout — {3}{R} 2/4 Elemental Scout. When another creature
/// enters, sacrifice this creature and it deals 4 damage to that creature.
pub fn flame_kin_war_scout() -> CardDefinition {
    CardDefinition {
        name: "Flame-Kin War Scout",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::SacrificePermanent {
                    what: Selector::This,
                },
                Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(4),
                },
            ]),
        }],
        ..Default::default()
    }
}
