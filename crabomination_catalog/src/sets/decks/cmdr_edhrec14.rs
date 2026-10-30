//! Commander: the one-card EDHREC gaps of commanders ranked 1001-1500 (the
//! 2026-10-12 top-1500 rescan) — each card completes at least one average
//! deck. Tests in `tests/recent_b/cmdr_edhrec14.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{
    battalion, blocks, cast_is_noncreature, enrage, etb, on_attack, target_any, target_filtered,
};
use crate::effect::{ActivatedAbility, Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost};
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

fn permanent(name: &'static str, mana: ManaCost, kind: CardType) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], ..Default::default() }
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

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn gain(n: i32) -> Effect {
    Effect::GainLife { who: Selector::You, amount: Value::Const(n) }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

fn may(description: &str, body: Effect) -> Effect {
    Effect::MayDo { description: description.into(), body: Box::new(body) }
}

/// Kindly Customer — {1}{W} 1/1 Human Citizen. When it enters, draw a card.
pub fn kindly_customer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        ..creature("Kindly Customer", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Citizen], 1, 1)
    }
}

/// Corridor Monitor — {1}{U} 1/4 artifact creature Construct. When it enters,
/// untap target artifact or creature you control.
pub fn corridor_monitor() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Untap {
            what: target_filtered(yours(R::Artifact.or(R::Creature))),
            up_to: None,
        })],
        ..creature("Corridor Monitor", cost(&[generic(1), u()]), vec![CreatureType::Construct], 1, 4)
    }
}

/// See Beyond — {1}{U} sorcery. Draw two, then shuffle a card from your hand
/// into your library.
pub fn see_beyond() -> CardDefinition {
    spell(
        "See Beyond",
        cost(&[generic(1), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            draw(2),
            Effect::PutOnLibraryFromHand { who: PlayerRef::You, count: Value::ONE },
            Effect::ShuffleLibrary { who: PlayerRef::You },
        ]),
    )
}

/// Goldenglow Moth — {W} 0/1 flying Insect. Whenever it blocks, you may gain 4
/// life.
pub fn goldenglow_moth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![blocks(may("gain 4 life", gain(4)))],
        ..creature("Goldenglow Moth", cost(&[w()]), vec![CreatureType::Insect], 0, 1)
    }
}

/// Howling Golem — {3} 2/3 artifact creature Golem. Whenever it attacks or
/// blocks, each player draws a card.
pub fn howling_golem() -> CardDefinition {
    let each_draws = || Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_attack(each_draws()), blocks(each_draws())],
        ..creature("Howling Golem", cost(&[generic(3)]), vec![CreatureType::Golem], 2, 3)
    }
}

/// Rendclaw Trow — {2}{B/G} 2/2 Troll with wither and persist.
pub fn rendclaw_trow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Wither, Keyword::Persist],
        ..creature(
            "Rendclaw Trow",
            cost(&[generic(2), hybrid(Color::Black, Color::Green)]),
            vec![CreatureType::Troll],
            2,
            2,
        )
    }
}

/// Spitemare — {2}{R/W}{R/W} 3/3 Elemental. Whenever it's dealt damage, it
/// deals that much damage to any target.
pub fn spitemare() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    CardDefinition {
        triggered_abilities: vec![enrage(Effect::DealDamage { to: target_any(), amount: Value::TriggerEventAmount })],
        ..creature("Spitemare", cost(&[generic(2), rw(), rw()]), vec![CreatureType::Elemental], 3, 3)
    }
}

/// Golden Egg — {2} artifact Food. When it enters, draw a card. {1}, {T},
/// sacrifice it: one mana of any color. {2}, {T}, sacrifice it: gain 3 life.
pub fn golden_egg() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        triggered_abilities: vec![etb(draw(1))],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: gain(3),
                ..Default::default()
            },
        ],
        ..permanent("Golden Egg", cost(&[generic(2)]), CardType::Artifact)
    }
}

/// Staff of the Death Magus — {3} artifact. Whenever you cast a black spell or
/// a Swamp you control enters, you gain 1 life.
pub fn staff_of_the_death_magus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasColor(Color::Black) },
                ),
                effect: gain(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasLandType(LandType::Swamp),
                    },
                ),
                effect: gain(1),
            },
        ],
        ..permanent("Staff of the Death Magus", cost(&[generic(3)]), CardType::Artifact)
    }
}

/// Lyla, Holographic Assistant — {3}{U} 2/2 legendary artifact creature
/// Illusion Advisor. Whenever you draw a card, put a +1/+1 counter on target
/// creature.
pub fn lyla_holographic_assistant() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..legend(
            "Lyla, Holographic Assistant",
            cost(&[generic(3), u()]),
            vec![CreatureType::Illusion, CreatureType::Advisor],
            2,
            2,
        )
    }
}

/// Tainted Treats — {1}{B}{G} instant. Destroy target artifact or creature; if
/// its mana value was 4 or less, create a Food token.
pub fn tainted_treats() -> CardDefinition {
    spell(
        "Tainted Treats",
        cost(&[generic(1), b(), g()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::ValueAtMost(Value::ManaValueOf(Box::new(Selector::Target(0))), Value::Const(4)),
            then: Box::new(Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Creature)) },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::food_token()),
                },
            ])),
            else_: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        },
    )
}

/// Haazda Marshal — {W} 1/1 Human Soldier. Whenever it and at least two other
/// creatures attack, create a 1/1 white Soldier token with lifelink.
pub fn haazda_marshal() -> CardDefinition {
    let soldier = TokenDefinition {
        keywords: vec![Keyword::Lifelink],
        ..token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier])
    };
    CardDefinition {
        triggered_abilities: vec![battalion(make(1, soldier))],
        ..creature("Haazda Marshal", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Magus of the Moat — {2}{W}{W} 0/3 Human Wizard. Creatures without flying
/// can't attack.
pub fn magus_of_the_moat() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Creatures without flying can't attack.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))))),
                keyword: Keyword::CantAttack,
            },
        )],
        ..creature(
            "Magus of the Moat",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            0,
            3,
        )
    }
}

/// Voltaic Servant — {2} 1/3 artifact creature Construct. At the beginning of
/// your end step, untap target artifact.
pub fn voltaic_servant() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![at_your(
            TurnStep::End,
            Effect::Untap { what: target_filtered(R::Artifact), up_to: None },
        )],
        ..creature("Voltaic Servant", cost(&[generic(2)]), vec![CreatureType::Construct], 1, 3)
    }
}

/// Samut, Voice of Dissent — {3}{R}{G} 3/4 legendary Human Warrior with flash,
/// double strike, vigilance, haste. Other creatures you control have haste.
/// {W}, {T}: untap another target creature.
pub fn samut_voice_of_dissent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::DoubleStrike, Keyword::Vigilance, Keyword::Haste],
        static_abilities: vec![static_ab(
            "Other creatures you control have haste.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Creature).and(R::OtherThanSource)),
                keyword: Keyword::Haste,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::Creature.and(R::OtherThanSource)), up_to: None },
            ..Default::default()
        }],
        ..legend(
            "Samut, Voice of Dissent",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            4,
        )
    }
}

