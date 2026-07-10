//! Modern Horizons 3 (MH3), batch 4 — Devoid Eldrazi (bounce/exile riders,
//! kicker choose-one/both, emerge tap-lock), a kicker exile with a mana-value
//! lifegain rider, and an umbra-armor Aura. Tests in `tests/mh3d.rs`.

use crate::card::{
    ActivatedAbility, Adventure, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    Predicate, Prototype, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{emerge, etb, on_cast, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest, ZoneRef};
use crate::mana::{b, colorless, cost, g, generic, r, u, w, Color};

/// Back for the MH3 mono-U modal DFC: "As this land enters, you may pay 3 life.
/// If you don't, it enters tapped. {T}: Add {U}."
fn u_pain_land(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
                Effect::Tap { what: Selector::This },
            ]),
        }],
        activated_abilities: vec![super::tap_add(Color::Blue)],
        ..Default::default()
    }
}

/// Ugin's Binding — {2}{U} Devoid instant. Bounce a nonland permanent you don't
/// control. From the graveyard, casting a colorless spell of mana value 7+ lets
/// you exile it to bounce every nonland permanent you don't control.
pub fn ugins_binding() -> CardDefinition {
    CardDefinition {
        name: "Ugin's Binding",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Move {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Colorless.and(R::ManaValueAtLeast(7)),
                },
            ),
            effect: Effect::MayDo {
                description: "Exile Ugin's Binding from your graveyard to bounce all?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: Selector::This },
                    Effect::Move {
                        what: Selector::EachMatching {
                            zone: ZoneRef::Battlefield,
                            filter: R::Permanent.and(R::Nonland).and(R::ControlledByOpponent),
                        },
                        to: ZoneDest::Hand(PlayerRef::EachOpponent),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Abstruse Appropriation — {2}{W}{B} Devoid instant. Exile a nonland
/// permanent; you may cast it for as long as it stays exiled, spending any
/// mana for its cost (the "colorless as any color" rider is generalized).
pub fn abstruse_appropriation() -> CardDefinition {
    CardDefinition {
        name: "Abstruse Appropriation",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Permanent.and(R::Nonland)) },
            Effect::GrantMayPlay {
                what: Selector::Target(0),
                duration: MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
        ]),
        ..Default::default()
    }
}

