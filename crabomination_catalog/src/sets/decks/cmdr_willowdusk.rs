//! Commander: the cards the **Witherbloom Witchcraft** precon (C21,
//! Willowdusk, Essence Seer) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_willowdusk.rs`.
//!
//! Residuals (each also on its card):
//! - **Revival Experiment** — the engine picks the cards: the highest mana
//!   value per type, a multi-typed card counting for the first type it fills.
//! - **Suffer the Past** — the cards are chosen as it resolves, not targeted.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, x, Color, ManaCost};

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

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

fn token(name: &str, ct: CreatureType, colors: Vec<Color>, pt: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: pt,
        toughness: pt,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn food() -> Arc<TokenDefinition> {
    Arc::new(crabomination_base::tokens::food_token())
}

fn gained() -> Value {
    Value::LifeGainedThisTurn(PlayerRef::You)
}

fn end_step_yours() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
}

fn if_gained_life() -> Predicate {
    Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE }
}

fn whenever_you_gain_life(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl), effect }
}

fn nontoken_of_yours_dies(scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::CreatureDied, scope)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken })
}

/// Willowdusk, Essence Seer — {1}, {T}: +1/+1 counters on another target
/// creature equal to the greater of the life you gained and lost this turn.
/// Sorcery speed.
pub fn willowdusk_essence_seer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Max(Box::new(gained()), Box::new(Value::LifeLostThisTurn(PlayerRef::You))),
            },
            ..Default::default()
        }],
        ..creature(
            "Willowdusk, Essence Seer",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Dryad, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Blight Mound — attacking Pests get +1/+0 and menace; a nontoken creature
/// of yours dying makes a Pest.
pub fn blight_mound() -> CardDefinition {
    let attacking_pests = || {
        Selector::EachPermanent(
            R::HasCreatureType(CreatureType::Pest).and(R::IsAttacking).and(R::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Blight Mound",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Attacking Pests you control get +1/+0.",
                effect: StaticEffect::PumpPT { applies_to: attacking_pests(), power: 1, toughness: 0 },
            },
            StaticAbility {
                description: "Attacking Pests you control have menace.",
                effect: StaticEffect::GrantKeyword { applies_to: attacking_pests(), keyword: Keyword::Menace },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: nontoken_of_yours_dies(EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::stx_pest_token()),
            },
        }],
        ..Default::default()
    }
}

/// Druidic Satchel — {2}, {T}: reveal the top card; a creature makes a
/// Saproling, a land goes onto the battlefield, anything else gains 2 life.
pub fn druidic_satchel() -> CardDefinition {
    let top = Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    let saproling = token("Saproling", CreatureType::Saproling, vec![Color::Green], 1, vec![]);
    CardDefinition {
        name: "Druidic Satchel",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::RevealTopThenIf {
                who: PlayerRef::You,
                filter: R::Creature,
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(saproling),
                }),
                else_: Some(Box::new(Effect::RevealTopThenIf {
                    who: PlayerRef::You,
                    filter: R::Land,
                    then: Box::new(Effect::Move {
                        what: top,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Some(Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) })),
                })),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Essence Pulse — gain 2 life, then each creature gets -X/-X, X the life you
/// gained this turn.
pub fn essence_pulse() -> CardDefinition {
    let minus = Value::Negate(Box::new(gained()));
    spell(
        "Essence Pulse",
        cost(&[generic(3), b()]),
        false,
        Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: minus.clone(),
                toughness: minus,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Ezzaroot Channeler — reach; your creature spells cost {X} less, X the life
/// you gained this turn; {T}: gain 2 life.
pub fn ezzaroot_channeler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast cost {X} less, X the life you gained this turn.",
            effect: StaticEffect::CostReductionByValue { filter: R::Creature, amount: gained() },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature(
            "Ezzaroot Channeler",
            cost(&[generic(5), g()]),
            vec![CreatureType::Treefolk, CreatureType::Druid],
            4,
            6,
        )
    }
}

/// Gift of Paradise — enchant land; enters: gain 3 life; enchanted land taps
/// for two mana of any one color.
pub fn gift_of_paradise() -> CardDefinition {
    CardDefinition {
        name: "Gift of Paradise",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Add two mana of any one color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyOneColor(Value::Const(2)),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Gluttonous Troll — trample; enters with a Food per opponent; {1}{G},
/// sacrifice another nonland permanent: +2/+2 until end of turn.
pub fn gluttonous_troll() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::OpponentCount,
            definition: food(),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_other_filter: Some((R::Nonland, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Gluttonous Troll", cost(&[generic(2), b(), g()]), vec![CreatureType::Troll], 3, 3)
    }
}

/// Gyome, Master Chef — trample; your end step makes a Food per nontoken
/// creature that entered under your control this turn; {1}, sacrifice a
/// Food: target creature gains indestructible and is tapped.
pub fn gyome_master_chef() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: end_step_yours(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::NontokenCreaturesEnteredThisTurn(PlayerRef::You),
                definition: food(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Gyome, Master Chef",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Troll, CreatureType::Warlock],
            5,
            3,
        )
    }
}

/// Marshland Bloodcaster — flying; {1}{B}, {T}: the next spell you cast this
/// turn may be paid for with life equal to its mana value.
pub fn marshland_bloodcaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::NextSpellThisTurnMayCostLife { who: PlayerRef::You },
            ..Default::default()
        }],
        ..creature(
            "Marshland Bloodcaster",
            cost(&[generic(4), b()]),
            vec![CreatureType::Vampire, CreatureType::Warlock],
            3,
            5,
        )
    }
}

