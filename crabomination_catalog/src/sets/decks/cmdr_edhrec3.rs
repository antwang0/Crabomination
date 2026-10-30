//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, third file (`cmdr_edhrec2` is the
//! second). Ketramose, the New Dawn, Niko, Light of Hope, Syr Gwyn, Hero of
//! Ashvale, Kastral, the Windcrested, Jodah, Archmage Eternal, Child of Alara,
//! Rakdos, Lord of Riots, Be'lakor's Demons, Myrel, Shield of Argive, Fire Lord
//! Zuko, Arna Kennerüd, Terra, Magical Adept, Atreus // Kratos, Kratos, God of
//! War, Eddie Brock, Nicol Bolas, the Ravager, Shilgengar, Norin, Katara, the
//! Fearless, Azlask, the Swelling Scourge, The Mycotyrant, Thranduil, the
//! Elvenking, Maralen, Fae Ascendant, Najeela, the Blade-Blossom, Toph, the First
//! Metalbender and Avatar Aang. Tests in
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
                exile_after: false,
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
            effect: Effect::OneSearch { body: Box::new(Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: basic_or_gate(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::Search { who: PlayerRef::You, filter: basic_or_gate(), to: ZoneDest::Hand(PlayerRef::You) },
            ])) },
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

/// Cryptolith Fragment // Aurora of Emrakul — {3} artifact, enters tapped.
/// {T}: one mana of any color, and each player loses 1 life. At the
/// beginning of your upkeep, if each player has 10 or less life, transform
/// it (CR 701.28) into Aurora of Emrakul, a colorless 1/4 flying deathtouch Eldrazi
/// Reflection whose attacks drain each opponent for 3.
pub fn cryptolith_fragment() -> CardDefinition {
    let aurora = CardDefinition {
        name: "Aurora of Emrakul",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Reflection],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
        }],
        ..Default::default()
    };
    let everyone_low = Predicate::ValueAtMost(Value::HighestLifeTotal, Value::Const(10));
    CardDefinition {
        name: "Cryptolith Fragment",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(everyone_low.clone()),
            effect: Effect::If {
                cond: everyone_low,
                then: Box::new(Effect::Transform { what: Selector::This }),
                else_: Box::new(Effect::Noop),
            },
        }],
        back_face: Some(Box::new(aurora)),
        ..Default::default()
    }
}

/// Orcus, Prince of Undeath — {X}{2}{B}{R} 5/3 legendary Demon, flying,
/// trample. On entry, choose one: each other creature gets -X/-X until end
/// of turn and you lose X life; or return up to X target creature cards with
/// total mana value X or less from your graveyard (CR 601.2c across the
/// slots), and they gain haste until end of turn.
pub fn orcus_prince_of_undeath() -> CardDefinition {
    let minus_x = || Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost));
    let wither = Effect::Seq(vec![
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
            power: minus_x(),
            toughness: minus_x(),
            duration: Duration::EndOfTurn,
        },
        Effect::LoseLife { who: Selector::You, amount: Value::XFromCost },
    ]);
    let reanimate = Effect::WithX {
        x: Value::XFromCost,
        body: Box::new(Effect::ReflexiveTrigger {
            body: Box::new(Effect::CapTargetsAt {
                amount: Value::XFromCost,
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature.and(R::InYourGraveyard).and(R::SlotsTotalManaValueAtMostX),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                        Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                    ])),
                }),
            }),
        }),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![wither, reanimate]))],
        ..legend(
            "Orcus, Prince of Undeath",
            cost(&[x(), generic(2), b(), r()]),
            vec![CreatureType::Demon],
            5,
            3,
        )
    }
}

/// Sanctum of Stone Fangs — {1}{B} legendary Shrine. At the beginning of
/// your precombat main phase, each opponent loses X life and you gain X,
/// X the number of Shrines you control.
pub fn sanctum_of_stone_fangs() -> CardDefinition {
    let shrines = || Value::count(Selector::EachPermanent(
        R::HasEnchantmentSubtype(EnchantmentSubtype::Shrine).and(R::ControlledByYou),
    ));
    CardDefinition {
        name: "Sanctum of Stone Fangs",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Shrine], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: shrines() },
                Effect::GainLife { who: Selector::You, amount: shrines() },
            ]),
        }],
        ..Default::default()
    }
}

/// Sarkhan's Unsealing — {3}{R} enchantment. Casting a creature spell with
/// power 4, 5 or 6 deals 4 damage to any target; with power 7 or greater, 4
/// to each opponent and each creature and planeswalker they control.
pub fn sarkhans_unsealing() -> CardDefinition {
    let cast = |filter: R| {
        EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::Creature.and(filter)))
    };
    CardDefinition {
        name: "Sarkhan's Unsealing",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: cast(R::PowerAtLeast(4).and(R::PowerAtMost(6))),
                effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(4) },
            },
            TriggeredAbility {
                event: cast(R::PowerAtLeast(7)),
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(4) },
                    Effect::DealDamage {
                        to: Selector::EachPermanent(
                            R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
                        ),
                        amount: Value::Const(4),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

fn devil() -> TokenDefinition {
    TokenDefinition {
        name: "Devil".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Ob Nixilis, the Adversary — {1}{B}{R} legendary planeswalker, loyalty 3.
/// Casualty X: the copy isn't legendary and starts with loyalty X, the
/// sacrificed creature's power (CR 702.153a). +1: each opponent loses 2 life
/// unless they discard a card; then with a Demon or Devil you gain 2. −2: a
/// 1/1 red Devil with "when this dies, 1 damage to any target". −7: target
/// player draws seven and loses 7 life.
pub fn ob_nixilis_the_adversary() -> CardDefinition {
    let fiend = R::HasCreatureType(CreatureType::Demon).or(R::HasCreatureType(CreatureType::Devil));
    CardDefinition {
        name: "Ob Nixilis, the Adversary",
        cost: cost(&[generic(1), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Nixilis], ..Default::default() },
        base_loyalty: 3,
        keywords: vec![Keyword::Casualty(0)],
        static_abilities: vec![StaticAbility {
            description: "Casualty X. The copy isn't legendary and has starting loyalty X.",
            effect: StaticEffect::CasualtyCopyLoyaltyFromSacrifice,
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::ForEachOpponent {
                        body: Box::new(Effect::UnlessPlayerPays {
                            who: PlayerRef::Triggerer,
                            cost: crate::card::WardCost::Discard(1),
                            then: Box::new(Effect::LoseLife {
                                who: Selector::Player(PlayerRef::Triggerer),
                                amount: Value::Const(2),
                            }),
                            if_paid: None,
                        }),
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::count(Selector::EachPermanent(fiend.and(R::ControlledByYou))),
                            Value::ONE,
                        ),
                        then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(devil()) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::TargetPlayerThen {
                    filter: R::Player,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(7) },
                        Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(7) },
                    ])),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dream Devourer — {1}{B} 0/3 Demon Cleric. Each nonland card in your hand
/// without foretell has foretell, at its mana cost less {2} (CR 702.143;
/// the cost stays with a card foretold this way). Whenever you foretell a
/// card, it gets +2/+0 until end of turn.
pub fn dream_devourer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each nonland card in your hand without foretell has foretell. Its foretell cost is equal to its mana cost reduced by {2}.",
            effect: StaticEffect::HandNonlandCardsHaveForetell { reduce: 2 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardExiled, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::OwnedByYou.and(R::ForetoldThisTurn),
            }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Dream Devourer", cost(&[generic(1), b()]), vec![CreatureType::Demon, CreatureType::Cleric], 0, 3)
    }
}

/// Raphael, Fiendish Savior — {3}{B}{R} 4/4 legendary Devil Noble, flying.
/// Other Demons, Devils, Imps and Tieflings you control get +1/+1 and have
/// lifelink. At the beginning of each end step, if a creature card was put
/// into your graveyard from anywhere this turn (CR 603.4), create a 1/1 red
/// Devil with "when this dies, 1 damage to any target".
pub fn raphael_fiendish_savior() -> CardDefinition {
    let fiends = [CreatureType::Demon, CreatureType::Devil, CreatureType::Imp, CreatureType::Tiefling]
        .into_iter()
        .map(R::HasCreatureType)
        .reduce(|a, b| a.or(b))
        .expect("four types");
    let fed = Predicate::ValueAtLeast(Value::CreatureCardsPutIntoGraveyardThisTurn(PlayerRef::You), Value::ONE);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other Demons, Devils, Imps, and Tieflings you control get +1/+1 and have lifelink.",
            effect: StaticEffect::AnthemForFilter {
                filter: fiends.and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Lifelink],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::End), EventScope::AnyPlayer)
                .with_filter(fed.clone()),
            effect: Effect::If {
                cond: fed,
                then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(devil()) }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..legend(
            "Raphael, Fiendish Savior",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Devil, CreatureType::Noble],
            4,
            4,
        )
    }
}

/// Varragoth, Bloodsky Sire — {2}{B} 2/3 legendary Demon Rogue, deathtouch.
/// Boast — {1}{B}: target player searches their library for a card, then
/// shuffles and puts it on top (CR 702.142: only if it attacked this turn,
/// once each turn).
pub fn varragoth_bloodsky_sire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![crate::effect::shortcut::boast(
            cost(&[generic(1), b()]),
            Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::Search {
                    who: PlayerRef::Target(0),
                    filter: R::Any,
                    to: ZoneDest::Library { who: PlayerRef::Target(0), pos: crate::effect::LibraryPosition::Top },
                }),
            },
        )],
        ..legend(
            "Varragoth, Bloodsky Sire",
            cost(&[generic(2), b()]),
            vec![CreatureType::Demon, CreatureType::Rogue],
            2,
            3,
        )
    }
}

/// Burning-Rune Demon — {4}{B}{B} 6/6 Demon Berserker, flying. On entry you
/// may search for two cards not named Burning-Rune Demon with different
/// names; an opponent picks one for your hand and the other goes to your
/// graveyard (the Gifts Ungiven split).
pub fn burning_rune_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for two cards with different names?".into(),
            body: Box::new(Effect::SearchSplitOpponentChooses {
                opponent: Selector::None,
                count: 2,
                opponent_picks: 1,
                chosen_to: ZoneDest::Hand(PlayerRef::You),
                rest_to: ZoneDest::Graveyard,
                filter: Some(R::Not(Box::new(R::HasName("Burning-Rune Demon".into())))),
            }),
        })],
        ..creature(
            "Burning-Rune Demon",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Demon, CreatureType::Berserker],
            6,
            6,
        )
    }
}

fn soldier_of(types: Vec<CreatureType>, colorless_artifact: bool) -> TokenDefinition {
    TokenDefinition {
        name: if types.len() > 1 { "Human Soldier".into() } else { "Soldier".into() },
        power: 1,
        toughness: 1,
        colors: if colorless_artifact { vec![] } else { vec![Color::White] },
        card_types: if colorless_artifact { vec![CardType::Artifact, CardType::Creature] } else { vec![CardType::Creature] },
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn soldier_creature() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Soldier))
}

/// Horn of Gondor — {3} legendary artifact. On entry, a 1/1 white Human
/// Soldier. {3}, {T}: X of them, X the number of Humans you control.
pub fn horn_of_gondor() -> CardDefinition {
    let human_soldier = || Arc::new(soldier_of(vec![CreatureType::Human, CreatureType::Soldier], false));
    CardDefinition {
        name: "Horn of Gondor",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: human_soldier() })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::count(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Human).and(R::ControlledByYou),
                )),
                definition: human_soldier(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Horn of Valhalla // Ysgard's Call — {1}{W} Equipment: equipped creature
/// gets +1/+1 for each creature you control; equip {3}. Adventure (CR
/// 715): Ysgard's Call, {X}{W}{W} sorcery — X 1/1 white Soldiers.
pub fn horn_of_valhalla() -> CardDefinition {
    CardDefinition {
        name: "Horn of Valhalla",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            scale: Some(crate::card::EquipScale {
                filter: R::Creature.and(R::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
                ..Default::default()
            }),
            ..Default::default()
        }),
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Ysgard's Call",
            cost: cost(&[x(), w(), w()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: Arc::new(soldier_of(vec![CreatureType::Soldier], false)),
            },
        })),
        ..Default::default()
    }
}

