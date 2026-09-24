//! Commander: the cards the **Painbow** precon (DMC, Jared Carthalion)
//! needed beyond what the catalog had. Tests in `tests/recent_b/cmdr_jared.rs`.
//!
//! Residuals (each also on its card):
//! - **Primeval Spawn** — it isn't exiled when it would enter without being
//!   cast, and its leave trigger casts one spell of mana value 10 or less
//!   rather than any number with total mana value 10 or less.
//! - **Knight of New Alara** — counts printed colors (a Kavu token is all
//!   colors; a static that paints a creature all colors isn't seen).
//! - **Unite the Coalition** — a repeated mode needs a different target the
//!   bot doesn't always find.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, r, u, w};
use crate::sets::enters_tapped;
use std::sync::Arc;

const WUBRG: [Color; 5] = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];

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

fn wubrg() -> ManaCost {
    cost(&[w(), u(), b(), r(), g()])
}

fn token(name: &str, colors: Vec<Color>, ct: CreatureType, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn mint(t: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(t) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// "Whenever you cast a multicolored spell, [effect]."
fn on_multicolored_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Multicolored }),
        effect,
    }
}

fn spell_colors() -> Value {
    Value::ColorCountOf(Box::new(Selector::TriggerSource))
}

/// Jared Carthalion — can be your commander; +1: an all-colors 3/3 trample
/// Kavu; −3: up to two creatures get a +1/+1 counter per color; −6: a
/// multicolored card back, and an all-colors one also draws and makes two
/// Treasures.
pub fn jared_carthalion() -> CardDefinition {
    CardDefinition {
        name: "Jared Carthalion",
        cost: wubrg(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Jared], ..Default::default() },
        base_loyalty: 5,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                x_cost: false,
                effect: mint(token("Kavu", WUBRG.to_vec(), CreatureType::Kavu, 3, 3, vec![Keyword::Trample])),
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                x_cost: false,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ColorCountOf(Box::new(Selector::Target(0))),
                    }),
                },
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                x_cost: false,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::Multicolored.and(R::InYourGraveyard)),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::AllColors },
                        then: Box::new(Effect::Seq(vec![
                            draw(1),
                            Effect::CreateToken {
                                who: PlayerRef::You,
                                count: Value::Const(2),
                                definition: Arc::new(crabomination_base::tokens::treasure_token()),
                            },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Archelos, Lagoon Mystic — tapped, other permanents enter tapped;
/// untapped, they enter untapped.
pub fn archelos_lagoon_mystic() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Other permanents enter tapped while this is tapped and untapped while it isn't.",
            effect: StaticEffect::OthersEnterWithSourceTapState,
        }],
        ..creature(
            "Archelos, Lagoon Mystic",
            cost(&[generic(1), b(), g(), u()]),
            vec![CreatureType::Turtle, CreatureType::Shaman],
            2,
            4,
        )
    }
}

/// Fallaji Wayfarer — all colors; your multicolored spells have convoke.
pub fn fallaji_wayfarer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Fallaji Wayfarer is all colors. This ability doesn't affect its color identity.",
                effect: StaticEffect::GrantAllColors { applies_to: Selector::This },
            },
            StaticAbility {
                description: "Multicolored spells you cast have convoke.",
                effect: StaticEffect::GrantConvokeToSpells { filter: R::Multicolored },
            },
        ],
        ..creature("Fallaji Wayfarer", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Scout], 2, 4)
    }
}

/// Fusion Elemental — an 8/8 for WUBRG.
pub fn fusion_elemental() -> CardDefinition {
    creature("Fusion Elemental", wubrg(), vec![CreatureType::Elemental], 8, 8)
}

/// Iridian Maelstrom — destroy each creature that isn't all colors.
pub fn iridian_maelstrom() -> CardDefinition {
    CardDefinition {
        name: "Iridian Maelstrom",
        cost: wubrg(),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::AllColors.negate())) },
        ..Default::default()
    }
}

