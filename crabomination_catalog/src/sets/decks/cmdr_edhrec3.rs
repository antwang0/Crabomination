//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, third file (`cmdr_edhrec2` is the
//! second). Ketramose, the New Dawn, Niko, Light of Hope, Syr Gwyn, Hero of
//! Ashvale, Kastral, the Windcrested, Jodah, Archmage Eternal and Child of
//! Alara. Tests in
//! `tests/recent_b/cmdr_edhrec3.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, TriggerZone, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, SpendRestriction};
use crabomination_base::tokens::treasure_token;
use std::sync::Arc;

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn legend(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    }
}

/// "Commander creatures you own have [ability]." (the Background shape,
/// `cmdr_familiars`).
fn grant_to_your_commanders(description: &'static str, ability: TriggeredAbility) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::GrantTriggeredAbility {
            filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
            ability: Box::new(ability),
        },
    }
}

/// Flickering Hound — {3}{W} 2/2 Dog. Whenever you cast a creature spell,
/// exile up to one other target creature you control, then return that card
/// to the battlefield under its owner's control.
pub fn flickering_hound() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature)),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileAndReturnToOwner {
                    what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                }),
            },
        }],
        ..creature("Flickering Hound", cost(&[generic(3), w()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Unlicensed Hearse — {2} */* Vehicle. {T}: exile up to two target cards
/// from a single graveyard; its power and toughness are each the number of
/// cards exiled with it (CR 604.3). Crew 2. The picks are made as the
/// ability resolves (`ExileUpToNFromGraveyards`, the Rag Dealer shape), then
/// linked to the Hearse.
pub fn unlicensed_hearse() -> CardDefinition {
    let count = || Value::CardsExiledWithSourceCount;
    CardDefinition {
        name: "Unlicensed Hearse",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        keywords: vec![Keyword::Crew(2)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::ExileUpToNFromGraveyards { count: Value::Const(2), of: None, single: true },
                Effect::LinkLastMovedExilesToSource,
            ]),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Unlicensed Hearse's power and toughness are each equal to the number of cards exiled with it.",
            effect: StaticEffect::SelfBasePtFromValue { power: count(), toughness: count() },
        }],
        ..Default::default()
    }
}

/// Abdel Adrian, Gorion's Ward — {4}{W} 4/4 legendary Human Warrior. On
/// entry, exile any number of other nonland permanents you control until it
/// leaves the battlefield (CR 610.3), and create a 1/1 white Soldier for
/// each one. Choose a Background.
pub fn abdel_adrian_gorions_ward() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        can_be_commander: true,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileAnyNumberUntilSourceLeaves { filter: R::Nonland, shelter: false },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CardsExiledWithSourceCount,
                definition: Arc::new(soldier()),
            },
        ]))],
        ..legend(
            "Abdel Adrian, Gorion's Ward",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Senu, Keen-Eyed Protector — {1}{W} 2/1 legendary Bird Scout, flying,
/// vigilance. {T}, exile it: gain 2 life and scry 2. From exile (CR 113.6,
/// `TriggerZone::WhileExiled`): when a legendary creature you control attacks
/// and isn't blocked, if this card is still exiled (CR 603.4), it enters
/// attacking (CR 508.4 — its controller picks what it attacks).
pub fn senu_keen_eyed_protector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                zone: TriggerZone::WhileExiled,
                ..EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::HasSupertype(Supertype::Legendary).and(R::Creature),
                        },
                        Predicate::EntityMatches { what: Selector::This, filter: R::InExile },
                    ]),
                )
            },
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: Selector::This, filter: R::InExile },
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::JoinCombatAttackingChosen { what: Selector::LastMoved, cleanup: Default::default() },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..legend(
            "Senu, Keen-Eyed Protector",
            cost(&[generic(1), w()]),
            vec![CreatureType::Bird, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Battle Angels of Tyr — {2}{W}{W} 4/4 Angel Knight, flying, myriad. On
/// combat damage to a player: draw if that player has more cards in hand
/// than each other player, then a Treasure if they control more lands than
/// each other player, then 3 life if they have more life than each other
/// player — each read in turn, after the one before.
pub fn battle_angels_of_tyr() -> CardDefinition {
    let them = || PlayerRef::TriggerEventPlayer;
    let leads = |value: Value| Predicate::PlayerValueExceedsEachOther { who: them(), value };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: Effect::Myriad },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::If {
                        cond: leads(Value::HandSizeOf(PlayerRef::You)),
                        then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                        else_: Box::new(Effect::Noop),
                    },
                    Effect::If {
                        cond: Predicate::PlayerControlsMostOf { who: them(), filter: R::Land },
                        then: Box::new(Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Arc::new(treasure_token()),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                    Effect::If {
                        cond: leads(Value::LifeOf(PlayerRef::You)),
                        then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..creature(
            "Battle Angels of Tyr",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Candlekeep Sage — {2}{U} legendary Background. Commander creatures you own
/// have "When this creature enters or leaves the battlefield, draw a card."
pub fn candlekeep_sage() -> CardDefinition {
    let draw = || Effect::Draw { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        name: "Candlekeep Sage",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![
            grant_to_your_commanders(
                "Commander creatures you own have \"When this creature enters, draw a card.\"",
                TriggeredAbility { event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource), effect: draw() },
            ),
            grant_to_your_commanders(
                "Commander creatures you own have \"When this creature leaves the battlefield, draw a card.\"",
                TriggeredAbility {
                    event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                    effect: draw(),
                },
            ),
        ],
        ..Default::default()
    }
}

/// The Shard token Niko makes: an enchantment with "{2}, Sacrifice this
/// token: Scry 1, then draw a card."
fn shard() -> TokenDefinition {
    TokenDefinition {
        name: "Shard".into(),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn shards(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(shard()) }
}

/// Niko Aris — {X}{W}{U}{U} legendary planeswalker, loyalty 3. Enters with X
/// Shards. +1: up to one target creature you control can't be blocked this
/// turn, and whenever it deals damage this turn it returns to its owner's
/// hand. −1: 2 damage to target tapped creature per card you've drawn this
/// turn. −1: a Shard.
pub fn niko_aris() -> CardDefinition {
    let bounce_on_damage = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
        effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))) },
    };
    CardDefinition {
        name: "Niko Aris",
        cost: cost(&[x(), w(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Niko], ..Default::default() },
        base_loyalty: 3,
        triggered_abilities: vec![etb(shards(Value::XFromCost))],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::GrantKeyword {
                            what: target_filtered(R::Creature.and(R::ControlledByYou)),
                            keyword: Keyword::Unblockable,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantTriggeredAbility {
                            what: Selector::Target(0),
                            trigger: Box::new(bounce_on_damage),
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::Tapped)),
                    amount: Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(Value::CardsDrawnThisTurn(PlayerRef::You)),
                    ),
                },
                ..Default::default()
            },
            LoyaltyAbility { loyalty_cost: -1, effect: shards(Value::ONE), ..Default::default() },
        ],
        ..Default::default()
    }
}

fn aura_card() -> R {
    R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
}

fn equipment_card() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Equipment)
}

fn tap_for(pool: ManaPayload) -> ActivatedAbility {
    ActivatedAbility { tap_cost: true, effect: Effect::AddMana { who: PlayerRef::You, pool }, ..Default::default() }
}

fn land(name: &'static str) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Land], ..Default::default() }
}

/// Axgard Armory — Land, enters tapped (CR 614.1c). {T}: Add {W}.
/// {1}{R}{R}{W}, {T}, sacrifice it: search for an Aura card and/or an
/// Equipment card, reveal them, put them into your hand, then shuffle — one
/// search per category, either of which may find nothing (CR 701.23b).
pub fn axgard_armory() -> CardDefinition {
    let to_hand = |filter: R| Effect::Search { who: PlayerRef::You, filter, to: ZoneDest::Hand(PlayerRef::You) };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![
            tap_for(ManaPayload::Colors(vec![Color::White])),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(1), r(), r(), w()]),
                effect: Effect::Seq(vec![to_hand(aura_card()), to_hand(equipment_card())]),
                ..Default::default()
            },
        ],
        ..land("Axgard Armory")
    }
}

/// Tournament Grounds — Land. {T}: Add {C}. {T}: Add {R}, {W}, or {B};
/// spend it only to cast a Knight or Equipment spell (CR 106.6).
pub fn tournament_grounds() -> CardDefinition {
    let restricted = |c: Color| {
        tap_for(ManaPayload::Restricted(
            Box::new(ManaPayload::Colors(vec![c])),
            SpendRestriction::KnightOrEquipmentSpells,
        ))
    };
    CardDefinition {
        activated_abilities: vec![
            tap_for(ManaPayload::Colorless(Value::ONE)),
            restricted(Color::Red),
            restricted(Color::White),
            restricted(Color::Black),
        ],
        ..land("Tournament Grounds")
    }
}

/// Danitha, Benalia's Hope — {4}{W} 4/4 legendary Human Knight, first strike,
/// vigilance, lifelink. On entry you may put an Aura or Equipment card from
/// your hand or graveyard onto the battlefield attached to it (CR 303.4f /
/// 301.5c).
pub fn danitha_benalias_hope() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance, Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Put an Aura or Equipment card from your hand or graveyard onto the battlefield attached to Danitha?".into(),
            body: Box::new(Effect::PutOntoBattlefieldAttached {
                zones: vec![crate::card::Zone::Hand, crate::card::Zone::Graveyard],
                filter: aura_card().or(equipment_card()),
                host: Some(Selector::This),
                max: Some(Value::ONE),
                creatures_only: false,
                equipment_unattached: false,
            }),
        })],
        ..legend(
            "Danitha, Benalia's Hope",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Merry, Esquire of Rohan — {R}{W} 2/2 legendary Halfling Knight, haste.
/// First strike while equipped. Whenever you attack with it and another
/// legendary creature, draw a card (one trigger per declaration, CR 508.1).
pub fn merry_esquire_of_rohan() -> CardDefinition {
    let other_legend_attacking = Value::count(Selector::EachPermanent(
        R::Creature
            .and(R::IsAttacking)
            .and(R::ControlledByYou)
            .and(R::OtherThanSource)
            .and(R::HasSupertype(Supertype::Legendary)),
    ));
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Merry has first strike as long as it's equipped.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsEquipped,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::ValueAtLeast(other_legend_attacking, Value::ONE)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend(
            "Merry, Esquire of Rohan",
            cost(&[r(), w()]),
            vec![CreatureType::Halfling, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Battlefield Raptor — {W} 1/2 Bird, flying, first strike.
pub fn battlefield_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..creature("Battlefield Raptor", cost(&[w()]), vec![CreatureType::Bird], 1, 2)
    }
}

/// Scouting Hawk — {2}{W} 1/1 Bird, flying. Keen Sight: on entry, if an
/// opponent controls more lands than you (CR 603.4), search for a basic
/// Plains and put it onto the battlefield tapped.
pub fn scouting_hawk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::OpponentControlsMoreLandsThanYou),
            effect: Effect::If {
                cond: Predicate::OpponentControlsMoreLandsThanYou,
                then: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand.and(R::HasLandType(crate::card::LandType::Plains)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Scouting Hawk", cost(&[generic(2), w()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Lofty Denial — {1}{U} instant. Counter target spell unless its controller
/// pays {1}, or {4} if you control a creature with flying (read on
/// resolution).
pub fn lofty_denial() -> CardDefinition {
    let counter = |n: u32| Effect::CounterUnlessPaid {
        what: target_filtered(R::IsSpellOnStack),
        mana_cost: cost(&[generic(n)]),
        exile: false,
        extra_generic: None,
        if_paid: None,
    };
    let flier = Value::count(Selector::EachPermanent(
        R::Creature.and(R::ControlledByYou).and(R::HasKeyword(Keyword::Flying)),
    ));
    CardDefinition {
        name: "Lofty Denial",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(flier, Value::ONE),
            then: Box::new(counter(4)),
            else_: Box::new(counter(1)),
        },
        ..Default::default()
    }
}

fn bird_soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Bird Soldier".into(),
        power: 4,
        toughness: 4,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird, CreatureType::Soldier], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// The Eagles Are Coming! — {1}{W} instant, kicker {2}{W}{W}. Return target
/// creature you own — kicked, any number of them — to your hand; at the
/// beginning of the next upkeep (CR 603.7), a 4/4 white flying Bird Soldier
/// for each one that reached your hand. The cast offers the kicked ceiling;
/// unkicked, `CapTargetsAt` keeps one (CR 601.2c). Each returned creature
/// registers its own delayed Bird, so the tokens arrive at the same upkeep.
pub fn the_eagles_are_coming() -> CardDefinition {
    let cap = Value::Sum(vec![
        Value::ONE,
        Value::Times(Box::new(Value::OneIf(Box::new(Predicate::SpellWasKicked))), Box::new(Value::Const(99))),
    ]);
    CardDefinition {
        name: "The Eagles Are Coming!",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[generic(2), w(), w()]))],
        effect: Effect::CapTargetsAt {
            amount: cap,
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 1,
                filter: R::Creature.and(R::OwnedByYou),
                effect: Box::new(Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) },
                    Effect::If {
                        cond: Predicate::SelectorExists(Selector::LastMoved),
                        then: Box::new(Effect::DelayUntil {
                            kind: crate::effect::DelayedTriggerKind::NextUpkeep,
                            body: Box::new(Effect::CreateToken {
                                who: PlayerRef::You,
                                count: Value::ONE,
                                definition: Arc::new(bird_soldier()),
                            }),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            }),
        },
        ..Default::default()
    }
}

/// Imoti, Celebrant of Bounty — {3}{G}{U} 3/1 legendary Snake Druid,
/// cascade. Spells you cast with mana value 6 or greater have cascade (CR
/// 702.85: cascade triggers on cast, below the spell's own mana value).
pub fn imoti_celebrant_of_bounty() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cascade],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::ValueAtLeast(
                Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                Value::Const(6),
            )),
            effect: Effect::Cascade { max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)), filter: None },
        }],
        ..legend(
            "Imoti, Celebrant of Bounty",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Snake, CreatureType::Druid],
            3,
            1,
        )
    }
}

/// Leyline Immersion — {3}{G} Aura, enchant legendary creature. Enchanted
/// creature has ward {2} and "{T}: Add five mana in any combination of
/// colors. Spend this mana only to cast spells."
pub fn leyline_immersion() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Leyline Immersion",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
        },
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature has ward {2}.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: host(),
                    keyword: Keyword::Ward(crate::card::WardCost::generic(2)),
                },
            },
            StaticAbility {
                description: "Enchanted creature has \"{T}: Add five mana in any combination of colors. Spend this mana only to cast spells.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: host(),
                    ability: tap_for(ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyColors(Value::Const(5))),
                        SpendRestriction::SpellsOnly,
                    )),
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Nicol Bolas, God-Pharaoh — {4}{U}{B}{R} legendary planeswalker, loyalty 7.
/// +2: target opponent exiles from the top until a nonland card; you may cast
/// it free this turn. +1: each opponent exiles two cards from their hand.
/// −4: 7 damage to target opponent, or creature or planeswalker an opponent
/// controls. −12: exile each nonland permanent your opponents control.
pub fn nicol_bolas_god_pharaoh() -> CardDefinition {
    CardDefinition {
        name: "Nicol Bolas, God-Pharaoh",
        cost: cost(&[generic(4), u(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Bolas], ..Default::default() },
        base_loyalty: 7,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::TargetPlayerThen {
                    filter: R::Player.and(R::ControlledByOpponent),
                    then: Box::new(Effect::ExileTopUntilNonlandMayPlay {
                        who: PlayerRef::Target(0),
                        duration: crate::card::MayPlayDuration::EndOfThisTurn,
                        free: true,
                        hand_unless_mv_below: None,
                        grant_to_exiling_player: false,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::ExileFromHand { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::DealDamage {
                    to: target_filtered(
                        R::OpponentPlayer
                            .or(R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent)),
                    ),
                    amount: Value::Const(7),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -12,
                effect: Effect::Exile { what: Selector::EachPermanent(R::Nonland.and(R::ControlledByOpponent)) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Emergent Ultimatum — {B}{B}{G}{G}{G}{U}{U} sorcery. Search for up to three
/// monocolored cards with different names and exile them; an opponent
/// chooses one to shuffle back; you may cast the others free (CR 118.9).
/// Exile it as it resolves.
pub fn emergent_ultimatum() -> CardDefinition {
    CardDefinition {
        name: "Emergent Ultimatum",
        cost: cost(&[b(), b(), g(), g(), g(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::SearchSplitOpponentChooses {
                opponent: Selector::None,
                count: 3,
                opponent_picks: 1,
                chosen_to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Shuffled },
                rest_to: ZoneDest::ExileWithSourceStamp,
                filter: Some(R::Monocolored),
            },
            Effect::CastAnyOrderWithoutPaying {
                what: Selector::CardExiledWithSource,
                source_zone: crate::card::Zone::Exile,
                filter: None,
                cap: None,
                total_mana_value: None,
            },
        ]),
        ..Default::default()
    }
}

fn gate() -> R {
    R::HasLandType(crate::card::LandType::Gate)
}

fn basic_or_gate() -> R {
    R::IsBasicLand.or(gate())
}

/// Navigation Orb — {3} artifact. {2}, {T}, sacrifice it: search for up to
/// two basic land and/or Gate cards, one onto the battlefield tapped and the
/// other into your hand (the Cultivate split, one search per destination).
pub fn navigation_orb() -> CardDefinition {
    CardDefinition {
        name: "Navigation Orb",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: basic_or_gate(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::Search { who: PlayerRef::You, filter: basic_or_gate(), to: ZoneDest::Hand(PlayerRef::You) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// District Guide — {2}{G} 2/2 Elf Scout. On entry you may search for a
/// basic land or Gate card and put it into your hand.
pub fn district_guide() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a basic land or Gate card?".into(),
            body: Box::new(Effect::Search { who: PlayerRef::You, filter: basic_or_gate(), to: ZoneDest::Hand(PlayerRef::You) }),
        })],
        ..creature("District Guide", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Scout], 2, 2)
    }
}

/// Nine-Fingers Keene — {1}{B}{G}{U} 4/4 legendary Human Rogue, menace,
/// ward—pay 9 life. Combat damage to a player: look at the top nine, you
/// may put a Gate from among them onto the battlefield; then with nine or
/// more Gates the rest go to your hand, else to the bottom in a random order.
pub fn nine_fingers_keene() -> CardDefinition {
    let nine_gates = Predicate::ValueAtLeast(
        Value::count(Selector::EachPermanent(gate().and(R::ControlledByYou))),
        Value::Const(9),
    );
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Ward(crate::card::WardCost::Life(9))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                count: Value::Const(9),
                pick_filter: Some(gate()),
                to_battlefield: true,
                optional: true,
                rest_bottom_random: true,
                rest_to_hand_if: Some(nine_gates),
                ..Default::default()
            })),
        }],
        ..legend(
            "Nine-Fingers Keene",
            cost(&[generic(1), b(), g(), u()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            4,
            4,
        )
    }
}

/// Guild Summit — {2}{U} enchantment. On entry, tap any number of untapped
/// Gates you control and draw a card for each. Whenever a Gate you control
/// enters, draw a card.
pub fn guild_summit() -> CardDefinition {
    CardDefinition {
        name: "Guild Summit",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::TapAnyNumberThenDraw { filter: gate() }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: gate() },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Nasty End — {1}{B} instant; as an additional cost, sacrifice a creature.
/// Draw two cards, or three if the sacrificed creature was legendary (read
/// from the sacrificed card, CR 608.2h).
pub fn nasty_end() -> CardDefinition {
    let draw = |n: i32| Effect::Draw { who: Selector::You, amount: Value::Const(n) };
    CardDefinition {
        name: "Nasty End",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 1 }],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::SacrificedCard,
                filter: R::HasSupertype(Supertype::Legendary),
            },
            then: Box::new(draw(3)),
            else_: Box::new(draw(2)),
        },
        ..Default::default()
    }
}
