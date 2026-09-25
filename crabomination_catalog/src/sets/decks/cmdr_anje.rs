//! Commander: the cards the **Merciless Rage** precon (C19, Anje
//! Falkenrath) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_anje.rs`.
//!
//! Residuals (each also on its card):
//! - **Archfiend of Spite** — the damaging player always sacrifices when
//!   they have that many permanents (the punisher's first affordable
//!   option), never choosing the life loss.
//! - **Boneyard Parley** — the up-to-five creature cards are an untargeted
//!   pick (the first five in seat order).
//! - **Chainer, Nightmare Adept** — the permission names one creature card
//!   (the first in your graveyard) as it resolves.
//! - **Hedonist's Trove** — the exiled cards stay playable after the Trove
//!   leaves, and the one-spell-a-turn cap isn't enforced.
//! - **K'rrik, Son of Yawgmoth** — life for {B} covers spells only, not
//!   activation costs.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{combat_partner_punisher, etb, on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, phyrexian, r, x};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, ty: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![ty], effect, ..Default::default() }
}

/// `def` with madness `cost` (CR 702.35) added to its keywords.
fn madness(def: CardDefinition, mana: ManaCost) -> CardDefinition {
    let mut keywords = def.keywords.clone();
    keywords.push(Keyword::Madness(mana));
    CardDefinition { keywords, ..def }
}

fn with_filter(t: TriggeredAbility, p: Predicate) -> TriggeredAbility {
    TriggeredAbility { event: t.event.with_filter(p), effect: t.effect }
}

fn hellbent() -> Predicate {
    Predicate::HellbentActive { who: PlayerRef::You }
}

/// "Whenever you discard a [filter] card" — a cycled card is discarded too.
fn on_you_discard(filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

fn zombie(tapped: bool) -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        tapped,
        ..Default::default()
    }
}

fn nonblack() -> R {
    R::Not(Box::new(R::HasColor(Color::Black)))
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

/// Anje Falkenrath — a hasty rummager that untaps whenever you discard a
/// card with madness.
pub fn anje_falkenrath() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: draw(Value::ONE),
            ..Default::default()
        }],
        triggered_abilities: vec![on_you_discard(
            R::HasMadness,
            Effect::Untap { what: Selector::This, up_to: None },
        )],
        ..legendary(creature(
            "Anje Falkenrath",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Vampire],
            1,
            3,
        ))
    }
}

/// Aeon Engine — enters tapped; tap and exile it to reverse the turn order.
pub fn aeon_engine() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::ReverseTurnOrder,
            ..Default::default()
        }],
        ..spell("Aeon Engine", cost(&[generic(5)]), CardType::Artifact, Effect::Noop)
    }
}

/// Alchemist's Greeting — 4 damage to a creature; madness {1}{R}.
pub fn alchemists_greeting() -> CardDefinition {
    madness(
        spell(
            "Alchemist's Greeting",
            cost(&[generic(4), r()]),
            CardType::Sorcery,
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
        ),
        cost(&[generic(1), r()]),
    )
}

/// Archfiend of Spite — whoever damages it loses that much life unless they
/// sacrifice that many permanents; madness {3}{B}{B}.
/// Residual: they always sacrifice when they can.
pub fn archfiend_of_spite() -> CardDefinition {
    let damager = || Selector::Player(PlayerRef::LastDamagerControllerOf(Box::new(Selector::This)));
    madness(
        CardDefinition {
            keywords: vec![Keyword::Flying],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource).dealt_by(R::ControlledByOpponent),
                effect: Effect::Punisher {
                    chooser: damager(),
                    options: vec![Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::You),
                        count: Value::TriggerEventAmount,
                        filter: R::Permanent,
                    }],
                    otherwise: Box::new(Effect::LoseLife { who: damager(), amount: Value::TriggerEventAmount }),
                },
            }],
            ..creature("Archfiend of Spite", cost(&[generic(5), b(), b()]), vec![CreatureType::Demon], 6, 6)
        },
        cost(&[generic(3), b(), b()]),
    )
}

/// Asylum Visitor — each upkeep of a player with an empty hand, you draw and
/// lose 1 life; madness {1}{B}.
pub fn asylum_visitor() -> CardDefinition {
    madness(
        CardDefinition {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                    .with_filter(Predicate::HellbentActive { who: PlayerRef::ActivePlayer }),
                effect: Effect::Seq(vec![
                    draw(Value::ONE),
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                ]),
            }],
            ..creature(
                "Asylum Visitor",
                cost(&[generic(1), b()]),
                vec![CreatureType::Vampire, CreatureType::Wizard],
                3,
                1,
            )
        },
        cost(&[generic(1), b()]),
    )
}

/// Bloodhall Priest — entering or attacking with an empty hand, 2 damage to
/// any target; madness {1}{B}{R}.
pub fn bloodhall_priest() -> CardDefinition {
    let ping = || Effect::DealDamage { to: target_any(), amount: Value::Const(2) };
    madness(
        CardDefinition {
            triggered_abilities: vec![
                with_filter(etb(ping()), hellbent()),
                with_filter(on_attack(ping()), hellbent()),
            ],
            ..creature(
                "Bloodhall Priest",
                cost(&[generic(2), b(), r()]),
                vec![CreatureType::Vampire, CreatureType::Cleric],
                4,
                4,
            )
        },
        cost(&[generic(1), b(), r()]),
    )
}

