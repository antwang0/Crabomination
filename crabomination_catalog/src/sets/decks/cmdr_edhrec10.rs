//! Commander: the one-card EDHREC gaps of EDHREC's top-1000 commanders, tenth
//! file (`cmdr_edhrec9` is the ninth) — each card completes at least one
//! EDHREC average deck the 2026-10-12 rescan found one card short. Tests in
//! `tests/recent_b/cmdr_edhrec10.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType,
    LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{colored_animation, etb, target_any, target_filtered, tokens_gain_until_eot};
use crate::effect::{ActivatedAbility, Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost};
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

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(who: PlayerRef, n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who, count: n, definition: Arc::new(def) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

fn yours() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn search_basic(to: ZoneDest) -> Effect {
    Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to }
}

/// Yargle, Glutton of Urborg — {4}{B} 9/3 legendary Frog Spirit.
pub fn yargle_glutton_of_urborg() -> CardDefinition {
    legend("Yargle, Glutton of Urborg", cost(&[generic(4), b()]), vec![CreatureType::Frog, CreatureType::Spirit], 9, 3)
}

/// Thistledown Liege — {1}{W/U}{W/U}{W/U} 1/3 Kithkin Knight with flash.
/// Other white creatures you control get +1/+1; other blue ones too.
pub fn thistledown_liege() -> CardDefinition {
    let lord = |c: Color, description: &'static str| {
        static_ab(
            description,
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(yours().and(R::HasColor(c)).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        )
    };
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![
            lord(Color::White, "Other white creatures you control get +1/+1."),
            lord(Color::Blue, "Other blue creatures you control get +1/+1."),
        ],
        ..creature(
            "Thistledown Liege",
            cost(&[generic(1), wu(), wu(), wu()]),
            vec![CreatureType::Kithkin, CreatureType::Knight],
            1,
            3,
        )
    }
}

/// Geist of Saint Thalia — {1}{U} 1/2 legendary flying Spirit Cleric.
/// Noncreature spells you cast cost {1} less.
pub fn geist_of_saint_thalia() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab(
            "Noncreature spells you cast cost {1} less to cast.",
            StaticEffect::CostReduction { filter: R::Noncreature, amount: 1 },
        )],
        ..legend("Geist of Saint Thalia", cost(&[generic(1), u()]), vec![CreatureType::Spirit, CreatureType::Cleric], 1, 2)
    }
}

/// Mass Calcify — {5}{W}{W} sorcery: destroy all nonwhite creatures.
pub fn mass_calcify() -> CardDefinition {
    spell(
        "Mass Calcify",
        cost(&[generic(5), w(), w()]),
        CardType::Sorcery,
        Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasColor(Color::White))))),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
    )
}

/// Union of the Third Path — {2}{W} instant: draw a card, then gain life
/// equal to the cards in your hand.
pub fn union_of_the_third_path() -> CardDefinition {
    spell(
        "Union of the Third Path",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![draw(1), Effect::GainLife { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You) }]),
    )
}

/// Warlord's Fury — {R} sorcery: creatures you control gain first strike
/// until end of turn. Draw a card.
pub fn warlords_fury() -> CardDefinition {
    spell(
        "Warlord's Fury",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::EachPermanent(yours()),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Pirate's Pillage — {3}{R} sorcery; discard a card as an additional cost.
/// Draw two cards and create two Treasures.
pub fn pirates_pillage() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::Discard { count: 1, filter: None }],
        ..spell(
            "Pirate's Pillage",
            cost(&[generic(3), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                draw(2),
                make(PlayerRef::You, Value::Const(2), crabomination_base::tokens::treasure_token()),
            ]),
        )
    }
}

/// Sheltering Word — {1}{G} instant: target creature you control gains
/// hexproof until end of turn; you gain life equal to its toughness.
pub fn sheltering_word() -> CardDefinition {
    spell(
        "Sheltering Word",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GrantKeyword { what: target_filtered(yours()), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
            Effect::GainLife { who: Selector::You, amount: Value::ToughnessOf(Box::new(Selector::Target(0))) },
        ]),
    )
}

/// Sultai Ascendancy — {B}{G}{U} enchantment: surveil 2 each upkeep.
pub fn sultai_ascendancy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![at_your(TurnStep::Upkeep, Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) })],
        ..spell("Sultai Ascendancy", cost(&[b(), g(), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Selfless Savior — {W} 1/1 Dog. Sacrifice it: another target creature you
/// control gains indestructible until end of turn.
pub fn selfless_savior() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(yours().and(R::OtherThanSource)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Selfless Savior", cost(&[w()]), vec![CreatureType::Dog], 1, 1)
    }
}

/// Ravenous Robots — {1}{R} 2/1 artifact Robot. Casting an artifact spell
/// makes a 1/1 colorless Robot artifact creature; {R}, {T}: creature tokens
/// you control gain haste until end of turn.
pub fn ravenous_robots() -> CardDefinition {
    let robot = TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..token("Robot", 1, 1, vec![], vec![CreatureType::Robot])
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Artifact)),
            effect: make(PlayerRef::You, Value::ONE, robot),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(yours().and(R::IsToken)),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Ravenous Robots", cost(&[generic(1), r()]), vec![CreatureType::Robot], 2, 1)
    }
}

