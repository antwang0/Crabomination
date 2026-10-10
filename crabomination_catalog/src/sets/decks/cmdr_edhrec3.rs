//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, third file (`cmdr_edhrec2` is the
//! second). Ketramose, the New Dawn, Niko, Light of Hope, Syr Gwyn, Hero of
//! Ashvale and Kastral, the Windcrested. Tests in
//! `tests/recent_b/cmdr_edhrec3.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, TriggerZone, ZoneDest};
use crate::mana::{cost, generic, r, u, w, x, Color, SpendRestriction};
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
