//! Commander: the cards the **Graveyard Overdrive** precon (M3C, Disa the
//! Restless) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_disa.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility,
    PlaneswalkerSubtype, SelectionRequirement as R, Selector, SplitCard, SplitHalf, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{encore, etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, x};
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

/// A Lhurgoyf: power = card types among cards in all graveyards, toughness
/// one more (CR 604.3 characteristic-defining).
fn goyf(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::DistinctTypesInAllGraveyards),
        ..creature(name, mana, vec![CreatureType::Lhurgoyf], 0, 1)
    }
}

/// "A Tarmogoyf token": a {1}{G} green Lhurgoyf whose P/T track the
/// graveyards. A token's `dynamic_pt` is read once as it is made, so the
/// characteristic-defining ability rides a static instead.
fn tarmogoyf_token() -> Arc<TokenDefinition> {
    let types = || Value::CardTypesInAllGraveyards;
    Arc::new(TokenDefinition {
        name: "Tarmogoyf".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Lhurgoyf], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Tarmogoyf's power is equal to the number of card types among cards in all graveyards and its toughness is equal to that number plus 1.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: types(),
                toughness: Value::Sum(vec![types(), Value::ONE]),
            },
        }],
        ..Default::default()
    })
}

fn make(n: Value, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: def }
}

fn flier(name: &str, color: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32, kw: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: color,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords: kw,
        ..Default::default()
    })
}

fn lhurgoyf_creature() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Lhurgoyf))
}

/// Disa the Restless — a Lhurgoyf card reaching your graveyard from anywhere
/// but the battlefield comes back; connecting makes a Tarmogoyf.
/// ⚠ "Not from the battlefield" is read as "not put there from the
/// battlefield this turn": a Lhurgoyf that died earlier this turn and then
/// reached the graveyard another way doesn't return.
pub fn disa_the_restless() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::PermanentCard
                            .and(R::HasCreatureType(CreatureType::Lhurgoyf))
                            .and(R::Not(Box::new(R::PutIntoGraveyardFromBattlefieldThisTurn))),
                    },
                ),
                effect: Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .once_per_batch_across_players(),
                effect: make(Value::ONE, tarmogoyf_token()),
            },
        ],
        ..creature(
            "Disa the Restless",
            cost(&[generic(2), b(), r(), g()]),
            vec![CreatureType::Human, CreatureType::Scout],
            5,
            6,
        )
    }
}

/// Barrowgoyf — deathtouch, lifelink; connecting may mill that many and take
/// a creature card.
pub fn barrowgoyf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Mill that many cards?".into(),
                body: Box::new(Effect::MillThenToHandN {
                    amount: Value::TriggerEventAmount,
                    filter: R::Creature,
                    take: Value::ONE,
                    otherwise: None,
                }),
            },
        }],
        ..goyf("Barrowgoyf", cost(&[generic(2), b()]))
    }
}

/// Polygoyf — trample, myriad (CR 702.116).
pub fn polygoyf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Myriad,
        }],
        ..goyf("Polygoyf", cost(&[generic(2), g()]))
    }
}

/// Pyrogoyf — it or another Lhurgoyf of yours entering deals its power to
/// any target.
pub fn pyrogoyf() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: lhurgoyf_creature() },
            ),
            effect: Effect::DealDamageEqualToPower { source: Selector::TriggerSource, target: target_any() },
        }],
        ..goyf("Pyrogoyf", cost(&[generic(3), r()]))
    }
}