/// Preeminent Captain — {2}{W} 2/2 Kithkin Soldier, first strike. Whenever
/// it attacks, you may put a Soldier creature card from your hand onto the
/// battlefield tapped and attacking (CR 508.4).
pub fn preeminent_captain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DeployCreatureFromHandAttacking { filter: soldier_creature(), return_to_hand_eot: false, same_defender: false },
        }],
        ..creature("Preeminent Captain", cost(&[generic(2), w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 2, 2)
    }
}

/// Rescue Retriever — {3}{W}{W} 3/3 Dog Soldier, flash. On entry, a +1/+1
/// counter on each other Soldier you control. Prevent all damage that would
/// be dealt to other attacking Soldiers you control (CR 615).
pub fn rescue_retriever() -> CardDefinition {
    let other_soldiers = || soldier_creature().and(R::ControlledByYou).and(R::OtherThanSource);
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(other_soldiers()),
            kind: crate::card::CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to other attacking Soldiers you control.",
            effect: StaticEffect::PreventAllDamageToAttackingMatching { filter: other_soldiers() },
        }],
        ..creature("Rescue Retriever", cost(&[generic(3), w(), w()]), vec![CreatureType::Dog, CreatureType::Soldier], 3, 3)
    }
}

/// Siege Veteran — {2}{W} 2/2 Human Soldier. At the beginning of combat on
/// your turn, a +1/+1 counter on target creature you control. Whenever
/// another nontoken Soldier you control dies, a 1/1 colorless Soldier
/// artifact creature token.
pub fn siege_veteran() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Soldier).and(R::NotToken),
                    },
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(soldier_of(vec![CreatureType::Soldier], true)),
                },
            },
        ],
        ..creature("Siege Veteran", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Valiant Veteran — {1}{W} 2/2 Kor Soldier. Other Soldiers you control get
/// +1/+1. {3}{W}{W}, exile it from your graveyard: a +1/+1 counter on each
/// Soldier you control.
pub fn valiant_veteran() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Soldiers you control get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: soldier_creature().and(R::ControlledByYou).and(R::OtherThanSource),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            exile_self_cost: true,
            mana_cost: cost(&[generic(3), w(), w()]),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(soldier_creature().and(R::ControlledByYou)),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Valiant Veteran", cost(&[generic(1), w()]), vec![CreatureType::Kor, CreatureType::Soldier], 2, 2)
    }
}

/// Balor — {3}{R}{R} 5/5 Demon, flying. Whenever it attacks or dies, choose
/// one or more, each mode targeting a different player (the Vindictive Lich
/// shape: a mode with no fresh opponent does nothing): an opponent draws
/// three then discards three at random; an opponent sacrifices a nontoken
/// artifact; Balor deals an opponent damage equal to the cards in their hand.
pub fn balor() -> CardDefinition {
    let opponent = || target_filtered(R::Player.and(R::ControlledByOpponent));
    let modes = || Effect::ChooseN { up_to: None,
        picks: vec![2, 1, 0],
        modes: vec![
            Effect::Seq(vec![
                Effect::Draw { who: opponent(), amount: Value::Const(3) },
                Effect::Discard { who: opponent(), amount: Value::Const(3), random: true },
            ]),
            Effect::Sacrifice { who: opponent(), count: Value::ONE, filter: R::Artifact.and(R::NotToken) },
            Effect::DealDamage { to: opponent(), amount: Value::HandSizeOf(PlayerRef::Target(0)) },
        ],
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: modes() },
            crate::effect::shortcut::on_dies(modes()),
        ],
        ..creature("Balor", cost(&[generic(3), r(), r()]), vec![CreatureType::Demon], 5, 5)
    }
}

/// Saving Grace — {1}{W} Aura, flash, enchant creature you control. ETB: all
/// damage that would be dealt this turn to you and permanents you control is
/// dealt to the enchanted creature instead (CR 614.9 — bound to that creature
/// as the trigger resolves, so it keeps redirecting if the Aura leaves).
/// Enchanted creature gets +0/+3.
pub fn saving_grace() -> CardDefinition {
    CardDefinition {
        name: "Saving Grace",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        equipped_bonus: Some(crate::card::EquipBonus { toughness: 3, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::RedirectYourDamageToChosen {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            creatures_only: false,
        })],
        ..Default::default()
    }
}

/// Martyrdom — {1}{W}{W} Instant. Until end of turn, target creature you
/// control gains "{0}: The next 1 damage that would be dealt to target
/// creature, planeswalker, or player this turn is dealt to this creature
/// instead." (CR 614.9). "Only you may activate" is its controller here,
/// which is you unless its control changes this turn.
pub fn martyrdom() -> CardDefinition {
    CardDefinition {
        name: "Martyrdom",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GainActivatedAbility {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            ability: Box::new(ActivatedAbility {
                effect: Effect::RedirectNextDamage {
                    target: target_filtered(R::Creature.or(R::Planeswalker).or(R::Player)),
                    to: Selector::This,
                    amount: Value::ONE,
                },
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Fire Lord Zuko (RWB) ────────────────────────────────────────────────────

/// Fire Nation Turret — {2}{R} Artifact. At the beginning of combat on your
/// turn, up to one target creature gets +2/+0 and gains firebending 2 until
/// end of turn. {R}: a charge counter. Remove fifty: 50 damage to any target.
pub fn fire_nation_turret() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Fire Nation Turret",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::Const(2),
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Firebending(2),
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                remove_counter_cost: Some((CounterType::Charge, 50)),
                effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(50) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Commander Liara Portyr — {3}{R}{W} 5/3. Whenever you attack, spells you
/// cast from exile this turn cost {X} less (X = players being attacked, fixed
/// as it resolves); exile your top X cards, castable until end of turn.
pub fn commander_liara_portyr() -> CardDefinition {
    let x = || Value::OpponentsAttackedThisCombat;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::SpellsCostLessThisTurnByValue { filter: R::InExile, amount: x() },
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: x(),
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                // "You may CAST spells": an exiled land can't be played.
                Effect::RestrictMayPlayToCasting { what: Selector::ExiledThisResolution { filter: R::Any } },
            ]),
        }],
        ..legend(
            "Commander Liara Portyr",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            5,
            3,
        )
    }
}

/// Fire Lord Ozai — {3}{B} 4/4. Attacking, you may sacrifice another creature
/// for {R} equal to its power, kept until end of combat (the firebending pool,
/// CR 702.189a). {6}: exile each opponent's top card; play one of them free
/// this turn.
pub fn fire_lord_ozai() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Sacrifice another creature for {R} equal to its power?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificeAndRemember {
                        who: PlayerRef::You,
                        filter: R::Creature.and(R::OtherThanSource),
                    },
                    Effect::Firebend { amount: Value::SacrificedPower },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            effect: Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::EachOpponent,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
                Effect::OneCastAmongGranted { what: Selector::ExiledThisResolution { filter: R::Any } },
            ]),
            ..Default::default()
        }],
        ..legend("Fire Lord Ozai", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Noble], 4, 4)
    }
}

/// Iroh, Dragon of the West — {2}{R}{R} 4/4 haste, mentor. At the beginning
/// of combat on your turn, each creature you control with a counter on it
/// gains firebending 2 until end of turn.
pub fn iroh_dragon_of_the_west() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            crate::effect::shortcut::mentor(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::WithAnyCounter)),
                    keyword: Keyword::Firebending(2),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..legend(
            "Iroh, Dragon of the West",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Ally],
            4,
            4,
        )
    }
}

/// Avatar Roku — The Legend of Roku's back face: 4/4 legendary Avatar,
/// firebending 4; {8}: a 4/4 red flying Dragon with firebending 4.
pub fn avatar_roku() -> CardDefinition {
    CardDefinition {
        // CR 105.2c — a transformed Saga's back face carries a color indicator.
        color_indicator: vec![Color::Red],
        keywords: vec![Keyword::Firebending(4)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Dragon".into(),
                    power: 4,
                    toughness: 4,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
                    keywords: vec![Keyword::Flying, Keyword::Firebending(4)],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..legend("Avatar Roku", crate::mana::ManaCost::default(), vec![CreatureType::Avatar], 4, 4)
    }
}

/// The Legend of Roku — {2}{R}{R} Saga. I: exile your top three, playable
/// until the end of your next turn. II: one mana of any color. III: exile it,
/// return it transformed (Avatar Roku).
pub fn the_legend_of_roku() -> CardDefinition {
    CardDefinition {
        name: "The Legend of Roku",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            ),
            (2, Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) }),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(avatar_roku())),
        ..Default::default()
    }
}

// ── Arna Kennerüd, Skycaptain (WUB) ─────────────────────────────────────────

fn equipment(name: &'static str, mana: crate::mana::ManaCost, equip: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        ..Default::default()
    }
}

/// Arna Kennerüd, Skycaptain — {2}{W}{U}{B} 4/4 flying, lifelink, ward—
/// discard a card. Whenever a modified creature you control attacks, double
/// each kind of counter on it (CR 701.10), then copy each nontoken permanent
/// attached to it, the copy entering attached to that creature (CR 707.2).
pub fn arna_kennerud_skycaptain() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Lifelink,
            Keyword::Ward(crate::card::WardCost::Discard(1)),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::IsModified),
            }),
            effect: Effect::Seq(vec![
                Effect::DoubleAllCountersOn { what: Selector::TriggerSource },
                // Inside the walk `TriggerSource` is the attachment, so its
                // host is the attacker; the list is fixed before any copy.
                Effect::ForEach {
                    selector: Selector::AttachedToMe(Box::new(Selector::TriggerSource)),
                    body: Box::new(Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsToken.negate() },
                        then: Box::new(Effect::CreateTokenCopyOfAttachedToEach {
                            source: Selector::TriggerSource,
                            hosts: Selector::AttachedTo(Box::new(Selector::TriggerSource)),
                        }),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
        }],
        ..legend(
            "Arna Kennerüd, Skycaptain",
            cost(&[generic(2), w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Assassin Gauntlet — {2}{U} Equipment. ETB: attach it to up to one target
/// creature you control, and tap all creatures target opponent controls.
/// Equipped +1/+1 and loots on combat damage to a player. Equip {2}.
pub fn assassin_gauntlet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::TargetPlayerThen {
                    filter: R::OpponentPlayer,
                    then: Box::new(Effect::ForEach {
                        selector: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                        body: Box::new(Effect::Tap { what: Selector::TriggerSource }),
                    }),
                },
                Effect::Attach {
                    what: Selector::This,
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) },
                },
            ])),
        })],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
            }],
            ..Default::default()
        }),
        ..equipment("Assassin Gauntlet", cost(&[generic(2), u()]), cost(&[generic(2)]))
    }
}

/// Biorganic Carapace — {2}{W}{U} Equipment. ETB: attach it to target
/// creature you control. Equipped +2/+2 and, on combat damage to a player,
/// draws a card per modified creature you control (CR 700.9). Equip {2}.
pub fn biorganic_carapace() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::IsModified),
                    ))),
                },
            }],
            ..Default::default()
        }),
        ..equipment("Biorganic Carapace", cost(&[generic(2), w(), u()]), cost(&[generic(2)]))
    }
}

/// Ardenn, Intrepid Archaeologist — {2}{W} 2/2, partner. At the beginning of
/// combat on your turn, you may attach any number of Auras and Equipment you
/// control to target permanent or player. Only what could legally attach
/// moves (CR 701.3b): Equipment onto a creature, a Curse onto a player.
pub fn ardenn_intrepid_archaeologist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::AttachAnyNumberTo {
                what: Selector::EachPermanent(
                    R::ControlledByYou.and(
                        R::HasArtifactSubtype(ArtifactSubtype::Equipment)
                            .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
                    ),
                ),
                to: target_filtered(R::Permanent.or(R::Player)),
            },
        }],
        ..legend(
            "Ardenn, Intrepid Archaeologist",
            cost(&[generic(2), w()]),
            vec![CreatureType::Kor, CreatureType::Scout],
            2,
            2,
        )
    }
}

