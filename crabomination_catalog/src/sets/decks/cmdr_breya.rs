//! Commander: the cards the **Invent Superiority** precon (C16, Breya,
//! Etherium Shaper) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_breya.rs`.
//!
//! Residuals (each also on its card):
//! - **Armory Automaton** — attaches every Equipment you control; "any number
//!   of target Equipment" also reaches other players' Equipment.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, generic, r, u, w, Color, ManaCost};
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

fn artifact_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn basic_landcycling() -> Keyword {
    Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))
}

fn attacks(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect }
}

fn token(name: &str, types: Vec<CardType>, subtype: CreatureType, color: Option<Color>, pt: (i32, i32), kws: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: pt.0,
        toughness: pt.1,
        card_types: types,
        colors: color.into_iter().collect(),
        subtypes: Subtypes { creature_types: vec![subtype], ..Default::default() },
        keywords: kws,
        ..Default::default()
    })
}

fn artifacts_you_control() -> Value {
    Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))))
}

/// Breya, Etherium Shaper — enters with two Thopters; {2}, sacrifice two
/// artifacts: 3 to a player or planeswalker, -4/-4, or 5 life.
pub fn breya_etherium_shaper() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: token(
                "Thopter",
                vec![CardType::Artifact, CardType::Creature],
                CreatureType::Thopter,
                Some(Color::Blue),
                (1, 1),
                vec![Keyword::Flying],
            ),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Artifact, 2)),
            sac_other_may_be_source: true,
            effect: Effect::ChooseMode(vec![
                Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::Const(3) },
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-4),
                    toughness: Value::Const(-4),
                    duration: Duration::EndOfTurn,
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
            ]),
            ..Default::default()
        }],
        ..artifact_creature("Breya, Etherium Shaper", cost(&[w(), u(), b(), r()]), vec![CreatureType::Human], 4, 4)
    }
}

/// Ancient Excavation — draw as many cards as are in your hand, then discard
/// that many; basic landcycling {2}.
pub fn ancient_excavation() -> CardDefinition {
    CardDefinition {
        keywords: vec![basic_landcycling()],
        ..spell(
            "Ancient Excavation",
            cost(&[generic(2), u(), b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You) },
                Effect::Discard { who: Selector::You, amount: Value::CardsDrawnThisEffect, random: false },
            ]),
        )
    }
}

/// Armory Automaton — entering or attacking, it may take on every Equipment
/// you control (other players' Equipment is out of reach).
pub fn armory_automaton() -> CardDefinition {
    let suit_up = || Effect::MayDo {
        description: "Attach your Equipment to Armory Automaton?".into(),
        body: Box::new(Effect::Attach {
            what: Selector::EachPermanent(R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou)),
            to: Selector::This,
        }),
    };
    CardDefinition {
        triggered_abilities: vec![etb(suit_up()), attacks(suit_up())],
        ..artifact_creature("Armory Automaton", cost(&[generic(3)]), vec![CreatureType::Construct], 2, 2)
    }
}

/// Bruse Tarl, Boorish Herder — partner; entering or attacking, a creature
/// of yours gains double strike and lifelink this turn.
pub fn bruse_tarl_boorish_herder() -> CardDefinition {
    let rally = || {
        let it = || target_filtered(R::Creature.and(R::ControlledByYou));
        Effect::Seq(vec![
            Effect::GrantKeyword { what: it(), keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
        ])
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![etb(rally()), attacks(rally())],
        ..creature(
            "Bruse Tarl, Boorish Herder",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Human, CreatureType::Ally],
            3,
            3,
        )
    }
}

/// Curse of Vengeance — a spite counter per spell the cursed player casts;
/// when they lose the game, you gain that much life and draw that many.
pub fn curse_of_vengeance() -> CardDefinition {
    CardDefinition {
        name: "Curse of Vengeance",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                    .with_filter(Predicate::SamePlayer(PlayerRef::Triggerer, PlayerRef::EnchantedPlayer)),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Spite, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EnchantedPlayerLeftGame, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
                    Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Ethersworn Adjudicator — flying; {1}{W}{B}, {T}: destroy target creature
/// or enchantment; {2}{U}: untap it.
pub fn ethersworn_adjudicator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w(), b()]),
                tap_cost: true,
                effect: Effect::Destroy { what: target_filtered(R::Creature.or(R::Enchantment)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
        ],
        ..artifact_creature(
            "Ethersworn Adjudicator",
            cost(&[generic(4), u()]),
            vec![CreatureType::Vedalken, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Faerie Artisans — flying; an opponent's nontoken creature entering makes
/// you an artifact token copy of it, and the older copies go.
pub fn faerie_artisans() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::NotToken) },
            ),
            // "Then exile all other tokens created with this creature" — the
            // older ones, exiled first so the new copy isn't among them.
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::TokensCreatedBySource, to: ZoneDest::Exile },
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    extra_card_types: vec![CardType::Artifact],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            ]),
        }],
        ..creature(
            "Faerie Artisans",
            cost(&[generic(3), u()]),
            vec![CreatureType::Faerie, CreatureType::Artificer],
            2,
            2,
        )
    }
}