/// Aegis Angel — {4}{W}{W} 5/5 flying Angel. When it enters, another target
/// permanent gains indestructible for as long as you control this creature.
pub fn aegis_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Permanent.and(R::OtherThanSource)),
            keyword: Keyword::Indestructible,
            duration: Duration::WhileYouControlSource,
        })],
        ..creature("Aegis Angel", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 5, 5)
    }
}

/// Ayula's Influence — {G}{G}{G} enchantment. Discard a land card: create a
/// 2/2 green Bear token.
pub fn ayulas_influence() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Land, 1)),
            effect: make(1, token("Bear", 2, 2, vec![Color::Green], vec![CreatureType::Bear])),
            ..Default::default()
        }],
        ..permanent("Ayula's Influence", cost(&[g(), g(), g()]), CardType::Enchantment)
    }
}

/// Plate Armor — {2}{W} Equipment. +3/+3 and ward {1}. Equip {3}, {1} less for
/// each other Equipment you control.
pub fn plate_armor() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        static_abilities: vec![static_ab(
            "This ability costs {1} less to activate for each other Equipment you control.",
            StaticEffect::EquipCostReducedPerControlled {
                filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::OtherThanSource),
            },
        )],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
            ..Default::default()
        }),
        ..permanent("Plate Armor", cost(&[generic(2), w()]), CardType::Artifact)
    }
}

/// Mouser Foundry — {1}{R} artifact. When it enters or leaves the
/// battlefield, create a 1/1 colorless Robot artifact creature token. {4}{R},
/// sacrifice it: 3 damage to target creature.
pub fn mouser_foundry() -> CardDefinition {
    let robot = || {
        make(
            1,
            TokenDefinition {
                card_types: vec![CardType::Artifact, CardType::Creature],
                ..token("Robot", 1, 1, vec![], vec![CreatureType::Robot])
            },
        )
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(robot()),
            TriggeredAbility { event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource), effect: robot() },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r()]),
            sac_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..permanent("Mouser Foundry", cost(&[generic(1), r()]), CardType::Artifact)
    }
}

/// Brightmare — {2}{W} 2/3 Unicorn. When it enters, tap up to one target
/// creature; gain life equal to its power.
pub fn brightmare() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::GainLife { who: Selector::You, amount: Value::PowerOf(Box::new(Selector::Target(0))) },
            ])),
        })],
        ..creature("Brightmare", cost(&[generic(2), w()]), vec![CreatureType::Unicorn], 2, 3)
    }
}

/// Heartwood Storyteller — {1}{G}{G} 2/3 Treefolk. Whenever a player casts a
/// noncreature spell, each of that player's opponents may draw a card.
pub fn heartwood_storyteller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(cast_is_noncreature()),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::OpponentOf(Box::new(PlayerRef::Triggerer)),
                body: Box::new(Effect::MayDo {
                    description: "Draw a card?".into(),
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                }),
            },
        }],
        ..creature("Heartwood Storyteller", cost(&[generic(1), g(), g()]), vec![CreatureType::Treefolk], 2, 3)
    }
}

/// Soldevi Adnate — {1}{B} 1/2 Human Cleric. {T}, sacrifice a black or
/// artifact creature: add {B} equal to its mana value.
pub fn soldevi_adnate() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::HasColor(Color::Black).or(R::Artifact)), 1)),
            sac_other_may_be_source: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Black, Value::SacrificedManaValue),
            },
            ..Default::default()
        }],
        ..creature("Soldevi Adnate", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 2)
    }
}

/// Lyra, Archangel of Dawn — {2}{W} 3/3 legendary flying Angel Knight.
/// Whenever you gain life, put a +1/+1 counter on each Angel you control.
pub fn lyra_archangel_of_dawn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Angel))),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..legend(
            "Lyra, Archangel of Dawn",
            cost(&[generic(2), w()]),
            vec![CreatureType::Angel, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Marwyn, the Preserver — {1}{G} 3/2 legendary Elf Druid. Lands you control
/// have hexproof. {2}: return target land card from your graveyard to hand.
pub fn marwyn_the_preserver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Lands you control have hexproof.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Land)),
                keyword: Keyword::Hexproof,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Move {
                what: target_filtered(R::Land.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..legend("Marwyn, the Preserver", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Druid], 3, 2)
    }
}

/// Wizards of Thay — {3}{U} 3/3 Human Wizard with myriad. Instant and sorcery
/// spells you cast cost {1} less; you may cast sorceries as though they had
/// flash.
pub fn wizards_of_thay() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::myriad()],
        static_abilities: vec![
            static_ab(
                "Instant and sorcery spells you cast cost {1} less to cast.",
                StaticEffect::CostReduction {
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    amount: 1,
                },
            ),
            static_ab(
                "You may cast sorcery spells as though they had flash.",
                StaticEffect::ControllerSpellsHaveFlash { filter: R::HasCardType(CardType::Sorcery) },
            ),
        ],
        ..creature("Wizards of Thay", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 3)
    }
}

/// Faerie Tauntings — {2}{B} Kindred Enchantment — Faerie. Whenever you cast
/// a spell during an opponent's turn, you may have each opponent lose 1 life.
pub fn faerie_tauntings() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
            effect: may(
                "each opponent loses 1 life",
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            ),
        }],
        ..permanent("Faerie Tauntings", cost(&[generic(2), b()]), CardType::Enchantment)
    }
}

/// Ingris Stingerquill — {B}{R}{R} 1/4 legendary flying Elder Sphinx. Whenever
/// a creature you control attacks, it deals 1 damage to each opponent. {4}:
/// create a 2/2 colorless Wizard Soldier named Cadet; creatures you control
/// gain haste until end of turn.
pub fn ingris_stingerquill() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::DealDamageFrom {
                source: Selector::TriggerSource,
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: Effect::Seq(vec![
                make(1, token("Cadet", 2, 2, vec![], vec![CreatureType::Wizard, CreatureType::Soldier])),
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(yours(R::Creature)),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Ingris Stingerquill",
            cost(&[b(), r(), r()]),
            vec![CreatureType::Elder, CreatureType::Sphinx],
            1,
            4,
        )
    }
}

/// Desolation of Smaug — {2}{R}{R} sorcery. 3 damage to each non-Dragon
/// creature; add four mana in any combination of colors, spent only on Dragon
/// spells.
pub fn desolation_of_smaug() -> CardDefinition {
    spell(
        "Desolation of Smaug",
        cost(&[generic(2), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Dragon))))),
                amount: Value::Const(3),
            },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyColors(Value::Const(4))),
                    crate::mana::SpendRestriction::CreatureSpellOfType(CreatureType::Dragon),
                ),
            },
        ]),
    )
}