/// Halvar, God of Battle // Sword of the Realms — {2}{W}{W} 4/4 God. Your
/// enchanted or equipped creatures have double strike; at the beginning of
/// each combat you may move an Aura or Equipment from one of your creatures
/// to another. The back ({1}{W} legendary Equipment): +2/+0 and vigilance,
/// and the equipped creature returns to its owner's hand when it dies.
pub fn halvar_god_of_battle() -> CardDefinition {
    let sword = CardDefinition {
        supertypes: vec![Supertype::Legendary],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 2,
            keywords: vec![Keyword::Vigilance],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
            }],
            ..Default::default()
        }),
        ..equipment("Sword of the Realms", cost(&[generic(1), w()]), cost(&[generic(1), w()]))
    };
    let mine = || R::Creature.and(R::ControlledByYou);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control that are enchanted or equipped have double strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(mine().and(R::IsEnchanted.or(R::IsEquipped))),
                keyword: Keyword::DoubleStrike,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat), EventScope::AnyPlayer),
            effect: Effect::MayDo {
                description: "Move an Aura or Equipment to another creature you control?".into(),
                body: Box::new(Effect::Attach {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment)
                            .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura))
                            .and(R::AttachedToCreatureYouControl),
                    },
                    to: Selector::TargetFiltered { slot: 1, filter: mine() },
                }),
            },
        }],
        back_face: Some(Box::new(sword)),
        ..legend("Halvar, God of Battle", cost(&[generic(2), w(), w()]), vec![CreatureType::God], 4, 4)
    }
}

// ── Terra, Magical Adept (RG, Esper Terra's WUBRG) ──────────────────────────

fn saga() -> Subtypes {
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() }
}

/// Tom Bombadil — {W}{U}{B}{R}{G} 4/4 God Bard. Hexproof and indestructible
/// while your Sagas hold four or more lore counters. Once each turn, when a
/// Saga of yours finishes (CR 714.2e), reveal until a Saga card and put it
/// onto the battlefield; the rest go to the bottom in a random order.
pub fn tom_bombadil() -> CardDefinition {
    use crate::card::CounterType;
    let lore_at_least_four = || {
        Predicate::ValueAtLeast(
            Value::CountersOn {
                what: Box::new(Selector::EachPermanent(
                    R::HasEnchantmentSubtype(EnchantmentSubtype::Saga).and(R::ControlledByYou),
                )),
                kind: CounterType::Lore,
            },
            Value::Const(4),
        )
    };
    let while_lore = |keyword: Keyword, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::SelfHasKeywordWhilePredicate { keyword, condition: lore_at_least_four() },
    };
    CardDefinition {
        static_abilities: vec![
            while_lore(Keyword::Hexproof, "Tom Bombadil has hexproof while your Sagas have four or more lore counters."),
            while_lore(
                Keyword::Indestructible,
                "Tom Bombadil has indestructible while your Sagas have four or more lore counters.",
            ),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SagaFinalChapterResolved, EventScope::YourControl).once_per_turn(),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::HasEnchantmentSubtype(EnchantmentSubtype::Saga),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                cap: Value::Const(100),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
        }],
        ..legend(
            "Tom Bombadil",
            cost(&[w(), u(), b(), r(), g()]),
            vec![CreatureType::God, CreatureType::Bard],
            4,
            4,
        )
    }
}

/// The Apprentice's Folly — {2}{U}{R} Saga. I, II: a hasty, nonlegendary
/// Reflection token copy of target nontoken creature you control that shares
/// no name with a token you control. III: sacrifice all your Reflections.
pub fn the_apprentices_folly() -> CardDefinition {
    let copy = || Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source: target_filtered(
            R::Creature.and(R::ControlledByYou).and(R::IsToken.negate()).and(R::NameNotSharedWithYourTokens),
        ),
        extra_creature_types: vec![CreatureType::Reflection],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: true,
        legendary: false,
        extra_keywords: vec![Keyword::Haste],
        no_mana_cost: false,
        enters_with_counters: None,
        remove_keywords: vec![],
    };
    CardDefinition {
        name: "The Apprentice's Folly",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga(),
        saga_chapters: vec![
            (1, copy()),
            (2, copy()),
            (
                3,
                Effect::SacrificeAllMatching {
                    who: Selector::You,
                    filter: R::HasCreatureType(CreatureType::Reflection),
                },
            ),
        ],
        ..Default::default()
    }
}

/// O-Kagachi Made Manifest — The Kami War's back face: a 6/6 flying, trample
/// Dragon Spirit that is all colors. Attacking, the defending player picks a
/// nonland card in your graveyard for your hand; it gets +X/+0 for its mana
/// value.
pub fn o_kagachi_made_manifest() -> CardDefinition {
    CardDefinition {
        name: "O-Kagachi Made Manifest",
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Spirit],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "O-Kagachi Made Manifest is all colors.",
            effect: StaticEffect::GrantAllColors { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ReturnFromGraveyardOpponentChooses {
                    filter: R::Nonland,
                    chooser: Some(PlayerRef::DefendingPlayer),
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ManaValueOf(Box::new(Selector::LastMoved)),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// The Kami War — {1}{W}{U}{B}{R}{G} Saga. I: exile target nonland permanent
/// an opponent controls. II: return up to one other target nonland permanent
/// to its owner's hand, then each opponent discards a card. III: exile it and
/// return it transformed (O-Kagachi Made Manifest).
pub fn the_kami_war() -> CardDefinition {
    CardDefinition {
        name: "The Kami War",
        cost: cost(&[generic(1), w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga(),
        saga_chapters: vec![
            (
                1,
                Effect::Exile {
                    what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
                },
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::Permanent.and(R::Nonland).and(R::OtherThanSource),
                        effect: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                        }),
                    },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::ONE,
                        random: false,
                    },
                ]),
            ),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(o_kagachi_made_manifest())),
        ..Default::default()
    }
}

/// Moonmist — {1}{G} Instant. Transform all Humans (only double-faced ones
/// turn over, CR 701.27c). Prevent all combat damage this turn dealt by
/// creatures other than Werewolves and Wolves (CR 615).
pub fn moonmist() -> CardDefinition {
    CardDefinition {
        name: "Moonmist",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Transform { what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Human)) },
            Effect::PreventCombatDamageExceptDealtBy {
                except: R::HasCreatureType(CreatureType::Werewolf).or(R::HasCreatureType(CreatureType::Wolf)),
            },
        ]),
        ..Default::default()
    }
}

// ── Atreus, Impulsive Son // Kratos, Stoic Father (URW) ─────────────────────

/// Djeru and Hazoret — {2}{R}{R}{W} 5/4 Human God. Vigilance and haste with
/// one or fewer cards in hand. Attacking, look at your top six; you may exile
/// a legendary creature card to cast free this turn; the rest go to the
/// bottom in a random order.
pub fn djeru_and_hazoret() -> CardDefinition {
    let hellbent = || Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::You), Value::ONE);
    let while_hellbent = |keyword: Keyword, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::SelfHasKeywordWhilePredicate { keyword, condition: hellbent() },
    };
    CardDefinition {
        static_abilities: vec![
            while_hellbent(Keyword::Vigilance, "Djeru and Hazoret has vigilance with one or fewer cards in your hand."),
            while_hellbent(Keyword::Haste, "Djeru and Hazoret has haste with one or fewer cards in your hand."),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::LookTopExileOneMayPlay {
                count: Value::Const(6),
                who: PlayerRef::You,
                grant: crate::effect::LookExileGrant::LegendaryCreatureFreeThisTurn,
            },
        }],
        ..legend(
            "Djeru and Hazoret",
            cost(&[generic(2), r(), r(), w()]),
            vec![CreatureType::Human, CreatureType::God],
            5,
            4,
        )
    }
}

/// Katara, Waterbending Master — {1}{U} 1/3. Casting a spell during an
/// opponent's turn gets you an experience counter; attacking, you may draw one
/// per experience counter, then discard a card.
pub fn katara_waterbending_master() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
                effect: Effect::AddExperience(Value::ONE),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Draw a card per experience counter, then discard a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ControllerExperience },
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    ])),
                },
            },
        ],
        ..legend(
            "Katara, Waterbending Master",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            1,
            3,
        )
    }
}

/// Lae'zel, Vlaakith's Champion — {2}{W} 3/3 Gith Warrior. Counters put on a
/// creature or planeswalker you control or on you come one more of each kind
/// (CR 614.16). Choose a Background.
pub fn laezel_vlaakiths_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        static_abilities: vec![StaticAbility {
            description: "If you would put one or more counters on a creature or planeswalker you control or on yourself, put that many plus one of each of those kinds instead.",
            effect: StaticEffect::ExtraCounterOnCreaturePlaneswalkerOrYou,
        }],
        ..legend(
            "Lae'zel, Vlaakith's Champion",
            cost(&[generic(2), w()]),
            vec![CreatureType::Gith, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Reidane, God of the Worthy // Valkmira, Protector's Shield — {2}{W} 2/3
/// flying, vigilance: opponents' snow lands enter tapped, their noncreature
/// spells of mana value 4+ cost {2} more. Back ({3}{W} legendary artifact):
/// opponents' sources deal you and yours 1 less damage, and you and your other
/// permanents have ward {1} against them (CR 702.21).
pub fn reidane_god_of_the_worthy() -> CardDefinition {
    let ward = || crate::card::WardCost::Mana(cost(&[generic(1)]));
    let valkmira = CardDefinition {
        name: "Valkmira, Protector's Shield",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "If a source an opponent controls would deal damage to you or a permanent you control, prevent 1 of that damage.",
                effect: StaticEffect::ReduceOpponentDamageToYouAndYoursBy(1),
            },
            StaticAbility {
                description: "Whenever you or another permanent you control becomes the target of a spell or ability an opponent controls, counter that spell or ability unless its controller pays {1}.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::ControlledByYou.and(R::OtherThanSource)),
                    keyword: Keyword::Ward(ward()),
                },
            },
            StaticAbility { description: "You have ward {1}.", effect: StaticEffect::ControllerHasWard(ward()) },
        ],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "Snow lands your opponents control enter tapped.",
                effect: StaticEffect::EntersTapped {
                    applies_to: Selector::EachPermanent(R::ControlledByOpponent.and(R::Land).and(R::IsSnow)),
                },
            },
            StaticAbility {
                description: "Noncreature spells your opponents cast with mana value 4 or greater cost {2} more to cast.",
                effect: StaticEffect::OpponentSpellsCostMore {
                    filter: R::Noncreature.and(R::ManaValueAtLeast(4)),
                    amount: 2,
                },
            },
        ],
        back_face: Some(Box::new(valkmira)),
        ..legend("Reidane, God of the Worthy", cost(&[generic(2), w()]), vec![CreatureType::God], 2, 3)
    }
}

/// Surtr, Fiery Jötun — {3}{R}{R} 5/5 trample. Casting a historic spell (CR
/// 700.6) deals 3 damage to any target.
pub fn surtr_fiery_jotun() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::historic())),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) },
        }],
        ..legend(
            "Surtr, Fiery Jötun",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Giant, CreatureType::God, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// World at War — {3}{R}{R} Sorcery, rebound. After the second main phase
/// this turn, an additional combat and main phase (CR 500.8: a combat banked
/// in the first main phase still follows the scheduled one); at the start of
/// that combat, untap every creature that attacked this turn.
pub fn world_at_war() -> CardDefinition {
    CardDefinition {
        name: "World at War",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Rebound],
        effect: Effect::Seq(vec![
            Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE },
            Effect::AtTheAddedCombat {
                body: Box::new(Effect::Untap {
                    what: Selector::EachPermanent(R::Creature.and(R::AttackedThisTurn)),
                    up_to: None,
                }),
            },
        ]),
        ..Default::default()
    }
}

// ── Kratos, God of War (R) ──────────────────────────────────────────────────

