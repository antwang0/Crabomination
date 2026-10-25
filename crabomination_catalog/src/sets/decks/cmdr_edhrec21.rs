//! Commander: The Emperor of Palamecia and Owen Grady, Raptor Trainer's last
//! gaps (`cmdr_edhrec20` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec21.rs`.

use crate::card::{
    Adventure, AdditionalCastCost, CardDefinition, CardType, CreatureType, DynamicPt, EnchantmentSubtype,
    Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, Value, Zone,
};
use crate::effect::shortcut::{etb, partner_with_search, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, LookPick, PlayerRef, ZoneDest, ZoneRef};
use crate::mana::{cost, g, generic, hybrid, r, u, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost, statics: Vec<StaticAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: statics,
        ..Default::default()
    }
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn grant(description: &'static str, keyword: Keyword) -> StaticAbility {
    StaticAbility { description, effect: StaticEffect::GrantKeyword { applies_to: enchanted(), keyword } }
}

/// "As long as enchanted creature is `color`, `inner`."
fn while_enchanted_is(description: &'static str, color: Color, inner: StaticEffect) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::WhileCondition {
            condition: Predicate::EntityMatches { what: enchanted(), filter: R::HasColor(color) },
            inner: Box::new(inner),
        },
    }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn spells_in_your_graveyard() -> Value {
    Value::count(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: instant_or_sorcery() })
}

fn dinosaur() -> R {
    R::HasCreatureType(CreatureType::Dinosaur)
}

// ── The Emperor of Palamecia ─────────────────────────────────────────────────

/// Kinetic Augur — {3}{R} */4 Human Shaman, trample. Power = instant and
/// sorcery cards in your graveyard. ETB: discard up to two, then draw that many.
pub fn kinetic_augur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(DynamicPt::InstantsSorceriesInControllerGraveyard { base_t: 4 }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DiscardAnyNumber { who: Selector::You, filter: R::Any, max: Some(Value::Const(2)) },
            Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisEffect },
        ]))],
        ..creature("Kinetic Augur", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Shaman], 0, 4)
    }
}

/// Clout of the Dominus — {U/R} Aura. Blue: +1/+1 and shroud. Red: +1/+1 and
/// haste.
pub fn clout_of_the_dominus() -> CardDefinition {
    let pump = || StaticEffect::PumpPT { applies_to: enchanted(), power: 1, toughness: 1 };
    let has = |keyword| StaticEffect::GrantKeyword { applies_to: enchanted(), keyword };
    aura(
        "Clout of the Dominus",
        cost(&[hybrid(Color::Blue, Color::Red)]),
        vec![
            while_enchanted_is("As long as enchanted creature is blue, it gets +1/+1.", Color::Blue, pump()),
            while_enchanted_is("As long as enchanted creature is blue, it has shroud.", Color::Blue, has(Keyword::Shroud)),
            while_enchanted_is("As long as enchanted creature is red, it gets +1/+1.", Color::Red, pump()),
            while_enchanted_is("As long as enchanted creature is red, it has haste.", Color::Red, has(Keyword::Haste)),
        ],
    )
}

/// Protective Bubble — {3}{U} Aura. Enchanted creature can't be blocked and
/// has shroud.
pub fn protective_bubble() -> CardDefinition {
    aura(
        "Protective Bubble",
        cost(&[generic(3), u()]),
        vec![
            grant("Enchanted creature can't be blocked.", Keyword::Unblockable),
            grant("Enchanted creature has shroud.", Keyword::Shroud),
        ],
    )
}

