//! Commander: the one-card EDHREC gaps of EDHREC's top-1000 commanders, tenth
//! file (`cmdr_edhrec9` is the ninth) — each card completes at least one
//! EDHREC average deck the 2026-10-12 rescan found one card short. Tests in
//! `tests/recent_b/cmdr_edhrec10.rs`.

use crate::card::{
    AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, EnchantmentSubtype, EntersAsCopy, EquipBonus, SplitCard,
    SplitHalf, CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType,
    LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    LevelBand, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone, MayPlayDuration,
};
use crate::effect::shortcut::{colored_animation, etb, on_attack, target_any, target_filtered, tokens_gain_until_eot};
use crate::effect::{
    ActivatedAbility, Duration, Effect, LibraryPosition, LookPick, ManaPayload, PlayerRef, VoteOption, VoteTally, ZoneDest,
};
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

// ── Creatures, auras and enchantments ──────────────────────────────────────

/// Treefolk Umbra — {2}{G} Aura. Enchanted creature gets +0/+2 and assigns
/// combat damage equal to its toughness. Umbra armor.
pub fn treefolk_umbra() -> CardDefinition {
    CardDefinition {
        name: "Treefolk Umbra",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::UmbraArmor],
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
        equipped_bonus: Some(EquipBonus {
            toughness: 2,
            keywords: vec![Keyword::AssignsCombatDamageByToughness],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Sigarda's Summons — {4}{W}{W} enchantment. Your creatures with +1/+1
/// counters have base power and toughness 4/4, flying, and are Angels.
pub fn sigardas_summons() -> CardDefinition {
    let countered = || Selector::EachPermanent(yours().and(R::WithCounter(CounterType::PlusOnePlusOne)));
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "Creatures you control with +1/+1 counters on them have base power and toughness 4/4.",
                StaticEffect::SetBasePtForFilter { applies_to: countered(), power: 4, toughness: 4 },
            ),
            static_ab(
                "… have flying …",
                StaticEffect::GrantKeyword { applies_to: countered(), keyword: Keyword::Flying },
            ),
            static_ab(
                "… and are Angels in addition to their other types.",
                StaticEffect::AddCreatureTypeToMatching { applies_to: countered(), creature_type: CreatureType::Angel },
            ),
        ],
        ..spell("Sigarda's Summons", cost(&[generic(4), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Kuldotha Forgemaster — {5} 3/5 artifact Construct. {T}, sacrifice three
/// artifacts: search for an artifact card and put it onto the battlefield.
pub fn kuldotha_forgemaster() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact.and(R::ControlledByYou), 3)),
            sac_other_may_be_source: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Artifact,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Kuldotha Forgemaster", cost(&[generic(5)]), vec![CreatureType::Construct], 3, 5)
    }
}

/// Outlaws' Merriment — {1}{R}{W}{W} enchantment. Each upkeep, a random one
/// of three red-and-white hasty Human tokens.
pub fn outlaws_merriment() -> CardDefinition {
    let human = |name: &str, p: i32, t: i32, kind: CreatureType, extra: Option<Keyword>| TokenDefinition {
        keywords: [Some(Keyword::Haste), extra].into_iter().flatten().collect(),
        ..token(name, p, t, vec![Color::Red, Color::White], vec![CreatureType::Human, kind])
    };
    let rogue = TokenDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage { to: target_any(), amount: Value::ONE })],
        ..human("Human Rogue", 1, 2, CreatureType::Rogue, None)
    };
    CardDefinition {
        triggered_abilities: vec![at_your(TurnStep::Upkeep, Effect::ChooseModeAtRandom(vec![
            make(PlayerRef::You, Value::ONE, human("Human Warrior", 3, 1, CreatureType::Warrior, Some(Keyword::Trample))),
            make(PlayerRef::You, Value::ONE, human("Human Cleric", 2, 1, CreatureType::Cleric, Some(Keyword::Lifelink))),
            make(PlayerRef::You, Value::ONE, rogue),
        ]))],
        ..spell("Outlaws' Merriment", cost(&[generic(1), r(), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Bard, King of Dale — {4}{W}{U} 3/5 legendary Human Noble Archer with
/// reach and vigilance. Draws past the first in each draw step are doubled;
/// tokens created under your control are doubled.
pub fn bard_king_of_dale() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        static_abilities: vec![
            static_ab(
                "If you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.",
                StaticEffect::ControllerDrawsDoubledExceptFirstEachDrawStep,
            ),
            static_ab(
                "If one or more tokens would be created under your control, twice that many of those tokens are created instead.",
                StaticEffect::DoubleTokens,
            ),
        ],
        ..legend(
            "Bard, King of Dale",
            cost(&[generic(4), w(), u()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Archer],
            3,
            5,
        )
    }
}

/// Defiler of Faith — {3}{W}{W} 5/5 vigilance Phyrexian Human. White
/// permanent spells may pay 2 life for {W}; casting one makes a 1/1 Soldier.
pub fn defiler_of_faith() -> CardDefinition {
    let white_permanent = || R::HasColor(Color::White).and(R::PermanentCard);
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![static_ab(
            "White permanent spells may pay 2 life for {W}.",
            StaticEffect::PhyrexianPipForSpells { filter: white_permanent(), color: Color::White },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(white_permanent())),
            effect: make(PlayerRef::You, Value::ONE, token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier])),
        }],
        ..creature("Defiler of Faith", cost(&[generic(3), w(), w()]), vec![CreatureType::Phyrexian, CreatureType::Human], 5, 5)
    }
}