/// Mjölnir, Storm Hammer — {4} legendary Equipment. ETB: attach it to target
/// legendary creature you control. Whenever the equipped creature attacks, tap
/// target creature the defending player controls and stun it, then deal each
/// opponent damage equal to the tapped creatures they control. Equip {4}.
pub fn mjolnir_storm_hammer() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou).and(R::HasSupertype(Supertype::Legendary))),
        })],
        equipped_bonus: Some(crate::card::EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)) },
                    Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                    Effect::ForEachOpponent {
                        body: Box::new(Effect::DealDamage {
                            to: Selector::Player(PlayerRef::Triggerer),
                            amount: Value::CountOf(Box::new(Selector::ControlledBy {
                                who: PlayerRef::Triggerer,
                                filter: R::Creature.and(R::Tapped),
                            })),
                        }),
                    },
                ]),
            }],
            ..Default::default()
        }),
        ..equipment("Mjölnir, Storm Hammer", cost(&[generic(4)]), cost(&[generic(4)]))
    }
}

/// Gornog, the Red Reaper — {2}{R} 2/3 haste. Cowards can't block Warriors.
/// When Warriors of yours attack a player, target creature that player
/// controls becomes a Coward (overwriting its creature types, no duration —
/// the rulings). Attacking Warriors you control get +1/+0 per Coward your
/// opponents control.
pub fn gornog_the_red_reaper() -> CardDefinition {
    let warrior = || R::HasCreatureType(CreatureType::Warrior);
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![
            StaticAbility {
                description: "Cowards can't block Warriors.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Coward)),
                    keyword: Keyword::CantBlockCreatureType(CreatureType::Warrior),
                },
            },
            StaticAbility {
                description: "Attacking Warriors you control get +X/+0, where X is the number of Cowards your opponents control.",
                effect: StaticEffect::PumpPTByValue {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::IsAttacking).and(warrior()),
                    ),
                    power: Value::CountOf(Box::new(Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Coward).and(R::ControlledByOpponent),
                    ))),
                    toughness: Value::Const(0),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: warrior().and(R::IsAttackingOpponentPlayer),
                })
                .once_per_batch_per_defender(),
            effect: Effect::BecomeCreatureType {
                what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                creature_types: vec![CreatureType::Coward],
                duration: Duration::Permanent,
            },
        }],
        ..legend(
            "Gornog, the Red Reaper",
            cost(&[generic(2), r()]),
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Spider-Punk — {1}{R} 2/1 Spider Human Hero, riot; other Spiders you
/// control have riot (CR 702.136). Spells and abilities can't be countered;
/// damage can't be prevented.
pub fn spider_punk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Riot],
        static_abilities: vec![
            StaticAbility {
                description: "Other Spiders you control have riot.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Spider).and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    keyword: Keyword::Riot,
                },
            },
            StaticAbility {
                description: "Spells and abilities can't be countered.",
                effect: StaticEffect::SpellsAndAbilitiesCantBeCountered,
            },
            StaticAbility { description: "Damage can't be prevented.", effect: StaticEffect::DamageCantBePrevented },
        ],
        ..legend(
            "Spider-Punk",
            cost(&[generic(1), r()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            2,
            1,
        )
    }
}

/// Maximum Carnage — {4}{R} Saga. I: until your next turn, every creature
/// attacks each combat if able and attacks a player other than you if able —
/// goad (CR 701.15b) on every creature, those entering later too. II: {R}{R}{R}.
/// III: 5 damage to each opponent.
pub fn maximum_carnage() -> CardDefinition {
    CardDefinition {
        name: "Maximum Carnage",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga(),
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Goad { what: Selector::EachPermanent(R::Creature) },
                    Effect::WheneverCreatureEntersUntilYourNextTurn {
                        filter: R::Creature,
                        body: Box::new(Effect::Goad { what: Selector::TriggerSource }),
                    },
                ]),
            ),
            (
                2,
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Red, Color::Red, Color::Red]),
                },
            ),
            (
                3,
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(5) },
            ),
        ],
        ..Default::default()
    }
}

// ── Eddie Brock // Venom, Lethal Protector (BRG) ────────────────────────────

/// Venom, Lethal Protector — Eddie Brock's back face, 5/5 menace, trample,
/// haste. Attacking, you may sacrifice another creature to draw X, then you
/// may put a permanent card of mana value X or less from your hand onto the
/// battlefield (X = the sacrificed creature's mana value).
pub fn venom_lethal_protector() -> CardDefinition {
    CardDefinition {
        // The face prints {3}{B}{R}{G}; carried as its colors, since the
        // catalog tells a nonmodal DFC by a back face with no cost (CR 712.2,
        // `cr_712_transforming_and_modal_dfcs_are_told_apart`). Its mana
        // value is the front's anyway (CR 712.8e).
        color_indicator: vec![Color::Black, Color::Red, Color::Green],
        keywords: vec![Keyword::Menace, Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Sacrifice another creature to draw cards and put a permanent onto the battlefield?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificeAndRemember { who: PlayerRef::You, filter: R::Creature.and(R::OtherThanSource) },
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(Effect::Seq(vec![
                            Effect::Draw { who: Selector::You, amount: Value::SacrificedManaValue },
                            Effect::PutFromHandOntoBattlefield {
                                who: PlayerRef::You,
                                filter: R::PermanentCard.and(R::ManaValueAtMostSacrificedPlus(0)),
                                count: Value::ONE,
                                tapped: false,
                                haste: false,
                                sacrifice_eot: false,
                                return_eot: false,
                                then: None,
                            },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        }],
        ..legend(
            "Venom, Lethal Protector",
            cost(&[generic(3), b(), r(), g()]),
            vec![CreatureType::Symbiote, CreatureType::Hero, CreatureType::Villain],
            5,
            5,
        )
    }
}

/// Eddie Brock — {2}{B} 3/3. ETB: return target creature card with mana
/// value 1 or less from your graveyard to the battlefield. {3}{B}{R}{G}:
/// transform (sorcery speed) into Venom, Lethal Protector.
pub fn eddie_brock() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::ManaValueAtMost(1)).from_your_graveyard()),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b(), r(), g()]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(venom_lethal_protector())),
        ..legend(
            "Eddie Brock",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Hero, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Carnage, Crimson Chaos — {2}{B}{R} 4/3 trample. ETB: return target
/// creature card with mana value 3 or less from your graveyard; it attacks
/// each combat if able and is sacrificed when it deals combat damage to a
/// player. Mayhem {B}{R}.
pub fn carnage_crimson_chaos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Mayhem(cost(&[b(), r()]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3)).from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::MustAttack, duration: Duration::Permanent },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::SacrificeSource,
                }),
                duration: Duration::Permanent,
            },
        ]))],
        ..legend(
            "Carnage, Crimson Chaos",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Symbiote, CreatureType::Villain],
            4,
            3,
        )
    }
}

/// Gwenom, Remorseless — {3}{B}{B} 4/4 deathtouch, lifelink. Attacking, until
/// end of turn you may play cards from the top of your library, a spell cast
/// this way costing life equal to its mana value (CR 401.6).
pub fn gwenom_remorseless() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantPlayFromTopPayLifeThisTurn,
        }],
        ..legend(
            "Gwenom, Remorseless",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Symbiote, CreatureType::Spider, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Venom, Deadly Devourer — {2}{B}{G} 4/4 vigilance, menace. {3}: exile
/// target creature card from a graveyard; when you do, put +1/+1 counters
/// equal to its toughness on target Symbiote — a reflexive trigger (CR
/// 603.12) that picks the Symbiote as it triggers.
pub fn venom_deadly_devourer() -> CardDefinition {
    use crate::card::CounterType;
    let exiled = || Selector::ExiledThisResolution { filter: R::Any };
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Creature.from_any_graveyard()) },
                Effect::If {
                    cond: Predicate::SelectorExists(exiled()),
                    then: Box::new(Effect::ReflexiveTrigger {
                        body: Box::new(Effect::AddCounter {
                            what: target_filtered(R::HasCreatureType(CreatureType::Symbiote)),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ToughnessOf(Box::new(exiled())),
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Venom, Deadly Devourer",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Symbiote, CreatureType::Villain],
            4,
            4,
        )
    }
}

/// Behold the Sinister Six! — {6}{B} Sorcery. Return up to six target
/// creature cards with different names from your graveyard to the
/// battlefield (a singleton Commander graveyard holds no two names alike).
pub fn behold_the_sinister_six() -> CardDefinition {
    CardDefinition {
        name: "Behold the Sinister Six!",
        cost: cost(&[generic(6), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 6,
            min_targets: 0,
            filter: R::Creature.from_your_graveyard().and(R::NameDiffersFromOtherTargets),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        },
        ..Default::default()
    }
}

// ── Nicol Bolas, the Ravager (UBR) ──────────────────────────────────────────

fn bolas_walker(name: &'static str, mana: crate::mana::ManaCost, loyalty: u32, abilities: Vec<LoyaltyAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Bolas], ..Default::default() },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

fn loyalty(cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() }
}

/// CR 701.71a — "empower Jace N": with no Jace planeswalker token, create a
/// blue one with 0 loyalty, "[−1]: Surveil 1" and "[−3]: Draw a card"; then
/// put N loyalty counters on a Jace token you control.
fn empower_jace(n: i32) -> Effect {
    use crate::card::CounterType;
    let jace_token = || R::IsToken.and(R::HasPlaneswalkerType(PlaneswalkerSubtype::Jace)).and(R::ControlledByYou);
    Effect::Seq(vec![
        Effect::If {
            cond: Predicate::ValueAtMost(
                Value::CountOf(Box::new(Selector::EachPermanent(jace_token()))),
                Value::Const(0),
            ),
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Jace".into(),
                    card_types: vec![CardType::Planeswalker],
                    colors: vec![Color::Blue],
                    subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace], ..Default::default() },
                    loyalty_abilities: vec![
                        loyalty(-1, Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }),
                        loyalty(-3, Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    ],
                    ..Default::default()
                }),
            }),
            else_: Box::new(Effect::Noop),
        },
        Effect::AddCounter {
            what: Selector::GreatestPowerTopN { filter: jace_token(), count: Value::ONE },
            kind: CounterType::Loyalty,
            amount: Value::Const(n),
        },
    ])
}

/// Sanctum Lurker — {2}{B} 3/2 Horror. ETB: empower Jace 1. Your
/// planeswalkers survive 0 loyalty (CR 704.5i) and have "[+2]: 1 damage to
/// each opponent and you gain 1 life."
pub fn sanctum_lurker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(empower_jace(1))],
        static_abilities: vec![
            StaticAbility {
                description: "Planeswalkers you control aren't put into their owners' graveyards for having 0 loyalty.",
                effect: StaticEffect::YourPlaneswalkersSurviveZeroLoyalty,
            },
            StaticAbility {
                description: "Planeswalkers you control have \"[+2]: This planeswalker deals 1 damage to each opponent and you gain 1 life.\"",
                effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                    abilities: vec![loyalty(
                        2,
                        Effect::Seq(vec![
                            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                            Effect::GainLife { who: Selector::You, amount: Value::ONE },
                        ]),
                    )],
                },
            },
        ],
        ..creature("Sanctum Lurker", cost(&[generic(2), b()]), vec![CreatureType::Horror], 3, 2)
    }
}