/// Jace, Memory Adept — {3}{U}{U}, loyalty 4. +1: draw a card, target player
/// mills one. 0: target player mills ten. −7: any number of target players
/// each draw twenty (eight slots: a pod has at most eight seats).
pub fn jace_memory_adept() -> CardDefinition {
    let mill = |n| Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(n) };
    CardDefinition {
        name: "Jace, Memory Adept",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![Effect::Draw { who: Selector::You, amount: Value::ONE }, mill(1)]),
                ..Default::default()
            },
            LoyaltyAbility { loyalty_cost: 0, effect: mill(10), ..Default::default() },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Player,
                    effect: Box::new(Effect::Draw {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::Const(20),
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Experimental Overload — {2}{U}{R} sorcery. An X/X blue and red Weird, X =
/// instant and sorcery cards in your graveyard; then you may return one of
/// them to your hand. Exile it.
pub fn experimental_overload() -> CardDefinition {
    let x = spells_in_your_graveyard;
    let weird = TokenDefinition {
        name: "Weird".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Weird], ..Default::default() },
        dynamic_pt: Some((x(), x())),
        ..Default::default()
    };
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Experimental Overload",
            cost(&[generic(2), u(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(weird) },
                Effect::MayDo {
                    description: "Return an instant or sorcery card from your graveyard to your hand?".into(),
                    body: Box::new(Effect::MoveChosen {
                        from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: instant_or_sorcery() },
                        filter: None,
                        count: Value::ONE,
                        up_to: false,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                },
            ]),
        )
    }
}

// ── Owen Grady, Raptor Trainer ───────────────────────────────────────────────

/// Owen Grady, Raptor Trainer — {1}{R}{G} 3/2 legendary Human Soldier
/// Scientist. Partner with Blue, Loyal Raptor. {T}: your choice of a menace,
/// trample, reach or haste counter on target Dinosaur (sorcery speed).
pub fn owen_grady_raptor_trainer() -> CardDefinition {
    let counter = |keyword| Effect::AddKeywordCounter { what: target_filtered(dinosaur()), keyword, amount: Value::ONE };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::PartnerWith("Blue, Loyal Raptor".into())],
        triggered_abilities: vec![partner_with_search("Blue, Loyal Raptor")],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::ChooseMode(vec![
                counter(Keyword::Menace),
                counter(Keyword::Trample),
                counter(Keyword::Reach),
                counter(Keyword::Haste),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Owen Grady, Raptor Trainer",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Scientist],
            3,
            2,
        )
    }
}

/// Blue, Loyal Raptor — {2}{G}{U} legendary 5/4 Dinosaur, partner with Owen
/// Grady, Raptor Trainer. For each kind of counter on Blue, each other
/// Dinosaur you control enters with a counter of that kind on it (CR 614.12;
/// keyword counters are kinds, CR 122.1b).
pub fn blue_loyal_raptor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::PartnerWith("Owen Grady, Raptor Trainer".into())],
        triggered_abilities: vec![partner_with_search("Owen Grady, Raptor Trainer")],
        static_abilities: vec![StaticAbility {
            description: "For each kind of counter on Blue, Loyal Raptor, each other Dinosaur you control enters with a counter of that kind on it.",
            effect: StaticEffect::MatchingEntersWith {
                filter: dinosaur(),
                effect: Box::new(Effect::AddCounterOfEachKindAmong {
                    onto: Selector::Target(0),
                    who: PlayerRef::You,
                    filter: R::IsSource,
                    or_plus_one: false,
                }),
            },
        }],
        ..creature("Blue, Loyal Raptor", cost(&[generic(2), g(), u()]), vec![CreatureType::Dinosaur], 5, 4)
    }
}

/// Cresting Mosasaurus — {6}{U}{U} 4/8 Dinosaur, emerge {6}{U}. When it
/// enters, if you cast it (CR 603.4), return each non-Dinosaur creature to
/// its owner's hand.
pub fn cresting_mosasaurus() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::effect::shortcut::emerge(cost(&[generic(6), u()]))),
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: crate::card::EventSpec::new(crate::card::EventKind::EntersBattlefield, crate::card::EventScope::SelfSource)
                .with_filter(Predicate::SourceWasCast),
            effect: Effect::Move {
                what: Selector::EachPermanent(R::Creature.and(dinosaur().negate())),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature("Cresting Mosasaurus", cost(&[generic(6), u(), u()]), vec![CreatureType::Dinosaur], 4, 8)
    }
}