/// Grimgrin, Corpse-Born — {3}{U}{B} 5/5 legendary Zombie Warrior. Enters
/// tapped and doesn't untap in your untap step; sacrifice another creature to
/// untap it with a +1/+1 counter; attacking destroys a creature the defending
/// player controls, then grows it.
pub fn grimgrin_corpse_born() -> CardDefinition {
    let grow = || Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE };
    CardDefinition {
        static_abilities: vec![
            crate::sets::enters_tapped(),
            static_ab(
                "Grimgrin doesn't untap during your untap step.",
                StaticEffect::PreventUntap { applies_to: Selector::This },
            ),
        ],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((yours(), 1)),
            effect: Effect::Seq(vec![Effect::Untap { what: Selector::This, up_to: None }, grow()]),
            ..Default::default()
        }],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)) },
            grow(),
        ]))],
        ..legend(
            "Grimgrin, Corpse-Born",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Zombie, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Inferno of the Star Mounts — {4}{R}{R} 6/6 legendary flying, hasty
/// Dragon that can't be countered. {R}: +1/+0 until end of turn; when its
/// power becomes 20 this way, it deals 20 damage to any target.
pub fn inferno_of_the_star_mounts() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::CantBeCountered],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::Const(0), duration: Duration::EndOfTurn },
                Effect::If {
                    cond: Predicate::ValueEquals(Value::PowerOf(Box::new(Selector::This)), Value::Const(20)),
                    then: Box::new(Effect::ReflexiveTrigger {
                        body: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(20) }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend("Inferno of the Star Mounts", cost(&[generic(4), r(), r()]), vec![CreatureType::Dragon], 6, 6)
    }
}

/// Guiding Hydra — {X}{W} 1/0 Hydra Horror; enters with X +1/+1 counters.
/// At the beginning of combat on your turn you may move one of its counters
/// onto each other creature you control.
pub fn guiding_hydra() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![at_your(TurnStep::BeginCombat, Effect::If {
            cond: Predicate::EntityMatches { what: Selector::This, filter: R::WithCounter(CounterType::PlusOnePlusOne) },
            then: Box::new(Effect::MayDo {
                description: "Remove a +1/+1 counter to put one on each other creature you control?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::RemoveCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::AddCounter {
                        what: Selector::EachPermanent(yours().and(R::OtherThanSource)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ])),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Guiding Hydra", cost(&[x(), w()]), vec![CreatureType::Hydra, CreatureType::Horror], 1, 0)
    }
}

/// Greasefang, Okiba Boss — {1}{W}{B} 4/3 legendary Rat Pilot. At the
/// beginning of combat on your turn, return target Vehicle card from your
/// graveyard to the battlefield; it gains haste and returns to its owner's
/// hand at your next end step.
pub fn greasefang_okiba_boss() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![at_your(TurnStep::BeginCombat, Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Vehicle).and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::ReturnToOwnersHandAtNextEndStep { what: Selector::LastMoved },
        ]))],
        ..legend("Greasefang, Okiba Boss", cost(&[generic(1), w(), b()]), vec![CreatureType::Rat, CreatureType::Pilot], 4, 3)
    }
}