/// Liliana's Talent — {B}{B} Aura, enchant planeswalker. It has "[−8]: Put
/// all creature cards from all graveyards onto the battlefield under your
/// control." A creature dealing damage to it is destroyed.
pub fn lilianas_talent() -> CardDefinition {
    CardDefinition {
        name: "Liliana's Talent",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Planeswalker) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted planeswalker has \"[−8]: Put all creature cards from all graveyards onto the battlefield under your control.\"",
            effect: StaticEffect::AttachedPlaneswalkerHasLoyaltyAbilities {
                abilities: vec![loyalty(
                    -8,
                    Effect::Move {
                        what: Selector::EachMatching {
                            zone: crate::effect::ZoneRef::Graveyard(PlayerRef::EachPlayer),
                            filter: R::Creature,
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                )],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::EnchantedBySource),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::LastDamagerOf(Box::new(Selector::AttachedTo(Box::new(Selector::This)))),
                    filter: R::Creature,
                },
                then: Box::new(Effect::Destroy {
                    what: Selector::LastDamagerOf(Box::new(Selector::AttachedTo(Box::new(Selector::This)))),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Nicol Bolas, Planeswalker — {4}{U}{B}{B}{R}, loyalty 5. +3: destroy target
/// noncreature permanent. −2: gain control of target creature. −9: 7 damage
/// to target player or planeswalker; that player (or its controller) discards
/// seven cards, then sacrifices seven permanents.
pub fn nicol_bolas_planeswalker() -> CardDefinition {
    let victim = || PlayerRef::ControllerOf(Box::new(Selector::Target(0)));
    bolas_walker(
        "Nicol Bolas, Planeswalker",
        cost(&[generic(4), u(), b(), b(), r()]),
        5,
        vec![
            loyalty(3, Effect::Destroy { what: target_filtered(R::Permanent.and(R::Noncreature)) }),
            loyalty(
                -2,
                Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::Permanent },
            ),
            loyalty(
                -9,
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::Const(7) },
                    Effect::Discard { who: Selector::Player(victim()), amount: Value::Const(7), random: false },
                    Effect::Sacrifice { who: Selector::Player(victim()), count: Value::Const(7), filter: R::Permanent },
                ]),
            ),
        ],
    )
}

/// Nicol Bolas, the Deceiver — {5}{U}{B}{R}, loyalty 5. +3: each opponent
/// loses 3 life unless they sacrifice a nonland permanent or discard a card.
/// −3: destroy target creature, draw a card. −11: 7 damage to each opponent,
/// you draw seven.
pub fn nicol_bolas_the_deceiver() -> CardDefinition {
    bolas_walker(
        "Nicol Bolas, the Deceiver",
        cost(&[generic(5), u(), b(), r()]),
        5,
        vec![
            loyalty(
                3,
                Effect::Punisher {
                    chooser: Selector::Player(PlayerRef::EachOpponent),
                    options: vec![
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                        Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Permanent.and(R::Nonland) },
                    ],
                    otherwise: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::Const(3),
                    }),
                },
            ),
            loyalty(
                -3,
                Effect::Seq(vec![
                    Effect::Destroy { what: target_filtered(R::Creature) },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
            ),
            loyalty(
                -11,
                Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(7) },
                    Effect::Draw { who: Selector::You, amount: Value::Const(7) },
                ]),
            ),
        ],
    )
}

/// Dark Intimations — {2}{U}{B}{R} Sorcery. Each opponent sacrifices a
/// creature or planeswalker, then discards a card; you return a creature or
/// planeswalker card from your graveyard to hand, then draw. From your
/// graveyard: casting a Bolas planeswalker spell exiles it, and that
/// planeswalker enters with an additional loyalty counter.
pub fn dark_intimations() -> CardDefinition {
    use crate::card::CounterType;
    let creature_or_walker = || R::Creature.or(R::Planeswalker);
    CardDefinition {
        name: "Dark Intimations",
        cost: cost(&[generic(2), u(), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: creature_or_walker(),
            },
            Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE, random: false },
            Effect::ReturnGraveyardCardsToHand { filter: creature_or_walker(), max: Value::ONE },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Planeswalker.and(R::HasPlaneswalkerType(PlaneswalkerSubtype::Bolas)),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Exile { what: Selector::This },
                Effect::SpellEntersWithCounters {
                    what: Selector::TriggerSource,
                    kind: CounterType::Loyalty,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Hour of Devastation — {3}{R}{R} Sorcery. Creatures lose indestructible
/// until end of turn; 5 damage to each creature and each non-Bolas
/// planeswalker.
pub fn hour_of_devastation() -> CardDefinition {
    CardDefinition {
        name: "Hour of Devastation",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LoseKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.or(R::Planeswalker.and(R::HasPlaneswalkerType(PlaneswalkerSubtype::Bolas).negate())),
                ),
                amount: Value::Const(5),
            },
        ]),
        ..Default::default()
    }
}

// ── Shilgengar, Sire of Famine (WB) ─────────────────────────────────────────

/// Angel of Suffering — {3}{B}{B} 5/3 Nightmare Angel, flying. Damage that
/// would be dealt to you is prevented, and you mill twice that many (CR 615).
pub fn angel_of_suffering() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to you, prevent that damage and mill twice that many cards.",
            effect: StaticEffect::PreventDamageToYouMillTwice,
        }],
        ..creature(
            "Angel of Suffering",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Nightmare, CreatureType::Angel],
            5,
            3,
        )
    }
}

/// Firja, Judge of Valor — {2}{W}{B}{B} 2/4 flying, lifelink. Your second
/// spell each turn: look at your top three, one to hand, the rest to your
/// graveyard.
pub fn firja_judge_of_valor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(2) }),
            effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: true,
                ..Default::default()
            })),
        }],
        ..legend(
            "Firja, Judge of Valor",
            cost(&[generic(2), w(), b(), b()]),
            vec![CreatureType::Angel, CreatureType::Cleric],
            2,
            4,
        )
    }
}

/// Renegade Reaper — {2}{B} 2/3 Angel Berserker, flying. ETB: mill four; 4
/// life if an Angel card was among them.
pub fn renegade_reaper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(4) },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CardsMilledThisEffectMatching { filter: R::HasCreatureType(CreatureType::Angel) },
                    Value::ONE,
                ),
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(4) }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature(
            "Renegade Reaper",
            cost(&[generic(2), b()]),
            vec![CreatureType::Angel, CreatureType::Berserker],
            2,
            3,
        )
    }
}

/// Sanctuary Warden — {4}{W}{W} 5/5 Angel Soldier, flying, enters with two
/// shield counters. Entering or attacking, you may remove a counter from a
/// creature or planeswalker you control to draw a card and make a 1/1
/// green and white Citizen.
pub fn sanctuary_warden() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::{choose_one_then, chosen_one};
    let cash_in = || Effect::MayDo {
        description: "Remove a counter to draw a card and make a Citizen?".into(),
        body: Box::new(choose_one_then(
            Selector::EachPermanent(R::Creature.or(R::Planeswalker).and(R::ControlledByYou).and(R::WithAnyCounter)),
            PlayerRef::You,
            Effect::Seq(vec![
                Effect::RemoveCountersUpTo { what: chosen_one(), amount: Value::ONE, only_helpful: false },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::CountersRemovedThisEffect, Value::ONE),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Arc::new(TokenDefinition {
                                name: "Citizen".into(),
                                power: 1,
                                toughness: 1,
                                card_types: vec![CardType::Creature],
                                colors: vec![Color::Green, Color::White],
                                subtypes: Subtypes { creature_types: vec![CreatureType::Citizen], ..Default::default() },
                                ..Default::default()
                            }),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::Shield, Value::Const(2))),
        triggered_abilities: vec![
            etb(cash_in()),
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: cash_in() },
        ],
        ..creature(
            "Sanctuary Warden",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Soldier],
            5,
            5,
        )
    }
}

/// Rampage of the Valkyries — {3}{W}{B} Enchantment. ETB: a 4/4 white flying,
/// vigilant Angel. Whenever an Angel you control dies, each other player
/// sacrifices a creature.
pub fn rampage_of_the_valkyries() -> CardDefinition {
    CardDefinition {
        name: "Rampage of the Valkyries",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Angel".into(),
                    power: 4,
                    toughness: 4,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
                    keywords: vec![Keyword::Flying, Keyword::Vigilance],
                    ..Default::default()
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Angel),
                    },
                ),
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachPlayerExceptControllerOf(Box::new(Selector::This))),
                    count: Value::ONE,
                    filter: R::Creature,
                },
            },
        ],
        ..Default::default()
    }
}

/// Lich-Knights' Conquest — {4}{B} Sorcery. Sacrifice any number of
/// artifacts, enchantments and/or tokens; return that many creature cards
/// from your graveyard to the battlefield (one per sacrifice).
pub fn lich_knights_conquest() -> CardDefinition {
    use crate::effect::shortcut::{choose_one_then, chosen_one};
    CardDefinition {
        name: "Lich-Knights' Conquest",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Artifact.or(R::Enchantment).or(R::IsToken),
            per_each: Box::new(choose_one_then(
                Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter: R::Creature },
                PlayerRef::You,
                Effect::Move {
                    what: chosen_one(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            )),
        },
        ..Default::default()
    }
}

// ── Norin the Wary (R) ──────────────────────────────────────────────────────

/// Alliance — "whenever another creature you control enters, `effect`".
fn alliance(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
        effect,
    }
}

/// Raphael, Tough Turtle — {1}{R} 1/3 Mutant Ninja Turtle. Alliance: 1 damage
/// to target opponent.
pub fn raphael_tough_turtle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![alliance(Effect::DealDamage {
            to: target_filtered(R::OpponentPlayer),
            amount: Value::ONE,
        })],
        ..legend(
            "Raphael, Tough Turtle",
            cost(&[generic(1), r()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            1,
            3,
        )
    }
}

/// Slash, Reptile Rampager — {3}{R}{R} 7/5. Alliance: 2 damage to each
/// opponent. Attacking, it makes a 2/2 red Mutant.
pub fn slash_reptile_rampager() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            alliance(Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition {
                        name: "Mutant".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        subtypes: Subtypes { creature_types: vec![CreatureType::Mutant], ..Default::default() },
                        ..Default::default()
                    }),
                },
            },
        ],
        ..legend(
            "Slash, Reptile Rampager",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Mutant, CreatureType::Berserker, CreatureType::Turtle],
            7,
            5,
        )
    }
}

/// Mechanized Warfare — {1}{R}{R} Enchantment. A red or artifact source you
/// control deals 1 more damage to an opponent or their permanent (CR 614.5).
pub fn mechanized_warfare() -> CardDefinition {
    CardDefinition {
        name: "Mechanized Warfare",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If a red or artifact source you control would deal damage to an opponent or a permanent an opponent controls, it deals that much damage plus 1 instead.",
            effect: StaticEffect::AddDamageToOpponents {
                source_color: Some(Color::Red),
                amount: 1,
                other: false,
                or_artifact: true,
            },
        }],
        ..Default::default()
    }
}

/// Powerbalance — {R}{R} Enchantment. Whenever an opponent casts a spell,
/// you may reveal your top card and cast it free if its mana value matches
/// that spell's; otherwise it stays on top.
pub fn powerbalance() -> CardDefinition {
    CardDefinition {
        name: "Powerbalance",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::WithX {
                x: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                body: Box::new(Effect::RevealTopMayCastFreeKeepOnTop { filter: R::ManaValueExactlyXFromCost }),
            },
        }],
        ..Default::default()
    }
}

/// Wild Magic Surge — {R}{R} Instant. Destroy target permanent an opponent
/// controls; its controller reveals until a permanent card sharing a card
/// type with it, puts that onto the battlefield, and bottoms the rest
/// randomly.
pub fn wild_magic_surge() -> CardDefinition {
    CardDefinition {
        name: "Wild Magic Surge",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Permanent.and(R::ControlledByOpponent)) },
            Effect::RevealUntilFind {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                find: R::PermanentCard.and(R::SharesCardTypeWithTarget),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
                cap: Value::Const(100),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
        ]),
        ..Default::default()
    }
}

/// Wheel of Misfortune — {2}{R} Sorcery. Secret numbers: the highest takes
/// that much damage, and everyone who didn't pick the lowest wheels for
/// seven.
pub fn wheel_of_misfortune() -> CardDefinition {
    CardDefinition {
        name: "Wheel of Misfortune",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::WheelOfMisfortune { max: 20 },
        ..Default::default()
    }
}

// ── Katara, the Fearless (GWU) ──────────────────────────────────────────────

fn ally() -> TokenDefinition {
    TokenDefinition {
        name: "Ally".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ally], ..Default::default() },
        ..Default::default()
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// Aang and Katara — {3}{G}{W}{U} 5/5 legendary Human Avatar Ally. Whenever
/// they enter or attack, create X 1/1 white Ally tokens, X = the tapped
/// artifacts and/or creatures you control (counted on resolution).
pub fn aang_and_katara() -> CardDefinition {
    let allies = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::CountOf(Box::new(yours(R::Tapped.and(R::Artifact.or(R::Creature))))),
        definition: Arc::new(ally()),
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(allies()),
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: allies() },
        ],
        ..legend(
            "Aang and Katara",
            cost(&[generic(3), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Avatar, CreatureType::Ally],
            5,
            5,
        )
    }
}

