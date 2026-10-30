//! Commander: the cards two-card EDHREC gaps share, eighth file
//! (`cmdr_edhrec7` is the seventh) — Preston, Kiora, Coppercoat Vanguard and
//! the other cards that complete more than one EDHREC top-600 deck, with
//! their partners. Tests in `tests/recent_b/cmdr_edhrec8.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector,
    SpellSubtype, StaticAbility, Zone, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{ActivatedAbility, DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost, SpendRestriction};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

fn blink(filter: R) -> Effect {
    Effect::OptionalTargets { min: 0, body: Box::new(Effect::ExileAndReturnToOwner { what: target_filtered(filter) }) }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

/// Slimefoot and Squee — {B}{R}{G} 3/3 Fungus Goblin. Entering or attacking
/// makes a Saproling; from your graveyard, {1}{B}{R}{G} and a Saproling
/// return it and up to one other creature card, as a sorcery.
pub fn slimefoot_and_squee() -> CardDefinition {
    let saproling = || make(1, token("Saproling", 1, 1, Color::Green, vec![CreatureType::Saproling]));
    CardDefinition {
        triggered_abilities: vec![etb(saproling()), on_attack(saproling())],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            sorcery_speed: true,
            mana_cost: cost(&[generic(1), b(), r(), g()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Saproling), 1)),
            effect: Effect::Seq(vec![
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Creature.from_your_graveyard().and(R::OtherThanSource)),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                },
                Effect::Move { what: Selector::This, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false } },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Slimefoot and Squee",
            cost(&[b(), r(), g()]),
            vec![CreatureType::Fungus, CreatureType::Goblin],
            3,
            3,
        )
    }
}

/// Sarkhan, Fireblood — {1}{R}{R} planeswalker, loyalty 3. +1 rummage; +1
/// two mana in any colors for Dragon spells; −7 four 5/5 flying Dragons.
pub fn sarkhan_fireblood() -> CardDefinition {
    let dragon = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Dragon", 5, 5, Color::Red, vec![CreatureType::Dragon])
    };
    CardDefinition {
        name: "Sarkhan, Fireblood",
        cost: cost(&[generic(1), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Sarkhan], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::MayDo {
                    description: "Discard a card to draw a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                        crate::effect::shortcut::if_discarded(draw(1)),
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyColors(Value::Const(2))),
                        SpendRestriction::CreatureOfType(CreatureType::Dragon),
                    ),
                },
                ..Default::default()
            },
            LoyaltyAbility { loyalty_cost: -7, effect: make(4, dragon), ..Default::default() },
        ],
        ..Default::default()
    }
}

/// Aurelia, Exemplar of Justice — {2}{R}{W} 2/5 flying Angel, mentor. Each of
/// your combats: up to one creature of yours gets +2/+0, trample if red and
/// vigilance if white.
pub fn aurelia_exemplar_of_justice() -> CardDefinition {
    let if_color = |color, keyword| Effect::If {
        cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::HasColor(color) },
        then: Box::new(Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn }),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            crate::effect::shortcut::mentor(),
            at_your(
                TurnStep::BeginCombat,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::PumpPT {
                            what: target_filtered(mine()),
                            power: Value::Const(2),
                            toughness: Value::Const(0),
                            duration: Duration::EndOfTurn,
                        },
                        if_color(Color::Red, Keyword::Trample),
                        if_color(Color::White, Keyword::Vigilance),
                    ])),
                },
            ),
        ],
        ..legend("Aurelia, Exemplar of Justice", cost(&[generic(2), r(), w()]), vec![CreatureType::Angel], 2, 5)
    }
}

/// Supernatural Stamina — {B} Instant. +2/+0 and "when this dies, return it
/// to the battlefield tapped" until end of turn.
pub fn supernatural_stamina() -> CardDefinition {
    spell(
        "Supernatural Stamina",
        cost(&[b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                    effect: Effect::ReturnSelfTapped,
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Agent of the Shadow Thieves — {1}{B} Background. Your commander creatures
/// attacking the life leader grow and gain deathtouch and indestructible.
pub fn agent_of_the_shadow_thieves() -> CardDefinition {
    let richest = Predicate::Not(Box::new(Predicate::ForAnyPlayer {
        who: PlayerRef::EachOpponent,
        pred: Box::new(Predicate::Not(Box::new(Predicate::ValueAtLeast(
            Value::LifeOf(PlayerRef::DefendingPlayer),
            Value::LifeOf(PlayerRef::Triggerer),
        )))),
    }));
    let grant = |keyword| Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn };
    let ability = TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::All(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::IsAttackingOpponentPlayer },
            richest,
        ])),
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            grant(Keyword::Deathtouch),
            grant(Keyword::Indestructible),
        ]),
    };
    CardDefinition {
        name: "Agent of the Shadow Thieves",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![static_ab(
            "Commander creatures you own have \"Whenever this creature attacks a player, if no opponent has more life than that player, put a +1/+1 counter on this creature. It gains deathtouch and indestructible until end of turn.\"",
            StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
                ability: Box::new(ability),
            },
        )],
        ..Default::default()
    }
}

