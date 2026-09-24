//! Commander: the cards the **Adaptive Enchantment** precon (C18, Estrid, the
//! Masked) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_estrid.rs`.
//!
//! Residuals (each also on its card):
//! - **Estrid, the Masked** — the −7's Auras land on hosts the engine picks.
//! - **Genesis Storm** — the revealed permanent always goes onto the
//!   battlefield ("you may" isn't offered).
//! - **Myth Unbound** — the discount counts both partners' casts together.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{blocks, etb, exalted, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, u, w};
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

fn aura(name: &'static str, mana: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura_card() -> R {
    R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
}

fn aura_or_equipment() -> R {
    aura_card().or(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment))
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// "When you cast this spell, copy it for each time you've cast your
/// commander from the command zone this game."
fn commander_storm() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
        effect: Effect::CopySpell {
            what: Selector::This,
            count: Value::CommanderCastsFromCommandZone(PlayerRef::You),
        },
    }
}

fn mask_token() -> TokenDefinition {
    TokenDefinition {
        name: "Mask".into(),
        card_types: vec![CardType::Enchantment],
        colors: vec![Color::White],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::UmbraArmor],
        ..Default::default()
    }
}

/// Estrid, the Masked — +2 untaps your enchanted permanents; −1 masks another
/// permanent with an umbra-armored Aura token; −7 mills seven and returns your
/// enchantments, then your Auras. ⚠ The −7's Auras go on engine-picked hosts.
pub fn estrid_the_masked() -> CardDefinition {
    CardDefinition {
        name: "Estrid, the Masked",
        cost: cost(&[generic(1), g(), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Estrid], ..Default::default() },
        base_loyalty: 3,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Untap { what: yours(R::IsEnchanted), up_to: None },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::CreateTokenAttachedTo {
                    target: target_filtered(R::Permanent.and(R::OtherThanSource)),
                    definition: Arc::new(mask_token()),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::Const(7) },
                    Effect::Move {
                        what: Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Enchantment.and(R::Not(Box::new(aura_card()))),
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::PutOntoBattlefieldAttached {
                        zones: vec![Zone::Graveyard],
                        filter: aura_card(),
                        host: None,
                        max: None,
                        creatures_only: false,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Arixmethes, Slumbering Isle — enters tapped with five slumber counters, a
/// land (not a creature) while any remain; each spell you cast may remove one;
/// taps for {G}{U}.
pub fn arixmethes_slumbering_isle() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        enters_with_counters: Some((CounterType::Slumber, Value::Const(5))),
        static_abilities: vec![
            StaticAbility {
                description: "Arixmethes enters tapped.",
                effect: StaticEffect::EntersTapped { applies_to: Selector::This },
            },
            StaticAbility {
                description: "As long as Arixmethes has a slumber counter on it, it's a land.",
                effect: StaticEffect::WhileCountersAtLeast {
                    kind: CounterType::Slumber,
                    n: 1,
                    inner: Box::new(StaticEffect::AddCardTypeToMatching {
                        applies_to: Selector::This,
                        card_type: CardType::Land,
                        artifact_subtype: None,
                    }),
                },
            },
            StaticAbility {
                description: "As long as Arixmethes has a slumber counter on it, it's not a creature.",
                effect: StaticEffect::NotCreatureUnless {
                    condition: Predicate::Not(Box::new(Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::Slumber,
                        n: 1,
                    })),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Remove a slumber counter from Arixmethes?".into(),
                body: Box::new(Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Slumber,
                    amount: Value::ONE,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Blue]) },
            ..Default::default()
        }],
        ..creature(
            "Arixmethes, Slumbering Isle",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Kraken],
            12,
            12,
        )
    }
}

/// Bruna, Light of Alabaster — flying, vigilance; attacking or blocking, you
/// may gather Auras onto it from the battlefield, your graveyard and hand.
pub fn bruna_light_of_alabaster() -> CardDefinition {
    let gather = || {
        Effect::Seq(vec![
            Effect::AttachAnyNumberTo { what: Selector::EachPermanent(aura_card()), to: Selector::This },
            Effect::PutOntoBattlefieldAttached {
                zones: vec![Zone::Graveyard, Zone::Hand],
                filter: aura_card(),
                host: Some(Selector::This),
                max: None,
                creatures_only: false,
            },
        ])
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![on_attack(gather()), blocks(gather())],
        ..creature(
            "Bruna, Light of Alabaster",
            cost(&[generic(3), w(), w(), u()]),
            vec![CreatureType::Angel],
            5,
            5,
        )
    }
}