/// Aang, Swift Savior // Aang and La, Ocean's Fury — flash, flying; ETB airbend
/// up to one other target creature or spell; Waterbend {8}: transform. The
/// back is a 5/5 reach, trample Avatar Spirit Ally whose attack puts a +1/+1
/// counter on each tapped creature you control.
pub fn aang_swift_savior() -> CardDefinition {
    let la = CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample],
        color_indicator: vec![Color::White, Color::Blue],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: yours(R::Creature.and(R::Tapped)),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..legend(
            "Aang and La, Ocean's Fury",
            crate::mana::ManaCost::default(),
            vec![CreatureType::Avatar, CreatureType::Spirit, CreatureType::Ally],
            5,
            5,
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature.and(R::OtherThanSource).or(R::IsSpellOnStack),
            effect: Box::new(Effect::Airbend { what: Selector::Target(0) }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            waterbend: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(la)),
        ..legend(
            "Aang, Swift Savior",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Avatar, CreatureType::Ally],
            2,
            3,
        )
    }
}

/// Aang, at the Crossroads // Aang, Destined Savior — flying; ETB look at the
/// top five, you may put a creature card with mana value 4 or less onto the
/// battlefield, the rest on the bottom in a random order; another creature of
/// yours leaving transforms him at the beginning of the next upkeep. The back
/// gives land creatures you control vigilance and earthbends 2 at the
/// beginning of combat on your turn.
pub fn aang_at_the_crossroads() -> CardDefinition {
    let destined = CardDefinition {
        keywords: vec![Keyword::Flying],
        color_indicator: vec![Color::White, Color::Blue, Color::Green],
        static_abilities: vec![StaticAbility {
            description: "Land creatures you control have vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Land.and(R::Creature)),
                keyword: Keyword::Vigilance,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Earthbend { n: Value::Const(2) },
        }],
        ..legend(
            "Aang, Destined Savior",
            crate::mana::ManaCost::default(),
            vec![CreatureType::Avatar, CreatureType::Ally],
            4,
            4,
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                count: Value::Const(5),
                pick_filter: Some(R::Creature.and(R::ManaValueAtMost(4))),
                to_battlefield: true,
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            }))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::OtherThanSource) },
                ),
                effect: Effect::AtNextTurnsUpkeep { body: Box::new(Effect::Transform { what: Selector::This }) },
            },
        ],
        back_face: Some(Box::new(destined)),
        ..legend(
            "Aang, at the Crossroads",
            cost(&[generic(2), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Avatar, CreatureType::Ally],
            3,
            3,
        )
    }
}

/// Appa, the Vigilant — {5}{W}{W} 6/6 legendary Bison Ally. Flying,
/// vigilance. Whenever Appa or another Ally you control enters, creatures you
/// control get +1/+1 and gain flying and vigilance until end of turn.
pub fn appa_the_vigilant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Ally) },
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: yours(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            ]),
        }],
        ..legend("Appa, the Vigilant", cost(&[generic(5), w(), w()]), vec![CreatureType::Bison, CreatureType::Ally], 6, 6)
    }
}

/// Tale of Katara and Toph — {2}{G} Enchantment. Creatures you control have
/// "Whenever this creature becomes tapped for the first time during each of
/// your turns, put a +1/+1 counter on it."
pub fn tale_of_katara_and_toph() -> CardDefinition {
    CardDefinition {
        name: "Tale of Katara and Toph",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have \"Whenever this creature becomes tapped for the first time \
                          during each of your turns, put a +1/+1 counter on it.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::ControlledByYou),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource).with_filter(Predicate::All(vec![
                        Predicate::IsTurnOf(PlayerRef::You),
                        Predicate::TriggerSourceFirstTappedThisTurn,
                    ])),
                    effect: Effect::AddCounter {
                        what: Selector::This,
                        kind: crate::card::CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                }),
            },
        }],
        ..Default::default()
    }
}

/// Tale of Momo — {2}{W} Sorcery. Costs {2} less if a creature left the
/// battlefield under your control this turn. Search your library and/or
/// graveyard for an Ally creature card, reveal it, put it into your hand
/// (shuffling if the library was searched).
pub fn tale_of_momo() -> CardDefinition {
    CardDefinition {
        name: "Tale of Momo",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if: Some((Predicate::CreatureLeftBattlefieldThisTurn { who: PlayerRef::You }, 2)),
        effect: Effect::SearchZones {
            who: PlayerRef::You,
            zones: vec![crate::card::Zone::Library, crate::card::Zone::Graveyard],
            filter: R::Creature.and(R::HasCreatureType(CreatureType::Ally)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

// ── Azlask, the Swelling Scourge (C) ────────────────────────────────────────

fn spawns(n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: Arc::new(crabomination_base::tokens::eldrazi_spawn_token()),
    }
}

/// Emrakul's Hatcher — {4}{R} 3/3 Eldrazi Drone. ETB three 0/1 Eldrazi Spawn
/// tokens with "Sacrifice this token: Add {C}."
pub fn emrakuls_hatcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(spawns(3))],
        ..creature("Emrakul's Hatcher", cost(&[generic(4), r()]), vec![CreatureType::Eldrazi, CreatureType::Drone], 3, 3)
    }
}

/// Brood Birthing — {1}{R} Sorcery. Three Eldrazi Spawn if you control an
/// Eldrazi Spawn, otherwise one (checked on resolution).
pub fn brood_birthing() -> CardDefinition {
    CardDefinition {
        name: "Brood Birthing",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Eldrazi)
                    .and(R::HasCreatureType(CreatureType::Spawn))
                    .and(R::ControlledByYou),
            )),
            then: Box::new(spawns(3)),
            else_: Box::new(spawns(1)),
        },
        ..Default::default()
    }
}

/// Minthara, Merciless Soul — {2}{W}{B} 2/2 legendary Elf Cleric. Ward {X}, X
/// = your experience counters. At the beginning of your end step, if a
/// permanent you controlled left the battlefield this turn, you get an
/// experience counter. Creatures you control get +1/+0 per experience counter.
pub fn minthara_merciless_soul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(crate::card::WardCost::GenericControllerExperience)],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+0 for each experience counter you have.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: yours(R::Creature),
                power: Value::ControllerExperience,
                toughness: Value::Const(0),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
            effect: Effect::AddExperience(Value::ONE),
        }],
        ..legend("Minthara, Merciless Soul", cost(&[generic(2), w(), b()]), vec![CreatureType::Elf, CreatureType::Cleric], 2, 2)
    }
}

/// Echoes of Eternity — {3}{C}{C}{C} Kindred Enchantment — Eldrazi. A
/// triggered ability of another colorless permanent you control triggers an
/// additional time; whenever you cast a colorless spell, copy it (new targets
/// allowed; a permanent spell's copy becomes a token).
pub fn echoes_of_eternity() -> CardDefinition {
    CardDefinition {
        name: "Echoes of Eternity",
        cost: crate::mana::ManaCost::new(vec![
            crate::mana::ManaSymbol::Generic(3),
            crate::mana::ManaSymbol::Colorless(3),
        ]),
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "If a triggered ability of a colorless spell you control or another colorless permanent \
                          you control triggers, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerTriggersMatching {
                filter: R::Colorless.and(R::OtherThanSource),
                include_spells: true,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Colorless,
            }),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
        }],
        ..Default::default()
    }
}

// ── The Mycotyrant (BG) ─────────────────────────────────────────────────────

fn saproling() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    }
}

fn saprolings(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(saproling()) }
}

/// Matzalantli, the Great Door // The Core — {3} legendary artifact. {T}:
/// loot. {4}, {T}: transform, only with four or more permanent types among
/// cards in your graveyard. The Core is a legendary land tapping for X mana of
/// any one color, X = permanent cards in your graveyard.
pub fn matzalantli_the_great_door() -> CardDefinition {
    let core = CardDefinition {
        name: "The Core",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_for(ManaPayload::AnyOneColor(Value::CardsInGraveyardMatching {
            who: PlayerRef::You,
            filter: R::PermanentCard,
        }))],
        ..Default::default()
    };
    CardDefinition {
        name: "Matzalantli, the Great Door",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                condition: Some(Predicate::ValueAtLeast(
                    Value::PermanentTypesInGraveyard(PlayerRef::You),
                    Value::Const(4),
                )),
                effect: Effect::Transform { what: Selector::This },
                ..Default::default()
            },
        ],
        back_face: Some(Box::new(core)),
        ..Default::default()
    }
}

/// Nemata, Primeval Warden — {2}{B}{G} 3/4 legendary Treefolk, reach. An
/// opponent's creature that would die is exiled instead, and you create a
/// Saproling (CR 614.1a + reflexive "when you do"). {G}, sacrifice a
/// Saproling: +2/+2 until end of turn. {1}{B}, sacrifice two Saprolings: draw.
pub fn nemata_primeval_warden() -> CardDefinition {
    let sap = || R::HasCreatureType(CreatureType::Saproling);
    CardDefinition {
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, exile it instead. When you do, create a \
                          1/1 green Saproling creature token.",
            effect: StaticEffect::ExileDyingOpponentCreatures {
                when_you_do: Some(Box::new(saprolings(Value::ONE))),
                tokens_too: true,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                sac_other_filter: Some((sap(), 1)),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                sac_other_filter: Some((sap(), 2)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..legend("Nemata, Primeval Warden", cost(&[generic(2), b(), g()]), vec![CreatureType::Treefolk], 3, 4)
    }
}

/// Slimefoot, the Stowaway — {1}{B}{G} 2/3 legendary Fungus. A Saproling of
/// yours dying drains each opponent for 1. {4}: create a Saproling.
pub fn slimefoot_the_stowaway() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Saproling) },
            ),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: saprolings(Value::ONE),
            ..Default::default()
        }],
        ..legend("Slimefoot, the Stowaway", cost(&[generic(1), b(), g()]), vec![CreatureType::Fungus], 2, 3)
    }
}

/// Sporecrown Thallid — {1}{G} 2/2 Fungus. Each other Fungus or Saproling
/// creature you control gets +1/+1.
pub fn sporecrown_thallid() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control that's a Fungus or Saproling gets +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(
                    R::Creature
                        .and(R::OtherThanSource)
                        .and(R::HasCreatureType(CreatureType::Fungus).or(R::HasCreatureType(CreatureType::Saproling))),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..creature("Sporecrown Thallid", cost(&[generic(1), g()]), vec![CreatureType::Fungus], 2, 2)
    }
}

/// Fungal Rebirth — {2}{G} Instant. Return target permanent card from your
/// graveyard to your hand; two Saprolings if a creature died this turn.
pub fn fungal_rebirth() -> CardDefinition {
    CardDefinition {
        name: "Fungal Rebirth",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::CreaturesDiedThisTurnTotal, Value::ONE),
                then: Box::new(saprolings(Value::Const(2))),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Fungal Sprouting — {3}{G} Sorcery. X Saprolings, X = the greatest power
/// among creatures you control.
pub fn fungal_sprouting() -> CardDefinition {
    CardDefinition {
        name: "Fungal Sprouting",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: saprolings(Value::GreatestPowerControlled { who: PlayerRef::You }),
        ..Default::default()
    }
}

// ── Thranduil, the Elvenking (BGU) ──────────────────────────────────────────

fn elf() -> R {
    R::HasCreatureType(CreatureType::Elf)
}

/// Elven Passage — Land. {T}, pay 1 life, sacrifice it: search for a basic
/// land onto the battlefield tapped; you may behold an Elf (an Elf you
/// control, or an Elf card revealed from hand), and if you do, untap it.
/// Beholding costs nothing, so it is taken whenever an Elf can be beheld.
pub fn elven_passage() -> CardDefinition {
    CardDefinition {
        name: "Elven Passage",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 1,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::If {
                    cond: Predicate::Any(vec![
                        Predicate::SelectorExists(yours(elf())),
                        Predicate::ValueAtLeast(
                            Value::CardsInHandMatching { who: PlayerRef::You, filter: elf() },
                            Value::ONE,
                        ),
                    ]),
                    then: Box::new(Effect::Untap { what: Selector::LastMoved, up_to: None }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dionus, Elvish Archdruid — {3}{G} 3/3 legendary Elf Druid. Elves you
/// control have "Whenever this creature becomes tapped during your turn, untap
/// it and put a +1/+1 counter on it. This ability triggers only once each
/// turn."
pub fn dionus_elvish_archdruid() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Elves you control have \"Whenever this creature becomes tapped during your turn, untap it \
                          and put a +1/+1 counter on it. This ability triggers only once each turn.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(elf()).and(R::ControlledByYou),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource)
                        .with_filter(Predicate::IsTurnOf(PlayerRef::You))
                        .once_per_turn(),
                    effect: Effect::Seq(vec![
                        Effect::Untap { what: Selector::This, up_to: None },
                        Effect::AddCounter {
                            what: Selector::This,
                            kind: crate::card::CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                    ]),
                }),
            },
        }],
        ..legend("Dionus, Elvish Archdruid", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Druid], 3, 3)
    }
}