/// Filigree Angel — flying; entering gains 3 life per artifact you control.
pub fn filigree_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(Box::new(Value::Const(3)), Box::new(artifacts_you_control())),
        })],
        ..artifact_creature("Filigree Angel", cost(&[generic(5), w(), w(), u()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Grave Upheaval — a creature card from any graveyard joins you with haste;
/// basic landcycling {2}.
pub fn grave_upheaval() -> CardDefinition {
    CardDefinition {
        keywords: vec![basic_landcycling()],
        ..spell(
            "Grave Upheaval",
            cost(&[generic(4), b(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.from_any_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::Permanent },
            ]),
        )
    }
}

/// Grip of Phyresis — steal an Equipment and put it on a new 0/0 Germ.
pub fn grip_of_phyresis() -> CardDefinition {
    spell(
        "Grip of Phyresis",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                to: None,
                duration: Duration::Permanent,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: token(
                    "Phyrexian Germ",
                    vec![CardType::Creature],
                    CreatureType::Phyrexian,
                    Some(Color::Black),
                    (0, 0),
                    vec![],
                ),
            },
            Effect::Attach { what: Selector::Target(0), to: Selector::LastCreatedToken },
        ]),
    )
}

/// Magus of the Will — {2}{B}, {T}, exile it: this turn you may play cards
/// from your graveyard, and cards bound there are exiled instead.
pub fn magus_of_the_will() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![Effect::PlayFromGraveyardThisTurn, Effect::ExileYourGraveyardBoundThisTurn]),
            ..Default::default()
        }],
        ..creature(
            "Magus of the Will",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Migratory Route — four 1/1 white Birds with flying; basic landcycling {2}.
pub fn migratory_route() -> CardDefinition {
    CardDefinition {
        keywords: vec![basic_landcycling()],
        ..spell(
            "Migratory Route",
            cost(&[generic(3), w(), u()]),
            CardType::Sorcery,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(4),
                definition: token(
                    "Bird",
                    vec![CardType::Creature],
                    CreatureType::Bird,
                    Some(Color::White),
                    (1, 1),
                    vec![Keyword::Flying],
                ),
            },
        )
    }
}

/// Parting Thoughts — destroy target creature; draw and lose life for each
/// counter on it (counted before it goes).
pub fn parting_thoughts() -> CardDefinition {
    let x = || Value::TotalCountersOn { what: Box::new(Selector::Target(0)) };
    spell(
        "Parting Thoughts",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: x() },
            Effect::LoseLife { who: Selector::You, amount: x() },
            Effect::Destroy { what: target_filtered(R::Creature) },
        ]),
    )
}

/// Sharuum the Hegemon — flying; entering, may return an artifact card from
/// your graveyard to the battlefield.
pub fn sharuum_the_hegemon() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return an artifact card from your graveyard?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Artifact.from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        })],
        ..artifact_creature(
            "Sharuum the Hegemon",
            cost(&[generic(3), w(), u(), b()]),
            vec![CreatureType::Sphinx],
            5,
            5,
        )
    }
}

/// Silas Renn, Seeker Adept — deathtouch, partner; connecting lets you cast
/// an artifact card from your graveyard this turn.
pub fn silas_renn_seeker_adept() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch, Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::GrantMayPlay {
                what: target_filtered(R::Artifact.from_your_graveyard()),
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        }],
        ..artifact_creature(
            "Silas Renn, Seeker Adept",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Human],
            2,
            2,
        )
    }
}

/// Sphinx Summoner — flying; entering, may tutor an artifact creature card.
pub fn sphinx_summoner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for an artifact creature card?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Artifact.and(R::Creature),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..artifact_creature("Sphinx Summoner", cost(&[generic(3), u(), b()]), vec![CreatureType::Sphinx], 3, 3)
    }
}

/// Sydri, Galvanic Genius — {U}: a noncreature artifact becomes a creature
/// with P/T equal to its mana value this turn; {W}{B}: an artifact creature
/// gains deathtouch and lifelink this turn.
pub fn sydri_galvanic_genius() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(Selector::Target(0)));
    let target_artifact_creature = || target_filtered(R::Artifact.and(R::Creature));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::BecomeCreature {
                    what: target_filtered(R::Artifact.and(R::Noncreature)),
                    power: mv(),
                    toughness: mv(),
                    creature_types: vec![],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w(), b()]),
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_artifact_creature(),
                        keyword: Keyword::Deathtouch,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
        ],
        ..creature(
            "Sydri, Galvanic Genius",
            cost(&[w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}
