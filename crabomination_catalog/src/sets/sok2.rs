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

/// Sweep (CR 207.2c ability word) — return any number of `ty` you control to
/// hand, then run `then`, which reads the count via
/// `Value::PermanentsReturnedThisEffect`.
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

// ── Sweep (ability word) ────────────────────────────────────────────────────

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

// ── Spirits that bounce themselves on spiritcraft ────────────────────────────

/// An "-Onna" Spirit: an ETB `etb_effect` plus "whenever you cast a Spirit or
/// Arcane spell, you may return this creature to its owner's hand."
fn onna(
    name: &'static str,
    c: crate::mana::ManaCost,
    p: i32,
    t: i32,
    etb_effect: Effect,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::etb(etb_effect),
            crate::effect::shortcut::spiritcraft(Effect::MayDo {
                description: "Return this creature to its owner's hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                }),
            }),
        ],
        ..creature(name, c, vec![CreatureType::Spirit], p, t)
    }
}

/// Haru-Onna — {3}{G} 2/1. Draws on entry, then recurs off spiritcraft.
pub fn haru_onna() -> CardDefinition {
    onna(
        "Haru-Onna",
        cost(&[generic(3), g()]),
        2,
        1,
        Effect::Draw { who: Selector::You, amount: Value::ONE },
    )
}

/// Kiri-Onna — {4}{U} 2/2. Bounces a creature on entry.
pub fn kiri_onna() -> CardDefinition {
    onna(
        "Kiri-Onna",
        cost(&[generic(4), u()]),
        2,
        2,
        Effect::Move {
            what: target_filtered(R::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
    )
}

/// Nikko-Onna — {2}{W} 2/2. Blows up an enchantment on entry.
pub fn nikko_onna() -> CardDefinition {
    onna(
        "Nikko-Onna",
        cost(&[generic(2), w()]),
        2,
        2,
        Effect::Destroy { what: target_filtered(R::Enchantment) },
    )
}

/// Yuki-Onna — {3}{R} 3/1. Blows up an artifact on entry.
pub fn yuki_onna() -> CardDefinition {
    onna(
        "Yuki-Onna",
        cost(&[generic(3), r()]),
        3,
        1,
        Effect::Destroy { what: target_filtered(R::Artifact) },
    )
}

// ── Kirins (the rest of the cycle) ──────────────────────────────────────────

/// Infernal Kirin — {2}{B}{B} 3/3 flier. Your Spirit/Arcane spells strip every
/// card sharing their mana value from a hand.
pub fn infernal_kirin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(
            Effect::RevealHandDiscardAllMatching {
                who: PlayerRef::Target(0),
                filter: R::ManaValueEqualsTriggerAmount,
            },
        )],
        ..legend(
            "Infernal Kirin",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Kirin, CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Skyfire Kirin — {2}{R}{R} 3/3 flier. Your Spirit/Arcane spells steal a
/// creature of matching mana value for the turn.
pub fn skyfire_kirin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(Effect::MayDo {
            description: "Gain control of a creature with that spell's mana value?".into(),
            body: Box::new(Effect::GainControl {
                what: target_filtered(R::Creature.and(R::ManaValueEqualsTriggerAmount)),
                to: None,
                duration: Duration::EndOfTurn,
            }),
        })],
        ..legend(
            "Skyfire Kirin",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Kirin, CreatureType::Spirit],
            3,
            3,
        )
    }
}

// ── Samurai / Snake bodies ──────────────────────────────────────────────────

/// Inner-Chamber Guard — {1}{W} 0/2 Samurai with bushido 2.
pub fn inner_chamber_guard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bushido(2)],
        ..creature(
            "Inner-Chamber Guard",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            0,
            2,
        )
    }
}

/// Kitsune Dawnblade — {4}{W} 2/3 Samurai with bushido 1 that taps a blocker
/// on entry.
pub fn kitsune_dawnblade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bushido(1)],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::MayDo {
            description: "Tap target creature?".into(),
            body: Box::new(Effect::Tap { what: target_filtered(R::Creature) }),
        })],
        ..creature(
            "Kitsune Dawnblade",
            cost(&[generic(4), w()]),
            vec![CreatureType::Fox, CreatureType::Samurai],
            2,
            3,
        )
    }
}