/// Psemilla, Meletian Poet — {2}{W} 1/1 legendary Human Bard. Your first
/// enchantment spell each turn makes a 2/2 white Nymph enchantment creature;
/// at the beginning of each combat, with five or more enchantments, she gets
/// +4/+4 and lifelink until end of turn.
pub fn psemilla_meletian_poet() -> CardDefinition {
    let nymph = TokenDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        ..token("Nymph", 2, 2, vec![Color::White], vec![CreatureType::Nymph])
    };
    let enchantments = Value::PermanentCountControlledByMatching(PlayerRef::You, R::Enchantment);
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                    Predicate::CastSpellMatches(R::Enchantment),
                    Predicate::SpellsOfTypeCastThisTurnAtMost {
                        who: PlayerRef::You,
                        card_type: CardType::Enchantment,
                        n: 1,
                    },
                ])),
                effect: make(PlayerRef::You, Value::ONE, nymph),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer)
                    .with_filter(Predicate::ValueAtLeast(enchantments, Value::Const(5))),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
                ]),
            },
        ],
        ..legend("Psemilla, Meletian Poet", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Bard], 1, 1)
    }
}

/// Feast of the Victorious Dead — {W}{B} enchantment. At your end step, if
/// creatures died this turn, gain that much life and distribute that many
/// +1/+1 counters among your creatures.
pub fn feast_of_the_victorious_dead() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::CreaturesDiedThisTurnTotal },
                Effect::DistributeCountersAmong {
                    what: Selector::EachPermanent(yours()),
                    total: Value::CreaturesDiedThisTurnTotal,
                    counter: CounterType::PlusOnePlusOne,
                },
            ]),
        }],
        ..spell("Feast of the Victorious Dead", cost(&[w(), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Anje, Maid of Dishonor — {2}{B}{R} 4/5 legendary Vampire. Once each turn,
/// Vampires of yours entering make a Blood token; {2}, sacrifice another
/// creature or a Blood token: each opponent loses 2 life and you gain 2.
pub fn anje_maid_of_dishonor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Vampire)),
                })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::blood_token()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((
                R::ControlledByYou.and(R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Blood))),
                1,
            )),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..legend("Anje, Maid of Dishonor", cost(&[generic(2), b(), r()]), vec![CreatureType::Vampire], 4, 5)
    }
}

/// Archghoul of Thraben — {2}{B} 3/2 Zombie Cleric. Whenever it or another
/// Zombie you control dies, look at the top card: a Zombie card goes to your
/// hand (the "may reveal" is always taken), anything else may be binned.
pub fn archghoul_of_thraben() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCreatureType(CreatureType::Zombie),
            }),
            effect: Effect::RevealTopAndDrawIf {
                who: PlayerRef::You,
                reveal_filter: R::HasCreatureType(CreatureType::Zombie),
                may_graveyard_miss: true,
            },
        }],
        ..creature("Archghoul of Thraben", cost(&[generic(2), b()]), vec![CreatureType::Zombie, CreatureType::Cleric], 3, 2)
    }
}

/// Emissary Green — {4}{G} 3/3 legendary Human Advisor. Attacking, each
/// player votes profit or security: two Treasures per profit vote, a +1/+1
/// counter on each of your creatures per security vote.
pub fn emissary_green() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Vote {
            options: vec![
                VoteOption::new(
                    "profit",
                    make(PlayerRef::You, Value::Const(2), crabomination_base::tokens::treasure_token()),
                ),
                VoteOption::new(
                    "security",
                    Effect::AddCounter {
                        what: Selector::EachPermanent(yours()),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ),
            ],
            tally: VoteTally::PerVote,
        })],
        ..legend("Emissary Green", cost(&[generic(4), g()]), vec![CreatureType::Human, CreatureType::Advisor], 3, 3)
    }
}