/// Elrond, Moon-Reader — {2}{U} 3/3 legendary Elf Noble. The first time each
/// turn you activate an ability of a creature, draw a card. {5}{U}{U}: exile
/// up to two other target nonland permanents you control; they return under
/// their owner's control at the beginning of the next end step.
///
/// Mana abilities count (ruling 2026-06-29): a `{T}` one is read off
/// `TappedForMana`, any other off `ActivationTrait::ManaWithoutTap`; the three
/// halves share one "first time each turn" (CR 603.3d).
pub fn elrond_moon_reader() -> CardDefinition {
    let others_unused = |a: u8, b: u8| {
        Predicate::All(vec![
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            Predicate::Not(Box::new(Predicate::SourceTriggerUsedThisTurn(a))),
            Predicate::Not(Box::new(Predicate::SourceTriggerUsedThisTurn(b))),
        ])
    };
    let draw = |kind: EventKind, a: u8, b: u8| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::YourControl).with_filter(others_unused(a, b)).once_per_turn(),
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
    };
    CardDefinition {
        triggered_abilities: vec![
            draw(EventKind::AbilityActivated, 1, 2),
            draw(EventKind::TappedForMana, 0, 2),
            draw(EventKind::AbilityActivatedWith(crate::effect::ActivationTrait::ManaWithoutTap), 0, 1),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u(), u()]),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Nonland.and(R::ControlledByYou).and(R::OtherThanSource),
                effect: Box::new(Effect::ExileReturnToOwnerNextEndStep { what: Selector::Target(0), tapped: false }),
            },
            ..Default::default()
        }],
        ..legend("Elrond, Moon-Reader", cost(&[generic(2), u()]), vec![CreatureType::Elf, CreatureType::Noble], 3, 3)
    }
}

/// Maralen, Fae Ascendant — {2}{B}{G}{U} 4/5 legendary Elf Faerie Noble.
/// Flying. Whenever Maralen or another Elf or Faerie you control enters,
/// exile the top two cards of target opponent's library. Once each turn, you
/// may cast a spell with mana value at most your Elves and Faeries from among
/// cards exiled with Maralen this turn without paying its mana cost. The cap
/// is read as the spell is cast (CR 601.2).
pub fn maralen_fae_ascendant() -> CardDefinition {
    let kin = || elf().or(R::HasCreatureType(CreatureType::Faerie));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: kin() }),
            effect: Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::Target(0),
                    count: Value::Const(2),
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
                // "You may CAST a spell": an exiled land isn't playable.
                Effect::RestrictMayPlayToCasting { what: Selector::ExiledThisResolution { filter: R::Any } },
                Effect::OneCastPerTurnAmongGranted { what: Selector::ExiledThisResolution { filter: R::Any } },
                Effect::CapGrantedCastsAtCast {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    max_mana_value: Value::CountOf(Box::new(yours(R::Creature.and(kin())))),
                },
            ]),
        }],
        ..legend(
            "Maralen, Fae Ascendant",
            cost(&[generic(2), b(), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Faerie, CreatureType::Noble],
            4,
            5,
        )
    }
}

/// Thranduil's Company — {2}{G}{U} 3/4 Elf Soldier. With another Elf, you may
/// play an additional land each turn. Landfall: two +1/+1 counters on target
/// creature you control, which gains vigilance until end of turn.
pub fn thranduils_company() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you control another Elf, you may play an additional land on each of your turns.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::SelectorExists(yours(elf().and(R::OtherThanSource))),
                inner: Box::new(StaticEffect::ExtraLandPerTurn),
            },
        }],
        triggered_abilities: vec![crate::effect::shortcut::landfall(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Thranduil's Company", cost(&[generic(2), g(), u()]), vec![CreatureType::Elf, CreatureType::Soldier], 3, 4)
    }
}

/// Woodland Weavemaster — {1}{G} 1/2 Elf Druid. Vigilance. Another Elf of
/// yours entering gives it +1/+1 until end of turn. {T}: add X mana of any one
/// color, X = its power; spend it only on Elf spells and abilities of Elf
/// sources.
pub fn woodland_weavemaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: elf().and(R::OtherThanSource) },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![tap_for(ManaPayload::Restricted(
            Box::new(ManaPayload::AnyOneColor(Value::PowerOf(Box::new(Selector::This)))),
            SpendRestriction::TypeSpellOrItsAbility(CreatureType::Elf),
        ))],
        ..creature("Woodland Weavemaster", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 2)
    }
}

// ── Maralen, Fae Ascendant (BGU) ────────────────────────────────────────────

/// Eclipsed Realms — Land. As it enters, choose a creature type (Elemental,
/// Elf, Faerie, Giant, Goblin, Kithkin, Merfolk or Treefolk). {T}: Add {C}.
/// {T}: Add one mana of any color, spent only on a spell of the chosen type
/// or an ability of a source of that type.
pub fn eclipsed_realms() -> CardDefinition {
    CardDefinition {
        name: "Eclipsed Realms",
        card_types: vec![CardType::Land],
        as_enters_effect: Some(Effect::NameCreatureTypeAmong {
            what: Selector::This,
            options: vec![
                CreatureType::Elemental,
                CreatureType::Elf,
                CreatureType::Faerie,
                CreatureType::Giant,
                CreatureType::Goblin,
                CreatureType::Kithkin,
                CreatureType::Merfolk,
                CreatureType::Treefolk,
            ],
        }),
        activated_abilities: vec![
            tap_for(ManaPayload::Colorless(Value::ONE)),
            tap_for(ManaPayload::RestrictedToChosenTypeSpellOrAbility(Box::new(ManaPayload::AnyOneColor(Value::ONE)))),
        ],
        ..Default::default()
    }
}

/// Champions of the Perfect — {3}{G} 6/6 Elf Warrior. As an additional cost,
/// behold an Elf and exile it. Whenever you cast a creature spell, draw a
/// card. When it leaves the battlefield, the exiled card returns to its
/// owner's hand.
pub fn champions_of_the_perfect() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::BeholdAndExile { filter: elf() }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::CardExiledWithSource))),
                },
            },
        ],
        ..creature("Champions of the Perfect", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 6, 6)
    }
}

/// Glen Elendra Guardian — {2}{U} 3/4 Faerie Wizard. Flash, flying; enters
/// with a -1/-1 counter. {1}{U}, remove a counter from it: counter target
/// noncreature spell; its controller draws a card.
pub fn glen_elendra_guardian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        enters_with_counters: Some((crate::card::CounterType::MinusOneMinusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            remove_counter_among_filter: Some((None, 1, R::OtherThanSource.negate())),
            effect: Effect::Seq(vec![
                Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)) },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Glen Elendra Guardian", cost(&[generic(2), u()]), vec![CreatureType::Faerie, CreatureType::Wizard], 3, 4)
    }
}

/// Morcant's Loyalist — {1}{B}{G} 3/2 Elf Warrior. Other Elves you control get
/// +1/+1. When it dies, return another target Elf card from your graveyard to
/// your hand.
pub fn morcants_loyalist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Elves you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(R::Creature.and(elf()).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(elf().and(R::InYourGraveyard).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..creature("Morcant's Loyalist", cost(&[generic(1), b(), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 3, 2)
    }
}

/// Selfless Safewright — {3}{G}{G} 4/2 Elf Warrior. Flash, convoke. On entry,
/// choose a creature type: other permanents you control of that type gain
/// hexproof and indestructible until end of turn.
pub fn selfless_safewright() -> CardDefinition {
    let kin = || yours(R::IsSourceChosenCreatureType.and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Convoke],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::NameCreatureType { what: Selector::This },
            Effect::GrantKeyword { what: kin(), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: kin(), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Selfless Safewright", cost(&[generic(3), g(), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 4, 2)
    }
}

/// Glen Elendra's Answer — {2}{U}{U} Instant. Can't be countered. Counter all
/// spells and abilities your opponents control; a 1/1 blue and black flying
/// Faerie for each one countered.
pub fn glen_elendras_answer() -> CardDefinition {
    CardDefinition {
        name: "Glen Elendra's Answer",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::CounterOpponentsSpellsAndAbilities {
            token: Arc::new(TokenDefinition {
                name: "Faerie".into(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Blue, Color::Black],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
                keywords: vec![Keyword::Flying],
                ..Default::default()
            }),
        },
        ..Default::default()
    }
}

// ── Najeela, the Blade-Blossom (WUBRG) ──────────────────────────────────────

fn warrior() -> R {
    R::HasCreatureType(CreatureType::Warrior)
}

/// Blood-Chin Rager — {1}{B} 2/2 Human Warrior. Attacking gives Warrior
/// creatures you control menace until end of turn.
pub fn blood_chin_rager() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: yours(R::Creature.and(warrior())),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Blood-Chin Rager", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2)
    }
}

/// Grand Warlord Radha — {2}{R}{G} 3/4 legendary Elf Warrior. Haste. Whenever
/// one or more creatures you control attack, add that much mana in any
/// combination of {R} and/or {G}, kept through step and phase ends this turn.
pub fn grand_warlord_radha() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::effect::shortcut::on_you_attack(Effect::AddManaKeptThisTurnOfColors {
            who: PlayerRef::You,
            colors: vec![Color::Red, Color::Green],
            amount: Value::CountOf(Box::new(yours(R::Creature.and(R::IsAttacking)))),
        })],
        ..legend("Grand Warlord Radha", cost(&[generic(2), r(), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 3, 4)
    }
}

/// Herald of Dromoka — {1}{W} 2/2 Human Warrior. Vigilance; other Warriors
/// you control have vigilance.
pub fn herald_of_dromoka() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Other Warrior creatures you control have vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(warrior()).and(R::OtherThanSource)),
                keyword: Keyword::Vigilance,
            },
        }],
        ..creature("Herald of Dromoka", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2)
    }
}

/// Kargan Warleader — {1}{R}{W} 3/3 Human Warrior. Other Warriors you control
/// get +1/+1.
pub fn kargan_warleader() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Warriors you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(R::Creature.and(warrior()).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        ..creature("Kargan Warleader", cost(&[generic(1), r(), w()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 3)
    }
}

/// Rushblade Commander — {B}{R} 2/2 Azra Warrior. Warriors your team controls
/// have haste (outside Two-Headed Giant your team is you, CR 810.1).
pub fn rushblade_commander() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Warriors your team controls have haste.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(R::Creature.and(warrior())), keyword: Keyword::Haste },
        }],
        ..creature("Rushblade Commander", cost(&[b(), r()]), vec![CreatureType::Azra, CreatureType::Warrior], 2, 2)
    }
}

/// Samut, Vizier of Naktamun — {1}{R}{G} 2/3 legendary Human Warrior Cleric.
/// First strike, vigilance, haste. A creature of yours that entered this turn
/// dealing combat damage to a player draws a card.
pub fn samut_vizier_of_naktamun() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::EnteredThisTurn) },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend(
            "Samut, Vizier of Naktamun",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Cleric],
            2,
            3,
        )
    }
}