/// Tarmogoyf Nest — Kindred Enchantment — Lhurgoyf Aura; the enchanted land
/// makes Tarmogoyf tokens.
pub fn tarmogoyf_nest() -> CardDefinition {
    CardDefinition {
        name: "Tarmogoyf Nest",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lhurgoyf],
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter: R::Land } },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{1}{G}, {T}: Create a Tarmogoyf token.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    mana_cost: cost(&[generic(1), g()]),
                    effect: make(Value::ONE, tarmogoyf_token()),
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Broodmate Tyrant — a 5/5 Dragon comes with it; encore.
pub fn broodmate_tyrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(make(
            Value::ONE,
            flier("Dragon", vec![Color::Red], vec![CreatureType::Dragon], 5, 5, vec![Keyword::Flying]),
        ))],
        activated_abilities: vec![encore(cost(&[generic(5), b(), r(), g()]))],
        ..creature("Broodmate Tyrant", cost(&[generic(4), b(), r(), g()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Coram, the Undertaker — +X/+0 for the biggest creature card in any
/// graveyard; attacking mills everyone one; plays from what was milled.
pub fn coram_the_undertaker() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "Coram gets +X/+0, where X is the greatest power among creature cards in all graveyards.",
                effect: StaticEffect::PumpPTByValue {
                    applies_to: Selector::This,
                    power: Value::GreatestPowerAmongCards(Box::new(Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: Zone::Graveyard,
                        filter: R::Creature,
                    })),
                    toughness: Value::Const(0),
                },
            },
            StaticAbility {
                description: "During each of your turns, you may play a land and cast a spell from among cards in graveyards that were put there from libraries this turn.",
                effect: StaticEffect::MayPlayCardsMilledThisTurn,
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
        }],
        ..creature(
            "Coram, the Undertaker",
            cost(&[generic(1), b(), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            0,
            5,
        )
    }
}

/// Exterminator Magmarch — an instant or sorcery aimed at one opponent's
/// nonland permanent is copied onto another opponent's; {1}{B}: regenerate.
pub fn exterminator_magmarch() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
                Predicate::CastSpellTargetsOnlyOneMatching(R::Nonland.and(R::ControlledByOpponent)),
            ])),
            effect: Effect::CopySpellOntoAnotherOpponentsPermanent { what: Selector::TriggerSource },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Exterminator Magmarch",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Construct],
            5,
            3,
        )
    }
}

/// Find // Finality — two creature cards back; or two counters, then -4/-4
/// to everything. ⚠ Find picks its two at resolution (the printed "target"
/// isn't declared on cast).
pub fn find_finality() -> CardDefinition {
    CardDefinition {
        name: "Find // Finality",
        cost: ManaCost::new(vec![hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ReturnGraveyardCardsToHand { filter: R::Creature, max: Value::Const(2) },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(4), b(), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::MayDo {
                        description: "Put two +1/+1 counters on a creature you control?".into(),
                        body: Box::new(Effect::AddCounter {
                            what: Selector::one_of(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(2),
                        }),
                    },
                    Effect::PumpPT {
                        what: Selector::EachPermanent(R::Creature),
                        power: Value::Const(-4),
                        toughness: Value::Const(-4),
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Gluttonous Hellkite — cast, each player sacrifices X creatures; it enters
/// with two +1/+1 counters per creature sacrificed this way.
pub fn gluttonous_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    count: Value::XFromCost,
                    filter: R::Creature,
                },
                Effect::SpellEntersWithCounters {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Times(Box::new(Value::SacrificedCount), Box::new(Value::Const(2))),
                },
            ]),
        }],
        ..creature(
            "Gluttonous Hellkite",
            cost(&[x(), x(), b(), r(), g()]),
            vec![CreatureType::Dragon],
            3,
            3,
        )
    }
}

/// Infested Thrinax — flash; this turn, each nontoken creature of yours that
/// dies leaves Saprolings equal to its power.
pub fn infested_thrinax() -> CardDefinition {
    let saproling = flier("Saproling", vec![Color::Green], vec![CreatureType::Saproling], 1, 1, vec![]);
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::CreaturesYouControlDyingThisTurn {
            body: Box::new(Effect::If {
                cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
                then: Box::new(make(Value::PowerOf(Box::new(Selector::TriggerSource)), saproling)),
                else_: Box::new(Effect::Noop),
            }),
        })],
        ..creature("Infested Thrinax", cost(&[generic(3), b(), g()]), vec![CreatureType::Lizard], 4, 4)
    }
}

