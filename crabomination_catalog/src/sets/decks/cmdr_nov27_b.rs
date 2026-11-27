//! Commander: one-card closers (scan of 2026-11-27) — Gone Missing (The Sixth
//! Doctor // Nyssa of Traken), The Needs of the Many (The War Doctor // Dan
//! Lewis), Troubling Tribbles (Jaheira // Noble Heritage), Cardassia Prime
//! (Sengir // Tymna); Gul Dukat and Unexplored Territory toward Mohiam's and
//! Benjamin Sisko's decks; McCoy and There's Coffee in that Nebula (their Clara
//! Oswald averages mix colors, so those decks never seat); Yume, Chronicler of
//! Valor (assist). Tests in
//! `tests/recent_b/cmdr_nov26.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{investigate, target_filtered, token_copy_of};
use crate::effect::{
    Duration, Effect, LibraryPosition, LookPick, ManaPayload, PlayerRef, StaticEffect, Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, u, w, Color};
use crate::sets::{enters_tapped, etb_gain_one};

/// McCoy, Not a Bricklayer — {3}{G}{W} legendary 0/4 Human Doctor. Whenever it
/// or another Doctor you control enters, put a +1/+1 counter on each creature
/// you control. Your creatures with counters have lifelink.
pub fn mccoy_not_a_bricklayer() -> CardDefinition {
    let yours = || R::Creature.and(R::ControlledByYou);
    CardDefinition {
        name: "McCoy, Not a Bricklayer",
        cost: cost(&[generic(3), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Doctor], ..Default::default() },
        power: 0,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsSource.or(R::Creature.and(R::HasCreatureType(CreatureType::Doctor))),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(yours()),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with counters on them have lifelink.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours().and(R::WithAnyCounter)),
                keyword: Keyword::Lifelink,
            },
        }],
        ..Default::default()
    }
}

/// "Beam me up {cost}": flashback that also returns a creature you control to
/// its owner's hand (Star Trek's graveyard cast).
fn beam_me_up(def: CardDefinition, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(mana)],
        flashback_additional_cost: vec![AdditionalCastCost::ReturnToHand {
            filter: R::Creature.and(R::ControlledByYou),
            count: 1,
            count_x: false,
        }],
        ..def
    }
}

/// There's Coffee in that Nebula — {1}{W} sorcery. Look at the top five; a
/// Plains card or a nonland permanent card with mana value 3 or less may go to
/// hand, the rest to the bottom at random. Beam me up {2}{W}.
pub fn theres_coffee_in_that_nebula() -> CardDefinition {
    beam_me_up(
        CardDefinition {
            name: "There's Coffee in that Nebula",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(
                    R::HasLandType(LandType::Plains)
                        .or(R::Permanent.and(R::Nonland).and(R::ManaValueAtMost(3))),
                ),
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            })),
            ..Default::default()
        },
        cost(&[generic(2), w()]),
    )
}

/// Gone Missing — {4}{U} sorcery. Put target permanent on top of its owner's
/// library. Investigate.
pub fn gone_missing() -> CardDefinition {
    CardDefinition {
        name: "Gone Missing",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Top,
                },
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// The Needs of the Many — {1}{W} instant; sacrifice a creature as an
/// additional cost. Your creatures gain hexproof and indestructible and you
/// gain hexproof until end of turn; prevent all damage to you this turn.
/// Exile it.
pub fn the_needs_of_the_many() -> CardDefinition {
    CardDefinition {
        name: "The Needs of the Many",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 1 }],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::GrantKeywords {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keywords: vec![Keyword::Hexproof, Keyword::Indestructible],
                duration: Duration::EndOfTurn,
            },
            Effect::PlayerHexproofThisTurn { who: Selector::You },
            Effect::PreventAllDamageToPlayerThisTurn { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

/// Troubling Tribbles — {2}{W} 0/1 Tribble, can't block. At the beginning of
/// your upkeep, create a token copy of it.
pub fn troubling_tribbles() -> CardDefinition {
    CardDefinition {
        name: "Troubling Tribbles",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Tribble], ..Default::default() },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: token_copy_of(PlayerRef::You, Value::ONE, Selector::This),
        }],
        ..Default::default()
    }
}