/// Druids' Repository — {1}{G}{G} Enchantment. A creature of yours attacking
/// puts a charge counter on it; remove one: add one mana of any color.
pub fn druids_repository() -> CardDefinition {
    CardDefinition {
        name: "Druids' Repository",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: crate::card::CounterType::Charge, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((crate::card::CounterType::Charge, 1)),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Toph, the First Metalbender (RGW) ───────────────────────────────────────

fn land_creature() -> R {
    R::Land.and(R::Creature)
}

/// The Stasis Coffin — {3} legendary artifact. {2}, {T}, exile it: you gain
/// protection from everything until your next turn.
pub fn the_stasis_coffin() -> CardDefinition {
    CardDefinition {
        name: "The Stasis Coffin",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::PlayerProtectionUntilNextTurn { who: PlayerRef::You },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Avatar Kyoshi, Earthbender — {5}{G}{G}{G} 6/6 legendary Human Avatar.
/// Hexproof during your turn. At the beginning of combat on your turn,
/// earthbend 8, then untap that land.
pub fn avatar_kyoshi_earthbender() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, Avatar Kyoshi has hexproof.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Hexproof }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::Earthbend { n: Value::Const(8) },
                Effect::Untap { what: Selector::Target(0), up_to: None },
            ]),
        }],
        ..legend(
            "Avatar Kyoshi, Earthbender",
            cost(&[generic(5), g(), g(), g()]),
            vec![CreatureType::Human, CreatureType::Avatar],
            6,
            6,
        )
    }
}

/// Bumi, Eclectic Earthbender — {3}{G}{G} 4/4 legendary Human Noble Ally. ETB
/// earthbend 1; attacking puts two +1/+1 counters on each land creature you
/// control.
pub fn bumi_eclectic_earthbender() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Earthbend { n: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: yours(land_creature()),
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            },
        ],
        ..legend(
            "Bumi, Eclectic Earthbender",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Ally],
            4,
            4,
        )
    }
}

/// Toph, Earthbending Master — {3}{G} 2/4 legendary Human Warrior Ally.
/// Landfall: you get an experience counter. Whenever you attack, earthbend X,
/// X = your experience counters.
pub fn toph_earthbending_master() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::landfall(Effect::AddExperience(Value::ONE)),
            crate::effect::shortcut::on_you_attack(Effect::Earthbend { n: Value::ControllerExperience }),
        ],
        ..legend(
            "Toph, Earthbending Master",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            2,
            4,
        )
    }
}

/// Toph, Greatest Earthbender — {2}{R}{G} 3/3 legendary Human Warrior Ally.
/// ETB earthbend X, X = the mana spent to cast her. Land creatures you
/// control have double strike.
pub fn toph_greatest_earthbender() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Land creatures you control have double strike.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(land_creature()), keyword: Keyword::DoubleStrike },
        }],
        triggered_abilities: vec![etb(Effect::Earthbend { n: Value::CastSpellManaSpent })],
        ..legend(
            "Toph, Greatest Earthbender",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            3,
            3,
        )
    }
}

/// Avatar Kyoshi — the back of The Legend of Kyoshi: a 5/4 legendary Avatar.
/// Lands you control have trample and hexproof. {T}: add X mana of any one
/// color, X = the greatest power among creatures you control.
pub fn avatar_kyoshi() -> CardDefinition {
    CardDefinition {
        // CR 105.2c — a transformed Saga's back face carries a color indicator.
        color_indicator: vec![Color::Green],
        static_abilities: vec![
            StaticAbility {
                description: "Lands you control have trample.",
                effect: StaticEffect::GrantKeyword { applies_to: yours(R::Land), keyword: Keyword::Trample },
            },
            StaticAbility {
                description: "Lands you control have hexproof.",
                effect: StaticEffect::GrantKeyword { applies_to: yours(R::Land), keyword: Keyword::Hexproof },
            },
        ],
        activated_abilities: vec![tap_for(ManaPayload::AnyOneColor(Value::GreatestPowerControlled {
            who: PlayerRef::You,
        }))],
        ..legend("Avatar Kyoshi", crate::mana::ManaCost::default(), vec![CreatureType::Avatar], 5, 4)
    }
}

/// The Legend of Kyoshi — {4}{G}{G} Saga. I: draw cards equal to the greatest
/// power among creatures you control. II: earthbend X, X = cards in your hand;
/// that land becomes an Island too. III: exile it, return it transformed
/// (Avatar Kyoshi).
pub fn the_legend_of_kyoshi() -> CardDefinition {
    CardDefinition {
        name: "The Legend of Kyoshi",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga(),
        saga_chapters: vec![
            (
                1,
                Effect::Draw { who: Selector::You, amount: Value::GreatestPowerControlled { who: PlayerRef::You } },
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::Earthbend { n: Value::HandSizeOf(PlayerRef::You) },
                    Effect::GainLandType {
                        what: Selector::Target(0),
                        land_type: crate::card::LandType::Island,
                        duration: Duration::Permanent,
                    },
                ]),
            ),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(avatar_kyoshi())),
        ..Default::default()
    }
}

/// Earthshape — {2}{W} Instant. Earthbend 3. Then each creature you control
/// with power at most that land's gains hexproof and indestructible until end
/// of turn, and you gain hexproof until end of turn.
pub fn earthshape() -> CardDefinition {
    let shielded = || {
        Predicate::ValueAtLeast(
            Value::PowerOf(Box::new(Selector::Target(0))),
            Value::PowerOf(Box::new(Selector::TriggerSource)),
        )
    };
    CardDefinition {
        name: "Earthshape",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Earthbend { n: Value::Const(3) },
            Effect::ForEach {
                selector: yours(R::Creature),
                body: Box::new(Effect::If {
                    cond: shielded(),
                    then: Box::new(Effect::Seq(vec![
                        Effect::GrantKeyword {
                            what: Selector::TriggerSource,
                            keyword: Keyword::Hexproof,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::TriggerSource,
                            keyword: Keyword::Indestructible,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                }),
            },
            Effect::PlayerHexproofThisTurn { who: Selector::You },
        ]),
        ..Default::default()
    }
}

// ── Avatar Aang (RGWU) ──────────────────────────────────────────────────────

/// Aang, Master of Elements — the back of Avatar Aang: a 6/6 flying Avatar
/// Ally. Spells you cast cost {W}{U}{B}{R}{G} less (an unmatched colored pip
/// takes a generic one). At the beginning of each upkeep, you may transform
/// him: if you do, gain 4 life, draw four, four +1/+1 counters on him, and 4
/// damage to each opponent.
pub fn aang_master_of_elements() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        color_indicator: vec![Color::White, Color::Blue, Color::Red, Color::Green],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast cost {W}{U}{B}{R}{G} less to cast.",
            effect: StaticEffect::ColoredCostReduction { filter: R::Any, less: cost(&[w(), u(), b(), r(), g()]) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::MayDo {
                description: "Transform Aang, Master of Elements?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Transform { what: Selector::This },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
                    Effect::Draw { who: Selector::You, amount: Value::Const(4) },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: crate::card::CounterType::PlusOnePlusOne,
                        amount: Value::Const(4),
                    },
                    Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(4) },
                ])),
            },
        }],
        ..legend(
            "Aang, Master of Elements",
            crate::mana::ManaCost::default(),
            vec![CreatureType::Avatar, CreatureType::Ally],
            6,
            6,
        )
    }
}

/// Avatar Aang // Aang, Master of Elements — {R}{G}{W}{U} 4/4 legendary Human
/// Avatar Ally. Flying, firebending 2. Whenever you waterbend, earthbend,
/// firebend or airbend, draw a card; then if you've done all four this turn,
/// transform him.
pub fn avatar_aang() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Firebending(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::Performed(crate::effect::KeywordAct::Bent(None)),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::BentAllFourThisTurn { who: PlayerRef::You },
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(aang_master_of_elements())),
        ..legend(
            "Avatar Aang",
            cost(&[r(), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Avatar, CreatureType::Ally],
            4,
            4,
        )
    }
}

/// Mai and Zuko — {1}{U}{B}{R} 3/5 legendary Human Noble Ally. Firebending 3.
/// You may cast Ally spells and artifact spells as though they had flash.
pub fn mai_and_zuko() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Firebending(3)],
        static_abilities: vec![StaticAbility {
            description: "You may cast Ally spells and artifact spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash {
                filter: R::HasCreatureType(CreatureType::Ally).or(R::Artifact),
            },
        }],
        ..legend(
            "Mai and Zuko",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Ally],
            3,
            5,
        )
    }
}

/// Monk Gyatso — {3}{W} 3/3 legendary Human Monk. Whenever another creature
/// you control becomes the target of a spell or ability, you may airbend it.
pub fn monk_gyatso() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourCreatureTargeted).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
            ),
            effect: Effect::MayDo {
                description: "Airbend that creature?".into(),
                body: Box::new(Effect::Airbend { what: Selector::TriggerSource }),
            },
        }],
        ..legend("Monk Gyatso", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Monk], 3, 3)
    }
}

/// Avatar Kuruk — the back of The Legend of Kuruk: a 4/3 legendary Avatar.
/// Whenever you cast a spell, a 1/1 colorless Spirit that can't block or be
/// blocked by non-Spirit creatures. Exhaust — Waterbend {20}: take an extra
/// turn after this one.
pub fn avatar_kuruk() -> CardDefinition {
    CardDefinition {
        color_indicator: vec![Color::Blue],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Spirit".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
                    keywords: vec![
                        Keyword::CantBlockMatching(Box::new(R::HasCreatureType(CreatureType::Spirit).negate())),
                        Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(CreatureType::Spirit))),
                    ],
                    ..Default::default()
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(20)]),
            waterbend: true,
            exhaust: true,
            effect: Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
            ..Default::default()
        }],
        ..legend("Avatar Kuruk", crate::mana::ManaCost::default(), vec![CreatureType::Avatar], 4, 3)
    }
}

/// The Legend of Kuruk — {2}{U}{U} Saga. I, II: scry 2, then draw a card.
/// III: exile it, return it transformed (Avatar Kuruk).
pub fn the_legend_of_kuruk() -> CardDefinition {
    let look = || {
        Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ])
    };
    CardDefinition {
        name: "The Legend of Kuruk",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga(),
        saga_chapters: vec![(1, look()), (2, look()), (3, Effect::ExileSelfReturnTransformed)],
        back_face: Some(Box::new(avatar_kuruk())),
        ..Default::default()
    }
}

/// Avatar Yangchen — the back of The Legend of Yangchen: a 4/5 flying
/// legendary Avatar. Whenever you cast your second spell each turn, airbend up
/// to one other target nonland permanent.
pub fn avatar_yangchen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        color_indicator: vec![Color::White],
        triggered_abilities: vec![crate::effect::shortcut::flurry(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Nonland.and(R::OtherThanSource),
            effect: Box::new(Effect::Airbend { what: Selector::Target(0) }),
        })],
        ..legend("Avatar Yangchen", crate::mana::ManaCost::default(), vec![CreatureType::Avatar], 4, 5)
    }
}

/// The Legend of Yangchen — {3}{W}{W} Saga. I: starting with you, each player
/// chooses up to one permanent with mana value 3 or greater among permanents
/// your opponents control; exile those. II: you may have target opponent draw
/// three cards; if you do, draw three. III: exile it, return it transformed
/// (Avatar Yangchen).
pub fn the_legend_of_yangchen() -> CardDefinition {
    CardDefinition {
        name: "The Legend of Yangchen",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga(),
        saga_chapters: vec![
            (1, Effect::EachPlayerChoosesOpponentPermanentExile { filter: R::ManaValueAtLeast(3) }),
            (
                2,
                Effect::MayDo {
                    description: "Have target opponent draw three cards, then draw three?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw {
                            who: target_filtered(R::Player.and(R::OpponentPlayer)),
                            amount: Value::Const(3),
                        },
                        Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                    ])),
                },
            ),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(avatar_yangchen())),
        ..Default::default()
    }
}