/// Jenson Carthalion, Druid Exile — each multicolored spell scries 1, an
/// all-colors one also makes a 4/4 Angel; {5}, {T}: WUBRG.
pub fn jenson_carthalion_druid_exile() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_multicolored_cast(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            Effect::If {
                cond: Predicate::ValueAtLeast(spell_colors(), Value::Const(5)),
                then: Box::new(mint(token(
                    "Angel",
                    vec![Color::White],
                    CreatureType::Angel,
                    4,
                    4,
                    vec![Keyword::Flying, Keyword::Vigilance],
                ))),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(5)]),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(WUBRG.to_vec()) },
            ..Default::default()
        }],
        ..creature(
            "Jenson Carthalion, Druid Exile",
            cost(&[g(), w()]),
            vec![CreatureType::Human, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Knight of New Alara — your other multicolored creatures get +1/+1 per
/// color.
/// Residual: printed colors are counted.
pub fn knight_of_new_alara() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each other multicolored creature you control gets +1/+1 for each of its colors.",
            effect: StaticEffect::PumpTeamByControlledPermanents {
                applies_to: R::Creature.and(R::Multicolored).and(R::OtherThanSource),
                count_filter: R::Any,
                per_power: 1,
                per_toughness: 1,
                count_graveyard: false,
                exclude_self: false,
                per_own_color: true,
            },
        }],
        ..creature("Knight of New Alara", cost(&[generic(2), g(), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Mana Cannons — each multicolored spell you cast deals its color count to
/// any target.
pub fn mana_cannons() -> CardDefinition {
    CardDefinition {
        name: "Mana Cannons",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![on_multicolored_cast(Effect::DealDamage { to: target_any(), amount: spell_colors() })],
        ..Default::default()
    }
}

/// Obsidian Obelisk — enters tapped; {T}: {C}; {T}: any color, multicolored
/// spells only.
pub fn obsidian_obelisk() -> CardDefinition {
    CardDefinition {
        name: "Obsidian Obelisk",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::MulticoloredSpell,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Path to the World Tree — a basic land to hand on entry; {2}{W}{U}{B}{R}{G},
/// sacrifice it: 2 life, two cards, an opponent loses 2, 2 damage to up to
/// one creature, and a Bear.
pub fn path_to_the_world_tree() -> CardDefinition {
    CardDefinition {
        name: "Path to the World Tree",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), u(), b(), r(), g()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                draw(2),
                Effect::LoseLife {
                    who: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer },
                    amount: Value::Const(2),
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    amount: Value::Const(2),
                },
                mint(token("Bear", vec![Color::Green], CreatureType::Bear, 2, 2, vec![])),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Primeval Spawn — vigilance, trample, lifelink; leaving, exile the top ten
/// and cast spells from among them free.
/// Residual: it isn't exiled when entering uncast, and the leave trigger
/// casts one spell of mana value 10 or less.
pub fn primeval_spawn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileLinked {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(10) },
                },
                Effect::CastAnyOrderWithoutPaying {
                    what: Selector::CardExiledWithSource,
                    source_zone: Zone::Exile,
                    filter: Some(R::ManaValueAtMost(10)),
                    cap: Some(Value::ONE),
                },
            ]),
        }],
        ..creature(
            "Primeval Spawn",
            cost(&[generic(5), w(), u(), b(), r(), g()]),
            vec![CreatureType::Avatar],
            10,
            10,
        )
    }
}

/// Rienne, Angel of Rebirth — flying; your other multicolored creatures get
/// +1/+0; one dying returns to its owner's hand at the next end step.
pub fn rienne_angel_of_rebirth() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other multicolored creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(R::Creature.and(R::Multicolored).and(R::OtherThanSource)),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Multicolored },
            ),
            effect: Effect::AtNextEndStep {
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                }),
            },
        }],
        ..creature(
            "Rienne, Angel of Rebirth",
            cost(&[generic(2), r(), g(), w()]),
            vec![CreatureType::Angel],
            5,
            4,
        )
    }
}

