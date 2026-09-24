//! Commander: the cards the **Symbiotic Swarm** precon (C20, Kathril, Aspect
//! Warper) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kathril.rs`.
//!
//! Residuals (each also on its card):
//! - **Cairn Wanderer** — landwalk and protection are copied for the five
//!   basic land types and the five colors only.
//! - **Slippery Bogbonder** — every counter on your other creatures moves
//!   (the engine's "any number").
//! - **Tayam, Luminous Enigma** — the vigilance counter arrives by trigger,
//!   not as the creature enters.
//! - **Vitality Hunter** — the lifelink counters go on your X greatest-power
//!   creatures, not X chosen targets.
//! - **Yannik, Scavenging Sentinel** — the engine exiles your highest mana
//!   value other creature, and X reads its printed power.
//! - **Archon of Valor's Reach** — the bot's pick of type is the first
//!   offered (instant).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    ExileReturnZone, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, w, x};
use std::sync::Arc;

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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

/// The keyword list Kathril and Majestic Myriarch share (bar haste).
fn kathril_keywords() -> Vec<Keyword> {
    vec![
        Keyword::Flying,
        Keyword::FirstStrike,
        Keyword::DoubleStrike,
        Keyword::Deathtouch,
        Keyword::Hexproof,
        Keyword::Indestructible,
        Keyword::Lifelink,
        Keyword::Menace,
        Keyword::Reach,
        Keyword::Trample,
        Keyword::Vigilance,
    ]
}

/// "This has [kw] as long as [count] ≥ 1", one static per keyword.
fn keyword_while(kws: Vec<Keyword>, count: impl Fn(R) -> Value) -> Vec<StaticAbility> {
    kws.into_iter()
        .map(|kw| StaticAbility {
            description: "Has this keyword while a matching creature card is there.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::ValueAtLeast(count(R::Creature.and(R::HasKeyword(kw.clone()))), Value::ONE),
                applies_to: Selector::This,
                power: 0,
                toughness: 0,
                keywords: vec![kw],
            },
        })
        .collect()
}

/// Kathril, Aspect Warper — entering, a counter of each listed keyword a
/// creature card in your graveyard has, on a creature you control; a +1/+1
/// counter on Kathril per counter placed.
pub fn kathril_aspect_warper() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::KeywordCountersFromGraveyard { keywords: kathril_keywords() })],
        ..creature(
            "Kathril, Aspect Warper",
            cost(&[generic(2), w(), b(), g()]),
            vec![CreatureType::Nightmare, CreatureType::Insect],
            3,
            3,
        )
    }
}

