//! Commander: the cards the **Draconic Dissent** precon (CLB, Firkraag,
//! Cunning Instigator) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Baeloth Barrityl** — the powers compared are each creature's own
//!   (printed, pumps, counters), not static anthems.
//! - **Firkraag** — the goaded creature is the engine's pick (greatest
//!   power), not a target; "had to attack this combat" reads as goaded or
//!   must-attack when the damage is dealt.
//! - **Rowan Kenrith** — the +2's forced attacks last until your next turn
//!   and reach the target player's creatures at resolution only.
//! - **Stuffy Doll** — the chosen player is the engine's most hostile
//!   opponent.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EntersAsCopy, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::catalog::sets::{enters_tapped_unless_you_control, tap_add};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, cast_is_noncreature, etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, RevealMissDest,
    StaticAbility, StaticEffect, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, r, u, x};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn opp_creature() -> R {
    R::Creature.and(R::ControlledByOpponent)
}

fn treasure() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::treasure_token()),
    }
}

/// Artificer Class — L1: your first artifact spell each turn costs {1} less;
/// L2: reveal until an artifact, to hand; L3: each end step, a token copy of
/// target artifact you control.
pub fn artificer_class() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "The first artifact spell you cast each turn costs {1} less to cast.",
            effect: StaticEffect::FirstMatchingSpellEachTurnCostsLess { filter: R::Artifact, amount: 1 },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(2)),
                effect: Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Artifact,
                    to: ZoneDest::Hand(PlayerRef::You),
                    cap: Value::Const(1000),
                    miss_dest: RevealMissDest::BottomRandom,
                    life_per_revealed: 0,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(R::Artifact.and(R::ControlledByYou)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            },
        ],
        activated_abilities: vec![
            level_up(cost(&[generic(1), u()]), 1),
            level_up(cost(&[generic(5), u()]), 2),
        ],
        ..enchantment("Artificer Class", cost(&[generic(1), u()]))
    }
}

/// Astral Dragon — flying; enters: two token copies of target noncreature
/// permanent that are also 3/3 flying Dragon creatures.
pub fn astral_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::Const(2),
            source: target_filtered(R::Permanent.and(R::Noncreature)),
            extra_creature_types: vec![CreatureType::Dragon],
            extra_card_types: vec![CardType::Creature],
            override_pt: Some((3, 3)),
            override_colors: None,
            enters_tapped: false,
            non_legendary: false,
            legendary: false,
            extra_keywords: vec![Keyword::Flying],
        })],
        ..creature("Astral Dragon", cost(&[generic(6), u(), u()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Baeloth Barrityl, Entertainer — opposing creatures with less power are
/// goaded; a goaded attacking or blocking creature dying makes you a
/// Treasure. Choose a Background. Residual: the powers compared are each
/// creature's own, not static anthems.
pub fn baeloth_barrityl_entertainer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::ChooseABackground],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control with power less than Baeloth \
                          Barrityl's power are goaded.",
            effect: StaticEffect::OpponentCreaturesWithLesserPowerAreGoaded,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsGoaded.and(R::IsAttacking.or(R::IsBlocking)),
                },
            ),
            effect: treasure(),
        }],
        ..creature(
            "Baeloth Barrityl, Entertainer",
            cost(&[generic(4), r()]),
            vec![CreatureType::Elf, CreatureType::Shaman],
            2,
            5,
        )
    }
}

/// Bothersome Quasit — menace; your opponents' goaded creatures can't block;
/// casting a noncreature spell goads target creature an opponent controls.
pub fn bothersome_quasit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Goaded creatures your opponents control can't block.",
            effect: StaticEffect::OpponentsGoadedCreaturesCantBlock,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_noncreature()),
            effect: Effect::Goad { what: target_filtered(opp_creature()) },
        }],
        ..creature("Bothersome Quasit", cost(&[generic(2), r()]), vec![CreatureType::Demon], 3, 2)
    }
}

/// Castle Vantress — enters tapped unless you control an Island; {T}: {U};
/// {2}{U}{U}, {T}: scry 2.
pub fn castle_vantress() -> CardDefinition {
    CardDefinition {
        name: "Castle Vantress",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped_unless_you_control(LandType::Island)],
        activated_abilities: vec![
            tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), u()]),
                tap_cost: true,
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Clan Crafter — Background: commander creatures you own have "{2},
/// sacrifice an artifact: a +1/+1 counter on this creature, draw a card."
pub fn clan_crafter() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Background],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"{2}, Sacrifice an artifact: Put a \
                          +1/+1 counter on this creature and draw a card.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::IsCommander).and(R::OwnedByYou),
                ),
                ability: ActivatedAbility {
                    mana_cost: cost(&[generic(2)]),
                    sac_other_filter: Some((R::Artifact, 1)),
                    effect: Effect::Seq(vec![
                        Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ]),
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..enchantment("Clan Crafter", cost(&[generic(1), u()]))
    }
}