/// Paradise Plume — as it enters, choose a color; a spell of that color may
/// gain you 1 life; taps for the chosen color.
pub fn paradise_plume() -> CardDefinition {
    CardDefinition {
        name: "Paradise Plume",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::HasChosenColorOfSource)),
            effect: Effect::MayDo {
                description: "Gain 1 life?".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Revival Experiment — for each permanent type, return up to one card of
/// that type from your graveyard; lose 3 life per card; exile it. Residual:
/// the engine picks the highest mana value per type.
pub fn revival_experiment() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Revival Experiment",
            cost(&[generic(4), b(), g()]),
            false,
            Effect::ReturnOnePerPermanentType { life_per_card: 3 },
        )
    }
}

/// Sapling of Colfenor — indestructible; attacking reveals the top card: a
/// creature card gains you its toughness, costs you its power, and goes to
/// your hand.
pub fn sapling_of_colfenor() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    let bg = || hybrid(Color::Black, Color::Green);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![on_attack(Effect::RevealTopThenIf {
            who: PlayerRef::You,
            filter: R::Creature,
            then: Box::new(Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ToughnessOf(Box::new(top())) },
                Effect::LoseLife { who: Selector::You, amount: Value::PowerOf(Box::new(top())) },
                Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) },
            ])),
            else_: None,
        })],
        ..creature(
            "Sapling of Colfenor",
            cost(&[generic(3), bg(), bg()]),
            vec![CreatureType::Treefolk, CreatureType::Shaman],
            2,
            5,
        )
    }
}

/// Suffer the Past — exile X cards from target player's graveyard; they lose
/// 1 life and you gain 1 per card. Residual: the cards are chosen as it
/// resolves rather than targeted.
pub fn suffer_the_past() -> CardDefinition {
    let exiled = || Value::CountOf(Box::new(Selector::LastMoved));
    spell(
        "Suffer the Past",
        cost(&[x(), b()]),
        true,
        Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(Effect::Seq(vec![
                Effect::ExileUpToNFromGraveyards {
                    count: Value::XFromCost,
                    of: Some(PlayerRef::Target(0)),
                    single: true,
                },
                Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: exiled() },
                Effect::GainLife { who: Selector::You, amount: exiled() },
            ])),
        },
    )
}

/// Tivash, Gloom Summoner — lifelink; your end step, if you gained life, you
/// may pay that much life for an X/X flying Demon.
pub fn tivash_gloom_summoner() -> CardDefinition {
    let demon = TokenDefinition {
        dynamic_pt: Some((gained(), gained())),
        ..token("Demon", CreatureType::Demon, vec![Color::Black], 0, vec![Keyword::Flying])
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: end_step_yours().with_filter(if_gained_life()),
            effect: Effect::MayPayLife {
                description: "Pay life equal to the life you gained this turn for a Demon?".into(),
                amount: gained(),
                body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(demon) }),
                else_: None,
            },
        }],
        ..creature(
            "Tivash, Gloom Summoner",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            4,
            4,
        )
    }
}

/// Trudge Garden — whenever you gain life, you may pay {2} for a 4/4
/// trampling Fungus Beast.
pub fn trudge_garden() -> CardDefinition {
    let beast = TokenDefinition {
        subtypes: Subtypes { creature_types: vec![CreatureType::Fungus, CreatureType::Beast], ..Default::default() },
        ..token("Fungus Beast", CreatureType::Fungus, vec![Color::Green], 4, vec![Keyword::Trample])
    };
    CardDefinition {
        name: "Trudge Garden",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![whenever_you_gain_life(Effect::MayPay {
            description: "Pay {2} for a 4/4 Fungus Beast?".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(beast) }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Veinwitch Coven — menace; whenever you gain life, you may pay {B} to
/// return a creature card from your graveyard to your hand.
pub fn veinwitch_coven() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![whenever_you_gain_life(Effect::MayPay {
            description: "Pay {B} to return a creature card?".into(),
            mana_cost: cost(&[b()]),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: None,
        })],
        ..creature(
            "Veinwitch Coven",
            cost(&[generic(2), b()]),
            vec![CreatureType::Vampire, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Yedora, Grave Gardener — another nontoken creature of yours dying may come
/// back face down as a Forest land.
pub fn yedora_grave_gardener() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: nontoken_of_yours_dies(EventScope::AnotherOfYours),
            effect: Effect::MayDo {
                description: "Return it face down as a Forest?".into(),
                body: Box::new(Effect::ReturnFaceDownAsForest { what: Selector::TriggerSource }),
            },
        }],
        ..creature(
            "Yedora, Grave Gardener",
            cost(&[generic(4), g()]),
            vec![CreatureType::Treefolk, CreatureType::Druid],
            5,
            5,
        )
    }
}