/// Creeping Renaissance — return every card of a chosen permanent type from
/// your graveyard to hand. Flashback {5}{G}{G}.
pub fn creeping_renaissance() -> CardDefinition {
    let back = |t: CardType| Effect::Move {
        what: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::HasCardType(t) },
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), g(), g()]))],
        ..spell(
            "Creeping Renaissance",
            cost(&[generic(3), g(), g()]),
            CardType::Sorcery,
            Effect::ChooseMode(vec![
                back(CardType::Creature),
                back(CardType::Land),
                back(CardType::Enchantment),
                back(CardType::Artifact),
                back(CardType::Planeswalker),
                back(CardType::Battle),
            ]),
        )
    }
}

/// Eel Umbra — flash; +1/+1 and umbra armor.
pub fn eel_umbra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::UmbraArmor],
        ..aura("Eel Umbra", cost(&[generic(1), u()]), EquipBonus { power: 1, toughness: 1, ..Default::default() })
    }
}

/// Elderwood Scion — trample, lifelink; your spells targeting it cost {2}
/// less, your opponents' {2} more.
pub fn elderwood_scion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Lifelink],
        static_abilities: vec![
            StaticAbility {
                description: "Spells you cast that target this creature cost {2} less to cast.",
                effect: StaticEffect::CostReductionForYourSpellsTargetingThis { amount: 2 },
            },
            StaticAbility {
                description: "Spells your opponents cast that target this creature cost {2} more to cast.",
                effect: StaticEffect::TaxOpponentSpellsTargetingThis { amount: 2 },
            },
        ],
        ..creature("Elderwood Scion", cost(&[generic(3), g(), w()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Empyrial Storm — a 4/4 flying Angel, copied once per cast of your
/// commander from the command zone.
pub fn empyrial_storm() -> CardDefinition {
    let angel = TokenDefinition {
        name: "Angel".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![commander_storm()],
        ..spell(
            "Empyrial Storm",
            cost(&[generic(4), w(), w()]),
            CardType::Sorcery,
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(angel) },
        )
    }
}

/// Epic Proportions — flash; +5/+5 and trample.
pub fn epic_proportions() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..aura(
            "Epic Proportions",
            cost(&[generic(4), g(), g()]),
            EquipBonus { power: 5, toughness: 5, keywords: vec![Keyword::Trample], ..Default::default() },
        )
    }
}

/// Estrid's Invocation — enters as a copy of an enchantment you control that
/// may blink itself each upkeep.
pub fn estrids_invocation() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(crate::card::EntersAsCopy {
            filter: R::Enchantment.and(R::ControlledByYou),
            extra_triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Exile this enchantment and return it?".into(),
                    body: Box::new(Effect::ExileAndReturnToOwner { what: Selector::This }),
                },
            }],
            ..Default::default()
        }),
        ..enchantment("Estrid's Invocation", cost(&[generic(2), u()]))
    }
}

/// Finest Hour — exalted; a lone attacker in the first combat untaps and a
/// combat phase follows this one.
pub fn finest_hour() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            exalted(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(Predicate::AttackingAlone),
                effect: Effect::If {
                    cond: Predicate::IsFirstCombatPhaseThisTurn,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: Selector::TriggerSource, up_to: None },
                        Effect::AdditionalCombatPhase { count: Value::ONE },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..enchantment("Finest Hour", cost(&[generic(2), g(), w(), u()]))
    }
}

/// Genesis Storm — reveal until a nonland permanent card and put it onto the
/// battlefield, copied once per cast of your commander from the command zone.
/// ⚠ The card always goes onto the battlefield.
pub fn genesis_storm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![commander_storm()],
        ..spell(
            "Genesis Storm",
            cost(&[generic(4), g(), g()]),
            CardType::Sorcery,
            Effect::RevealUntilOneToBattlefieldRestBottom {
                filter: R::PermanentCard.and(R::Nonland),
                damage_controller: false,
            },
        )
    }
}

