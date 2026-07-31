//! Saviors of Kamigawa (SOK) gap closure, wave 2 — Sweep, the Shinen channel
//! cycle, and the hand-size-matters block. Tests in `classic_sets/sok2`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    StaticAbility, Subtypes, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{target_any, target_filtered},
};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

use super::bok::{arcane_instant, arcane_sorcery, creature, instant, legend, sorcery};
use super::bok2::on_upkeep;
use super::sok::channel_ability as channel;

/// Cards in the controller's hand.
fn hand() -> Value {
    Value::HandSizeOf(PlayerRef::You)
}

/// "You have more cards in hand than each opponent."
fn hand_advantage() -> Predicate {
    Predicate::Not(Box::new(Predicate::AnOpponentHasMoreCardsInHand))
}

/// Sweep (CR 702.60) — return any number of `ty` you control to hand, then run
/// `then`, which reads the count via `Value::PermanentsReturnedThisEffect`.
fn sweep(ty: LandType, then: Effect) -> Effect {
    Effect::Seq(vec![
        Effect::ReturnAnyNumberToHand { filter: R::HasLandType(ty) },
        then,
    ])
}

/// The number of lands a Sweep just returned.
fn swept() -> Value {
    Value::PermanentsReturnedThisEffect
}

// ── Sweep (CR 702.60) ───────────────────────────────────────────────────────

/// Barrel Down Sokenzan — {2}{R} Arcane instant. Sweep Mountains for twice
/// that many damage to a creature.
pub fn barrel_down_sokenzan() -> CardDefinition {
    arcane_instant(
        "Barrel Down Sokenzan",
        cost(&[generic(2), r()]),
        sweep(
            LandType::Mountain,
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Times(Box::new(swept()), Box::new(Value::Const(2))),
            },
        ),
    )
}

/// Charge Across the Araba — {4}{W} Arcane instant. Sweep Plains to pump your
/// whole team.
pub fn charge_across_the_araba() -> CardDefinition {
    arcane_instant(
        "Charge Across the Araba",
        cost(&[generic(4), w()]),
        sweep(
            LandType::Plains,
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: swept(),
                toughness: swept(),
                duration: Duration::EndOfTurn,
            },
        ),
    )
}

/// Plow Through Reito — {1}{W} Arcane instant. Sweep Plains to pump one
/// creature.
pub fn plow_through_reito() -> CardDefinition {
    arcane_instant(
        "Plow Through Reito",
        cost(&[generic(1), w()]),
        sweep(
            LandType::Plains,
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: swept(),
                toughness: swept(),
                duration: Duration::EndOfTurn,
            },
        ),
    )
}

/// Sink into Takenuma — {3}{B} Arcane sorcery. Sweep Swamps to strip a hand.
pub fn sink_into_takenuma() -> CardDefinition {
    arcane_sorcery(
        "Sink into Takenuma",
        cost(&[generic(3), b()]),
        sweep(
            LandType::Swamp,
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: swept(),
                random: false,
            },
        ),
    )
}

// ── The Shinen cycle — a body plus the keyword it channels out ───────────────

/// A Shinen: `{cost}` Spirit with `keywords`, whose Channel grants
/// `granted` to a target creature for the turn.
fn shinen(
    name: &'static str,
    c: crate::mana::ManaCost,
    p: i32,
    t: i32,
    keywords: Vec<Keyword>,
    channel_cost: crate::mana::ManaCost,
    granted: Keyword,
) -> CardDefinition {
    CardDefinition {
        keywords,
        activated_abilities: vec![channel(
            channel_cost,
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: granted,
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature(name, c, vec![CreatureType::Spirit], p, t)
    }
}

pub fn shinen_of_fears_chill() -> CardDefinition {
    shinen(
        "Shinen of Fear's Chill",
        cost(&[generic(4), b()]),
        3,
        2,
        vec![Keyword::CantBlock],
        cost(&[generic(1), b()]),
        Keyword::CantBlock,
    )
}

pub fn shinen_of_flights_wings() -> CardDefinition {
    shinen(
        "Shinen of Flight's Wings",
        cost(&[generic(4), u()]),
        3,
        3,
        vec![Keyword::Flying],
        cost(&[u()]),
        Keyword::Flying,
    )
}

pub fn shinen_of_furys_fire() -> CardDefinition {
    shinen(
        "Shinen of Fury's Fire",
        cost(&[generic(2), r()]),
        2,
        1,
        vec![Keyword::Haste],
        cost(&[r()]),
        Keyword::Haste,
    )
}

pub fn shinen_of_lifes_roar() -> CardDefinition {
    shinen(
        "Shinen of Life's Roar",
        cost(&[generic(1), g()]),
        1,
        2,
        vec![Keyword::AllMustBlock],
        cost(&[generic(2), g(), g()]),
        Keyword::AllMustBlock,
    )
}

pub fn shinen_of_stars_light() -> CardDefinition {
    shinen(
        "Shinen of Stars' Light",
        cost(&[generic(2), w()]),
        2,
        1,
        vec![Keyword::FirstStrike],
        cost(&[generic(1), w()]),
        Keyword::FirstStrike,
    )
}

/// Jiwari, the Earth Aflame — {3}{R}{R} 3/3. Snipes ground creatures, or
/// channels out of hand to sweep them.
pub fn jiwari_the_earth_aflame() -> CardDefinition {
    let grounded = || R::Creature.and(R::HasKeyword(Keyword::Flying).negate());
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[crate::mana::x(), r()]),
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: target_filtered(grounded()),
                    amount: Value::XFromCost,
                },
                ..Default::default()
            },
            channel(
                cost(&[crate::mana::x(), r(), r(), r()]),
                Effect::DealDamage {
                    to: Selector::EachPermanent(grounded()),
                    amount: Value::XFromCost,
                },
            ),
        ],
        ..legend(
            "Jiwari, the Earth Aflame",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Spirit],
            3,
            3,
        )
    }
}