/// The Millennium Calendar — {1} legendary artifact. A time counter for each
/// permanent you untap in your untap step; {2}, {T}: double them; at 1,000
/// it is sacrificed and each opponent loses 1,000 life.
pub fn the_millennium_calendar() -> CardDefinition {
    let time = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Time };
    CardDefinition {
        name: "The Millennium Calendar",
        cost: cost(&[generic(1)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesUntapped, EventScope::YourControl)
                    .with_filter(Predicate::CurrentStepIs(TurnStep::Untap)),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Time, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::Time), EventScope::SelfSource)
                    .with_filter(Predicate::ValueAtLeast(time(), Value::Const(1000))),
                effect: Effect::Seq(vec![
                    Effect::SacrificePermanent { what: Selector::This },
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1000) },
                ]),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Time, amount: time() },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Confounding Conundrum — {1}{U} enchantment. ETB: draw. Whenever a land an
/// opponent controls enters, if they had another land enter this turn, they
/// return a land they control to its owner's hand.
pub fn confounding_conundrum() -> CardDefinition {
    let them = || PlayerRef::ControllerOf(Box::new(Selector::TriggerSource));
    CardDefinition {
        triggered_abilities: vec![
            etb(draw(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl).with_filter(
                    Predicate::All(vec![
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
                        Predicate::LandsEnteredThisTurnAtLeast { who: them(), at_least: 2 },
                    ]),
                ),
                effect: Effect::PlayerReturnsPermanentsToHand { who: them(), count: Value::ONE, filter: R::Land, up_to: false },
            },
        ],
        ..spell("Confounding Conundrum", cost(&[generic(1), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Danitha, New Benalia's Light — {1}{G}{W} 2/2 legendary Human Knight with
/// vigilance, trample and lifelink. Once during each of your turns you may
/// cast an Aura or Equipment spell from your graveyard.
pub fn danitha_new_benalias_light() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Lifelink],
        static_abilities: vec![static_ab(
            "Once during each of your turns, you may cast an Aura or Equipment spell from your graveyard.",
            StaticEffect::GraveyardCastOncePerTurn {
                filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                    .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                exile_after: false,
                mv_at_most_counters: None,
                enters_tapped: false,
            },
        )],
        ..legend(
            "Danitha, New Benalia's Light",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Astor, Bearer of Blades — {2}{R}{W} 4/4 legendary Human Warrior. ETB:
/// look at the top seven for an Equipment or Vehicle card (rest on the
/// bottom). Your Equipment have equip {1}; your Vehicles have crew 1.
pub fn astor_bearer_of_blades() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(7),
            pick_filter: Some(
                R::HasArtifactSubtype(ArtifactSubtype::Equipment).or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
            ),
            optional: true,
            ..Default::default()
        })))],
        static_abilities: vec![
            static_ab(
                "Equipment you control have equip {1}.",
                StaticEffect::EquipmentYouControlHaveEquip { cost: cost(&[generic(1)]) },
            ),
            static_ab(
                "Vehicles you control have crew 1.",
                StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::HasArtifactSubtype(ArtifactSubtype::Vehicle).and(R::ControlledByYou),
                    ),
                    keyword: Keyword::Crew(1),
                },
            ),
        ],
        ..legend(
            "Astor, Bearer of Blades",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Claim // Fame — {B} sorcery: return target creature card with mana value
/// 2 or less from your graveyard to the battlefield. Fame ({1}{R}, aftermath
/// sorcery): target creature gets +2/+0 and haste until end of turn.
pub fn claim_fame() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(1), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::Const(2),
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
            },
            fuse: false,
            aftermath: true,
        })),
        ..spell(
            "Claim // Fame",
            cost(&[b()]),
            CardType::Sorcery,
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(2))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        )
    }
}

