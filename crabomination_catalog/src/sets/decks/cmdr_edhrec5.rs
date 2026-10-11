//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, fifth file (`cmdr_edhrec4` is the
//! fourth). Lord of the Nazgûl's Wraiths and Ring spells. Tests in
//! `tests/recent_b/cmdr_edhrec5.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{ActivatedAbility, DelayedTriggerKind, Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{b, cost, generic, u, Color, ManaCost};

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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn tempt() -> Effect {
    Effect::RingTempts { who: PlayerRef::You }
}

fn target_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::Target(0), filter }
}

// ── Lord of the Nazgûl (UB) ──────────────────────────────────────────────────

/// Gandalf, Friend of the Shire — {3}{U} 2/4 flash. Your sorceries have
/// flash; the Ring tempting you onto a bearer other than Gandalf draws a card.
pub fn gandalf_friend_of_the_shire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "You may cast sorcery spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: R::HasCardType(CardType::Sorcery) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            // CR 701.54c — the chosen bearer is the event's subject; no
            // creature chosen leaves the player there, which isn't "other".
            event: EventSpec::new(EventKind::RingTempted, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend(
            "Gandalf, Friend of the Shire",
            cost(&[generic(3), u()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            2,
            4,
        )
    }
}

/// Ringwraiths — {4}{B}{B} 5/5. On entry an opponent's creature gets -3/-3
/// and a legendary one's controller loses 3; the Ring tempting you returns it
/// from your graveyard to your hand.
pub fn ringwraiths() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: target_is(R::HasSupertype(Supertype::Legendary)),
                    then: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                        amount: Value::Const(3),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::RingTempted, EventScope::FromYourGraveyard),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            },
        ],
        ..creature(
            "Ringwraiths",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Wraith, CreatureType::Knight],
            5,
            5,
        )
    }
}

/// Sauron, the Necromancer — {3}{B}{B} 4/4 menace. Attacking exiles a creature
/// card from your graveyard for a tapped, attacking 3/3 black menace Wraith
/// copy of it, exiled at the next end step unless Sauron is your Ring-bearer.
pub fn sauron_the_necromancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            // CR 707.2 — the copy reads the card's printed values, so it is
            // minted before the card leaves; the exile follows in the same
            // resolution.
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::Creature.from_your_graveyard()),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: Some((3, 3)),
                override_colors: Some(vec![Color::Black]),
                enters_tapped: true,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![Keyword::Menace],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
            Effect::Exile { what: Selector::Target(0) },
            // CR 707.9b — "a Wraith instead of its other creature types".
            Effect::AmendCopiableValues {
                what: Selector::LastCreatedToken,
                name: None,
                set_creature_types: Some(vec![CreatureType::Wraith]),
                add_creature_types: vec![],
                legendary: false,
                add_colors: vec![],
                set_card_types: None,
            },
            Effect::JoinCombatAttackingChosen {
                cleanup: crate::effect::AttackingTokenCleanup::None,
                what: Selector::LastCreatedToken,
            },
            Effect::DelayUntilWithCapture {
                kind: DelayedTriggerKind::NextEndStep,
                capture: Selector::LastCreatedToken,
                body: Box::new(Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::This, filter: R::IsRingBearer },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::Exile { what: Selector::Target(0) }),
                }),
            },
        ]))],
        ..legend(
            "Sauron, the Necromancer",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Avatar, CreatureType::Horror],
            4,
            4,
        )
    }
}

/// Witch-king of Angmar — {3}{B}{B} 5/3 flying. Combat damage to you makes
/// each opponent sacrifice a creature that dealt it, and the Ring tempts you;
/// discard a card: indestructible until end of turn, then tap it.
pub fn witch_king_of_angmar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::SelfSource).once_per_batch(),
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Creature.and(R::CombatDamagedSourceControllerThisTurn),
                },
                tempt(),
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Witch-king of Angmar",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Wraith, CreatureType::Noble],
            5,
            3,
        )
    }
}

/// Witch-king, Bringer of Ruin — {4}{B}{B} 5/3 flying. Attacking makes the
/// defending player sacrifice a creature with the least power they control.
pub fn witch_king_bringer_of_ruin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::DefendingPlayer),
            count: Value::ONE,
            filter: R::Creature.and(R::HasLeastPowerAmongControlled(Box::new(R::Creature))),
        })],
        ..legend(
            "Witch-king, Bringer of Ruin",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Wraith, CreatureType::Noble],
            5,
            3,
        )
    }
}

/// Glorious Gale — {1}{U} instant. Counter target creature spell; a legendary
/// one tempts you with the Ring.
pub fn glorious_gale() -> CardDefinition {
    spell(
        "Glorious Gale",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Creature))),
            },
            Effect::If {
                cond: target_is(R::HasSupertype(Supertype::Legendary)),
                then: Box::new(tempt()),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Sauron's Ransom — {1}{U}{B} instant. An opponent splits your top four into
/// a pile for your hand and one for your graveyard; the Ring tempts you.
pub fn saurons_ransom() -> CardDefinition {
    spell(
        "Sauron's Ransom",
        cost(&[generic(1), u(), b()]),
        CardType::Instant,
        Effect::Seq(vec![Effect::FactOrFiction { count: Value::Const(4), to_bottom: false }, tempt()]),
    )
}

/// Ringsight — {1}{U}{B} sorcery. The Ring tempts you, then search for a card
/// sharing a color with a legendary creature you control.
pub fn ringsight() -> CardDefinition {
    spell(
        "Ringsight",
        cost(&[generic(1), u(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            tempt(),
            Effect::Search {
                who: PlayerRef::You,
                filter: R::SharesColorWithYourPermanentMatching(Box::new(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                )),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
    )
}

/// Sam's Desperate Rescue — {B} sorcery. Return a creature card from your
/// graveyard to your hand; the Ring tempts you.
pub fn sams_desperate_rescue() -> CardDefinition {
    spell(
        "Sam's Desperate Rescue",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            tempt(),
        ]),
    )
}