/// Dino DNA — {1} artifact. Imprint — {1}, {T}: exile target creature card
/// from a graveyard (sorcery speed). {6}: a token copy of a creature card
/// exiled with it, except a 6/6 green Dinosaur with trample (sorcery speed).
pub fn dino_dna() -> CardDefinition {
    let exiled = || R::ExiledWithSource.and(R::Creature);
    CardDefinition {
        name: "Dino DNA",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::ExileLinked { what: target_filtered(R::Creature.and(R::InGraveyard)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(6)]),
                sorcery_speed: true,
                condition: Some(Predicate::SelectorExists(Selector::EachMatching {
                    zone: ZoneRef::Exile,
                    filter: exiled(),
                })),
                effect: Effect::ChooseOneAmong {
                    what: Selector::EachMatching { zone: ZoneRef::Exile, filter: exiled() },
                    chooser: PlayerRef::You,
                    chosen: Box::new(Effect::Seq(vec![
                        Effect::CreateTokenCopyOf {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            source: Selector::SeparatedPile { chosen: true },
                            extra_creature_types: vec![],
                            extra_card_types: vec![],
                            override_pt: Some((6, 6)),
                            override_colors: Some(vec![Color::Green]),
                            enters_tapped: false,
                            non_legendary: false,
                            legendary: false,
                            extra_keywords: vec![Keyword::Trample],
                            no_mana_cost: false,
                            enters_with_counters: None,
                            remove_keywords: vec![],
                        },
                        Effect::AmendCopiableValues {
                            what: Selector::LastCreatedToken,
                            name: None,
                            set_creature_types: Some(vec![CreatureType::Dinosaur]),
                            add_creature_types: vec![],
                            legendary: false,
                            add_colors: vec![],
                            set_card_types: None,
                        },
                    ])),
                    other: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Monster Manual // Zoological Study — {3}{G} artifact — Book: {1}{G}, {T}:
/// you may put a creature card from your hand onto the battlefield. Adventure
/// {2}{G} sorcery: mill five, then return a creature card milled this way.
pub fn monster_manual() -> CardDefinition {
    CardDefinition {
        name: "Monster Manual",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Book], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature,
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
            ..Default::default()
        }],
        adventure: Some(Box::new(Adventure {
            name: "Zoological Study",
            cost: cost(&[generic(2), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::MillThenToHand { amount: Value::Const(5), filter: R::Creature, otherwise: None },
        })),
        ..Default::default()
    }
}

/// Tranquil Frillback — {2}{G} 3/3 Dinosaur. ETB: pay {G} up to three times,
/// then choose up to that many of: destroy target artifact or enchantment;
/// exile target player's graveyard; gain 4 life. Each payment buys one mode.
pub fn tranquil_frillback() -> CardDefinition {
    let pay = |description: &str, body: Effect| Effect::MayPay {
        description: description.into(),
        mana_cost: cost(&[g()]),
        body: Box::new(Effect::ReflexiveTrigger { body: Box::new(body) }),
        else_: None,
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            pay(
                "Pay {G}: destroy target artifact or enchantment?",
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ),
            pay(
                "Pay {G}: exile target player's graveyard?",
                Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
            ),
            pay("Pay {G}: gain 4 life?", Effect::GainLife { who: Selector::You, amount: Value::Const(4) }),
        ]))],
        ..creature("Tranquil Frillback", cost(&[generic(2), g()]), vec![CreatureType::Dinosaur], 3, 3)
    }
}

/// Commune with Dinosaurs — {G} sorcery. Look at the top five; you may reveal
/// a Dinosaur or land card and put it into your hand; the rest on the bottom.
pub fn commune_with_dinosaurs() -> CardDefinition {
    spell(
        "Commune with Dinosaurs",
        cost(&[g()]),
        CardType::Sorcery,
        Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(5),
            pick_filter: Some(dinosaur().or(R::Land)),
            optional: true,
            ..Default::default()
        })),
    )
}

/// Savage Order — {2}{G}{G} sorcery. Additional cost: sacrifice a creature
/// with power 4 or greater. Search for a Dinosaur creature card onto the
/// battlefield; it gains indestructible until your next turn.
pub fn savage_order() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
            count: 1,
        }],
        ..spell(
            "Savage Order",
            cost(&[generic(2), g(), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature.and(dinosaur()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword {
                    what: Selector::LastMoved,
                    keyword: Keyword::Indestructible,
                    duration: Duration::UntilYourNextUntap,
                },
            ]),
        )
    }
}