/// Evil Twin — {2}{U}{B} 0/0 Shapeshifter. Enters as a copy of any creature,
/// with "{U}{B}, {T}: Destroy target creature with the same name as this
/// creature."
pub fn evil_twin() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_activated: vec![ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[u(), b()]),
                effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::SharesNameWithSource)) },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..creature("Evil Twin", cost(&[generic(2), u(), b()]), vec![CreatureType::Shapeshifter], 0, 0)
    }
}

// ── Batch 6 ─────────────────────────────────────────────────────────────────

/// The Mind Stone — {1}{W} legendary indestructible Infinity Stone artifact.
/// {T}: Add {W}. {5}{W}, {T}: harness it. ∞ — at your end step, blink up to
/// one other target nonland permanent you control.
pub fn the_mind_stone() -> CardDefinition {
    CardDefinition {
        name: "The Mind Stone",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Infinity, ArtifactSubtype::Stone], ..Default::default() },
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::White]) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(5), w()]),
                effect: Effect::Harness { what: Selector::This },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::SourceIsHarnessed),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileAndReturnToOwner {
                    what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByYou).and(R::OtherThanSource)),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Claim the Kingdom — {1}{G} enchantment — Plan. Landfall: a +1/+1 counter
/// on target creature you control and a plan counter here. The fourth plan
/// counter sacrifices it; when you do, an indestructible counter on target
/// creature you control.
pub fn claim_the_kingdom() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Plan], ..Default::default() },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
                effect: Effect::Seq(vec![
                    Effect::AddCounter { what: target_filtered(yours()), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Plan, amount: Value::ONE },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::Plan), EventScope::SelfSource)
                    .with_filter(Predicate::SourceHasCountersAtLeast { counter: CounterType::Plan, n: 4 }),
                effect: Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(Effect::ReflexiveTrigger {
                            body: Box::new(Effect::AddCounter {
                                what: target_filtered(yours()),
                                kind: CounterType::Indestructible,
                                amount: Value::ONE,
                            }),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..spell("Claim the Kingdom", cost(&[generic(1), g()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Lady Octopus, Inspired Inventor — {U} 0/2 legendary Human Scientist
/// Villain. Your first and second draws each turn add an ingenuity counter;
/// {T}: you may cast an artifact spell from your hand with mana value up to
/// her ingenuity counters without paying its mana cost.
pub fn lady_octopus_inspired_inventor() -> CardDefinition {
    let counter = || Effect::AddCounter { what: Selector::This, kind: CounterType::Ingenuity, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::FirstCardDrawnThisTurn, EventScope::YourControl),
                effect: counter(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
                effect: counter(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::MayCastFromHandFreeMatching {
                filter: R::Artifact,
                max_mv: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Ingenuity },
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..legend(
            "Lady Octopus, Inspired Inventor",
            cost(&[u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Villain],
            0,
            2,
        )
    }
}

/// Lara Croft, Tomb Raider — {G}{U}{R} 3/4 legendary Human Ranger with first
/// strike and reach. Attacking, she exiles up to one target legendary
/// artifact or land card from a graveyard with a discovery counter; you may
/// play a card with a discovery counter from exile this turn. Raid — at end
/// of combat on your turn, if you attacked, a Treasure. Residual: every
/// discovery card is playable that turn, not "a card" (one).
pub fn lara_croft_tomb_raider() -> CardDefinition {
    let relic = R::HasSupertype(Supertype::Legendary).and(R::Artifact.or(R::Land)).and(R::InGraveyard);
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Reach],
        triggered_abilities: vec![
            on_attack(Effect::Seq(vec![
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move { what: target_filtered(relic), to: ZoneDest::Exile },
                        Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Discovery, amount: Value::ONE },
                    ])),
                },
                Effect::GrantMayPlay {
                    what: Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: Zone::Exile,
                        filter: R::WithCounter(CounterType::Discovery),
                    },
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::EndCombat), EventScope::YourControl)
                    .with_filter(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
                effect: make(PlayerRef::You, Value::ONE, crabomination_base::tokens::treasure_token()),
            },
        ],
        ..legend(
            "Lara Croft, Tomb Raider",
            cost(&[g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Ranger],
            3,
            4,
        )
    }
}

/// Widespread Thieving — {2}{R} enchantment, hideaway 5. Casting a
/// multicolored spell makes a Treasure; then you may pay {W}{U}{B}{R}{G} to
/// play the hidden card free.
pub fn widespread_thieving() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Hideaway { count: Value::Const(5) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(R::Multicolored)),
                effect: Effect::Seq(vec![
                    make(PlayerRef::You, Value::ONE, crabomination_base::tokens::treasure_token()),
                    Effect::MayPay {
                        description: "Pay {W}{U}{B}{R}{G} to play the hidden card free?".into(),
                        mana_cost: cost(&[w(), u(), b(), r(), g()]),
                        body: Box::new(Effect::Seq(vec![
                            Effect::PlayLandAmongNow { what: Selector::CardExiledWithSource },
                            Effect::CastWithoutPayingImmediate {
                                reduce_generic: 0,
                                pay_own_cost: false,
                                what: Selector::CardExiledWithSource,
                                source_zone: Zone::Exile,
                                exile_after: false,
                                copy: false,
                            },
                        ])),
                        else_: None,
                    },
                ]),
            },
        ],
        ..spell("Widespread Thieving", cost(&[generic(2), r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// The Watcher in the Water — {3}{U}{U} 9/9 legendary Kraken. Enters tapped
/// with nine stun counters. Drawing during an opponent's turn makes a 1/1
/// blue Tentacle; a Tentacle of yours dying untaps up to one target Kraken
/// and puts a stun counter on up to one target nonland permanent.
pub fn the_watcher_in_the_water() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Stun, Value::Const(9))),
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                    .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
                effect: make(PlayerRef::You, Value::ONE, token("Tentacle", 1, 1, vec![Color::Blue], vec![CreatureType::Tentacle])),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Tentacle),
                }),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Untap {
                            what: Selector::TargetFiltered { slot: 0, filter: R::HasCreatureType(CreatureType::Kraken) },
                            up_to: None,
                        },
                        Effect::AddCounter {
                            what: Selector::TargetFiltered { slot: 1, filter: R::Permanent.and(R::Nonland) },
                            kind: CounterType::Stun,
                            amount: Value::ONE,
                        },
                    ])),
                },
            },
        ],
        ..legend("The Watcher in the Water", cost(&[generic(3), u(), u()]), vec![CreatureType::Kraken], 9, 9)
    }
}