/// Teferi, Mage of Zhalfir — {2}{U}{U}{U} 3/4 legendary Human Wizard with
/// flash. Your creature cards have flash; opponents cast spells only at
/// sorcery speed.
pub fn teferi_mage_of_zhalfir() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![
            static_ab(
                "Creature cards you own that aren't on the battlefield have flash.",
                StaticEffect::ControllerSpellsHaveFlash { filter: R::HasCardType(CardType::Creature) },
            ),
            static_ab(
                "Each opponent can cast spells only any time they could cast a sorcery.",
                StaticEffect::OpponentsSorceryTimingOnly,
            ),
        ],
        ..legend(
            "Teferi, Mage of Zhalfir",
            cost(&[generic(2), u(), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// "Exile two [color] cards from your hand rather than pay this spell's mana
/// cost" (Fury of the Horde, Commandeer).
fn exile_two(color: Color) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        exile_filter: Some(R::HasColor(color)),
        exile_filter_extra: 1,
        ..Default::default()
    }
}

/// Fury of the Horde — {5}{R}{R} sorcery; exile two red cards instead. Untap
/// every creature that attacked this turn; an additional combat and main
/// phase follow this one.
pub fn fury_of_the_horde() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(exile_two(Color::Red)),
        ..spell(
            "Fury of the Horde",
            cost(&[generic(5), r(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Untap { what: Selector::EachPermanent(R::Creature.and(R::AttackedThisTurn)), up_to: None },
                Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE },
            ]),
        )
    }
}

/// Commandeer — {5}{U}{U} instant; exile two blue cards instead. Gain control
/// of target noncreature spell.
pub fn commandeer() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(exile_two(Color::Blue)),
        ..spell(
            "Commandeer",
            cost(&[generic(5), u(), u()]),
            CardType::Instant,
            // "… You may choose new targets for it" (CR 115.7d).
            Effect::Seq(vec![
                Effect::GainControlOfSpell {
                    what: target_filtered(R::IsSpellOnStack.and(R::Not(Box::new(R::HasCardType(CardType::Creature))))),
                },
                Effect::ChooseNewTargetsForSpell { what: Selector::Target(0) },
            ]),
        )
    }
}

/// Invert Polarity — {U}{U}{R} instant. Choose target spell, flip a coin: win,
/// gain control of it; lose, counter it.
pub fn invert_polarity() -> CardDefinition {
    spell(
        "Invert Polarity",
        cost(&[u(), u(), r()]),
        CardType::Instant,
        Effect::FlipCoin {
            count: Value::ONE,
            on_heads: Box::new(Effect::Seq(vec![
                Effect::GainControlOfSpell { what: target_filtered(R::IsSpellOnStack) },
                Effect::ChooseNewTargetsForSpell { what: Selector::Target(0) },
            ])),
            on_tails: Box::new(Effect::CounterSpell { what: Selector::Target(0) }),
        },
    )
}

/// "Gain control of [target] until end of turn. Untap it. It gains haste
/// until end of turn," then `rider`.
fn threaten(filter: R, rider: Vec<Effect>) -> Effect {
    let mut steps = vec![
        Effect::GainControl { what: target_filtered(filter), to: None, duration: Duration::EndOfTurn },
        Effect::Untap { what: Selector::Target(0), up_to: None },
        Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
    ];
    steps.extend(rider);
    Effect::Seq(steps)
}

/// Kari Zev's Expertise — {1}{R}{R} sorcery. Threaten target creature or
/// Vehicle; you may cast a spell with mana value 2 or less from your hand free.
pub fn kari_zevs_expertise() -> CardDefinition {
    spell(
        "Kari Zev's Expertise",
        cost(&[generic(1), r(), r()]),
        CardType::Sorcery,
        threaten(
            R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
            vec![Effect::CastFromHandWithoutPaying { filter: Some(R::ManaValueAtMost(2)) }],
        ),
    )
}

/// Furnace Reins — {2}{R} sorcery. Threaten target creature; until end of turn
/// it has "whenever this deals combat damage to a player or battle, create a Treasure".
pub fn furnace_reins() -> CardDefinition {
    spell(
        "Furnace Reins",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        threaten(
            R::Creature,
            vec![Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).or_battle(),
                    effect: Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(crabomination_base::tokens::treasure_token()),
                    },
                }),
                duration: Duration::EndOfTurn,
            }],
        ),
    )
}

/// Qala, Ajani's Pridemate — {3}{W} 3/3 legendary Cat Warrior. Attacking:
/// other attackers get +X/+0, X its counters. Life gain grows it. {3}{W}: gain
/// 1 life.
pub fn qala_ajanis_pridemate() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::PumpPT {
                what: Selector::EachPermanent(yours(R::IsAttacking).and(R::OtherThanSource)),
                power: Value::TotalCountersOn { what: Box::new(Selector::This) },
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[generic(3), w()]), effect: gain(1), ..Default::default() }],
        ..legend("Qala, Ajani's Pridemate", cost(&[generic(3), w()]), vec![CreatureType::Cat, CreatureType::Warrior], 3, 3)
    }
}

/// Tui and La, Moon and Ocean — {3}{U} 3/3 legendary Fish Spirit. Becoming
/// tapped draws a card; becoming untapped puts a +1/+1 counter on it.
pub fn tui_and_la_moon_and_ocean() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource), effect: draw(1) },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..legend("Tui and La, Moon and Ocean", cost(&[generic(3), u()]), vec![CreatureType::Fish, CreatureType::Spirit], 3, 3)
    }
}

/// Shaman of the Great Hunt — {3}{R} 4/2 Orc Shaman with haste. Your creatures
/// that deal combat damage to a player get a +1/+1 counter. Ferocious —
/// {2}{G/U}{G/U}: draw a card per creature you control with power 4 or more.
pub fn shaman_of_the_great_hunt() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), gu(), gu()]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(yours(R::Creature).and(R::PowerAtLeast(4)))),
            },
            ..Default::default()
        }],
        ..creature("Shaman of the Great Hunt", cost(&[generic(3), r()]), vec![CreatureType::Orc, CreatureType::Shaman], 4, 2)
    }
}

/// Pride Sovereign — {2}{G} 2/2 Cat; +1/+1 for each other Cat you control.
/// {W}, {T}, exert it: create two 1/1 white Cat tokens with lifelink.
pub fn pride_sovereign() -> CardDefinition {
    let cat = TokenDefinition {
        keywords: vec![Keyword::Lifelink],
        ..token("Cat", 1, 1, vec![Color::White], vec![CreatureType::Cat])
    };
    CardDefinition {
        static_abilities: vec![static_ab(
            "This creature gets +1/+1 for each other Cat you control.",
            StaticEffect::PumpPTPerOtherOfType { creature_type: CreatureType::Cat, power: 1, toughness: 1 },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            exert_cost: true,
            effect: make(2, cat),
            ..Default::default()
        }],
        ..creature("Pride Sovereign", cost(&[generic(2), g()]), vec![CreatureType::Cat], 2, 2)
    }
}

fn background(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Background],
            ..Default::default()
        },
        ..permanent(name, mana, CardType::Enchantment)
    }
}

/// "Commander creatures you own have [ability]."
fn grant_to_your_commanders(description: &'static str, ability: TriggeredAbility) -> StaticAbility {
    static_ab(
        description,
        StaticEffect::GrantTriggeredAbility {
            filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
            ability: Box::new(ability),
        },
    )
}