/// Abzan Ascendancy — a +1/+1 counter on each of your creatures on entry;
/// your nontoken creatures dying leave 1/1 flying Spirits.
pub fn abzan_ascendancy() -> CardDefinition {
    CardDefinition {
        name: "Abzan Ascendancy",
        cost: cost(&[w(), b(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::AddCounter { what: yours(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition {
                        name: "Spirit".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Ajani Unyielding — +2: nonland permanents from the top three to hand;
/// −2: exile a creature, its controller gains its power in life; −9: five
/// +1/+1 counters on each creature and five loyalty on each other
/// planeswalker you control.
pub fn ajani_unyielding() -> CardDefinition {
    CardDefinition {
        name: "Ajani Unyielding",
        cost: cost(&[generic(4), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Ajani], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                x_cost: false,
                effect: Effect::RevealTopTakeMatchingToHand {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    filter: R::Permanent.and(R::Nonland),
                    distinct_powers: false,
                },
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                x_cost: false,
                effect: Effect::Seq(vec![
                    Effect::GainLife {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                        amount: Value::PowerOf(Box::new(target_filtered(R::Creature))),
                    },
                    Effect::Exile { what: Selector::Target(0) },
                ]),
            },
            LoyaltyAbility {
                loyalty_cost: -9,
                x_cost: false,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: yours(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(5),
                    },
                    Effect::AddCounter {
                        what: yours(R::Planeswalker.and(R::OtherThanSource)),
                        kind: CounterType::Loyalty,
                        amount: Value::Const(5),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Archon of Valor's Reach — flying, vigilance, trample; as it enters,
/// choose artifact, enchantment, instant, sorcery or planeswalker: no player
/// casts spells of that type.
/// Residual: the bot's pick is the first offered (instant).
pub fn archon_of_valors_reach() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample],
        as_enters_effect: Some(Effect::ChooseCardTypeAmongForSource(vec![
            CardType::Instant,
            CardType::Sorcery,
            CardType::Artifact,
            CardType::Enchantment,
            CardType::Planeswalker,
        ])),
        static_abilities: vec![StaticAbility {
            description: "Players can't cast spells of the chosen type.",
            effect: StaticEffect::NoOneCastsChosenCardType,
        }],
        ..creature("Archon of Valor's Reach", cost(&[generic(4), g(), w()]), vec![CreatureType::Archon], 5, 6)
    }
}

/// Avenging Huntbonder — double strike; attacking, a double strike counter on
/// another attacking creature.
pub fn avenging_huntbonder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![on_attack(Effect::AddKeywordCounter {
            what: target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
            keyword: Keyword::DoubleStrike,
            amount: Value::ONE,
        })],
        ..creature(
            "Avenging Huntbonder",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Bonder's Ornament — {T}: any color; {4}, {T}: each player controlling a
/// Bonder's Ornament draws.
pub fn bonders_ornament() -> CardDefinition {
    CardDefinition {
        name: "Bonder's Ornament",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                effect: Effect::Draw {
                    who: Selector::PlayersControlling(R::HasName("Bonder's Ornament".into())),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cairn Wanderer — changeling; has each listed keyword a creature card in
/// any graveyard has.
/// Residual: landwalk and protection are copied for the five basic land types
/// and the five colors only.
pub fn cairn_wanderer() -> CardDefinition {
    let mut kws = vec![
        Keyword::Flying,
        Keyword::Fear,
        Keyword::FirstStrike,
        Keyword::DoubleStrike,
        Keyword::Deathtouch,
        Keyword::Haste,
        Keyword::Lifelink,
        Keyword::Reach,
        Keyword::Trample,
        Keyword::Shroud,
        Keyword::Vigilance,
    ];
    kws.extend(
        [LandType::Plains, LandType::Island, LandType::Swamp, LandType::Mountain, LandType::Forest]
            .into_iter()
            .map(Keyword::Landwalk),
    );
    kws.extend([Color::White, Color::Blue, Color::Black, Color::Red, Color::Green].into_iter().map(Keyword::Protection));
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        static_abilities: keyword_while(kws, |f| Value::CardsInAllGraveyardsMatching { filter: f }),
        ..creature("Cairn Wanderer", cost(&[generic(4), b()]), vec![CreatureType::Shapeshifter], 4, 4)
    }
}

/// Ever After — up to two creature cards back from your graveyard as black
/// Zombies in addition; it goes to the bottom of its owner's library.
pub fn ever_after() -> CardDefinition {
    CardDefinition {
        name: "Ever After",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        library_bottom_on_resolve: true,
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::InYourGraveyard),
            effect: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::BecomeColor {
                    what: Selector::Target(0),
                    colors: vec![Color::Black],
                    duration: Duration::Permanent,
                    additive: true,
                },
                Effect::AddCreatureTypes {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Zombie],
                    duration: Duration::Permanent,
                },
            ])),
        },
        ..Default::default()
    }
}

/// Majestic Myriarch — twice your creature count in power and toughness; at
/// each combat, gains each listed keyword a creature you control has.
pub fn majestic_myriarch() -> CardDefinition {
    let pt = || Value::Times(Box::new(Value::Const(2)), Box::new(Value::CountOf(Box::new(yours(R::Creature)))));
    let mut kws = kathril_keywords();
    kws.insert(4, Keyword::Haste);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Its power and toughness are each equal to twice the number of creatures you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: pt(), toughness: pt() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
            effect: Effect::GainKeywordsYourCreaturesHave { what: Selector::This, keywords: kws },
        }],
        ..creature("Majestic Myriarch", cost(&[generic(4), g()]), vec![CreatureType::Chimera], 0, 0)
    }
}

/// Nesting Grounds — {T}: {C}; {1}, {T}: move a counter from one of your
/// permanents onto another permanent (sorcery speed).
pub fn nesting_grounds() -> CardDefinition {
    CardDefinition {
        name: "Nesting Grounds",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                sorcery_speed: true,
                effect: Effect::MoveOneCounter {
                    from: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::WithAnyCounter)),
                    to: Selector::TargetFiltered { slot: 1, filter: R::Permanent },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Netherborn Altar — {T}, a soul counter on it: your commander from the
/// command zone to your hand, then 3 life lost per soul counter.
pub fn netherborn_altar() -> CardDefinition {
    CardDefinition {
        name: "Netherborn Altar",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            add_counter_cost: Some((CounterType::Soul, 1)),
            effect: Effect::Seq(vec![
                Effect::CommanderToHand { who: PlayerRef::You },
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Times(
                        Box::new(Value::Const(3)),
                        Box::new(Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Soul }),
                    ),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nikara, Lair Scavenger — partner with Yannik; menace; another creature of
/// yours leaving with counters on it draws you a card for 1 life.
pub fn nikara_lair_scavenger() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::PartnerWith("Yannik, Scavenging Sentinel".into()), Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                    Predicate::TriggerSourceHadCounters,
                ]),
            ),
            effect: Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]),
        }],
        ..creature(
            "Nikara, Lair Scavenger",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Predatory Impetus — +3/+3; must be blocked if able; goaded.
pub fn predatory_impetus() -> CardDefinition {
    CardDefinition {
        name: "Predatory Impetus",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::MustBeBlocked],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is goaded.",
            effect: StaticEffect::AttachedIsGoaded,
        }],
        ..Default::default()
    }
}