/// Distinguished Conjurer — {1}{W} 1/2 Human Wizard. Another creature of
/// yours entering gains you 1; {4}{W}, {T}: flicker another creature of yours.
pub fn distinguished_conjurer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            tap_cost: true,
            effect: Effect::ExileAndReturnToOwner { what: target_filtered(mine().and(R::OtherThanSource)) },
            ..Default::default()
        }],
        ..creature("Distinguished Conjurer", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 2)
    }
}

/// Preston, the Vanisher — {3}{W} 2/5 Rabbit Wizard. Another nontoken
/// creature of yours entering without being cast is copied as a 0/1 white
/// Illusion; {1}{W}, sacrifice five Illusions: exile a nonland permanent.
pub fn preston_the_vanisher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::IsToken.negate()) },
                Predicate::Not(Box::new(Predicate::TriggerSourceWasCast)),
            ])),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: Some((0, 1)),
                    override_colors: Some(vec![Color::White]),
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                },
                // CR 707.9b — "it's a 0/1 white Illusion" replaces its
                // creature types.
                Effect::AmendCopiableValues {
                    what: Selector::LastCreatedToken,
                    name: None,
                    set_creature_types: Some(vec![CreatureType::Illusion]),
                    add_creature_types: vec![],
                    legendary: false,
                    add_colors: vec![],
                    set_card_types: None,
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Illusion), 5)),
            effect: Effect::Exile { what: target_filtered(R::Permanent.and(R::Nonland)) },
            ..Default::default()
        }],
        ..legend("Preston, the Vanisher", cost(&[generic(3), w()]), vec![CreatureType::Rabbit, CreatureType::Wizard], 2, 5)
    }
}

/// Lae'zel's Acrobatics — {3}{W} Instant. Exile your nontoken creatures, then
/// roll a d20: 1-9 returns those cards at the next end step; 10-20 returns
/// them now, then exiles them again until the next end step.
pub fn laezels_acrobatics() -> CardDefinition {
    let back = || ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false };
    let at_end_step = Effect::ForEach {
        selector: Selector::LastMoved,
        body: Box::new(Effect::DelayUntilWithCapture {
            kind: DelayedTriggerKind::NextEndStep,
            capture: Selector::TriggerSource,
            body: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
            }),
        }),
    };
    spell(
        "Lae'zel's Acrobatics",
        cost(&[generic(3), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::Move { what: Selector::EachPermanent(mine().and(R::IsToken.negate())), to: ZoneDest::Exile },
            Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::Const(0),
                reroll_at_most: 0,
                ignore_lowest: 0,
                on_doubles: None,
                results: vec![
                    (1, 9, at_end_step),
                    (10, 20, Effect::Seq(vec![
                        Effect::Move { what: Selector::LastMoved, to: back() },
                        Effect::ExileReturnToOwnerNextEndStep { what: Selector::LastMoved, tapped: false },
                    ])),
                ],
            },
        ]),
    )
}

/// Icewind Stalwart — {3}{W} 3/3 Tiefling Warrior. ETB flickers up to one
/// non-Warrior creature of yours.
pub fn icewind_stalwart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(blink(mine().and(R::HasCreatureType(CreatureType::Warrior).negate())))],
        ..creature("Icewind Stalwart", cost(&[generic(3), w()]), vec![CreatureType::Tiefling, CreatureType::Warrior], 3, 3)
    }
}

/// Plagon, Lord of the Beach — {2}{U} 0/3 Starfish Wizard. ETB draws per
/// creature of yours with toughness greater than power; {W/U}: a creature
/// of yours assigns combat damage by toughness this turn.
pub fn plagon_lord_of_the_beach() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::PermanentCountControlledByMatching(PlayerRef::You, R::Creature.and(R::ToughnessGreaterThanPower)),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[hybrid(Color::White, Color::Blue)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(mine()),
                keyword: Keyword::AssignsCombatDamageByToughness,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Plagon, Lord of the Beach",
            cost(&[generic(2), u()]),
            vec![CreatureType::Starfish, CreatureType::Wizard],
            0,
            3,
        )
    }
}