/// Corporeal Projection — {U}{R} sorcery: target creature you control gains
/// myriad until end of turn. Overload {3}{U}{U}{R}{R}: each creature you
/// control does.
pub fn corporeal_projection() -> CardDefinition {
    let grant = |what: Selector| Effect::GrantTriggeredAbility {
        what,
        trigger: Box::new(crate::effect::shortcut::myriad()),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), u(), u(), r(), r()]),
            effect_override: Some(grant(Selector::EachPermanent(yours()))),
            ..Default::default()
        }),
        ..spell("Corporeal Projection", cost(&[u(), r()]), CardType::Sorcery, grant(target_filtered(yours())))
    }
}

/// Ruthless Ripper — {B} 1/1 deathtouch Human Assassin. Morph—reveal a black
/// card in your hand. When it's turned face up, target player loses 2 life.
pub fn ruthless_ripper() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Deathtouch,
            Keyword::MorphCost(Box::new(WardCost::RevealMatching(Box::new(R::HasColor(Color::Black))))),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::LoseLife { who: target_filtered(R::Player), amount: Value::Const(2) },
        }],
        ..creature("Ruthless Ripper", cost(&[b()]), vec![CreatureType::Human, CreatureType::Assassin], 1, 1)
    }
}

/// Edge of Autumn — {1}{G} sorcery: with four or fewer lands, a basic land
/// onto the battlefield tapped. Cycling—Sacrifice a land.
pub fn edge_of_autumn() -> CardDefinition {
    let lands = Value::PermanentCountControlledByMatching(PlayerRef::You, R::Land);
    CardDefinition {
        keywords: vec![Keyword::CyclingSacrifice(Box::new(R::Land))],
        ..spell(
            "Edge of Autumn",
            cost(&[generic(1), g()]),
            CardType::Sorcery,
            Effect::If {
                cond: Predicate::Not(Box::new(Predicate::ValueAtLeast(lands, Value::Const(5)))),
                then: Box::new(search_basic(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true })),
                else_: Box::new(Effect::Noop),
            },
        )
    }
}