/// Iizuka the Ruthless — {3}{R}{R} 3/3 with bushido 2; feeds a Samurai to give
/// the rest double strike.
pub fn iizuka_the_ruthless() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bushido(2)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Samurai), 1)),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Samurai).and(R::ControlledByYou),
                ),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Iizuka the Ruthless",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            3,
            3,
        )
    }
}

/// Matsu-Tribe Birdstalker — {2}{G}{G} 2/2 Snake with the tap-lock and a reach
/// pump.
pub fn matsu_tribe_birdstalker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![super::chk::snake_tap_lock()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Matsu-Tribe Birdstalker",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Snake, CreatureType::Warrior, CreatureType::Archer],
            2,
            2,
        )
    }
}

/// Kashi-Tribe Elite — {1}{G}{G} 2/3 Snake with the tap-lock; your legendary
/// Snakes have shroud.
pub fn kashi_tribe_elite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![super::chk::snake_tap_lock()],
        static_abilities: vec![StaticAbility {
            description: "Legendary Snakes you control have shroud.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Snake)
                        .and(R::HasSupertype(crate::card::Supertype::Legendary))
                        .and(R::ControlledByYou),
                ),
                keyword: Keyword::Shroud,
            },
        }],
        ..creature(
            "Kashi-Tribe Elite",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Snake, CreatureType::Warrior],
            2,
            3,
        )
    }
}

// ── Upkeep bounce bodies ────────────────────────────────────────────────────

/// A big body whose upkeep returns one of your `color` creatures to hand.
fn upkeep_bounce(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    color: Color,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_upkeep(Effect::Move {
            what: Selector::take(
                Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::HasColor(color)),
                },
                Value::ONE,
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(name, c, types, p, t)
    }
}

/// Oni of Wild Places — {5}{R} 6/5 hasty Demon that bounces a red creature
/// every upkeep.
pub fn oni_of_wild_places() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        ..upkeep_bounce(
            "Oni of Wild Places",
            cost(&[generic(5), r()]),
            vec![CreatureType::Demon, CreatureType::Spirit],
            6,
            5,
            Color::Red,
        )
    }
}

/// Stampeding Serow — {2}{G}{G} 5/4 trampler that bounces a green creature
/// every upkeep.
pub fn stampeding_serow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..upkeep_bounce(
            "Stampeding Serow",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Antelope, CreatureType::Beast],
            5,
            4,
            Color::Green,
        )
    }
}

/// Skull Collector — {1}{B}{B} 3/3 that regenerates but bounces a black
/// creature every upkeep.
pub fn skull_collector() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..upkeep_bounce(
            "Skull Collector",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            3,
            3,
            Color::Black,
        )
    }
}

// ── Moonfolk (land-bounce activations) ──────────────────────────────────────

/// A Moonfolk ability: `{2}`, return a land you control to its owner's hand.
fn moonfolk_ability(effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        return_permanent_cost: Some(R::Land),
        effect,
        ..Default::default()
    }
}