/// Animation Module — {1} Artifact. +1/+1 counters on a permanent of yours
/// may buy a Servo for {1}; {3}, {T}: another counter of a kind a target
/// permanent or player already has.
pub fn animation_module() -> CardDefinition {
    let servo = TokenDefinition {
        name: "Servo".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Servo], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Animation Module",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} for a Servo?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(make(1, servo)),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::AddCounterOfPresentKind { what: target_filtered(R::Permanent.or(R::Player)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chandra, Acolyte of Flame — {1}{R}{R} planeswalker, loyalty 4. 0: a
/// loyalty counter on each red planeswalker of yours; 0: two hasty 1/1
/// Elementals, sacrificed at the next end step; −2: recast a cheap instant or
/// sorcery from your graveyard, exiled afterwards.
pub fn chandra_acolyte_of_flame() -> CardDefinition {
    let elemental = TokenDefinition {
        keywords: vec![Keyword::Haste],
        ..token("Elemental", 1, 1, Color::Red, vec![CreatureType::Elemental])
    };
    CardDefinition {
        name: "Chandra, Acolyte of Flame",
        cost: cost(&[generic(1), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Chandra], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(R::Planeswalker.and(R::HasColor(Color::Red)).and(R::ControlledByYou)),
                    kind: CounterType::Loyalty,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    make(2, elemental),
                    Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedTokens },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                // Cast as the ability resolves, its costs paid; exiled after.
                effect: Effect::CastImmediatePayingExtra {
                    what: target_filtered(
                        R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery))
                            .and(R::ManaValueAtMost(3))
                            .from_your_graveyard(),
                    ),
                    source_zone: Zone::Graveyard,
                    extra: ManaCost::default(),
                    exile_after: true,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kiora, Sovereign of the Deep — {3}{G}{U} 4/5 vigilance, ward {3}. A
/// Kraken, Leviathan, Octopus or Serpent spell cast from your hand digs its
/// mana value deep for a cheaper free spell.
pub fn kiora_sovereign_of_the_deep() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(Selector::TriggerSource));
    let deep = R::HasCreatureType(CreatureType::Kraken)
        .or(R::HasCreatureType(CreatureType::Leviathan))
        .or(R::HasCreatureType(CreatureType::Octopus))
        .or(R::HasCreatureType(CreatureType::Serpent));
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Ward(WardCost::generic(3))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::All(vec![Predicate::CastFromHand, Predicate::CastSpellMatches(deep)])),
            effect: Effect::RevealTopMayCastOneFree {
                count: mv(),
                max_mv: Value::Diff(Box::new(mv()), Box::new(Value::ONE)),
                filter: None,
            },
        }],
        ..legend(
            "Kiora, Sovereign of the Deep",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Noble],
            4,
            5,
        )
    }
}

/// Verdant Kraken — {4}{G}{G}{G} 6/6 Plant Kraken. Each player's upkeep makes
/// you a 3/3 Forest Tentacle land creature.
pub fn verdant_kraken() -> CardDefinition {
    let tentacle = TokenDefinition {
        name: "Forest Tentacle".into(),
        power: 3,
        toughness: 3,
        colors: vec![Color::Green],
        card_types: vec![CardType::Land, CardType::Creature],
        // A printed basic land type grants no intrinsic mana ability here
        // (`intrinsic_land_mana_abilities_with` skips printed types), so the
        // token's "{T}: Add {G}" is spelled out, as on the Forest Dryads.
        activated_abilities: vec![crate::sets::tap_add(Color::Green)],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Tentacle],
            land_types: vec![LandType::Forest],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: make(1, tentacle),
        }],
        ..creature(
            "Verdant Kraken",
            cost(&[generic(4), g(), g(), g()]),
            vec![CreatureType::Plant, CreatureType::Kraken],
            6,
            6,
        )
    }
}