/// Magister Sphinx — {4}{W}{U}{B} 5/5 flying artifact Sphinx. When it
/// enters, target player's life total becomes 10.
pub fn magister_sphinx() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::SetLifeTotal { who: target_filtered(R::Player), amount: Value::Const(10) })],
        ..creature("Magister Sphinx", cost(&[generic(4), w(), u(), b()]), vec![CreatureType::Sphinx], 5, 5)
    }
}

/// Polluted Bonds — {3}{B}{B} enchantment. Whenever a land an opponent
/// controls enters, that player loses 2 life and you gain 2 life.
pub fn polluted_bonds() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                to: Selector::You,
                amount: Value::Const(2),
            },
        }],
        ..spell("Polluted Bonds", cost(&[generic(3), b(), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Troop of Ponies — {2} 2/1 Horse. {2}, {T}, sacrifice it: search for up to
/// two basic lands, one onto the battlefield tapped and the other to hand.
pub fn troop_of_ponies() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                search_basic(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true }),
                search_basic(ZoneDest::Hand(PlayerRef::You)),
            ]),
            ..Default::default()
        }],
        ..creature("Troop of Ponies", cost(&[generic(2)]), vec![CreatureType::Horse], 2, 1)
    }
}

/// Scrollshift — {2}{W} instant: exile up to one target artifact, creature,
/// or enchantment you control, then return it under its owner's control.
/// Draw a card.
pub fn scrollshift() -> CardDefinition {
    let what = R::Artifact.or(R::Creature).or(R::Enchantment).and(R::ControlledByYou);
    spell(
        "Scrollshift",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileAndReturnToOwner { what: target_filtered(what) }),
            },
            draw(1),
        ]),
    )
}

/// Dragon Turtle — {1}{U}{U} 3/5 flash Dragon Turtle. Drag Below: when it
/// enters, tap it and up to one target creature an opponent controls; they
/// don't untap during their controllers' next untap steps.
pub fn dragon_turtle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: Selector::This },
            Effect::SkipNextUntap { what: Selector::This },
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                    Effect::SkipNextUntap { what: Selector::Target(0) },
                ])),
            },
        ]))],
        ..creature("Dragon Turtle", cost(&[generic(1), u(), u()]), vec![CreatureType::Dragon, CreatureType::Turtle], 3, 5)
    }
}

/// Eiganjo Uprising — {X}{R}{W} sorcery: X 2/2 white vigilance Samurai that
/// gain menace and haste until end of turn; each opponent creates X minus one.
pub fn eiganjo_uprising() -> CardDefinition {
    let samurai = || TokenDefinition {
        keywords: vec![Keyword::Vigilance],
        ..token("Samurai", 2, 2, vec![Color::White], vec![CreatureType::Samurai])
    };
    spell(
        "Eiganjo Uprising",
        cost(&[x(), r(), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            tokens_gain_until_eot(make(PlayerRef::You, Value::XFromCost, samurai()), &[Keyword::Menace, Keyword::Haste]),
            make(
                PlayerRef::EachOpponent,
                Value::Diff(Box::new(Value::XFromCost), Box::new(Value::ONE)),
                samurai(),
            ),
        ]),
    )
}

// ── Planeswalkers ───────────────────────────────────────────────────────────

fn walker(
    name: &'static str,
    mana: ManaCost,
    kind: PlaneswalkerSubtype,
    loyalty: u32,
    abilities: Vec<LoyaltyAbility>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![kind], ..Default::default() },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

fn loyalty(cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() }
}

fn emblem(name: &'static str, triggered: Vec<TriggeredAbility>, statics: Vec<StaticAbility>) -> Effect {
    Effect::CreateEmblem { who: PlayerRef::You, name: name.into(), triggered, statics }
}

fn lands_of(kind: LandType) -> R {
    R::Land.and(R::HasLandType(kind)).and(R::ControlledByYou)
}

/// "Lands of `kind` you control have '{T}: `ability`'" — an emblem's grant.
fn land_grant(description: &'static str, kind: LandType, effect: Effect) -> StaticAbility {
    static_ab(
        description,
        StaticEffect::GrantActivatedAbility {
            applies_to: Selector::EachPermanent(lands_of(kind)),
            ability: ActivatedAbility { tap_cost: true, effect, ..Default::default() },
            condition: None,
        },
    )
}