// ── The second rescan's one-card gaps ───────────────────────────────────────

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// Trail of Evidence — {2}{U} enchantment: investigate whenever you cast an
/// instant or sorcery spell.
pub fn trail_of_evidence() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(instant_or_sorcery())),
            effect: crate::effect::shortcut::investigate(1),
        }],
        ..spell("Trail of Evidence", cost(&[generic(2), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Sylvok Replica — {3} 1/3 artifact Shaman. {G}, sacrifice it: destroy
/// target artifact or enchantment.
pub fn sylvok_replica() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ..Default::default()
        }],
        ..creature("Sylvok Replica", cost(&[generic(3)]), vec![CreatureType::Shaman], 1, 3)
    }
}

/// Virulent Plague — {2}{B} enchantment: creature tokens get -2/-2.
pub fn virulent_plague() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Creature tokens get -2/-2.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsToken)),
                power: -2,
                toughness: -2,
            },
        )],
        ..spell("Virulent Plague", cost(&[generic(2), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Rescuer Chwinga — {1}{W} 2/2 flash Elemental Spirit. Natural Shelter: when
/// it enters, you may return another permanent you control to its owner's
/// hand.
pub fn rescuer_chwinga() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PlayerReturnsPermanentsToHand {
            who: PlayerRef::You,
            count: Value::ONE,
            filter: R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource),
            up_to: true,
        })],
        ..creature("Rescuer Chwinga", cost(&[generic(1), w()]), vec![CreatureType::Elemental, CreatureType::Spirit], 2, 2)
    }
}

/// Thieves' Tools — {1}{B} Equipment. ETB: a Treasure. The equipped creature
/// can't be blocked while its power is 3 or less. Equip {2}.
pub fn thieves_tools() -> CardDefinition {
    CardDefinition {
        name: "Thieves' Tools",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(make(PlayerRef::You, Value::ONE, crabomination_base::tokens::treasure_token()))],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::UnblockableWhilePowerAtMost(3)], ..Default::default() }),
        ..Default::default()
    }
}

/// Eradicator Valkyrie — {2}{B}{B} 4/3 flying, lifelink Angel Berserker.
/// Boast — {1}{B}, sacrifice a creature: each opponent sacrifices a creature
/// or planeswalker. Residual: "hexproof from planeswalkers" is not modelled.
pub fn eradicator_valkyrie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((yours(), 1)),
            sac_other_may_be_source: true,
            ..crate::effect::shortcut::boast(
                cost(&[generic(1), b()]),
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Creature.or(R::Planeswalker),
                },
            )
        }],
        ..creature(
            "Eradicator Valkyrie",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Angel, CreatureType::Berserker],
            4,
            3,
        )
    }
}

/// Sentinel Tower — {4} artifact. An instant or sorcery spell cast during
/// your turn makes it deal 1 plus the instants and sorceries cast before it
/// this turn to any target. Residual: only your own earlier casts count.
pub fn sentinel_tower() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(instant_or_sorcery()),
                Predicate::IsTurnOf(PlayerRef::You),
            ])),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::Sum(vec![
                    Value::ONE,
                    Value::Diff(
                        Box::new(Value::InstantsOrSorceriesCastThisTurn(PlayerRef::You)),
                        Box::new(Value::ONE),
                    ),
                ]),
            },
        }],
        ..spell("Sentinel Tower", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// Mishra's Self-Replicator — {5} 2/2 artifact Assembly-Worker. Casting a
/// historic spell, you may pay {1} for a token copy of it.
pub fn mishras_self_replicator() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::historic())),
            effect: Effect::MayPay {
                description: "Pay {1} to copy Mishra's Self-Replicator?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                }),
                else_: None,
            },
        }],
        ..creature("Mishra's Self-Replicator", cost(&[generic(5)]), vec![CreatureType::AssemblyWorker], 2, 2)
    }
}