/// Engulf the Shore — {3}{U} Instant. Bounce every creature with toughness at
/// most your Island count.
pub fn engulf_the_shore() -> CardDefinition {
    spell(
        "Engulf the Shore",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::Move {
            what: Selector::EachPermanent(R::Creature.and(R::ToughnessAtMostYourCount(Box::new(
                R::HasLandType(LandType::Island).and(R::ControlledByYou),
            )))),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Coppercoat Vanguard — {1}{W} 2/2 Human Soldier. Your other Humans get
/// +1/+0 and ward {1}.
pub fn coppercoat_vanguard() -> CardDefinition {
    let humans = || Selector::EachPermanent(R::HasCreatureType(CreatureType::Human).and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        static_abilities: vec![
            static_ab("Each other Human you control gets +1/+0.", StaticEffect::PumpPT { applies_to: humans(), power: 1, toughness: 0 }),
            static_ab(
                "Each other Human you control has ward {1}.",
                StaticEffect::GrantKeyword { applies_to: humans(), keyword: Keyword::Ward(WardCost::generic(1)) },
            ),
        ],
        ..creature("Coppercoat Vanguard", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Court of Ardenvale — {2}{W}{W} Enchantment. ETB monarch; your upkeep
/// returns a permanent card with mana value 3 or less — to the battlefield
/// while you're the monarch, else to hand.
pub fn court_of_ardenvale() -> CardDefinition {
    let card = || target_filtered(R::Permanent.and(R::ManaValueAtMost(3)).from_your_graveyard());
    CardDefinition {
        name: "Court of Ardenvale",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            at_your(
                TurnStep::Upkeep,
                Effect::If {
                    cond: Predicate::IsMonarch { who: PlayerRef::You },
                    then: Box::new(Effect::Move {
                        what: card(),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Box::new(Effect::Move { what: card(), to: ZoneDest::Hand(PlayerRef::You) }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Torens, Fist of the Angels — {1}{G}{W} 2/2 Human Cleric, training. Each
/// creature spell you cast makes a 1/1 Human Soldier with training.
pub fn torens_fist_of_the_angels() -> CardDefinition {
    let soldier = TokenDefinition {
        colors: vec![Color::Green, Color::White],
        triggered_abilities: vec![crate::effect::shortcut::training()],
        ..token("Human Soldier", 1, 1, Color::Green, vec![CreatureType::Human, CreatureType::Soldier])
    };
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::training(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(R::Creature)),
                effect: make(1, soldier),
            },
        ],
        ..legend(
            "Torens, Fist of the Angels",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Picnic Ruiner // Stolen Goodies — {1}{R} 2/2 Goblin Rogue; attacking
/// beside a power-4 creature it gains double strike. Adventure {3}{G}:
/// distribute three +1/+1 counters among your creatures.
pub fn picnic_ruiner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::SelectorExists(
                Selector::EachPermanent(mine().and(R::PowerAtLeast(4))),
            )),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Stolen Goodies",
            cost: cost(&[generic(3), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DistributeCounters {
                total: Value::Const(3),
                counter: CounterType::PlusOnePlusOne,
                filter: mine(),
                max_targets: 3,
            },
        })),
        ..creature("Picnic Ruiner", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Rogue], 2, 2)
    }
}

/// Scrappy Bruiser — {3}{R} 3/4 Raccoon Warrior. Attacking, up to one
/// attacker gets +2/+0 and trample and returns to hand at end of combat.
pub fn scrappy_bruiser() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::IsAttacking)),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                Effect::AtEndOfCombat {
                    body: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    }),
                },
            ])),
        })],
        ..creature("Scrappy Bruiser", cost(&[generic(3), r()]), vec![CreatureType::Raccoon, CreatureType::Warrior], 3, 4)
    }
}

/// Solid Ground — {3}{G} Enchantment. ETB earthbend 3; +1/+1 counters on
/// your permanents come one extra.
pub fn solid_ground() -> CardDefinition {
    CardDefinition {
        name: "Solid Ground",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Earthbend { n: Value::Const(3) })],
        static_abilities: vec![static_ab(
            "If one or more +1/+1 counters would be put on a permanent you control, that many plus one +1/+1 counters are put on it instead.",
            StaticEffect::ExtraPlusOneCountersMatching { filter: R::Permanent },
        )],
        ..Default::default()
    }
}

/// Cracked Earth Technique — {4}{G} Sorcery — Lesson. Earthbend 3 twice and
/// gain 3 life.
pub fn cracked_earth_technique() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        ..spell(
            "Cracked Earth Technique",
            cost(&[generic(4), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Earthbend { n: Value::Const(3) },
                Effect::Earthbend { n: Value::Const(3) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ]),
        )
    }
}