/// Bone Miser — a discarded creature makes a Zombie, a land {B}{B}, anything
/// else a card.
pub fn bone_miser() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_you_discard(
                R::Creature,
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(zombie(false)) },
            ),
            on_you_discard(
                R::Land,
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Black, Color::Black]) },
            ),
            on_you_discard(R::Noncreature.and(R::Nonland), draw(Value::ONE)),
        ],
        ..creature(
            "Bone Miser",
            cost(&[generic(4), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Boneyard Parley — exile up to five creature cards from graveyards; an
/// opponent splits them and the pile you pick comes back under your control.
/// Residual: the five are an untargeted pick.
pub fn boneyard_parley() -> CardDefinition {
    spell(
        "Boneyard Parley",
        cost(&[generic(5), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: Zone::Graveyard,
                        filter: R::Creature,
                    }),
                    count: Box::new(Value::Const(5)),
                },
                to: ZoneDest::Exile,
            },
            Effect::SeparateIntoPiles {
                what: Selector::LastMoved,
                splitter: PlayerRef::OpponentOf(Box::new(PlayerRef::You)),
                chooser: PlayerRef::You,
                chosen: Box::new(Effect::Move {
                    what: Selector::SeparatedPile { chosen: true },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                other: Box::new(Effect::Move { what: Selector::SeparatedPile { chosen: false }, to: ZoneDest::Graveyard }),
            },
        ]),
    )
}

/// Call to the Netherworld — a black creature card back to hand; madness {0}.
pub fn call_to_the_netherworld() -> CardDefinition {
    madness(
        spell(
            "Call to the Netherworld",
            cost(&[b()]),
            CardType::Sorcery,
            Effect::Move {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black)).and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ),
        cost(&[]),
    )
}

/// Chainer, Nightmare Adept — discard to cast a creature from your graveyard
/// this turn; creatures you didn't cast from hand get haste.
/// Residual: the permission names one card as it resolves.
pub fn chainer_nightmare_adept() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            once_per_turn: true,
            effect: Effect::GrantMayPlay {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Creature,
                    }),
                    count: Box::new(Value::ONE),
                },
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken).and(R::Not(Box::new(R::WasCastFromHand))),
                },
            ),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Haste,
                duration: Duration::UntilNextTurn,
            },
        }],
        ..legendary(creature(
            "Chainer, Nightmare Adept",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Human, CreatureType::Minion],
            3,
            2,
        ))
    }
}

/// Curse of Fool's Wisdom — each card the enchanted player draws drains 2;
/// madness {3}{B}.
pub fn curse_of_fools_wisdom() -> CardDefinition {
    madness(
        CardDefinition {
            subtypes: Subtypes {
                enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
                ..Default::default()
            },
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::EnchantedBySource),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EnchantedPlayer), amount: Value::Const(2) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            }],
            ..spell(
                "Curse of Fool's Wisdom",
                cost(&[generic(4), b(), b()]),
                CardType::Enchantment,
                Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
            )
        },
        cost(&[generic(3), b()]),
    )
}

/// Dark Withering — destroy a nonblack creature; madness {B}.
pub fn dark_withering() -> CardDefinition {
    madness(
        spell(
            "Dark Withering",
            cost(&[generic(4), b(), b()]),
            CardType::Instant,
            Effect::Destroy { what: target_filtered(R::Creature.and(nonblack())) },
        ),
        cost(&[b()]),
    )
}

/// Faith of the Devoted — each cycle or discard may pay {1} to drain 2.
pub fn faith_of_the_devoted() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_you_discard(
            R::Any,
            Effect::MayPay {
                description: "Pay {1}: each opponent loses 2 life and you gain 2 life?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ])),
                else_: None,
            },
        )],
        ..spell("Faith of the Devoted", cost(&[generic(2), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Flayer of the Hatebound — undying; a creature entering from your
/// graveyard deals damage equal to its power to any target.
pub fn flayer_of_the_hatebound() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Undying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::All(
                vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                    Predicate::TriggerSourceEnteredFromGraveyard,
                ],
            )),
            effect: Effect::DealDamageEqualToPower { source: Selector::TriggerSource, target: target_any() },
        }],
        ..creature("Flayer of the Hatebound", cost(&[generic(5), r()]), vec![CreatureType::Devil], 4, 2)
    }
}

/// From Under the Floorboards — three tapped Zombies and 3 life, or X of
/// each for its madness {X}{B}{B}.
pub fn from_under_the_floorboards() -> CardDefinition {
    let rise = |n: Value| {
        Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: n.clone(), definition: Arc::new(zombie(true)) },
            Effect::GainLife { who: Selector::You, amount: n },
        ])
    };
    madness(
        spell(
            "From Under the Floorboards",
            cost(&[generic(3), b(), b()]),
            CardType::Sorcery,
            Effect::If {
                cond: Predicate::SpellWasMadness,
                then: Box::new(rise(Value::XFromCost)),
                else_: Box::new(rise(Value::Const(3))),
            },
        ),
        cost(&[x(), b(), b()]),
    )
}