/// Expel the Unworthy — {1}{W} sorcery, Kicker {2}{W}. Exile a creature of mana
/// value 3 or less (any creature if kicked); its controller gains life equal to
/// its mana value.
pub fn expel_the_unworthy() -> CardDefinition {
    CardDefinition {
        name: "Expel the Unworthy",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(2), w()]))],
        // Gain life first (while the target still has a controller), then exile.
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Exile { what: target_filtered(R::Creature) }),
                else_: Box::new(Effect::Exile {
                    what: target_filtered(R::Creature.and(R::ManaValueAtMost(3))),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Twisted Riddlekeeper — {8} 5/5 Eldrazi Sphinx with flying and Emerge
/// {5}{C}{U}. When cast, tap up to two target permanents and stun each.
pub fn twisted_riddlekeeper() -> CardDefinition {
    CardDefinition {
        name: "Twisted Riddlekeeper",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Sphinx],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(emerge(cost(&[generic(5), colorless(1), u()]))),
        triggered_abilities: vec![on_cast(Effect::ApplyToTargets {
            max_targets: 2,
            filter: R::Permanent,
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Depth Defiler — {3}{U}{U} 3/5 Devoid Eldrazi, Kicker {C}. When cast, choose
/// one — bounce a creature, or you draw two then discard a card. If
/// kicked, choose both.
pub fn depth_defiler() -> CardDefinition {
    let bounce = Effect::Move {
        what: target_filtered(R::Creature),
        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
    };
    let draw_discard = Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
    ]);
    CardDefinition {
        name: "Depth Defiler",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Devoid, Keyword::Kicker(cost(&[colorless(1)]))],
        triggered_abilities: vec![on_cast(Effect::If {
            cond: Predicate::CastSpellWasKicked,
            then: Box::new(Effect::Seq(vec![bounce.clone(), draw_discard.clone()])),
            else_: Box::new(Effect::ChooseMode(vec![bounce, draw_discard])),
        })],
        ..Default::default()
    }
}

/// Dog Umbra — {1}{W} flash Aura. Enchant creature with umbra armor. (The
/// "can't attack/block while another player controls it" rider is dropped.)
pub fn dog_umbra() -> CardDefinition {
    CardDefinition {
        name: "Dog Umbra",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::UmbraArmor],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
        },
        ..Default::default()
    }
}

/// Thief of Existence — {1}{C}{G} 3/4 Devoid Eldrazi. ETB: exile up to one
/// noncreature, nonland permanent an opponent controls with mana value 4 or
/// less; if you do, it gains "When this leaves the battlefield, draw a card."
pub fn thief_of_existence() -> CardDefinition {
    CardDefinition {
        name: "Thief of Existence",
        cost: cost(&[generic(1), colorless(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: R::Permanent
                .and(R::Noncreature)
                .and(R::Nonland)
                .and(R::ControlledByOpponent)
                .and(R::ManaValueAtMost(4)),
            effect: Box::new(Effect::Seq(vec![
                Effect::Exile { what: Selector::Target(0) },
                Effect::GrantTriggeredAbility {
                    what: Selector::This,
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::PermanentLeavesBattlefield,
                            EventScope::SelfSource,
                        ),
                        effect: Effect::Draw {
                            who: Selector::Player(PlayerRef::You),
                            amount: Value::Const(1),
                        },
                    }),
                    duration: Duration::Permanent,
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Depth Charge Colossus — {9} 9/9 Dreadnought artifact creature, Prototype
/// {4}{U}{U} — 6/6. Doesn't untap during your untap step; {3}: untap it.
pub fn depth_charge_colossus() -> CardDefinition {
    CardDefinition {
        name: "Depth Charge Colossus",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dreadnought],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        prototype: Some(Box::new(Prototype { cost: cost(&[generic(4), u(), u()]), power: 6, toughness: 6 })),
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Amphibian Downpour — {2}{U} flash Aura with Storm. Enchanted creature loses
/// all abilities and is a blue Frog with base power and toughness 1/1.
pub fn amphibian_downpour() -> CardDefinition {
    CardDefinition {
        name: "Amphibian Downpour",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Storm],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
        },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((1, 1)),
            set_creature_types: Some(vec![CreatureType::Frog]),
            set_colors: Some(vec![Color::Blue]),
            remove_abilities: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Herigast, Erupting Nullkite — {9} 6/6 Eldrazi Dragon with flying and Emerge
/// {6}{R}{R}. When cast, you may exile your hand; if you do, draw three cards.
/// (The "each creature spell you cast has emerge" static is omitted.)
pub fn herigast_erupting_nullkite() -> CardDefinition {
    CardDefinition {
        name: "Herigast, Erupting Nullkite",
        cost: cost(&[generic(9)]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Dragon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(emerge(cost(&[generic(6), r(), r()]))),
        triggered_abilities: vec![on_cast(Effect::MayDo {
            description: "Exile your hand to draw three cards?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Exile {
                    what: Selector::EachMatching {
                        zone: ZoneRef::Hand(PlayerRef::You),
                        filter: R::Any,
                    },
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            ])),
        })],
        ..Default::default()
    }
}

/// Ondu Knotmaster // Throw a Line — {2}{W}{B} 2/2 Kor Rogue with lifelink.
/// Whenever another modified creature you control dies, put two +1/+1 counters
/// on it. Adventure: Throw a Line {W}{B} — distribute two +1/+1 counters among
/// one or two target creatures.
pub fn ondu_knotmaster() -> CardDefinition {
    CardDefinition {
        name: "Ondu Knotmaster",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsModified,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Throw a Line",
            cost: cost(&[w(), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DistributeCounters {
                total: Value::Const(2),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature,
                max_targets: 2,
            },
        })),
        ..Default::default()
    }
}

/// Hydroelectric Specimen // Hydroelectric Laboratory — {2}{U} 1/4 Weird with
/// flash. ETB: you may change the target of a single-target instant or sorcery
/// to this creature. Back is a mono-U pain land.
pub fn hydroelectric_specimen() -> CardDefinition {
    CardDefinition {
        name: "Hydroelectric Specimen",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Weird], ..Default::default() },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Change the target of an instant or sorcery to Hydroelectric Specimen?"
                .into(),
            body: Box::new(Effect::RedirectSpellTargetToSelf {
                what: target_filtered(R::IsSpellOnStack.and(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                )),
            }),
        })],
        back_face: Some(Box::new(u_pain_land("Hydroelectric Laboratory"))),
        ..Default::default()
    }
}