/// Liliana of the Dark Realms — {2}{B}{B}, loyalty 3. +1: a Swamp card to
/// hand. −3: target creature gets +X/+X or −X/−X (X = your Swamps). −6: your
/// Swamps tap for {B}{B}{B}{B}. Residual: the −3's sign follows the target's
/// controller (+X on yours, −X on anyone else's) rather than a free choice.
pub fn liliana_of_the_dark_realms() -> CardDefinition {
    let swamps = || Value::PermanentCountControlledByMatching(PlayerRef::You, lands_of(LandType::Swamp));
    let pump = |sign: bool| Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: if sign { swamps() } else { Value::Diff(Box::new(Value::Const(0)), Box::new(swamps())) },
        toughness: if sign { swamps() } else { Value::Diff(Box::new(Value::Const(0)), Box::new(swamps())) },
        duration: Duration::EndOfTurn,
    };
    walker(
        "Liliana of the Dark Realms",
        cost(&[generic(2), b(), b()]),
        PlaneswalkerSubtype::Liliana,
        3,
        vec![
            loyalty(1, Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Swamp),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            loyalty(-3, Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::ControlledByYou },
                then: Box::new(pump(true)),
                else_: Box::new(pump(false)),
            }),
            loyalty(-6, emblem("Liliana of the Dark Realms", vec![], vec![land_grant(
                "Swamps you control have '{T}: Add {B}{B}{B}{B}.'",
                LandType::Swamp,
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Black, Value::Const(4)) },
            )])),
        ],
    )
}

/// Koth of the Hammer — {2}{R}{R}, loyalty 3. +1: untap target Mountain; it
/// becomes a 4/4 red Elemental until end of turn. −2: {R} per Mountain you
/// control. −5: your Mountains gain "{T}: 1 damage to any target".
pub fn koth_of_the_hammer() -> CardDefinition {
    let mountain = R::Land.and(R::HasLandType(LandType::Mountain));
    walker(
        "Koth of the Hammer",
        cost(&[generic(2), r(), r()]),
        PlaneswalkerSubtype::Koth,
        3,
        vec![
            loyalty(1, Effect::Seq(vec![
                Effect::Untap { what: target_filtered(mountain), up_to: None },
                colored_animation(
                    Effect::BecomeCreature {
                        what: Selector::Target(0),
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        creature_types: vec![CreatureType::Elemental],
                        keywords: vec![],
                        duration: Duration::EndOfTurn,
                    },
                    &[Color::Red],
                ),
            ])),
            loyalty(-2, Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Red,
                    Value::PermanentCountControlledByMatching(PlayerRef::You, lands_of(LandType::Mountain)),
                ),
            }),
            loyalty(-5, emblem("Koth of the Hammer", vec![], vec![land_grant(
                "Mountains you control have '{T}: This land deals 1 damage to any target.'",
                LandType::Mountain,
                Effect::DealDamage { to: target_any(), amount: Value::ONE },
            )])),
        ],
    )
}

/// Basri Ket — {1}{W}{W}, loyalty 3. +1: a +1/+1 counter and indestructible
/// on up to one target creature. −2: this turn, each attacking nontoken
/// creature brings a tapped and attacking 1/1 Soldier (one trigger per
/// attacker — the count matches "that many"). −6: a combat-step emblem.
pub fn basri_ket() -> CardDefinition {
    let soldier = || token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier]);
    walker(
        "Basri Ket",
        cost(&[generic(1), w(), w()]),
        PlaneswalkerSubtype::Basri,
        3,
        vec![
            loyalty(1, Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
            loyalty(-2, Effect::OnMatchingAttacksThisTurn {
                filter: R::Creature.and(R::Not(Box::new(R::IsToken))).and(R::ControlledByYou),
                body: Box::new(Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(soldier()),
                    cleanup: Default::default(),
                    defender: None,
                }),
            }),
            loyalty(-6, emblem(
                "Basri Ket",
                vec![at_your(TurnStep::BeginCombat, Effect::Seq(vec![
                    make(PlayerRef::You, Value::ONE, soldier()),
                    Effect::AddCounter {
                        what: Selector::EachPermanent(yours()),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ]))],
                vec![],
            )),
        ],
    )
}

/// Teferi, Temporal Pilgrim — {3}{U}{U}, loyalty 4. Each card you draw adds a
/// loyalty counter. 0: draw. −2: a 2/2 vigilance Spirit that grows as you
/// draw. −12: target opponent shuffles each nonland permanent they control
/// into its owner's library. Residual: the −12's "chooses a permanent they
/// control and returns it to its owner's hand" first is not modelled.
pub fn teferi_temporal_pilgrim() -> CardDefinition {
    let on_draw = |effect: Effect| TriggeredAbility { event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl), effect };
    let spirit = TokenDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_draw(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..token("Spirit", 2, 2, vec![Color::Blue], vec![CreatureType::Spirit])
    };
    CardDefinition {
        triggered_abilities: vec![on_draw(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Loyalty,
            amount: Value::ONE,
        })],
        ..walker(
            "Teferi, Temporal Pilgrim",
            cost(&[generic(3), u(), u()]),
            PlaneswalkerSubtype::Teferi,
            4,
            vec![
                loyalty(0, draw(1)),
                loyalty(-2, make(PlayerRef::You, Value::ONE, spirit)),
                loyalty(-12, Effect::TargetPlayerThen {
                    filter: R::OpponentPlayer,
                    then: Box::new(Effect::Move {
                        what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Permanent.and(R::Not(Box::new(R::Land))) },
                        to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Shuffled },
                    }),
                }),
            ],
        )
    }
}