// ── Hand size matters ───────────────────────────────────────────────────────

/// Kiyomaro, First to Stand — {3}{W}{W} */*. Hand-sized, vigilant at four,
/// and pays out 7 life when it connects with a full grip.
pub fn kiyomaro_first_to_stand() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(crate::card::DynamicPt::ControllerHandSize),
        static_abilities: vec![StaticAbility {
            description: "As long as you have four or more cards in hand, Kiyomaro has vigilance.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Vigilance,
                condition: Predicate::ValueAtLeast(hand(), Value::Const(4)),
            },
        }],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource)
                .with_filter(Predicate::ValueAtLeast(hand(), Value::Const(7))),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(7) },
        }],
        ..legend(
            "Kiyomaro, First to Stand",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Spirit],
            0,
            0,
        )
    }
}

/// Okina Nightwatch — {4}{G} 4/3 that grows while you're ahead on cards.
pub fn okina_nightwatch() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you have more cards in hand than each opponent, \
                          this creature gets +3/+3.",
            effect: StaticEffect::PumpSelfIf {
                condition: hand_advantage(),
                power: 3,
                toughness: 3,
                keywords: vec![],
            },
        }],
        ..creature(
            "Okina Nightwatch",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Monk],
            4,
            3,
        )
    }
}

/// Secretkeeper — {3}{U} 2/2 that flies and grows while you're ahead on cards.
pub fn secretkeeper() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you have more cards in hand than each opponent, \
                          this creature gets +2/+2 and has flying.",
            effect: StaticEffect::PumpSelfIf {
                condition: hand_advantage(),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..creature("Secretkeeper", cost(&[generic(3), u()]), vec![CreatureType::Spirit], 2, 2)
    }
}

/// Descendant of Kiyomaro — {1}{W}{W} 2/3 that grows and drains while you're
/// ahead on cards.
pub fn descendant_of_kiyomaro() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you have more cards in hand than each opponent, \
                          this creature gets +1/+2 and has a combat-damage lifegain trigger.",
            effect: StaticEffect::PumpSelfIf {
                condition: hand_advantage(),
                power: 1,
                toughness: 2,
                keywords: vec![],
            },
        }],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamage, EventScope::SelfSource)
                .with_filter(hand_advantage()),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        }],
        ..creature(
            "Descendant of Kiyomaro",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Kitsune Loreweaver — {1}{W} 2/1 that armors up by your hand size.
pub fn kitsune_loreweaver() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: hand(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Kitsune Loreweaver",
            cost(&[generic(1), w()]),
            vec![CreatureType::Fox, CreatureType::Cleric],
            2,
            1,
        )
    }
}

/// Kitsune Bonesetter — {2}{W} 0/1. Shields a creature while you're ahead on
/// cards.
pub fn kitsune_bonesetter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(hand_advantage()),
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..creature(
            "Kitsune Bonesetter",
            cost(&[generic(2), w()]),
            vec![CreatureType::Fox, CreatureType::Cleric],
            0,
            1,
        )
    }
}