/// Oboro Breezecaller — {1}{U} 1/1 flier that untaps a land.
pub fn oboro_breezecaller() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![moonfolk_ability(Effect::Untap {
            what: target_filtered(R::Land),
            up_to: None,
        })],
        ..creature(
            "Oboro Breezecaller",
            cost(&[generic(1), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Oboro Envoy — {3}{U} 1/3 flier that shrinks a creature by your hand size.
pub fn oboro_envoy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![moonfolk_ability(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Diff(Box::new(Value::Const(0)), Box::new(hand())),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Oboro Envoy",
            cost(&[generic(3), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Moonbow Illusionist — {2}{U} 2/1 flier that rewrites a land's basic type.
pub fn moonbow_illusionist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![moonfolk_ability(Effect::LandsBecomeChosenBasicType {
            what: target_filtered(R::Land),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Moonbow Illusionist",
            cost(&[generic(2), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Wizard],
            2,
            1,
        )
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// Oboro, Palace in the Clouds — Legendary Land that returns itself for {1}.
pub fn oboro_palace_in_the_clouds() -> CardDefinition {
    CardDefinition {
        name: "Oboro, Palace in the Clouds",
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Miren, the Moaning Well — Legendary Land that eats a creature for its
/// toughness in life.
pub fn miren_the_moaning_well() -> CardDefinition {
    CardDefinition {
        name: "Miren, the Moaning Well",
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::SacrificedToughness,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Artifacts, enchantments, spells ─────────────────────────────────────────

/// Manriki-Gusari — {2} Equipment. +1/+2 and a tap to smash other Equipment.
pub fn manriki_gusari() -> CardDefinition {
    CardDefinition {
        name: "Manriki-Gusari",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 2,
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Destroy {
                    what: target_filtered(R::HasArtifactSubtype(
                        crate::card::ArtifactSubtype::Equipment,
                    )),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Soratami Cloud Chariot — {5} Artifact. Grants flight, or a Maze-of-Ith fog
/// on one creature.
pub fn soratami_cloud_chariot() -> CardDefinition {
    CardDefinition {
        name: "Soratami Cloud Chariot",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::PreventAllCombatDamageInvolving {
                    target: target_filtered(R::Creature.and(R::ControlledByYou)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Wine of Blood and Iron — {3} Artifact. Doubles a creature's power, then
/// goes away at end of turn.
pub fn wine_of_blood_and_iron() -> CardDefinition {
    CardDefinition {
        name: "Wine of Blood and Iron",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::PowerOf(Box::new(Selector::Target(0))),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::HasName("Wine of Blood and Iron".into()),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reverence — {2}{W}{W} Enchantment. Small creatures can't attack you.
pub fn reverence() -> CardDefinition {
    CardDefinition {
        name: "Reverence",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures with power 2 or less can't attack you.",
            effect: StaticEffect::CreaturesCantAttackController {
                protect_planeswalkers: false,
                filter: Some(R::PowerAtMost(2)),
            },
        }],
        ..Default::default()
    }
}

/// Seed the Land — {2}{G}{G} Enchantment. Every land entering mints a Snake
/// for its controller.
pub fn seed_the_land() -> CardDefinition {
    CardDefinition {
        name: "Seed the Land",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Land,
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                count: Value::ONE,
                definition: crate::card::TokenDefinition {
                    name: "Snake".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Snake],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Molting Skin — {2}{G} Enchantment. Bounce it to regenerate a creature.
pub fn molting_skin() -> CardDefinition {
    CardDefinition {
        name: "Molting Skin",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            return_self_cost: true,
            effect: Effect::Regenerate { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Razorjaw Oni — {3}{B} 4/5 Demon. Black creatures can't block.
pub fn razorjaw_oni() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Black creatures can't block.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Black))),
                keyword: Keyword::CantBlock,
            },
        }],
        ..creature(
            "Razorjaw Oni",
            cost(&[generic(3), b()]),
            vec![CreatureType::Demon, CreatureType::Spirit],
            4,
            5,
        )
    }
}

/// Raving Oni-Slave — {1}{B} 3/3 that costs you 3 life coming and going
/// without a Demon.
pub fn raving_oni_slave() -> CardDefinition {
    let toll = || {
        Effect::If {
            cond: Predicate::Not(Box::new(Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Demon),
            }))),
            then: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(3) }),
            else_: Box::new(Effect::Noop),
        }
    };
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::etb(toll()),
            crate::card::TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: toll(),
            },
        ],
        ..creature(
            "Raving Oni-Slave",
            cost(&[generic(1), b()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Reki, the History of Kamigawa — {2}{G} 1/2. Your legendary spells draw.
pub fn reki_the_history_of_kamigawa() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(R::HasSupertype(crate::card::Supertype::Legendary)),
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend(
            "Reki, the History of Kamigawa",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            1,
            2,
        )
    }
}

/// Maga, Traitor to Mortals — {X}{B}{B}{B} 0/0 that drains for its counters.
pub fn maga_traitor_to_mortals() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((crate::card::CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::LoseLife {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::CountersOn {
                what: Box::new(Selector::This),
                kind: crate::card::CounterType::PlusOnePlusOne,
            },
        })],
        ..legend(
            "Maga, Traitor to Mortals",
            cost(&[crate::mana::x(), b(), b(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            0,
            0,
        )
    }
}

/// Torii Watchward — {4}{W} 3/3 vigilant Spirit with soulshift 4.
pub fn torii_watchward() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![crate::effect::shortcut::soulshift(4)],
        ..creature("Torii Watchward", cost(&[generic(4), w()]), vec![CreatureType::Spirit], 3, 3)
    }
}

/// Kami of the Tended Garden — {3}{G} 4/4 Spirit on a {G} upkeep lease, with
/// soulshift 3.
pub fn kami_of_the_tended_garden() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_upkeep(Effect::SacrificeSourceUnlessPay { cost: cost(&[g()]) }),
            crate::effect::shortcut::soulshift(3),
        ],
        ..creature(
            "Kami of the Tended Garden",
            cost(&[generic(3), g()]),
            vec![CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Moonwing Moth — {1}{W}{W} 2/1 flier that can armor up for {W}.
pub fn moonwing_moth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Moonwing Moth", cost(&[generic(1), w(), w()]), vec![CreatureType::Insect], 2, 1)
    }
}

/// Path of Anger's Flame — {2}{R} Arcane instant. Team +2/+0.
pub fn path_of_angers_flame() -> CardDefinition {
    arcane_instant(
        "Path of Anger's Flame",
        cost(&[generic(2), r()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Sunder from Within — {2}{R}{R} Arcane sorcery. Destroy an artifact or land.
pub fn sunder_from_within() -> CardDefinition {
    arcane_sorcery(
        "Sunder from Within",
        cost(&[generic(2), r(), r()]),
        Effect::Destroy { what: target_filtered(R::Artifact.or(R::Land)) },
    )
}

/// Ideas Unbound — {U}{U} Arcane sorcery. Draw three now, pitch three later.
pub fn ideas_unbound() -> CardDefinition {
    arcane_sorcery(
        "Ideas Unbound",
        cost(&[u(), u()]),
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::AtNextEndStep {
                body: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(3),
                    random: false,
                }),
            },
        ]),
    )
}

/// Overwhelming Intellect — {4}{U}{U} instant. Counter a creature spell and
/// draw its mana value.
pub fn overwhelming_intellect() -> CardDefinition {
    instant(
        "Overwhelming Intellect",
        cost(&[generic(4), u(), u()]),
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Creature))) },
            Effect::Draw { who: Selector::You, amount: Value::CounteredSpellManaValue },
        ]),
    )
}

/// Twincast — {U}{U} instant. Copy an instant or sorcery spell.
pub fn twincast() -> CardDefinition {
    instant(
        "Twincast",
        cost(&[u(), u()]),
        Effect::CopySpellMayChooseTargets {
            what: target_filtered(R::IsSpellOnStack.and(
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            )),
            count: Value::ONE,
        },
    )
}

/// Endless Swarm — {5}{G}{G}{G} Epic sorcery. A Snake per card in hand, every
/// upkeep, forever.
pub fn endless_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Epic],
        ..sorcery(
            "Endless Swarm",
            cost(&[generic(5), g(), g(), g()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: hand(),
                definition: crate::card::TokenDefinition {
                    name: "Snake".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Snake],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
            },
        )
    }
}

// ── Graveyard-triggered recursion ───────────────────────────────────────────

/// Akuta, Born of Ash — {2}{B}{B} 3/2 haste that buys itself back out of the
/// graveyard for a Swamp while you're ahead on cards.
pub fn akuta_born_of_ash() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            )
            .with_filter(hand_advantage()),
            effect: Effect::MayDo {
                description: "Sacrifice a Swamp to return Akuta to the battlefield?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::HasLandType(LandType::Swamp),
                    },
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ])),
            },
        }],
        ..legend("Akuta, Born of Ash", cost(&[generic(2), b(), b()]), vec![CreatureType::Spirit], 3, 2)
    }
}