/// Eladamri, Korvecdal — {1}{G}{G} 3/3 Elf Warrior. Play with the top card of
/// your library revealed and cast creature spells from the top of your library.
/// (The {G},{T},tap-two: reveal-and-drop-a-creature activated ability is
/// omitted for want of a tap-two-creatures cost.)
pub fn eladamri_korvecdal() -> CardDefinition {
    CardDefinition {
        name: "Eladamri, Korvecdal",
        cost: cost(&[generic(1), g(), g()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![
            StaticAbility {
                description: "Play with the top card of your library revealed.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may cast creature spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Creature },
            },
        ],
        ..Default::default()
    }
}

/// Party Thrasher — {1}{R} 1/4 Lizard Wizard. At the beginning of your first
/// main phase, you may discard a card; if you do, exile the top two cards of
/// your library and you may play them this turn (printed "one of them"; the
/// "noncreature spells cast from exile have convoke" static is omitted).
pub fn party_thrasher() -> CardDefinition {
    CardDefinition {
        name: "Party Thrasher",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::MayPay {
                description: "Discard a card to dig two?".into(),
                mana_cost: cost(&[]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        duration: MayPlayDuration::EndOfThisTurn,
                        pay_any_color: true,
                        uncast_penalty: None,
                    },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Back for an MH3 dual-color modal DFC land: enters tapped, taps for either of
/// its two colors.
fn dual_tapland(name: &'static str, a: Color, b: Color) -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![a, b], Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Suppression Ray // Orderly Plaza — {3}{W/U}{W/U} Sorcery. Tap all creatures
/// target player controls. Back is a W/U tapland. (The "pay {E} to stun that
/// many" rider is omitted.)
pub fn suppression_ray() -> CardDefinition {
    use crate::mana::hybrid;
    CardDefinition {
        name: "Suppression Ray",
        cost: cost(&[generic(3), hybrid(Color::White, Color::Blue), hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Tap {
            what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
        },
        back_face: Some(Box::new(dual_tapland("Orderly Plaza", Color::White, Color::Blue))),
        ..Default::default()
    }
}

/// Bloodsoaked Insight // Sanguine Morass — {5}{B/R}{B/R} Sorcery. Target
/// opponent exiles the top three cards of their library; until the end of your
/// next turn you may play them, spending mana of any type. Back is a B/R
/// tapland. (The "costs {1} less per life your opponents lost this turn" rider
/// is omitted.)
pub fn bloodsoaked_insight() -> CardDefinition {
    use crate::mana::hybrid;
    CardDefinition {
        name: "Bloodsoaked Insight",
        cost: cost(&[
            generic(5),
            hybrid(Color::Black, Color::Red),
            hybrid(Color::Black, Color::Red),
        ]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::Target(0),
            count: Value::Const(3),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: true,
            uncast_penalty: None,
        },
        back_face: Some(Box::new(dual_tapland("Sanguine Morass", Color::Black, Color::Red))),
        ..Default::default()
    }
}

/// Collective Resistance — {1}{G} instant, Escalate {G}. Choose one or more:
/// destroy target artifact; destroy target enchantment; target creature gains
/// hexproof and indestructible until end of turn. (The escalate cost is modeled
/// as a per-extra-mode {G} payment at resolution.)
pub fn collective_resistance() -> CardDefinition {
    CardDefinition {
        name: "Collective Resistance",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Escalate {
            cost: Box::new(Effect::PayManaOrElse {
                mana_cost: cost(&[g()]),
                otherwise: Box::new(Effect::Noop),
            }),
            modes: vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::Destroy { what: target_filtered(R::Enchantment) },
                Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Hexproof,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ],
        },
        ..Default::default()
    }
}

/// Ripples of Undeath — {1}{B} enchantment. At the beginning of your first main
/// phase, mill three cards; you may put one of them into your hand. (The
/// printed "pay {1} and 3 life" cost on the return is omitted.)
pub fn ripples_of_undeath() -> CardDefinition {
    CardDefinition {
        name: "Ripples of Undeath",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::MillThenToHandN {
                amount: Value::Const(3),
                filter: R::Any,
                take: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Genku, Future Shaper — {2}{W}{U} 2/5 Moonfolk Wizard. Whenever another
/// nontoken permanent you control leaves the battlefield, create one of three
/// creature tokens (Fox / Moonfolk / Rat). {3}{W}{U}: put a +1/+1 counter on
/// each creature you control. (The "hasn't been chosen this turn" rotation is
/// modeled as a free choice among the three.)
pub fn genku_future_shaper() -> CardDefinition {
    use crate::card::TokenDefinition;
    let fox = TokenDefinition {
        name: "Fox".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fox], ..Default::default() },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    let moonfolk = TokenDefinition {
        name: "Moonfolk".into(),
        power: 1,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Moonfolk], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    let rat = TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    };
    let mk = |def| Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: def };
    CardDefinition {
        name: "Genku, Future Shaper",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Moonfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                }),
            effect: Effect::ChooseMode(vec![mk(fox), mk(moonfolk), mk(rat)]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), u()]),
            effect: Effect::AddCounter {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