/// Death Kiss — an opposing creature attacking another of your opponents
/// has its power doubled; monstrosity X; monstrous, goad up to X target
/// creatures your opponents control.
pub fn death_kiss() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::OpponentControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::IsAttackingAnOpponent,
                    },
                ),
                effect: Effect::DoublePower {
                    what: Selector::TriggerSource,
                    times: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameMonstrous, EventScope::SelfSource),
                effect: Effect::CapTargetsAt {
                    amount: Value::TriggerEventAmount,
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 20,
                        min_targets: 0,
                        filter: opp_creature(),
                        effect: Box::new(Effect::Goad { what: Selector::Target(0) }),
                    }),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), x(), r()]),
            effect: Effect::Monstrosity { n: Value::XFromCost },
            ..Default::default()
        }],
        ..creature("Death Kiss", cost(&[generic(5), r()]), vec![CreatureType::Beholder], 5, 5)
    }
}

/// Dissipation Field — whenever a permanent deals damage to you, return it
/// to its owner's hand.
pub fn dissipation_field() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Permanent },
            ),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..enchantment("Dissipation Field", cost(&[generic(2), u(), u()]))
    }
}

/// Firkraag, Cunning Instigator — flying, haste; Dragons you control
/// attacking an opponent goad a creature of theirs; a creature that had to
/// attack dealing combat damage to your opponent grows Firkraag and draws.
/// Residuals: the goaded creature is the engine's pick; "had to attack" is
/// goaded or must-attack when the damage is dealt.
pub fn firkraag_cunning_instigator() -> CardDefinition {
    let dragon = || R::HasCreatureType(CreatureType::Dragon);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: dragon() })
                    .once_per_batch(),
                effect: Effect::GoadACreatureOfEachOpponentAttackedBy { attackers: dragon() },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                    .with_filter(Predicate::All(vec![
                        Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer },
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::IsGoaded.or(R::HasKeyword(Keyword::MustAttack)),
                        },
                    ])),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
            },
        ],
        ..creature(
            "Firkraag, Cunning Instigator",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Dragon],
            3,
            3,
        )
    }
}

/// Loot Dispute — enters: take the initiative and a Treasure; attacking the
/// player with the initiative, a Treasure; completing a dungeon, a 5/5
/// flying Dragon.
pub fn loot_dispute() -> CardDefinition {
    let dragon = Arc::new(TokenDefinition {
        name: "Dragon".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![Effect::TakeInitiative { who: PlayerRef::You }, treasure()])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::OpponentOfYoursAttacked)
                    .with_filter(Predicate::All(vec![
                        Predicate::SamePlayer(PlayerRef::Target(0), PlayerRef::You),
                        Predicate::HasInitiative { who: PlayerRef::Triggerer },
                    ])),
                effect: treasure(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DungeonCompleted, EventScope::YourControl),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: dragon },
            },
        ],
        ..enchantment("Loot Dispute", cost(&[generic(3), r()]))
    }
}

/// Mocking Doppelganger — flash; may enter as a copy of an opponent's
/// creature whose other same-named creatures are goaded.
pub fn mocking_doppelganger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        enters_as_copy: Some(EntersAsCopy {
            filter: opp_creature(),
            extra_static: vec![StaticAbility {
                description: "Other creatures with the same name as this creature are goaded.",
                effect: StaticEffect::OthersNamedLikeThisAreGoaded,
            }],
            ..Default::default()
        }),
        ..creature(
            "Mocking Doppelganger",
            cost(&[generic(3), u()]),
            vec![CreatureType::Shapeshifter],
            0,
            0,
        )
    }
}