/// Locust Miser — {2}{B}{B} 2/2 Rat Shaman that shrinks opposing hands.
pub fn locust_miser() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each opponent's maximum hand size is reduced by two.",
            effect: StaticEffect::OpponentsMaxHandSizeReduced(2),
        }],
        ..creature(
            "Locust Miser",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Rat, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Minamo Scrollkeeper — {1}{U} 2/3 wall that widens your grip by one.
pub fn minamo_scrollkeeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        static_abilities: vec![StaticAbility {
            description: "Your maximum hand size is increased by one.",
            effect: StaticEffect::ControllerMaxHandSizeIncreased(1),
        }],
        ..creature(
            "Minamo Scrollkeeper",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Trusted Advisor — {U} 1/2 that widens your grip by two but bounces a blue
/// creature every upkeep.
pub fn trusted_advisor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your maximum hand size is increased by two.",
            effect: StaticEffect::ControllerMaxHandSizeIncreased(2),
        }],
        triggered_abilities: vec![on_upkeep(Effect::Move {
            what: Selector::take(
                Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::HasColor(Color::Blue)),
                },
                Value::ONE,
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Trusted Advisor",
            cost(&[u()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            2,
        )
    }
}

/// Meishin, the Mind Cage — {4}{U}{U}{U} Legendary Enchantment. Every creature
/// shrinks by your hand size.
pub fn meishin_the_mind_cage() -> CardDefinition {
    CardDefinition {
        name: "Meishin, the Mind Cage",
        cost: cost(&[generic(4), u(), u(), u()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![crate::card::Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "All creatures get -X/-0, where X is the number of cards in your hand.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::EachPermanent(R::Creature),
                power: Value::Diff(Box::new(Value::Const(0)), Box::new(hand())),
                toughness: Value::Const(0),
            },
        }],
        ..Default::default()
    }
}

/// Ivory Crane Netsuke — {2} Artifact. Four life each upkeep on a full grip.
pub fn ivory_crane_netsuke() -> CardDefinition {
    CardDefinition {
        name: "Ivory Crane Netsuke",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(hand(), Value::Const(7))),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        }],
        ..Default::default()
    }
}

/// Scroll of Origins — {2} Artifact. Draws while your grip stays full.
pub fn scroll_of_origins() -> CardDefinition {
    CardDefinition {
        name: "Scroll of Origins",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            condition: Some(Predicate::ValueAtLeast(hand(), Value::Const(7))),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Presence of the Wise — {2}{W}{W} sorcery. Two life per card in hand.
pub fn presence_of_the_wise() -> CardDefinition {
    sorcery(
        "Presence of the Wise",
        cost(&[generic(2), w(), w()]),
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(Box::new(hand()), Box::new(Value::Const(2))),
        },
    )
}

/// Spiraling Embers — {3}{R} Arcane sorcery. Damage equal to your hand size.
pub fn spiraling_embers() -> CardDefinition {
    arcane_sorcery(
        "Spiraling Embers",
        cost(&[generic(3), r()]),
        Effect::DealDamage { to: target_any(), amount: hand() },
    )
}

/// Inner Fire — {3}{R} sorcery. Your hand size in red mana.
pub fn inner_fire() -> CardDefinition {
    sorcery(
        "Inner Fire",
        cost(&[generic(3), r()]),
        Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, hand()) },
    )
}

/// One with Nothing — {B} instant. Discard your hand.
pub fn one_with_nothing() -> CardDefinition {
    instant(
        "One with Nothing",
        cost(&[b()]),
        Effect::Discard { who: Selector::You, amount: Value::Const(100), random: false },
    )
}

/// Oppressive Will — {2}{U} instant. Counter unless they pay your hand size.
pub fn oppressive_will() -> CardDefinition {
    instant(
        "Oppressive Will",
        cost(&[generic(2), u()]),
        Effect::CounterUnlessPaid {
            what: Selector::Target(0),
            mana_cost: cost(&[]),
            exile: false,
            extra_generic: Some(hand()),
        },
    )
}

/// Kagemaro's Clutch — {3}{B} Aura. The host shrinks by your hand size.
pub fn kagemaros_clutch() -> CardDefinition {
    CardDefinition {
        name: "Kagemaro's Clutch",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            scale: Some(crate::card::EquipScale {
                filter: R::Any,
                per_power: -1,
                per_toughness: -1,
                count_source_controller_hand: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Rending Vines — {1}{G}{G} Arcane instant. Blows up a cheap artifact or
/// enchantment and replaces itself.
pub fn rending_vines() -> CardDefinition {
    arcane_instant(
        "Rending Vines",
        cost(&[generic(1), g(), g()]),
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    R::Artifact.or(R::Enchantment).and(R::ManaValueAtMostControllerHand),
                ),
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Thoughts of Ruin — {2}{R}{R} sorcery. Every player sacrifices a land per
/// card in your hand.
pub fn thoughts_of_ruin() -> CardDefinition {
    sorcery(
        "Thoughts of Ruin",
        cost(&[generic(2), r(), r()]),
        Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: hand(),
            filter: R::Land,
        },
    )
}