/// Izoni, Thousand-Eyed — an Insect per creature card in your graveyard;
/// {B}{G}, sacrifice another creature: gain 1 and draw.
pub fn izoni_thousand_eyed() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(make(
            Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
            flier("Insect", vec![Color::Black, Color::Green], vec![CreatureType::Insect], 1, 1, vec![]),
        ))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), g()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Izoni, Thousand-Eyed",
            cost(&[generic(2), b(), b(), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Shaman],
            2,
            3,
        )
    }
}

/// Liliana, Death's Majesty — Zombies, a reanimated Zombie, and a sweep of
/// everything that isn't one.
pub fn liliana_deaths_majesty() -> CardDefinition {
    let zombie = flier("Zombie", vec![Color::Black], vec![CreatureType::Zombie], 2, 2, vec![]);
    CardDefinition {
        name: "Liliana, Death's Majesty",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Liliana], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    make(Value::ONE, zombie),
                    Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                ]),
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::AddCreatureTypes {
                        what: Selector::LastMoved,
                        creature_types: vec![CreatureType::Zombie],
                        duration: Duration::Permanent,
                    },
                    Effect::BecomeColor {
                        what: Selector::LastMoved,
                        colors: vec![Color::Black],
                        duration: Duration::Permanent,
                        additive: true,
                    },
                ]),
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Zombie)))),
                    ),
                },
                x_cost: false,
            },
        ],
        ..Default::default()
    }
}

/// Riveteers Charm — edict the biggest, impulse three until your next end
/// step, or exile a graveyard.
pub fn riveteers_charm() -> CardDefinition {
    let big = || R::Creature.or(R::Planeswalker);
    CardDefinition {
        name: "Riveteers Charm",
        cost: cost(&[b(), r(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Sacrifice {
                who: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: big().and(R::HasGreatestManaValueAmongControlled(Box::new(big()))),
            },
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(3),
                duration: crate::card::MayPlayDuration::UntilYourNextEndStep,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
        ]),
        ..Default::default()
    }
}

/// Sawhorn Nemesis — choose a player; damage to them and their permanents is
/// doubled.
pub fn sawhorn_nemesis() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::RememberPlayerOnSource { who: PlayerRef::HostileOpponent }),
        static_abilities: vec![StaticAbility {
            description: "If a source would deal damage to the chosen player or a permanent they control, it deals double that damage instead.",
            effect: StaticEffect::DoubleDamageToChosenPlayer,
        }],
        ..creature("Sawhorn Nemesis", cost(&[generic(3), r()]), vec![CreatureType::Dinosaur], 2, 4)
    }
}

/// Tempt with Mayhem — tempting offer: copy target instant or sorcery; each
/// opponent may copy it too, and you copy it again per opponent who does.
pub fn tempt_with_mayhem() -> CardDefinition {
    CardDefinition {
        name: "Tempt with Mayhem",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::TemptingOffer {
            body: Box::new(Effect::CopySpellMayChooseTargets {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
                count: Value::ONE,
            }),
        },
        ..Default::default()
    }
}

/// Ziatora, the Incinerator — at your end step you may sacrifice another
/// creature; when you do, its power in damage to any target and three
/// Treasures. ⚠ The sacrifice picks your weakest other creature
/// (`MaySacrifice`'s auto-pick), not the one you'd choose.
pub fn ziatora_the_incinerator() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::MaySacrifice {
                description: "Sacrifice another creature?".into(),
                filter: R::Creature.and(R::OtherThanSource),
                count: Value::ONE,
                then: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::Seq(vec![
                        Effect::DealDamage { to: target_any(), amount: Value::SacrificedPower },
                        crate::effect::shortcut::mint_treasures(3),
                    ])),
                }),
                else_: None,
            },
        }],
        ..creature(
            "Ziatora, the Incinerator",
            cost(&[generic(3), b(), r(), g()]),
            vec![CreatureType::Demon, CreatureType::Dragon],
            6,
            6,
        )
    }
}