/// Sword Coast Sailor — {1}{U} Background. Commander creatures you own have
/// "whenever this attacks a player, if no opponent has more life than that
/// player, it can't be blocked this turn."
pub fn sword_coast_sailor() -> CardDefinition {
    let richest = Predicate::Not(Box::new(Predicate::ForAnyPlayer {
        who: PlayerRef::EachOpponent,
        pred: Box::new(Predicate::Not(Box::new(Predicate::ValueAtLeast(
            Value::LifeOf(PlayerRef::DefendingPlayer),
            Value::LifeOf(PlayerRef::Triggerer),
        )))),
    }));
    let trigger = TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::All(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::IsAttackingOpponentPlayer },
            richest,
        ])),
        effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
    };
    CardDefinition {
        static_abilities: vec![grant_to_your_commanders(
            "Commander creatures you own have \"Whenever this creature attacks a player, if no opponent has more \
             life than that player, this creature can't be blocked this turn.\"",
            trigger,
        )],
        ..background("Sword Coast Sailor", cost(&[generic(1), u()]))
    }
}

/// Cloakwood Hermit — {2}{G} Background. Commander creatures you own have "at
/// the beginning of your end step, if a creature card was put into your
/// graveyard from anywhere this turn, create two tapped 1/1 green Squirrels."
pub fn cloakwood_hermit() -> CardDefinition {
    let squirrel = TokenDefinition {
        tapped: true,
        ..token("Squirrel", 1, 1, vec![Color::Green], vec![CreatureType::Squirrel])
    };
    let trigger = TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
            .with_filter(Predicate::CreatureCardsToGraveyardThisTurnAtLeast(1)),
        effect: make(2, squirrel),
    };
    CardDefinition {
        static_abilities: vec![grant_to_your_commanders(
            "Commander creatures you own have \"At the beginning of your end step, if a creature card was put \
             into your graveyard from anywhere this turn, create two tapped 1/1 green Squirrel creature tokens.\"",
            trigger,
        )],
        ..background("Cloakwood Hermit", cost(&[generic(2), g()]))
    }
}

/// Phantom Blade — {4}{B} Equipment. When it enters, attach it to up to one
/// target creature you control; destroy up to one other target creature.
/// Equipped creature gets +1/+1 and has menace. Equip {2}.
pub fn phantom_blade() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: yours(R::Creature),
                effect: Box::new(Effect::Attach { what: Selector::This, to: Selector::Target(0) }),
            },
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::Not(Box::new(R::ControlledByYou))),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
        ]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Menace], ..Default::default() }),
        ..permanent("Phantom Blade", cost(&[generic(4), b()]), CardType::Artifact)
    }
}

/// Sigil of Valor — {2} Equipment. Whenever equipped creature attacks alone,
/// it gets +1/+1 for each other creature you control. Equip {1}.
pub fn sigil_of_valor() -> CardDefinition {
    let others = || Value::count(Selector::EachPermanent(yours(R::Creature).and(R::Not(Box::new(R::IsAttacking)))));
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::AttackingAlone),
                effect: Effect::PumpPT { what: Selector::This, power: others(), toughness: others(), duration: Duration::EndOfTurn },
            }],
            ..Default::default()
        }),
        ..permanent("Sigil of Valor", cost(&[generic(2)]), CardType::Artifact)
    }
}

/// Firbolg Flutist — {4}{R}{R} 4/4 Giant Bard. When it enters, gain control of
/// target creature you don't control until end of turn, untap it; it gains
/// haste and myriad until end of turn.
pub fn firbolg_flutist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(threaten(
            R::Creature.and(R::Not(Box::new(R::ControlledByYou))),
            vec![Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(crate::effect::shortcut::myriad()),
                duration: Duration::EndOfTurn,
            }],
        ))],
        ..creature("Firbolg Flutist", cost(&[generic(4), r(), r()]), vec![CreatureType::Giant, CreatureType::Bard], 4, 4)
    }
}

/// Frenzied Gorespawn — {3}{B}{R} 4/4 Horror. When it enters, for each
/// opponent, goad target creature that player controls. Whenever creatures
/// attack one of your opponents, they gain menace until end of turn.
pub fn frenzied_gorespawn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::ForEachOpponentTarget {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 15,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByOpponent),
                    effect: Box::new(Effect::Goad { what: Selector::Target(0) }),
                }),
            }),
            // Once per declaration, whoever declares it (Frontier Warmonger's
            // shape): every creature attacking one of your opponents.
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayerAttacks),
                effect: Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::IsAttackingAnOpponent),
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..creature("Frenzied Gorespawn", cost(&[generic(3), b(), r()]), vec![CreatureType::Horror], 4, 4)
    }
}

/// Bhaal, Lord of Murder — {2}{B}{R}{G} 4/4 legendary God. Indestructible
/// while your life is at most half your starting life. Whenever another
/// nontoken creature you control dies, put a +1/+1 counter on target creature
/// and goad it.
pub fn bhaal_lord_of_murder() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "As long as your life total is less than or equal to half your starting life total, Bhaal has indestructible.",
            StaticEffect::WhileCondition {
                condition: Predicate::PlayerLifeAtMostHalfStarting { who: PlayerRef::You },
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Indestructible }),
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Goad { what: Selector::Target(0) },
            ]),
        }],
        ..legend("Bhaal, Lord of Murder", cost(&[generic(2), b(), r(), g()]), vec![CreatureType::God], 4, 4)
    }
}

/// Kroxa and Kunoros — {3}{R}{W}{B} 6/6 legendary Elder Giant Dog with
/// vigilance, menace, lifelink. Entering or attacking: you may exile five cards
/// from your graveyard; when you do, return target creature card from your
/// graveyard to the battlefield.
pub fn kroxa_and_kunoros() -> CardDefinition {
    let reanimate = || {
        Effect::If {
            cond: Predicate::ValueAtLeast(Value::GraveyardSizeOf(PlayerRef::You), Value::Const(5)),
            then: Box::new(may(
                "exile five cards from your graveyard to return a creature card",
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: crate::card::Zone::Graveyard,
                                filter: R::Any,
                            }),
                            count: Box::new(Value::Const(5)),
                        },
                        to: ZoneDest::Exile,
                    },
                    Effect::Reflexive {
                        body: Box::new(Effect::Move {
                            what: target_filtered(R::Creature.from_your_graveyard()),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                    },
                ]),
            )),
            else_: Box::new(Effect::Noop),
        }
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Menace, Keyword::Lifelink],
        triggered_abilities: vec![etb(reanimate()), on_attack(reanimate())],
        ..legend(
            "Kroxa and Kunoros",
            cost(&[generic(3), r(), w(), b()]),
            vec![CreatureType::Elder, CreatureType::Giant, CreatureType::Dog],
            6,
            6,
        )
    }
}