/// Exile into Darkness — {4}{B} sorcery. A cheap edict that buys itself back
/// while you're ahead on cards.
pub fn exile_into_darkness() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            )
            .with_filter(hand_advantage()),
            effect: Effect::MayDo {
                description: "Return Exile into Darkness to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..sorcery(
            "Exile into Darkness",
            cost(&[generic(4), b()]),
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Creature.and(R::ManaValueAtMost(3)),
            },
        )
    }
}

// ── The Ascendant flip cycle (CR 710) ───────────────────────────────────────

/// A flipped Essence — a costless Legendary Enchantment bottom face.
fn essence(name: &'static str, abilities: Vec<StaticAbility>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Enchantment],
        supertypes: vec![crate::card::Supertype::Legendary],
        static_abilities: abilities,
        ..Default::default()
    }
}

/// Erayo, Soratami Ascendant — {1}{U} 1/1 flier that flips on the turn's
/// fourth spell; Erayo's Essence counters each opponent's first spell.
pub fn erayo_soratami_ascendant() -> CardDefinition {
    let mut flipped = essence("Erayo's Essence", vec![]);
    flipped.triggered_abilities = vec![crate::card::TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).once_per_turn(),
        effect: Effect::CounterSpell { what: Selector::TriggerSource },
    }];
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::ValueAtLeast(
                    Value::SpellsCastThisTurnTotal,
                    Value::Const(4),
                ))
                .once_per_turn(),
            effect: Effect::Flip { what: Selector::This },
        }],
        flip_face: Some(Box::new(flipped)),
        ..legend(
            "Erayo, Soratami Ascendant",
            cost(&[generic(1), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Monk],
            1,
            1,
        )
    }
}