/// Pursued Whale — enters: each opponent gets a 1/1 Pirate that can't block
/// and makes their creatures attack each combat; spells your opponents cast
/// targeting it cost {3} more.
pub fn pursued_whale() -> CardDefinition {
    let pirate = Arc::new(TokenDefinition {
        name: "Pirate".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pirate], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control attack each combat if able.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::MustAttack,
            },
        }],
        ..Default::default()
    });
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells your opponents cast that target this creature cost {3} more.",
            effect: StaticEffect::TaxOpponentSpellsTargetingThis { amount: 3 },
        }],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::EachOpponent,
            count: Value::ONE,
            definition: pirate,
        })],
        ..creature("Pursued Whale", cost(&[generic(5), u(), u()]), vec![CreatureType::Whale], 8, 8)
    }
}

fn kenrith(
    name: &'static str,
    mana: ManaCost,
    subtype: PlaneswalkerSubtype,
    partner: &str,
    loyalty_abilities: Vec<LoyaltyAbility>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![subtype], ..Default::default() },
        keywords: vec![Keyword::PartnerWith(partner.into())],
        triggered_abilities: vec![crate::effect::shortcut::partner_with_search(partner)],
        base_loyalty: 4,
        can_be_commander: true,
        loyalty_abilities,
        ..Default::default()
    }
}

/// Rowan Kenrith — +2: target player's creatures attack if able (residual:
/// until your next turn, those it has at resolution); −2: 3 damage to each
/// tapped creature target player controls; −8: an emblem copying their
/// activated abilities. Partner with Will Kenrith.
pub fn rowan_kenrith() -> CardDefinition {
    let theirs = |filter: R| Selector::ControlledBy { who: PlayerRef::Target(0), filter };
    kenrith(
        "Rowan Kenrith",
        cost(&[generic(4), r(), r()]),
        PlaneswalkerSubtype::Rowan,
        "Will Kenrith",
        vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::GrantKeyword {
                    what: theirs(R::Creature),
                    keyword: Keyword::MustAttack,
                    duration: Duration::UntilNextTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::DealDamage {
                    to: theirs(R::Creature.and(R::Tapped)),
                    amount: Value::Const(3),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::Target(0),
                    name: "Rowan Kenrith".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::AbilityActivated, EventScope::YourControl),
                        effect: Effect::CopyActivatedAbilityMayChooseTargets,
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
    )
}

/// Sly Instigator — {U}, {T}: until your next turn, target opposing creature
/// can't be blocked; goad it.
pub fn sly_instigator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(opp_creature()),
                    keyword: Keyword::Unblockable,
                    duration: Duration::UntilNextTurn,
                },
                Effect::Goad { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Sly Instigator",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            4,
        )
    }
}

/// Stuffy Doll — indestructible; damage dealt to it is dealt to the chosen
/// player; {T}: 1 damage to itself. Residual: the chosen player is the
/// engine's most hostile opponent.
pub fn stuffy_doll() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Indestructible],
        as_enters_effect: Some(Effect::RememberPlayerOnSource { who: PlayerRef::HostileOpponent }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: Selector::This, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Stuffy Doll", cost(&[generic(5)]), vec![CreatureType::Construct], 0, 1)
    }
}

/// Thunder Dragon — flying; enters: 3 damage to each creature without flying.
pub fn thunder_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))))),
            amount: Value::Const(3),
        })],
        ..creature("Thunder Dragon", cost(&[generic(5), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Will Kenrith — +2: up to two creatures become 0/3 with no abilities until
/// your next turn; −2: target player draws two and their instants, sorceries
/// and planeswalkers cost {2} less until your next turn; −8: an emblem
/// copying their instants and sorceries. Partner with Rowan Kenrith.
pub fn will_kenrith() -> CardDefinition {
    kenrith(
        "Will Kenrith",
        cost(&[generic(4), u(), u()]),
        PlaneswalkerSubtype::Will,
        "Rowan Kenrith",
        vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Seq(vec![
                        Effect::SetBasePT {
                            what: Selector::Target(0),
                            power: Value::Const(0),
                            toughness: Value::Const(3),
                            duration: Duration::UntilNextTurn,
                        },
                        Effect::LoseAllAbilities { what: Selector::Target(0), duration: Duration::UntilNextTurn },
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: target_filtered(R::Player), amount: Value::Const(2) },
                    Effect::SpellDiscountUntilYourNextTurn {
                        who: PlayerRef::Target(0),
                        amount: 2,
                        filter: R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery))
                            .or(R::Planeswalker),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::Target(0),
                    name: "Will Kenrith".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                            .with_filter(cast_is_instant_or_sorcery()),
                        effect: Effect::CopySpellMayChooseTargets {
                            what: Selector::TriggerSource,
                            count: Value::ONE,
                        },
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
    )
}