/// Search for Dagger — {1}{W} enchantment. Whenever your commander enters or
/// attacks, look at the top six cards; you may reveal a legendary creature
/// card and put it into your hand; the rest go to the bottom at random.
pub fn search_for_dagger() -> CardDefinition {
    let commander = || Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter: R::IsCommander.and(R::OwnedByYou),
    };
    let dig = || {
        Effect::LookPickToHand(Box::new(crate::effect::LookPick {
            count: Value::Const(6),
            pick_filter: Some(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
            rest_bottom_random: true,
            optional: true,
            ..Default::default()
        }))
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(commander()),
                effect: dig(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(commander()),
                effect: dig(),
            },
        ],
        ..permanent("Search for Dagger", cost(&[generic(1), w()]), CardType::Enchantment)
    }
}

/// Flagstones of Trokair — legendary land. {T}: add {W}. When it's put into a
/// graveyard from the battlefield, you may search for a Plains card and put it
/// onto the battlefield tapped.
pub fn flagstones_of_trokair() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![crate::sets::tap_add(Color::White)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: may(
                "search for a Plains card",
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasLandType(LandType::Plains),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            ),
        }],
        ..permanent("Flagstones of Trokair", ManaCost::default(), CardType::Land)
    }
}

/// Loyal Inventor — {2}{U} 2/3 Human Artificer with vigilance. When it enters,
/// you may search for an artifact card: to your hand if you control an
/// Assassin, otherwise on top of your library.
pub fn loyal_inventor() -> CardDefinition {
    let search = |to: ZoneDest| Effect::Search { who: PlayerRef::You, filter: R::Artifact, to };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(may(
            "search for an artifact card",
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Assassin)))),
                then: Box::new(search(ZoneDest::Hand(PlayerRef::You))),
                else_: Box::new(search(ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top })),
            },
        ))],
        ..creature("Loyal Inventor", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 3)
    }
}

/// Angrath, the Flame-Chained — {3}{B}{R} planeswalker, loyalty 4. +1: each
/// opponent discards a card and loses 2 life. −3: threaten target creature;
/// sacrifice it at the next end step if its mana value is 3 or less. −8: each
/// opponent loses life equal to the cards in their graveyard.
pub fn angrath_the_flame_chained() -> CardDefinition {
    let loyalty = |cost: i32, effect: Effect| crate::card::LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() };
    let each_opp = || Selector::Player(PlayerRef::EachOpponent);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Angrath],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::Discard { who: each_opp(), amount: Value::ONE, random: false },
                    Effect::LoseLife { who: each_opp(), amount: Value::Const(2) },
                ]),
            ),
            loyalty(
                -3,
                threaten(
                    R::Creature,
                    vec![Effect::If {
                        cond: Predicate::ValueAtMost(Value::ManaValueOf(Box::new(Selector::Target(0))), Value::Const(3)),
                        then: Box::new(Effect::SacrificeAtNextEndStep { what: Selector::Target(0) }),
                        else_: Box::new(Effect::Noop),
                    }],
                ),
            ),
            loyalty(
                -8,
                Effect::ForEachOpponent {
                    body: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::GraveyardSizeOf(PlayerRef::Triggerer),
                    }),
                },
            ),
        ],
        ..permanent("Angrath, the Flame-Chained", cost(&[generic(3), b(), r()]), CardType::Planeswalker)
    }
}

/// Fumulus, the Infestation — {3}{B} 2/2 legendary flying, deathtouch Vampire
/// Insect. Whenever a player sacrifices a nontoken creature, create a 1/1 black
/// flying Insect. Whenever an Insect, Leech, Slug or Worm you control attacks,
/// defending player loses 1 life and you gain 1 life.
pub fn fumulus_the_infestation() -> CardDefinition {
    let insect = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Insect", 1, 1, vec![Color::Black], vec![CreatureType::Insect])
    };
    let crawlers = [CreatureType::Insect, CreatureType::Leech, CreatureType::Slug, CreatureType::Worm]
        .into_iter()
        .map(R::HasCreatureType)
        .reduce(|a, b| a.or(b))
        .expect("four types");
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
                ),
                effect: make(1, insect),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: crawlers }),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE },
                    gain(1),
                ]),
            },
        ],
        ..legend(
            "Fumulus, the Infestation",
            cost(&[generic(3), b()]),
            vec![CreatureType::Vampire, CreatureType::Insect],
            2,
            2,
        )
    }
}

/// Drafna, Founder of Lat-Nam — {1}{U} 2/1 legendary Human Artificer Advisor.
/// {1}{U}: return target artifact you control to its owner's hand. {3}, {T}:
/// copy target artifact spell you control.
pub fn drafna_founder_of_lat_nam() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                effect: Effect::Move {
                    what: target_filtered(yours(R::Artifact)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::CopySpell {
                    what: target_filtered(R::IsSpellOnStack.and(R::Artifact).and(R::ControlledByYou)),
                    count: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Drafna, Founder of Lat-Nam",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Advisor],
            2,
            1,
        )
    }
}

/// Saradoc, Master of Buckland — {3}{W} 2/4 legendary Halfling Citizen.
/// Whenever it or another nontoken creature you control with power 2 or less
/// enters, create a 1/1 white Halfling. Tap two other untapped Halflings you
/// control: +2/+0 and lifelink until end of turn.
pub fn saradoc_master_of_buckland() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken).and(R::PowerAtMost(2)),
                },
            ),
            effect: make(1, token("Halfling", 1, 1, vec![Color::White], vec![CreatureType::Halfling])),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::HasCreatureType(CreatureType::Halfling).and(R::OtherThanSource), 2)),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: Selector::This, power: Value::Const(2), toughness: Value::Const(0), duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Saradoc, Master of Buckland",
            cost(&[generic(3), w()]),
            vec![CreatureType::Halfling, CreatureType::Citizen],
            2,
            4,
        )
    }
}

/// Zoyowa's Justice — {1}{R} instant. The owner of target artifact or creature
/// with mana value 1 or greater shuffles it into their library, then discovers
/// X, X its mana value.
pub fn zoyowas_justice() -> CardDefinition {
    spell(
        "Zoyowa's Justice",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::AsPlayer {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::Artifact.or(R::Creature).and(R::ManaValueAtLeast(1))),
                        to: ZoneDest::Library {
                            who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                            pos: crate::effect::LibraryPosition::Shuffled,
                        },
                    },
                    // The card's mana value read from the library it was
                    // shuffled into.
                    Effect::Discover { n: Value::ManaValueOf(Box::new(Selector::Target(0))), filter: None },
                ])),
            },
        ]),
    )
}

fn non_legendary_copies() -> Effect {
    let mut copies = crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::Const(2), Selector::This);
    if let Effect::CreateTokenCopyOf { non_legendary, .. } = &mut copies {
        *non_legendary = true;
    }
    copies
}

/// The Notary Hobbits — {3}{G}{G} 1/1 legendary Halfling Advisor. When they
/// enter, if they're not a token, create two non-legendary token copies. {T}:
/// add {C} for each Halfling you control.
pub fn the_notary_hobbits() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::NotToken }),
            effect: non_legendary_copies(),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::count(Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Halfling))))),
            },
            ..Default::default()
        }],
        ..legend("The Notary Hobbits", cost(&[generic(3), g(), g()]), vec![CreatureType::Halfling, CreatureType::Advisor], 1, 1)
    }
}