/// Gorgon Recluse — destroys the nonblack creatures it blocks or is blocked
/// by at end of combat; madness {B}{B}.
pub fn gorgon_recluse() -> CardDefinition {
    madness(
        CardDefinition {
            triggered_abilities: combat_partner_punisher(R::Creature.and(nonblack())),
            ..creature("Gorgon Recluse", cost(&[generic(3), b(), b()]), vec![CreatureType::Gorgon], 2, 4)
        },
        cost(&[b(), b()]),
    )
}

/// Grave Scrabbler — cast for madness, it returns a creature card from a
/// graveyard to its owner's hand.
pub fn grave_scrabbler() -> CardDefinition {
    madness(
        CardDefinition {
            triggered_abilities: vec![with_filter(
                etb(Effect::MayDo {
                    description: "Return target creature card from a graveyard to its owner's hand?".into(),
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Creature.from_any_graveyard()),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                }),
                Predicate::SpellWasMadness,
            )],
            ..creature("Grave Scrabbler", cost(&[generic(3), b()]), vec![CreatureType::Zombie], 2, 2)
        },
        cost(&[generic(1), b()]),
    )
}

/// Greven, Predator Captain — +X/+0 for the life you lost this turn; its
/// attack may sacrifice a creature to draw its power and lose its toughness.
pub fn greven_predator_captain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Greven gets +X/+0, where X is the amount of life you've lost this turn.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::LifeLostThisTurn(PlayerRef::You),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice another creature to draw cards equal to its power?".into(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::Seq(vec![
                draw(Value::SacrificedPower),
                Effect::LoseLife { who: Selector::You, amount: Value::SacrificedToughness },
            ])),
            else_: None,
        })],
        ..legendary(creature(
            "Greven, Predator Captain",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Warrior],
            5,
            5,
        ))
    }
}

/// Hedonist's Trove — exile an opponent's graveyard and play from it.
/// Residual: no one-spell-a-turn cap; playable after the Trove leaves.
pub fn hedonists_trove() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
            Effect::GrantMayPlay {
                what: Selector::ExiledThisResolution { filter: R::Any },
                duration: crate::card::MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ]))],
        ..spell("Hedonist's Trove", cost(&[generic(5), b(), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// K'rrik, Son of Yawgmoth — lifelink; {B} in your spells' costs may be paid
/// with 2 life; each black spell you cast grows it.
/// Residual: spells only.
pub fn krrik_son_of_yawgmoth() -> CardDefinition {
    let pb = phyrexian(Color::Black);
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "For each {B} in a cost, you may pay 2 life rather than pay that mana.",
            effect: StaticEffect::PhyrexianPipsForAllSpells { color: Color::Black },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::Black))),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legendary(creature(
            "K'rrik, Son of Yawgmoth",
            ManaCost { symbols: vec![crate::mana::ManaSymbol::Generic(4), pb, pb, pb] },
            vec![CreatureType::Phyrexian, CreatureType::Horror, CreatureType::Minion],
            2,
            2,
        ))
    }
}

/// Malevolent Whispers — threaten with +2/+0; madness {3}{R}.
pub fn malevolent_whispers() -> CardDefinition {
    madness(
        spell(
            "Malevolent Whispers",
            cost(&[generic(3), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ]),
        ),
        cost(&[generic(3), r()]),
    )
}

/// Nightshade Assassin — reveal X black cards to give a creature -X/-X;
/// madness {1}{B}.
pub fn nightshade_assassin() -> CardDefinition {
    let x_neg = Value::Diff(Box::new(Value::ZERO), Box::new(Value::CardsRevealedThisEffect));
    madness(
        CardDefinition {
            keywords: vec![Keyword::FirstStrike],
            triggered_abilities: vec![etb(Effect::RevealAnyNumberFromHand {
                filter: R::HasColor(Color::Black),
                then: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: x_neg.clone(),
                    toughness: x_neg,
                    duration: Duration::EndOfTurn,
                }),
            })],
            ..creature(
                "Nightshade Assassin",
                cost(&[generic(2), b(), b()]),
                vec![CreatureType::Human, CreatureType::Assassin],
                2,
                1,
            )
        },
        cost(&[generic(1), b()]),
    )
}

/// Sanitarium Skeleton — {2}{B}: back to hand from your graveyard.
pub fn sanitarium_skeleton() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature("Sanitarium Skeleton", cost(&[b()]), vec![CreatureType::Skeleton], 1, 2)
    }
}

/// Skyfire Phoenix — returns from your graveyard when you cast your
/// commander.
pub fn skyfire_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander.and(R::OwnedByYou) },
            ),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..creature("Skyfire Phoenix", cost(&[generic(2), r(), r()]), vec![CreatureType::Phoenix], 3, 3)
    }
}