/// Homura, Human Ascendant — {4}{R}{R} 4/4 that can't block and comes back
/// flipped; Homura's Essence is a firebreathing anthem.
pub fn homura_human_ascendant() -> CardDefinition {
    let flipped = essence(
        "Homura's Essence",
        vec![
            StaticAbility {
                description: "Creatures you control get +2/+2 and have flying.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: 2,
                    toughness: 2,
                },
            },
            StaticAbility {
                description: "Creatures you control have flying.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Flying,
                },
            },
            StaticAbility {
                description: "Creatures you control have \"{R}: This creature gets +1/+0.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    ability: ActivatedAbility {
                        mana_cost: cost(&[r()]),
                        effect: Effect::PumpPT {
                            what: Selector::This,
                            power: Value::ONE,
                            toughness: Value::Const(0),
                            duration: Duration::EndOfTurn,
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
    );
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Seq(vec![
            Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::Flip { what: Selector::LastMoved },
        ]))],
        flip_face: Some(Box::new(flipped)),
        ..legend(
            "Homura, Human Ascendant",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Human, CreatureType::Monk],
            4,
            4,
        )
    }
}

/// Kuon, Ogre Ascendant — {B}{B}{B} 2/4 that flips after a bloody turn;
/// Kuon's Essence is a per-upkeep edict on everyone.
pub fn kuon_ogre_ascendant() -> CardDefinition {
    let mut flipped = essence("Kuon's Essence", vec![]);
    flipped.triggered_abilities = vec![crate::card::TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(crate::game::TurnStep::Upkeep),
            EventScope::AnyPlayer,
        ),
        effect: Effect::Sacrifice {
            who: Selector::Player(PlayerRef::ActivePlayer),
            count: Value::ONE,
            filter: R::Creature,
        },
    }];
    CardDefinition {
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::Const(3),
            }),
            effect: Effect::Flip { what: Selector::This },
        }],
        flip_face: Some(Box::new(flipped)),
        ..legend(
            "Kuon, Ogre Ascendant",
            cost(&[b(), b(), b()]),
            vec![CreatureType::Ogre, CreatureType::Monk],
            2,
            4,
        )
    }
}

/// Rune-Tail, Kitsune Ascendant — {2}{W} 2/2 that flips at 30 life;
/// Rune-Tail's Essence shields every creature you control.
pub fn rune_tail_kitsune_ascendant() -> CardDefinition {
    let flipped = essence(
        "Rune-Tail's Essence",
        vec![StaticAbility {
            description: "Prevent all damage that would be dealt to creatures you control.",
            effect: StaticEffect::PreventAllDamageToYourCreatures,
        }],
    );
    CardDefinition {
        flip_when_predicate: Some(Predicate::PlayerLifeAtLeast {
            who: PlayerRef::You,
            life: 30,
        }),
        flip_face: Some(Box::new(flipped)),
        ..legend(
            "Rune-Tail, Kitsune Ascendant",
            cost(&[generic(2), w()]),
            vec![CreatureType::Fox, CreatureType::Monk],
            2,
            2,
        )
    }
}

/// Sasaya, Orochi Ascendant — {1}{G}{G} 2/3 that flips on a land-flooded hand;
/// Sasaya's Essence doubles up on same-named lands.
pub fn sasaya_orochi_ascendant() -> CardDefinition {
    let flipped = essence(
        "Sasaya's Essence",
        vec![StaticAbility {
            description: "Whenever a land you control is tapped for mana, add an additional \
                          one mana of any type that land produced for each other land you \
                          control with the same name as it.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: false,
                filter: R::Land.and(R::ControlledByYou),
                extra: crate::effect::ExtraManaKind::Mirror,
                while_monarch: false,
            },
        }],
    );
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            condition: Some(Predicate::ValueAtLeast(
                Value::CardsInHandMatching { who: PlayerRef::You, filter: R::Land },
                Value::Const(7),
            )),
            effect: Effect::Flip { what: Selector::This },
            ..Default::default()
        }],
        flip_face: Some(Box::new(flipped)),
        ..legend(
            "Sasaya, Orochi Ascendant",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Snake, CreatureType::Monk],
            2,
            3,
        )
    }
}