/// Rite of Harmony — {G}{W} instant, flashback {2}{G}{W}. Until end of turn,
/// whenever a creature or enchantment you control enters, draw a card.
pub fn rite_of_harmony() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(2), g(), w()]))],
        ..spell(
            "Rite of Harmony",
            cost(&[g(), w()]),
            CardType::Instant,
            Effect::TriggersThisTurn {
                triggered: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::Creature.or(R::Enchantment),
                        },
                    ),
                    effect: draw(1),
                }],
            },
        )
    }
}

/// Bonus Round — {1}{R}{R} sorcery. Until end of turn, whenever a player casts
/// an instant or sorcery spell, that player copies it.
pub fn bonus_round() -> CardDefinition {
    spell(
        "Bonus Round",
        cost(&[generic(1), r(), r()]),
        CardType::Sorcery,
        Effect::TriggersThisTurn {
            triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                }),
                effect: Effect::AsPlayer {
                    who: PlayerRef::Triggerer,
                    body: Box::new(Effect::CopySpell { what: Selector::TriggerSource, count: Value::ONE }),
                },
            }],
        },
    )
}

/// Uncover the Moon-Letters — {3}{U} enchantment. Whenever you cast a
/// noncreature spell, you may draw X cards, X the mana spent on it; if you
/// do, discard two cards.
pub fn uncover_the_moon_letters() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(cast_is_noncreature()),
            effect: may(
                "draw X cards, then discard two",
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::CastSpellManaSpent },
                    Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
                ]),
            ),
        }],
        ..permanent("Uncover the Moon-Letters", cost(&[generic(3), u()]), CardType::Enchantment)
    }
}

/// Brazen Cannonade — {3}{R} enchantment. Whenever an attacking creature you
/// control dies, 2 damage to each opponent. Raid — at the beginning of each of
/// your postcombat main phases, if you attacked this turn, exile your top card;
/// you may play it until end of combat on your next turn.
pub fn brazen_cannonade() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsAttacking },
                ),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::YourControl)
                    .with_filter(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::UntilEndOfCombatOnYourNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            },
        ],
        ..permanent("Brazen Cannonade", cost(&[generic(3), r()]), CardType::Enchantment)
    }
}

/// Offspring's Revenge — {2}{R}{W}{B} enchantment. At the beginning of combat
/// on your turn, exile target red, white or black creature card from your
/// graveyard; create a 1/1 token copy of it that gains haste until your next
/// turn.
pub fn offsprings_revenge() -> CardDefinition {
    let mut copy = crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, Selector::Target(0));
    if let Effect::CreateTokenCopyOf { override_pt, .. } = &mut copy {
        *override_pt = Some((1, 1));
    }
    CardDefinition {
        triggered_abilities: vec![at_your(
            TurnStep::BeginCombat,
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Creature
                            .and(R::HasColor(Color::Red).or(R::HasColor(Color::White)).or(R::HasColor(Color::Black)))
                            .from_your_graveyard(),
                    ),
                    to: ZoneDest::Exile,
                },
                copy,
                // "It gains haste until your next turn" — not a copiable keyword.
                Effect::GrantKeyword {
                    what: Selector::LastCreatedToken,
                    keyword: Keyword::Haste,
                    duration: Duration::UntilNextTurn,
                },
            ]),
        )],
        ..permanent("Offspring's Revenge", cost(&[generic(2), r(), w(), b()]), CardType::Enchantment)
    }
}

/// Hex Magic — {2}{R} sorcery — Arcane. Exile your hand, draw that many; you
/// may play the exiled cards until the end of your next turn.
pub fn hex_magic() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { spell_subtypes: vec![crate::card::SpellSubtype::Arcane], ..Default::default() },
        ..spell(
            "Hex Magic",
            cost(&[generic(2), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move {
                    what: Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Hand, filter: R::Any },
                    to: ZoneDest::Exile,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
                Effect::Draw { who: Selector::You, amount: Value::count(Selector::LastMoved) },
            ]),
        )
    }
}

/// Key to the Side-Door — {1} artifact. {2}, {T}: target creature can't be
/// blocked this turn. {1}, {T}, discard a legendary card sharing a name with a
/// legendary permanent you control: draw two cards.
pub fn key_to_the_side_door() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                discard_cost: Some((
                    R::HasSupertype(Supertype::Legendary).and(R::Not(Box::new(R::NameNotSharedWithYourPermanents))),
                    1,
                )),
                effect: draw(2),
                ..Default::default()
            },
        ],
        ..permanent("Key to the Side-Door", cost(&[generic(1)]), CardType::Artifact)
    }
}

/// Demonic Gifts — {1}{B} instant. Until end of turn, target creature gets
/// +2/+0 and has "when this creature dies, return it to the battlefield under
/// its owner's control."
pub fn demonic_gifts() -> CardDefinition {
    spell(
        "Demonic Gifts",
        cost(&[generic(1), b()]),
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
                trigger: Box::new(crate::effect::shortcut::on_dies(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOf(Box::new(Selector::This)), tapped: false },
                })),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Monumental Corruption — {3}{B}{B} sorcery. Target player draws X cards and
/// loses X life, X the artifacts you control.
pub fn monumental_corruption() -> CardDefinition {
    let x = || Value::count(Selector::EachPermanent(yours(R::Artifact)));
    spell(
        "Monumental Corruption",
        cost(&[generic(3), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: x() },
            Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: x() },
        ]),
    )
}

/// Fomori Vault — land. {T}: add {C}. {3}, {T}, discard a card: look at the top
/// X cards, X your artifacts; one to hand, the rest on the bottom at random.
pub fn fomori_vault() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                    count: Value::count(Selector::EachPermanent(yours(R::Artifact))),
                    rest_bottom_random: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
        ],
        ..permanent("Fomori Vault", ManaCost::default(), CardType::Land)
    }
}

/// Arcbond — {2}{R} instant. Choose target creature; whenever it's dealt
/// damage this turn, it deals that much damage to each other creature and each
/// player.
pub fn arcbond() -> CardDefinition {
    spell(
        "Arcbond",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::GrantTriggeredAbility {
            what: target_filtered(R::Creature),
            trigger: Box::new(enrage(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                    amount: Value::TriggerEventAmount,
                },
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::TriggerEventAmount },
            ]))),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Fatal Grudge — {B}{R} sorcery; as an additional cost, sacrifice a nonland
/// permanent. Each opponent sacrifices a permanent sharing a card type with
/// it. Draw a card.
pub fn fatal_grudge() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Nonland.and(R::ControlledByYou),
            count: 1,
        }],
        ..spell(
            "Fatal Grudge",
            cost(&[b(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::EachPlayerDoes {
                    who: PlayerRef::EachOpponent,
                    body: Box::new(Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::SharesCardTypeWithFirstSacrificed.and(R::ControlledByYou),
                    }),
                },
                draw(1),
            ]),
        )
    }
}