/// Heavenly Blademaster — flying, double strike; entering, you may attach any
/// number of your Auras and Equipment to it; your other creatures get +1/+1
/// per attachment on it.
pub fn heavenly_blademaster() -> CardDefinition {
    let per = || Value::AttachmentsOn { what: Box::new(Selector::This), filter: aura_or_equipment() };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike],
        triggered_abilities: vec![etb(Effect::AttachAnyNumberTo {
            what: yours(aura_or_equipment()),
            to: Selector::This,
        })],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+1 for each Aura and Equipment attached to this creature.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                power: per(),
                toughness: per(),
            },
        }],
        ..creature("Heavenly Blademaster", cost(&[generic(5), w()]), vec![CreatureType::Angel], 3, 6)
    }
}

/// Loyal Unicorn — vigilance; lieutenant: with your commander out, your
/// combats prevent combat damage to your creatures, and the rest get vigilance.
pub fn loyal_unicorn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl)
                .with_filter(Predicate::ControlsOwnCommander { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::PreventCombatDamageToTargetThisTurn { target: yours(R::Creature) },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature("Loyal Unicorn", cost(&[generic(3), w()]), vec![CreatureType::Unicorn], 3, 4)
    }
}

/// Myth Unbound — your commander costs {1} less per cast from the command
/// zone; whenever it's put into the command zone, draw a card. ⚠ Partners'
/// casts are counted together.
pub fn myth_unbound() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your commander costs {1} less to cast for each time it's been cast from the command zone this game.",
            effect: StaticEffect::CostReductionByValue {
                filter: R::IsCommander,
                amount: Value::CommanderCastsFromCommandZone(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommanderPutIntoCommandZone, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..enchantment("Myth Unbound", cost(&[generic(2), g()]))
    }
}

/// Nylea's Colossus — constellation: double target creature's power and
/// toughness until end of turn.
pub fn nyleas_colossus() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
            ),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::PowerOf(Box::new(Selector::Target(0))),
                toughness: Value::ToughnessOf(Box::new(Selector::Target(0))),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Nylea's Colossus", cost(&[generic(6), g()]), vec![CreatureType::Giant], 6, 6)
    }
}

/// Octopus Umbra — base 8/8, and attacking may tap a creature with power 8 or
/// less; umbra armor.
pub fn octopus_umbra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::UmbraArmor],
        ..aura(
            "Octopus Umbra",
            cost(&[generic(3), u(), u()]),
            EquipBonus {
                set_base_pt: Some((8, 8)),
                triggered_abilities: vec![on_attack(Effect::MayDo {
                    description: "Tap target creature with power 8 or less?".into(),
                    body: Box::new(Effect::Tap { what: target_filtered(R::Creature.and(R::PowerAtMost(8))) }),
                })],
                ..Default::default()
            },
        )
    }
}

/// Ravenous Slime — can't be blocked by power 2 or less; an opponent's dying
/// creature is exiled instead and grows it by that creature's power.
pub fn ravenous_slime() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerAtMost(2)],
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, instead exile it and put +1/+1 counters equal to its power on this.",
            effect: StaticEffect::ExileDyingOpponentCreaturesGrowingThis,
        }],
        ..creature("Ravenous Slime", cost(&[generic(2), g()]), vec![CreatureType::Ooze], 1, 1)
    }
}

/// Tuvasa the Sunlit — +1/+1 per enchantment you control; your first
/// enchantment spell each turn draws a card.
pub fn tuvasa_the_sunlit() -> CardDefinition {
    let n = || Value::CountOf(Box::new(yours(R::Enchantment)));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Tuvasa gets +1/+1 for each enchantment you control.",
            effect: StaticEffect::PumpPTByValue { applies_to: Selector::This, power: n(), toughness: n() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
                Predicate::SpellsOfTypeCastThisTurnAtMost {
                    who: PlayerRef::You,
                    card_type: CardType::Enchantment,
                    n: 1,
                },
            ])),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Tuvasa the Sunlit",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Shaman],
            1,
            1,
        )
    }
}