// ── Splice / combat tricks ──────────────────────────────────────────────────

/// Into the Fray — {R} Arcane instant that drags a creature into combat.
/// Splices onto Arcane for {R}.
pub fn into_the_fray() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Splice(cost(&[r()]), crate::card::SpellSubtype::Arcane)],
        ..arcane_instant(
            "Into the Fray",
            cost(&[r()]),
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Shifting Borders — {3}{U} Arcane instant that swaps two lands. Splices onto
/// Arcane for {3}{U}.
pub fn shifting_borders() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Splice(
            cost(&[generic(3), u()]),
            crate::card::SpellSubtype::Arcane,
        )],
        ..arcane_instant(
            "Shifting Borders",
            cost(&[generic(3), u()]),
            Effect::ExchangeControl {
                a: target_filtered(R::Land),
                b: Selector::Target(1),
            },
        )
    }
}

/// Rushing-Tide Zubera — {2}{U}{U} 3/3 that cashes in for three cards when it
/// dies to four or more damage.
pub fn rushing_tide_zubera() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(
                Predicate::ValueAtLeast(
                    Value::DamageDealtToSourceThisTurn,
                    Value::Const(4),
                ),
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        }],
        ..creature(
            "Rushing-Tide Zubera",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Zubera, CreatureType::Spirit],
            3,
            3,
        )
    }
}

// ── Epic (CR 702.50) ────────────────────────────────────────────────────────

/// Eternal Dominion — {7}{U}{U}{U} Epic sorcery. Rip a permanent out of an
/// opponent's library, every upkeep, forever.
pub fn eternal_dominion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Epic],
        ..sorcery(
            "Eternal Dominion",
            cost(&[generic(7), u(), u(), u()]),
            Effect::SearchPickedBy {
                who: PlayerRef::Target(0),
                picker: PlayerRef::You,
                filter: R::Artifact.or(R::Creature).or(R::Enchantment).or(R::Land),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        )
    }
}

/// Neverending Torment — {4}{B}{B} Epic sorcery. Exiles a hand's worth of a
/// library, every upkeep, forever.
pub fn neverending_torment() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Epic],
        ..sorcery(
            "Neverending Torment",
            cost(&[generic(4), b(), b()]),
            Effect::Repeat {
                count: hand(),
                body: Box::new(Effect::SearchPickedBy {
                    who: PlayerRef::Target(0),
                    picker: PlayerRef::You,
                    filter: R::Any,
                    to: ZoneDest::Exile,
                }),
            },
        )
    }
}

/// Undying Flames — {4}{R}{R} Epic sorcery. Digs to the first nonland card and
/// burns for its mana value, every upkeep, forever.
pub fn undying_flames() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Epic],
        ..sorcery(
            "Undying Flames",
            cost(&[generic(4), r(), r()]),
            Effect::Seq(vec![
                Effect::ExileTopUntilNonland { who: PlayerRef::You },
                Effect::DealDamage { to: target_any(), amount: Value::LastExiledManaValue },
            ]),
        )
    }
}

// ── The rest ────────────────────────────────────────────────────────────────

/// Curtain of Light — {1}{W} combat trick. An unblocked attacker becomes
/// blocked, and you replace the card.
pub fn curtain_of_light() -> CardDefinition {
    CardDefinition {
        cast_only_after_blockers: true,
        ..instant(
            "Curtain of Light",
            cost(&[generic(1), w()]),
            Effect::Seq(vec![
                Effect::BecomeBlocked { what: target_filtered(R::Creature.and(R::IsUnblocked)) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        )
    }
}

/// Michiko Konda, Truth Seeker — {3}{W} 2/2. Any damage an opponent's source
/// deals you costs them a permanent. Approximation: the sacrifice hits every
/// opponent rather than only the damage's controller (exact at two players).
pub fn michiko_konda_truth_seeker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::OpponentSourceDamagedYou),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Any,
            },
        }],
        ..legend(
            "Michiko Konda, Truth Seeker",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            2,
            2,
        )
    }
}