/// Outrider en-Kor — {2}{W} 2/2 Kor Rebel Knight with flanking. {0}: the next
/// 1 damage that would be dealt to it this turn is dealt to target creature
/// you control instead.
pub fn outrider_en_kor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::RedirectNextDamage {
                target: Selector::This,
                to: target_filtered(yours(R::Creature)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Outrider en-Kor",
            cost(&[generic(2), w()]),
            vec![CreatureType::Kor, CreatureType::Rebel, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Ballad of the Black Flag — {1}{U}{U} Saga. I, II, III: mill three; you may
/// put a historic card from among them into your hand. IV: historic spells
/// you cast this turn cost {2} less.
pub fn ballad_of_the_black_flag() -> CardDefinition {
    let dig = || Effect::MillThenToHand { amount: Value::Const(3), filter: R::historic(), otherwise: None };
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, dig()),
            (2, dig()),
            (3, dig()),
            (4, Effect::SpellsCostLessThisTurn { filter: R::historic(), amount: 2 }),
        ],
        ..permanent("Ballad of the Black Flag", cost(&[generic(1), u(), u()]), CardType::Enchantment)
    }
}

/// Zurgo and Ojutai — {2}{U}{R}{W} 4/4 legendary flying, haste Orc Dragon;
/// hexproof the turn it entered. Whenever one or more Dragons you control deal
/// combat damage to a player or battle, look at the top three: one to hand, the rest on
/// the bottom; you may return one of those Dragons to its owner's hand.
pub fn zurgo_and_ojutai() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        static_abilities: vec![static_ab(
            "Zurgo and Ojutai has hexproof as long as it entered this turn.",
            StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn },
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Hexproof }),
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).or_battle()
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dragon),
                })
                .once_per_batch(),
            effect: Effect::Seq(vec![
                Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                    count: Value::Const(3),
                    ..Default::default()
                })),
                may(
                    "return the Dragon to its owner's hand",
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                    },
                ),
            ]),
        }],
        ..legend(
            "Zurgo and Ojutai",
            cost(&[generic(2), u(), r(), w()]),
            vec![CreatureType::Orc, CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Irencrag Feat — {1}{R}{R}{R} sorcery. Add seven {R}. You can cast only one
/// more spell this turn.
pub fn irencrag_feat() -> CardDefinition {
    spell(
        "Irencrag Feat",
        cost(&[generic(1), r(), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::Const(7)) },
            Effect::CapSpellsThisTurn { more: 1 },
        ]),
    )
}

/// Phyrexian Censor — {2}{W} 3/3 Phyrexian Wizard. Each player can't cast more
/// than one non-Phyrexian spell each turn. Non-Phyrexian creatures enter
/// tapped.
pub fn phyrexian_censor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "Each player can't cast more than one non-Phyrexian spell each turn.",
                StaticEffect::OneNonPhyrexianSpellPerTurn,
            ),
            static_ab(
                "Non-Phyrexian creatures enter tapped.",
                StaticEffect::EntersTapped {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Phyrexian)))),
                    ),
                },
            ),
        ],
        ..creature("Phyrexian Censor", cost(&[generic(2), w()]), vec![CreatureType::Phyrexian, CreatureType::Wizard], 3, 3)
    }
}

/// Peter Parker's Camera — {1} artifact; enters with three film counters. {2},
/// {T}, remove a film counter: copy target activated or triggered ability you
/// control.
pub fn peter_parkers_camera() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Film, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Film, 1)),
            effect: Effect::CopyAbility {
                what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByYou)),
                times: Value::ONE,
            },
            ..Default::default()
        }],
        ..permanent("Peter Parker's Camera", cost(&[generic(1)]), CardType::Artifact)
    }
}

/// Olag and Miau, New Friends — {W}{B}{G} 3/3 legendary Zombie Cat. Another
/// Zombie entering grows a Cat of yours; another Cat entering grows a Zombie.
/// At your end step, each opponent loses life equal to your tapped Cats and/or
/// Zombies.
pub fn olag_and_miau_new_friends() -> CardDefinition {
    let other_enters = |kind: CreatureType, grow: CreatureType| TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(kind) },
        ),
        effect: Effect::AddCounter {
            what: target_filtered(yours(R::HasCreatureType(grow))),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
    };
    let tapped_kin = R::Tapped.and(R::HasCreatureType(CreatureType::Cat).or(R::HasCreatureType(CreatureType::Zombie)));
    CardDefinition {
        triggered_abilities: vec![
            other_enters(CreatureType::Zombie, CreatureType::Cat),
            other_enters(CreatureType::Cat, CreatureType::Zombie),
            at_your(
                TurnStep::End,
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::count(Selector::EachPermanent(yours(tapped_kin))),
                },
            ),
        ],
        ..legend(
            "Olag and Miau, New Friends",
            cost(&[w(), b(), g()]),
            vec![CreatureType::Zombie, CreatureType::Cat],
            3,
            3,
        )
    }
}

/// Harald Unites the Elves — {2}{B}{G} Saga. I: mill three; you may put an Elf
/// or Tyvar card from your graveyard onto the battlefield. II: a +1/+1 counter
/// on each Elf you control. III: this turn, whenever an Elf you control
/// attacks, target creature an opponent controls gets -1/-1.
pub fn harald_unites_the_elves() -> CardDefinition {
    let elf_or_tyvar = R::HasCreatureType(CreatureType::Elf)
        .or(R::HasPlaneswalkerType(crate::card::PlaneswalkerSubtype::Tyvar));
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::Const(3) },
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: elf_or_tyvar.from_your_graveyard(),
                        effect: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                    },
                ]),
            ),
            (
                2,
                Effect::AddCounter {
                    what: Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Elf))),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ),
            (
                3,
                Effect::TriggersThisTurn {
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                            Predicate::EntityMatches {
                                what: Selector::TriggerSource,
                                filter: R::HasCreatureType(CreatureType::Elf),
                            },
                        ),
                        effect: Effect::PumpPT {
                            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                            power: Value::Const(-1),
                            toughness: Value::Const(-1),
                            duration: Duration::EndOfTurn,
                        },
                    }],
                },
            ),
        ],
        ..permanent("Harald Unites the Elves", cost(&[generic(2), b(), g()]), CardType::Enchantment)
    }
}

/// Golden Argosy — {4} 3/6 legendary artifact Vehicle, crew 1. Whenever it
/// attacks, exile each creature that crewed it this turn; return them tapped
/// under their owners' control at the beginning of the next end step.
pub fn golden_argosy() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 3,
        toughness: 6,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![on_attack(Effect::ExileReturnToOwnerNextEndStep {
            what: Selector::EachPermanent(R::Creature.and(R::CrewedSourceThisTurn)),
            tapped: true,
        })],
        ..permanent("Golden Argosy", cost(&[generic(4)]), CardType::Artifact)
    }
}