/// Joraga Treespeaker — {G} 1/1 Elf Druid, level up {1}{G}. LEVEL 1-4: 1/2,
/// {T}: Add {G}{G}. LEVEL 5+: 1/4, Elves you control have "{T}: Add {G}{G}."
pub fn joraga_treespeaker() -> CardDefinition {
    let gg = || ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Green]) },
        ..Default::default()
    };
    let level = |n: u32| Predicate::SourceHasCountersAtLeast { counter: CounterType::Level, n };
    CardDefinition {
        level_bands: vec![
            LevelBand { min: 1, max: Some(4), power: 1, toughness: 2, keywords: vec![] },
            LevelBand { min: 5, max: None, power: 1, toughness: 4, keywords: vec![] },
        ],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                sorcery_speed: true,
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Level, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                condition: Some(Predicate::All(vec![level(1), Predicate::Not(Box::new(level(5)))])),
                ..gg()
            },
        ],
        static_abilities: vec![static_ab(
            "LEVEL 5+ — Elves you control have \"{T}: Add {G}{G}.\"",
            StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Elf).and(R::ControlledByYou)),
                ability: gg(),
                condition: Some(level(5)),
            },
        )],
        ..creature("Joraga Treespeaker", cost(&[g()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 1)
    }
}

/// Teferi, Master of Time — {2}{U}{U}, loyalty 3; its loyalty abilities work
/// at instant speed on any turn. +1: loot. −3: target creature you don't
/// control phases out. −10: two extra turns.
pub fn teferi_master_of_time() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "You may activate loyalty abilities of Teferi on any player's turn any time you could cast an instant.",
            StaticEffect::LoyaltyAbilitiesAtInstantSpeed,
        )],
        ..walker(
            "Teferi, Master of Time",
            cost(&[generic(2), u(), u()]),
            PlaneswalkerSubtype::Teferi,
            3,
            vec![
                loyalty(1, Effect::Seq(vec![
                    draw(1),
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ])),
                loyalty(-3, Effect::PhaseOut {
                    what: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
                    until_source_leaves: false,
                }),
                loyalty(-10, Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::Const(2) }),
            ],
        )
    }
}

/// Lupine Prototype — {2} 5/5 artifact Wolf Construct; can't attack or block
/// unless a player has no cards in hand.
pub fn lupine_prototype() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::CantAttackOrBlockUnlessAHandIsEmpty],
        ..creature("Lupine Prototype", cost(&[generic(2)]), vec![CreatureType::Wolf, CreatureType::Construct], 5, 5)
    }
}

/// Majestic Genesis — {6}{G}{G} sorcery. Reveal the top X cards (X = the
/// greatest mana value of a commander you own on the battlefield or in the
/// command zone); put any number of permanent cards among them onto the
/// battlefield, the rest on the bottom.
pub fn majestic_genesis() -> CardDefinition {
    spell(
        "Majestic Genesis",
        cost(&[generic(6), g(), g()]),
        CardType::Sorcery,
        Effect::RevealTopNPutMatchingToBattlefield {
            who: PlayerRef::You,
            count: Value::GreatestCommanderManaValueInPlayOrCommandZone(PlayerRef::You),
            filter: R::PermanentCard,
        },
    )
}

/// Tetzimoc, Primal Death — {4}{B}{B} 6/6 legendary deathtouch Elder
/// Dinosaur. {B}, reveal it from your hand (your turn only): a prey counter on
/// target creature. When it enters, destroy each creature your opponents
/// control with a prey counter on it.
pub fn tetzimoc_primal_death() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            from_hand: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::Prey, amount: Value::ONE },
            ..Default::default()
        }],
        triggered_abilities: vec![etb(Effect::ForEach {
            selector: Selector::EachPermanent(
                R::Creature.and(R::ControlledByOpponent).and(R::WithCounter(CounterType::Prey)),
            ),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        })],
        ..legend(
            "Tetzimoc, Primal Death",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Elder, CreatureType::Dinosaur],
            6,
            6,
        )
    }
}