/// Selective Adaptation — reveal seven; a card per listed keyword: one onto
/// the battlefield, the others to hand, the rest to the graveyard.
pub fn selective_adaptation() -> CardDefinition {
    let mut kws = kathril_keywords();
    kws.insert(4, Keyword::Haste);
    CardDefinition {
        name: "Selective Adaptation",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealTopChooseByKeyword { count: Value::Const(7), keywords: kws },
        ..Default::default()
    }
}

/// Slippery Bogbonder — flash, hexproof; entering, a hexproof counter on a
/// creature, then your other creatures' counters move onto it.
/// Residual: every counter moves (the engine's "any number").
pub fn slippery_bogbonder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Hexproof],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddKeywordCounter {
                what: target_filtered(R::Creature),
                keyword: Keyword::Hexproof,
                amount: Value::ONE,
            },
            Effect::MoveCountersFromAmongOnto { onto: Selector::Target(0) },
        ]))],
        ..creature(
            "Slippery Bogbonder",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Soulflayer — delve; has each listed keyword a creature card exiled with
/// its delve has.
pub fn soulflayer() -> CardDefinition {
    let mut kws = kathril_keywords();
    kws.retain(|k| *k != Keyword::Menace);
    kws.insert(4, Keyword::Haste);
    CardDefinition {
        keywords: vec![Keyword::Delve],
        links_delved_cards: true,
        static_abilities: keyword_while(kws, Value::CardsExiledWithSourceMatching),
        ..creature("Soulflayer", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 4, 4)
    }
}

/// Tayam, Luminous Enigma — your other creatures get a vigilance counter as
/// they enter; {3}, remove three counters from among your creatures: mill
/// three, then a permanent card of mana value 3 or less from your graveyard
/// onto the battlefield.
/// Residual: the vigilance counter arrives by trigger.
pub fn tayam_luminous_enigma() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::AddKeywordCounter {
                what: Selector::TriggerSource,
                keyword: Keyword::Vigilance,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            remove_counter_among_filter: Some((None, 3, R::Creature)),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(3) },
                Effect::PutGraveyardCardOntoBattlefield {
                    filter: R::Permanent.and(R::ManaValueAtMost(3)).and(R::InYourGraveyard),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Tayam, Luminous Enigma",
            cost(&[generic(1), w(), b(), g()]),
            vec![CreatureType::Nightmare, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Vitality Hunter — lifelink; {X}{W}{W}: monstrosity X; becoming monstrous,
/// lifelink counters on up to X creatures.
/// Residual: the counters go on your X greatest-power creatures.
pub fn vitality_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), w(), w()]),
            effect: Effect::Monstrosity { n: Value::XFromCost },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameMonstrous, EventScope::SelfSource),
            effect: Effect::AddKeywordCounter {
                what: Selector::GreatestPowerTopN { filter: R::Creature, count: Value::TriggerEventAmount },
                keyword: Keyword::Lifelink,
                amount: Value::ONE,
            },
        }],
        ..creature("Vitality Hunter", cost(&[generic(3), w()]), vec![CreatureType::Nightmare], 3, 4)
    }
}

/// Yannik, Scavenging Sentinel — partner with Nikara; vigilance; entering,
/// exile another creature you control until Yannik leaves, then distribute
/// its power in +1/+1 counters.
/// Residual: the engine exiles your highest mana value other creature, and X
/// reads its printed power.
pub fn yannik_scavenging_sentinel() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::PartnerWith("Nikara, Lair Scavenger".into()), Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileUntilSourceLeaves {
                what: Selector::GreatestManaValueControlledMatching {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::OtherThanSource),
                },
                return_to: ExileReturnZone::Battlefield,
            },
            Effect::DistributeCounters {
                total: Value::PowerOf(Box::new(Selector::CardExiledWithSource)),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature,
                max_targets: 5,
            },
        ]))],
        ..creature(
            "Yannik, Scavenging Sentinel",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Hyena, CreatureType::Beast],
            3,
            3,
        )
    }
}