/// Tamiyo, Compleated Sage — {2}{G}{G/U/P}{U} planeswalker, loyalty 5,
/// compleated. +1: tap up to one target artifact or creature; it doesn't untap
/// during its controller's next untap step. −X: exile target nonland permanent
/// card with mana value X from your graveyard; create a token copy of it.
/// −7: create Tamiyo's Notebook.
pub fn tamiyo_compleated_sage() -> CardDefinition {
    let loyalty = |cost: i32, effect: Effect| crate::card::LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() };
    let notebook = TokenDefinition {
        name: "Tamiyo's Notebook".into(),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Book], ..Default::default() },
        static_abilities: vec![static_ab(
            "Spells you cast cost {2} less to cast.",
            StaticEffect::CostReduction { filter: R::Any, amount: 2 },
        )],
        activated_abilities: vec![ActivatedAbility { tap_cost: true, effect: draw(1), ..Default::default() }],
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Tamiyo],
            ..Default::default()
        },
        keywords: vec![Keyword::Compleated],
        base_loyalty: 5,
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Artifact.or(R::Creature),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: Selector::Target(0) },
                        Effect::SkipNextUntap { what: Selector::Target(0) },
                    ])),
                },
            ),
            crate::card::LoyaltyAbility { condition: None,
                loyalty_cost: 0,
                x_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(
                            R::Permanent.and(R::Nonland).and(R::ManaValueExactlyXFromCost).from_your_graveyard(),
                        ),
                        to: ZoneDest::Exile,
                    },
                    crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, Selector::Target(0)),
                ]),
            },
            loyalty(-7, make(1, notebook)),
        ],
        ..permanent(
            "Tamiyo, Compleated Sage",
            cost(&[generic(2), g(), crate::mana::phyrexian_hybrid(Color::Green, Color::Blue), u()]),
            CardType::Planeswalker,
        )
    }
}

/// Teferi, Who Slows the Sunset — {2}{W}{U} planeswalker, loyalty 4. +1: up to
/// one target artifact, creature and land — untap yours, tap the rest; gain 2
/// life. −2: top three, one to hand, the rest on the bottom. −7: an emblem
/// untapping your permanents and drawing you a card on each opponent's turn.
pub fn teferi_who_slows_the_sunset() -> CardDefinition {
    let loyalty = |cost: i32, effect: Effect| crate::card::LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() };
    // "Up to one target artifact, up to one target creature, and up to one
    // target land" — a slot each (CR 601.2c); untap yours, tap the rest.
    let toggle = |slot: u8, filter: R| Effect::If {
        cond: Predicate::EntityMatches { what: Selector::Target(slot), filter: R::ControlledByYou },
        then: Box::new(Effect::Untap { what: Selector::TargetFiltered { slot, filter: filter.clone() }, up_to: None }),
        else_: Box::new(Effect::Tap { what: Selector::TargetFiltered { slot, filter } }),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Teferi],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::Seq(vec![
                            toggle(0, R::Artifact),
                            toggle(1, R::Creature),
                            toggle(2, R::Land),
                        ])),
                    },
                    gain(2),
                ]),
            ),
            loyalty(
                -2,
                Effect::LookPickToHand(Box::new(crate::effect::LookPick { count: Value::Const(3), ..Default::default() })),
            ),
            loyalty(
                -7,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Teferi, Who Slows the Sunset".into(),
                    triggered: vec![],
                    // CR 502.3 — untapped in each opponent's untap step; the
                    // draw follows theirs without using the stack (ruling).
                    statics: vec![
                        crate::card::StaticAbility {
                            description: "Untap all permanents you control during each opponent's untap step.",
                            effect: crate::effect::StaticEffect::UntapAllYoursEachUntapStep,
                        },
                        crate::card::StaticAbility {
                            description: "You draw a card during each opponent's draw step.",
                            effect: crate::effect::StaticEffect::DrawDuringEachOpponentsDrawStep,
                        },
                    ],
                },
            ),
        ],
        ..permanent("Teferi, Who Slows the Sunset", cost(&[generic(2), w(), u()]), CardType::Planeswalker)
    }
}

/// Lagrella, the Magpie — {G}{W}{U} 2/3 legendary Human Soldier. When it
/// enters, exile any number of other target creatures controlled by different
/// players until Lagrella leaves the battlefield; one returning under your
/// control gets two +1/+1 counters (put on as it returns, not by a trigger).
pub fn lagrella_the_magpie() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ForEachPlayerTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature.and(R::OtherThanSource),
                effect: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::Target(0),
                    return_to: crate::card::ExileReturnZone::BattlefieldCountersIfYours(2),
                }),
            }),
        })],
        ..legend(
            "Lagrella, the Magpie",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Toluz, Clever Conductor — {W/U}{U}{U/B} 3/1 legendary Human Rogue. When it
/// enters, it connives. Whenever you discard cards, exile them from your
/// graveyard. When it dies, the cards exiled with it go to their owners' hands.
pub fn toluz_clever_conductor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(crate::effect::shortcut::connive(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
                effect: Effect::Move { what: Selector::TriggerSource, to: ZoneDest::ExileWithSourceStamp },
            },
            crate::effect::shortcut::on_dies(Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Exile,
                    filter: R::ExiledWithSource,
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        ],
        ..legend(
            "Toluz, Clever Conductor",
            cost(&[hybrid(Color::White, Color::Blue), u(), hybrid(Color::Blue, Color::Black)]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            1,
        )
    }
}

/// Amareth, the Lustrous — {3}{G}{W}{U} 6/6 legendary flying Dragon. Whenever
/// another permanent you control enters, look at the top card of your library;
/// if it shares a card type with that permanent, you may put it into your hand.
pub fn amareth_the_lustrous() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    let shares = Predicate::Any(
        [CardType::Artifact, CardType::Creature, CardType::Enchantment, CardType::Land, CardType::Planeswalker, CardType::Battle]
            .into_iter()
            .map(|t| {
                Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCardType(t.clone()) },
                    Predicate::EntityMatches { what: top(), filter: R::HasCardType(t) },
                ])
            })
            .collect(),
    );
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours),
            effect: Effect::If {
                cond: shares,
                then: Box::new(may(
                    "reveal it and put it into your hand",
                    Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) },
                )),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..legend(
            "Amareth, the Lustrous",
            cost(&[generic(3), g(), w(), u()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Sin Prodder — {2}{R} 3/2 menace Devil. At the beginning of your upkeep,
/// reveal the top card: any opponent may have it put into your graveyard and
/// take damage equal to its mana value; otherwise it goes to your hand.
pub fn sin_prodder() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![at_your(
            TurnStep::Upkeep,
            Effect::AnyPlayerMayAccept {
                who: PlayerRef::EachOpponent,
                prompt: "Put Sin Prodder's revealed card into its owner's graveyard and take damage equal to its mana value?".into(),
                accepted: Box::new(Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::AcceptingPlayer),
                        amount: Value::ManaValueOf(Box::new(top())),
                    },
                    Effect::Move { what: top(), to: ZoneDest::Graveyard },
                ])),
                otherwise: Box::new(Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) }),
            },
        )],
        ..creature("Sin Prodder", cost(&[generic(2), r()]), vec![CreatureType::Devil], 3, 2)
    }
}