/// Surrak Dragonclaw — flash, can't be countered; your creature spells can't
/// be countered; your other creatures have trample.
pub fn surrak_dragonclaw() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash, Keyword::CantBeCountered],
        static_abilities: vec![
            StaticAbility {
                description: "Creature spells you control can't be countered.",
                effect: StaticEffect::CreatureSpellsCantBeCountered,
            },
            StaticAbility {
                description: "Other creatures you control have trample.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: yours(R::Creature.and(R::OtherThanSource)),
                    keyword: Keyword::Trample,
                },
            },
        ],
        ..creature(
            "Surrak Dragonclaw",
            cost(&[generic(2), g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            6,
            6,
        )
    }
}

/// Tiller Engine — a land of yours entering tapped untaps, or taps an
/// opponent's nonland permanent.
pub fn tiller_engine() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land.and(R::Tapped) },
            ),
            effect: Effect::ChooseMode(vec![
                Effect::Untap { what: Selector::TriggerSource, up_to: None },
                Effect::Tap {
                    what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
                },
            ]),
        }],
        ..creature("Tiller Engine", cost(&[generic(2)]), vec![CreatureType::Construct], 1, 3)
    }
}

/// Two-Headed Hellkite — flying, menace, haste; attacking draws two.
pub fn two_headed_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Menace, Keyword::Haste],
        triggered_abilities: vec![on_attack(draw(2))],
        ..creature(
            "Two-Headed Hellkite",
            cost(&[generic(1), w(), u(), b(), r(), g()]),
            vec![CreatureType::Dragon],
            5,
            5,
        )
    }
}

/// Unite the Coalition — choose five, repeats allowed: phase out a
/// permanent, a player draws, exile a graveyard, 2 damage, destroy an
/// artifact or enchantment.
/// Residual: a repeated mode needs a different target the bot doesn't always
/// find.
pub fn unite_the_coalition() -> CardDefinition {
    CardDefinition {
        name: "Unite the Coalition",
        cost: cost(&[generic(2), w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::PhaseOut { what: target_filtered(R::Permanent), until_source_leaves: false },
                Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
                Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
                Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ],
            min: 5,
            max: 5,
            allow_repeats: true,
        },
        ..Default::default()
    }
}

/// Xyris, the Writhing Storm — flying; an opponent's extra draw makes a 1/1
/// Snake; its combat damage draws you and that player that many.
pub fn xyris_the_writhing_storm() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl).with_filter(
                    Predicate::Not(Box::new(Predicate::All(vec![
                        Predicate::CurrentStepIs(crate::game::types::TurnStep::Draw),
                        Predicate::IsTurnOf(PlayerRef::Triggerer),
                        Predicate::ValueAtMost(Value::CardsDrawnThisStep(PlayerRef::Triggerer), Value::Const(1)),
                    ]))),
                ),
                effect: mint(token("Snake", vec![Color::Green], CreatureType::Snake, 1, 1, vec![])),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
                    Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::TriggerEventAmount },
                ]),
            },
        ],
        ..creature(
            "Xyris, the Writhing Storm",
            cost(&[generic(2), g(), u(), r()]),
            vec![CreatureType::Snake, CreatureType::Leviathan],
            3,
            5,
        )
    }
}

/// Zaxara, the Exemplary — deathtouch; {T}: two mana of one color; each X
/// spell you cast makes a Hydra with X +1/+1 counters.
pub fn zaxara_the_exemplary() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(2)) },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::cast_has_x_trigger(Effect::Seq(vec![
                mint(token("Hydra", vec![Color::Green], CreatureType::Hydra, 0, 0, vec![])),
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                },
            ]))],
        ..creature(
            "Zaxara, the Exemplary",
            cost(&[generic(1), b(), g(), u()]),
            vec![CreatureType::Nightmare, CreatureType::Hydra],
            2,
            3,
        )
    }
}