/// Cardassia Prime — Land — Planet. Enters tapped; when it enters, you gain 1
/// life. {T}: {W} or {B}.
pub fn cardassia_prime() -> CardDefinition {
    CardDefinition {
        name: "Cardassia Prime",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        static_abilities: vec![enters_tapped()],
        triggered_abilities: vec![etb_gain_one()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![Color::White, Color::Black], Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gul Dukat, Colonial Prefect — {4}{B}{B}{B} legendary 6/6 Cardassian Rogue,
/// menace. At the beginning of each end step, each opponent loses life equal
/// to the life they lost this turn, and you gain the life lost this way.
pub fn gul_dukat_colonial_prefect() -> CardDefinition {
    CardDefinition {
        name: "Gul Dukat, Colonial Prefect",
        cost: cost(&[generic(4), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cardassian, CreatureType::Rogue], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::ForEachOpponent {
                body: Box::new(Effect::DrainLifeLost {
                    from: Selector::Player(PlayerRef::Triggerer),
                    to: Selector::You,
                    amount: Value::LifeLostThisTurn(PlayerRef::Triggerer),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Unexplored Territory — land. {T}: {C}. {1}, {T}, sacrifice it: search for a
/// basic land card, put it onto the battlefield tapped, then shuffle.
pub fn unexplored_territory() -> CardDefinition {
    CardDefinition {
        name: "Unexplored Territory",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Yume, Chronicler of Valor — {1}{W} legendary 1/2 Human Druid. The first
/// spell you cast during each of your turns has assist (CR 702.132a); when you
/// cast it, if another player spent mana on it, you and that player each draw.
pub fn yume_chronicler_of_valor() -> CardDefinition {
    let first_on_your_turn = |n: i32| {
        vec![Predicate::IsTurnOf(PlayerRef::You), Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(n) }]
    };
    CardDefinition {
        name: "Yume, Chronicler of Valor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Druid], ..Default::default() },
        power: 1,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "The first spell you cast during each of your turns has assist.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::All(first_on_your_turn(0)),
                inner: Box::new(StaticEffect::GrantAssistToSpells { filter: R::Any }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(
                [vec![Predicate::SpellWasAssisted { what: Selector::TriggerSource }], first_on_your_turn(1)].concat(),
            )),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::Const(1) },
                Effect::Draw { who: Selector::Player(PlayerRef::AssisterOfTriggerSpell), amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Benjamin Sisko, Besieged — {4}{G}{G} legendary 5/5 Human Officer. Play with
/// the top card of your library revealed; you may play lands and cast creature
/// spells from it.
pub fn benjamin_sisko_besieged() -> CardDefinition {
    CardDefinition {
        name: "Benjamin Sisko, Besieged",
        cost: cost(&[generic(4), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Officer], ..Default::default() },
        power: 5,
        toughness: 5,
        static_abilities: vec![
            StaticAbility { description: "Play with the top card of your library revealed.", effect: StaticEffect::TopOfLibraryRevealed },
            StaticAbility {
                description: "You may play lands and cast creature spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Land.or(R::Creature) },
            },
        ],
        ..Default::default()
    }
}

/// Final Frontier — {2}{G} sorcery. Search for a basic land or Planet card,
/// onto the battlefield tapped. Beam me up {1}{G}.
pub fn final_frontier() -> CardDefinition {
    beam_me_up(
        CardDefinition {
            name: "Final Frontier",
            cost: cost(&[generic(2), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.or(R::HasLandType(LandType::Planet)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        },
        cost(&[generic(1), g()]),
    )
}

/// Grakk the Pacifist — {1}{W} legendary 4/3 Orc. Can't attack or block.
/// Whenever you cast an Aura spell that targets a creature an opponent
/// controls, draw a card; only once each turn.
pub fn grakk_the_pacifist() -> CardDefinition {
    CardDefinition {
        name: "Grakk the Pacifist",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Orc], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Aura),
                    },
                    Predicate::CastSpellTargetsMatch(R::Creature.and(R::ControlledByOpponent)),
                ]))
                .once_per_turn(),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: crate::mana::ManaCost, filter: R, keywords: Vec<Keyword>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter } },
        equipped_bonus: Some(crate::card::EquipBonus { keywords, ..Default::default() }),
        ..Default::default()
    }
}

/// Guard Duty — {W} Aura. Enchanted creature has defender.
pub fn guard_duty() -> CardDefinition {
    aura("Guard Duty", cost(&[w()]), R::Creature, vec![Keyword::Defender])
}

/// Realmbreaker's Grasp — {1}{W} Aura, enchant artifact or creature. It can't
/// attack or block, and its non-mana activated abilities can't be activated.
pub fn realmbreakers_grasp() -> CardDefinition {
    aura(
        "Realmbreaker's Grasp",
        cost(&[generic(1), w()]),
        R::Artifact.or(R::Creature),
        vec![Keyword::CantAttack, Keyword::CantBlock, Keyword::CantActivateNonManaAbilities],
    )
}

/// Earth, Sector 001 — Land — Planet. Enters tapped; when it enters, you gain
/// 1 life. {T}: {G} or {U}. (The Traveler's deck.)
pub fn earth_sector_001() -> CardDefinition {
    CardDefinition { name: "Earth, Sector 001", ..planet_pair(Color::Green, Color::Blue) }
}

/// A tapped, life-gain Planet dual (the TRK Planet cycle's shape).
fn planet_pair(a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        static_abilities: vec![enters_tapped()],
        triggered_abilities: vec![etb_gain_one()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![a, b], Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Traveler — {G}{U} legendary */3 Scout with reach and vigilance; power =
/// differently named lands you control (strange new worlds).
pub fn the_traveler() -> CardDefinition {
    CardDefinition {
        name: "The Traveler",
        cost: cost(&[g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Scout], ..Default::default() },
        toughness: 3,
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        dynamic_pt: Some(crate::card::DynamicPt::DistinctLandNamesPower { base_t: 3 }),
        ..Default::default()
    }
}