/// Evereth, Viceroy of Plunder — {2}{B} 2/2 flying Vampire Soldier. Sorcery
/// speed, sacrifice another creature or artifact: a +1/+1 counter, and
/// lifelink if it was a Treasure; dying may pay {1}{B/R} to hit each
/// opponent for its power.
pub fn evereth_viceroy_of_plunder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            sac_other_filter: Some((R::Creature.or(R::Artifact).and(R::OtherThanSource), 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::SacrificedCard,
                        filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Treasure),
                    },
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Lifelink,
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {1}{B/R} to have Evereth hit each opponent for its power?".into(),
                mana_cost: cost(&[generic(1), hybrid(Color::Black, Color::Red)]),
                body: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                }),
                else_: None,
            },
        }],
        ..legend(
            "Evereth, Viceroy of Plunder",
            cost(&[generic(2), b()]),
            vec![CreatureType::Vampire, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Bramble Familiar // Fetch Quest — {1}{G} 2/2 Elemental Raccoon. {T}: {G};
/// {1}{G}, {T}, discard a card: back to hand. Adventure {5}{G}{G}: mill seven,
/// then put a creature, enchantment or land card milled this way onto the
/// battlefield.
pub fn bramble_familiar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![Color::Green], Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
                ..Default::default()
            },
        ],
        adventure: Some(Box::new(Adventure {
            name: "Fetch Quest",
            cost: cost(&[generic(5), g(), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(7) },
                Effect::MoveChosen {
                    from: Selector::LastMoved,
                    filter: Some(R::Creature.or(R::HasCardType(CardType::Enchantment)).or(R::Land)),
                    count: Value::ONE,
                    up_to: false,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ]),
        })),
        ..creature("Bramble Familiar", cost(&[generic(1), g()]), vec![CreatureType::Elemental, CreatureType::Raccoon], 2, 2)
    }
}

/// Talent of the Telepath — {2}{U}{U} Sorcery. A target opponent reveals the
/// top seven; you may cast an instant or sorcery among them free (two with
/// spell mastery — two or more instants and sorceries in your graveyard),
/// and the rest go to their graveyard. The cast count is read off your
/// spells-cast tally around the casts, so the six or five left on top are
/// exactly the revealed rest.
pub fn talent_of_the_telepath() -> CardDefinition {
    let is = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    let cast_now = || Value::Diff(Box::new(Value::SpellsCastThisTurn(PlayerRef::You)), Box::new(Value::XFromCost));
    spell(
        "Talent of the Telepath",
        cost(&[generic(2), u(), u()]),
        CardType::Sorcery,
        Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::WithX {
                x: Value::SpellsCastThisTurn(PlayerRef::You),
                body: Box::new(Effect::Seq(vec![
                    Effect::CastAnyOrderWithoutPaying {
                        what: Selector::TopOfLibrary { who: PlayerRef::Target(0), count: Value::Const(7) },
                        source_zone: Zone::Library,
                        filter: Some(is()),
                        cap: Some(Value::IfAtLeast {
                            value: Box::new(Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: is() }),
                            threshold: 2,
                            then: Box::new(Value::Const(2)),
                            else_: Box::new(Value::ONE),
                        }),
                        total_mana_value: None,
                        exile_after: false,
                    },
                    Effect::Move {
                        what: Selector::TopOfLibrary {
                            who: PlayerRef::Target(0),
                            count: Value::Diff(Box::new(Value::Const(7)), Box::new(cast_now())),
                        },
                        to: ZoneDest::Graveyard,
                    },
                ])),
            }),
        },
    )
}

/// Xanathar, Guild Kingpin — {4}{U}{B} 5/6 Beholder. Each upkeep of yours, a
/// target opponent can't cast spells this turn, and you may look at and play
/// the top card of their library, spending mana as though any color.
pub fn xanathar_guild_kingpin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![at_your(
            TurnStep::Upkeep,
            Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::Seq(vec![
                    Effect::PlayerCantCastMatchingThisTurn { who: PlayerRef::Target(0), filter: R::Any },
                    Effect::MayPlayLibraryTopOfThisTurn { who: PlayerRef::Target(0) },
                ])),
            },
        )],
        ..legend(
            "Xanathar, Guild Kingpin",
            cost(&[generic(4), u(), b()]),
            vec![CreatureType::Beholder],
            5,
            6,
        )
    }
}

/// Sen Triplets — {2}{W}{U}{B} 3/3 legendary artifact creature, Human Wizard.
/// Each upkeep of yours, a target opponent can't cast spells or activate
/// abilities this turn and plays with their hand revealed, and you may play
/// lands and cast spells from that hand this turn.
pub fn sen_triplets() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![at_your(
            TurnStep::Upkeep,
            Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::Seq(vec![
                    Effect::PlayerCantCastMatchingThisTurn { who: PlayerRef::Target(0), filter: R::Any },
                    Effect::PlayerCantActivateNonManaAbilitiesThisTurn { who: PlayerRef::Target(0) },
                    Effect::MayPlayHandOfThisTurn { who: PlayerRef::Target(0) },
                ])),
            },
        )],
        ..legend(
            "Sen Triplets",
            cost(&[generic(2), w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}