/// Measure of Wickedness — {3}{B} Enchantment. A hot potato that costs 8 life
/// if you're still holding it at your end step.
pub fn measure_of_wickedness() -> CardDefinition {
    CardDefinition {
        name: "Measure of Wickedness",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            crate::card::TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::YourControl,
                ),
                effect: Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::HasName("Measure of Wickedness".into()),
                    },
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(8) },
                ]),
            },
            crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl),
                // Approximation: the printed "target opponent" is modeled as
                // your opponent (exact at two players).
                effect: Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::EachOpponent),
                    duration: Duration::Permanent,
                },
            },
        ],
        ..Default::default()
    }
}

/// Iname as One — {8}{B}{B}{G}{G} 8/8. Fetches a Spirit on the way in and
/// reanimates one on the way out.
pub fn iname_as_one() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::CastFromHand),
                effect: Effect::MayDo {
                    description: "Search your library for a Spirit permanent card?".into(),
                    body: Box::new(Effect::Search {
                        who: PlayerRef::You,
                        filter: R::PermanentCard.and(R::HasCreatureType(CreatureType::Spirit)),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    }),
                },
            },
            crate::effect::shortcut::on_dies(Effect::MayDo {
                description: "Exile Iname to reanimate a Spirit?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: Selector::This },
                    Effect::Move {
                        what: target_filtered(
                            R::InYourGraveyard
                                .and(R::PermanentCard)
                                .and(R::HasCreatureType(CreatureType::Spirit)),
                        ),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                ])),
            }),
        ],
        ..legend(
            "Iname as One",
            cost(&[generic(8), b(), b(), g(), g()]),
            vec![CreatureType::Spirit],
            8,
            8,
        )
    }
}

/// Sakashima the Impostor — {2}{U}{U} 3/1 that enters as a copy of any
/// creature, keeping its own name and a bounce escape hatch.
pub fn sakashima_the_impostor() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(crate::card::EntersAsCopy {
            filter: R::Creature,
            keep_name: true,
            legendary: true,
            extra_activated: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), u()]),
                effect: Effect::AtNextEndStep {
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                    }),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..legend(
            "Sakashima the Impostor",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            1,
        )
    }
}

/// Shape Stealer — {U}{U} 1/1 that copies whatever it meets in combat.
pub fn shape_stealer() -> CardDefinition {
    let mimic = |from: Selector| Effect::SetBasePT {
        what: Selector::This,
        power: Value::PowerOf(Box::new(from.clone())),
        toughness: Value::ToughnessOf(Box::new(from)),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![
            crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: mimic(Selector::take(Selector::BlockedAttacker, Value::ONE)),
            },
            crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: mimic(Selector::take(Selector::BlockingCreatures, Value::ONE)),
            },
        ],
        ..creature(
            "Shape Stealer",
            cost(&[u(), u()]),
            vec![CreatureType::Shapeshifter, CreatureType::Spirit],
            1,
            1,
        )
    }
}

/// Sokenzan Renegade — {2}{R} 3/3 with bushido 1 that defects to whoever is
/// holding the most cards.
pub fn sokenzan_renegade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bushido(1)],
        triggered_abilities: vec![on_upkeep(Effect::If {
            cond: Predicate::AnOpponentHasMoreCardsInHand,
            then: Box::new(Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::MostCardsInHand),
                duration: Duration::Permanent,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Sokenzan Renegade",
            cost(&[generic(2), r()]),
            vec![CreatureType::Ogre, CreatureType::Samurai, CreatureType::Mercenary],
            3,
            3,
        )
    }
}

/// Tomb of Urami — Legendary Land. Painful black mana, or trade your whole
/// mana base for a 5/5 flying Demon.
pub fn tomb_of_urami() -> CardDefinition {
    CardDefinition {
        name: "Tomb of Urami",
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colors(vec![Color::Black]),
                    },
                    Effect::If {
                        cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                            Selector::ControlledBy {
                                who: PlayerRef::You,
                                filter: R::HasCreatureType(CreatureType::Ogre),
                            },
                        ))),
                        then: Box::new(Effect::DealDamage {
                            to: Selector::You,
                            amount: Value::ONE,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b(), b()]),
                tap_cost: true,
                sac_cost: true,
                sac_all_matching_cost: Some(R::Land),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crate::card::TokenDefinition {
                        name: "Urami".into(),
                        card_types: vec![CardType::Creature],
                        supertypes: vec![crate::card::Supertype::Legendary],
                        colors: vec![Color::Black],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Demon, CreatureType::Spirit],
                            ..Default::default()
                        },
                        power: 5,
                        toughness: 5,
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
