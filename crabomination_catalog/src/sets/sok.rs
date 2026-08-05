//! Saviors of Kamigawa (SOK) gap closure. Tests in `classic_sets/sok`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{soulshift, spiritcraft, target_any, target_filtered},
};
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

use super::bok::{arcane_instant, creature, instant, legend, sorcery};
use super::bok2::on_upkeep;

/// A Channel ability (CR 207.2c ability word) — "{cost}, Discard this card:
/// `effect`", activated from hand at instant speed.
pub(crate) fn channel_ability(mana: crate::mana::ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        from_hand: true,
        discard_self_cost: true,
        effect,
        ..Default::default()
    }
}

/// "You have more cards in hand than each opponent."
fn hand_advantage() -> Predicate {
    Predicate::Not(Box::new(Predicate::AnOpponentHasMoreCardsInHand))
}

// ── Kirin cycle — spiritcraft scaled by the spell's mana value ──────────────

/// Bounteous Kirin — {5}{G}{G} 4/4 flier. Your Spirit/Arcane spells feed you
/// life equal to their mana value.
pub fn bounteous_kirin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![spiritcraft(Effect::MayDo {
            description: "Gain life equal to that spell's mana value?".into(),
            body: Box::new(Effect::GainLife {
                who: Selector::You,
                amount: Value::TriggerEventAmount,
            }),
        })],
        ..legend(
            "Bounteous Kirin",
            cost(&[generic(5), g(), g()]),
            vec![CreatureType::Kirin, CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Celestial Kirin — {2}{W}{W} 3/3 flier. Your Spirit/Arcane spells wipe every
/// permanent sharing their mana value.
pub fn celestial_kirin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![spiritcraft(Effect::Destroy {
            what: Selector::EachPermanent(R::ManaValueEqualsTriggerAmount),
        })],
        ..legend(
            "Celestial Kirin",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Kirin, CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Cloudhoof Kirin — {3}{U}{U} 4/4 flier. Your Spirit/Arcane spells mill a
/// player for their mana value.
pub fn cloudhoof_kirin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![spiritcraft(Effect::MayDo {
            description: "Mill that spell's mana value?".into(),
            body: Box::new(Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::TriggerEventAmount,
            }),
        })],
        ..legend(
            "Cloudhoof Kirin",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Kirin, CreatureType::Spirit],
            4,
            4,
        )
    }
}

// ── Channel (CR 702.58) ─────────────────────────────────────────────────────

/// Arashi, the Sky Asunder — {3}{G}{G} 5/5. Snipes fliers on the battlefield,
/// or channels out of hand to sweep them.
pub fn arashi_the_sky_asunder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[x(), g()]),
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                    amount: Value::XFromCost,
                },
                ..Default::default()
            },
            channel_ability(
                cost(&[x(), g(), g()]),
                Effect::DealDamage {
                    to: Selector::EachPermanent(
                        R::Creature.and(R::HasKeyword(Keyword::Flying)),
                    ),
                    amount: Value::XFromCost,
                },
            ),
        ],
        ..legend(
            "Arashi, the Sky Asunder",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Spirit],
            5,
            5,
        )
    }
}

/// Ghost-Lit Nourisher — {2}{G} 2/1. Pumps on the battlefield, or channels a
/// bigger pump out of hand.
pub fn ghost_lit_nourisher() -> CardDefinition {
    let pump = |amount: i32| Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(amount),
        toughness: Value::Const(amount),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                tap_cost: true,
                effect: pump(2),
                ..Default::default()
            },
            channel_ability(cost(&[generic(3), g()]), pump(4)),
        ],
        ..creature(
            "Ghost-Lit Nourisher",
            cost(&[generic(2), g()]),
            vec![CreatureType::Spirit],
            2,
            1,
        )
    }
}

// ── Spiritcraft bodies ──────────────────────────────────────────────────────

/// Briarknit Kami — {3}{G}{G} 3/3. Your Spirit/Arcane spells grow a creature.
pub fn briarknit_kami() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![spiritcraft(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Briarknit Kami",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Dreamcatcher — {U} 1/1. Cash it in on a Spirit or Arcane spell for a card.
pub fn dreamcatcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![spiritcraft(Effect::MayDo {
            description: "Sacrifice Dreamcatcher to draw a card?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::SacrificePermanent { what: Selector::This },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ])),
        })],
        ..creature("Dreamcatcher", cost(&[u()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Elder Pine of Jukai — {2}{G} 2/1 with soulshift 2. Your Spirit/Arcane
/// spells dig three deep for lands.
pub fn elder_pine_of_jukai() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            spiritcraft(Effect::LookPickToHand {
                then_if_picked: None,
                who: PlayerRef::You,
                count: Value::Const(3),
                pick_filter: Some(R::Land),
                take: Some(Value::Const(3)),
                rest_to_graveyard: false,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            }),
            soulshift(2),
        ],
        ..creature(
            "Elder Pine of Jukai",
            cost(&[generic(2), g()]),
            vec![CreatureType::Spirit],
            2,
            1,
        )
    }
}

/// Fiddlehead Kami — {4}{G} 3/3 that shrugs off removal on every Spirit or
/// Arcane spell.
pub fn fiddlehead_kami() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![spiritcraft(Effect::Regenerate { what: Selector::This })],
        ..creature(
            "Fiddlehead Kami",
            cost(&[generic(4), g()]),
            vec![CreatureType::Spirit],
            3,
            3,
        )
    }
}

// ── Hand-size matters ───────────────────────────────────────────────────────

/// Deathmask Nezumi — {2}{B} 2/2 that grows and gains fear with a full hand.
pub fn deathmask_nezumi() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::card::StaticAbility {
            description: "As long as you have seven or more cards in hand, this creature \
                          gets +2/+1 and has fear.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::HandSizeOf(PlayerRef::You),
                    Value::Const(7),
                ),
                power: 2,
                toughness: 1,
                keywords: vec![Keyword::Fear],
            },
        }],
        ..creature(
            "Deathmask Nezumi",
            cost(&[generic(2), b()]),
            vec![CreatureType::Rat, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Gnat Miser — {B} 1/1 that shaves a card off every opponent's hand size.
pub fn gnat_miser() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::card::StaticAbility {
            description: "Each opponent's maximum hand size is reduced by one.",
            effect: StaticEffect::OpponentsMaxHandSizeReduced(1),
        }],
        ..creature(
            "Gnat Miser",
            cost(&[b()]),
            vec![CreatureType::Rat, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Ebony Owl Netsuke — {2} Artifact. Punishes an opponent hoarding seven cards.
pub fn ebony_owl_netsuke() -> CardDefinition {
    CardDefinition {
        name: "Ebony Owl Netsuke",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::OpponentControl,
            )
            .with_filter(Predicate::ValueAtLeast(
                Value::HandSizeOf(PlayerRef::ActivePlayer),
                Value::Const(7),
            )),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(4),
            },
        }],
        ..Default::default()
    }
}

/// Gaze of Adamaro — {2}{R}{R} Arcane instant. Damage equal to the target's
/// own hand.
pub fn gaze_of_adamaro() -> CardDefinition {
    arcane_instant(
        "Gaze of Adamaro",
        cost(&[generic(2), r(), r()]),
        Effect::DealDamage {
            to: target_filtered(R::Player),
            amount: Value::HandSizeOf(PlayerRef::Target(0)),
        },
    )
}

/// Descendant of Soramaro — {3}{U} 2/3. Sifts as deep as your hand is wide.
pub fn descendant_of_soramaro() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::HandSizeOf(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Descendant of Soramaro",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Death of a Thousand Stings — {4}{B} Arcane instant. Drains for 1, and
/// climbs back out of the graveyard while you're ahead on cards.
pub fn death_of_a_thousand_stings() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            )
            .with_filter(hand_advantage()),
            effect: Effect::MayDo {
                description: "Return this card from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..arcane_instant(
            "Death of a Thousand Stings",
            cost(&[generic(4), b()]),
            Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        )
    }
}

// ── The rest ────────────────────────────────────────────────────────────────

/// Aether Shockwave — {3}{W} Instant. Tap all Spirits, or everything else.
pub fn aether_shockwave() -> CardDefinition {
    instant(
        "Aether Shockwave",
        cost(&[generic(3), w()]),
        Effect::ChooseMode(vec![
            Effect::Tap {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::HasCreatureType(CreatureType::Spirit)),
                    ),
            },
            Effect::Tap {
                what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(
                    R::HasCreatureType(CreatureType::Spirit),
                )))),
            },
        ]),
    )
}

/// Araba Mothrider — {1}{W} 1/1 flier with bushido 1.
pub fn araba_mothrider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Bushido(1)],
        ..creature(
            "Araba Mothrider",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            1,
            1,
        )
    }
}

/// Ayumi, the Last Visitor — {3}{G}{G} 7/3 with legendary landwalk.
pub fn ayumi_the_last_visitor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::LandwalkFiltered(Box::new(
            R::Land.and(R::HasSupertype(crate::card::Supertype::Legendary)),
        ))],
        ..legend(
            "Ayumi, the Last Visitor",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Spirit],
            7,
            3,
        )
    }
}

/// Burning-Eye Zubera — {2}{R}{R} 3/3 that goes off if it soaked 4+ damage.
pub fn burning_eye_zubera() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::If {
            cond: Predicate::ValueAtLeast(Value::MarkedDamageOn(Box::new(Selector::This)), Value::Const(4)),
            then: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(3) }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Burning-Eye Zubera",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Zubera, CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Captive Flame — {2}{R} Enchantment. A repeatable {R} pump.
pub fn captive_flame() -> CardDefinition {
    CardDefinition {
        name: "Captive Flame",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cut the Earthly Bond — {U} Arcane instant. Bounce whatever is wearing an
/// Aura.
pub fn cut_the_earthly_bond() -> CardDefinition {
    arcane_instant(
        "Cut the Earthly Bond",
        cost(&[u()]),
        Effect::Move {
            what: target_filtered(R::IsEnchanted),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
    )
}

/// Death Denied — {X}{B}{B} Arcane instant. Rake X creatures out of your
/// graveyard.
pub fn death_denied() -> CardDefinition {
    arcane_instant(
        "Death Denied",
        cost(&[x(), b(), b()]),
        Effect::Move {
            what: Selector::take(
                Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Creature,
                },
                Value::XFromCost,
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Deathknell Kami — {1}{B} 0/1 flier with soulshift 1 that can trade itself
/// up for a swing.
pub fn deathknell_kami() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![soulshift(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::SacrificePermanent { what: Selector::This }),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Deathknell Kami",
            cost(&[generic(1), b()]),
            vec![CreatureType::Spirit],
            0,
            1,
        )
    }
}

/// Dense Canopy — {1}{G} Enchantment. Fliers can only block fliers.
pub fn dense_canopy() -> CardDefinition {
    CardDefinition {
        name: "Dense Canopy",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Creatures with flying can block only creatures with flying.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::CanBlockOnlyFlying],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Dosan's Oldest Chant — {4}{G} Sorcery. Six life and a card.
pub fn dosans_oldest_chant() -> CardDefinition {
    sorcery(
        "Dosan's Oldest Chant",
        cost(&[generic(4), g()]),
        Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(6) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Eiganjo Free-Riders — {3}{W} 3/4 flier that has to keep bouncing a white
/// creature every upkeep.
pub fn eiganjo_free_riders() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_upkeep(Effect::Move {
            what: Selector::take(
                Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::HasColor(Color::White)),
                },
                Value::ONE,
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Eiganjo Free-Riders",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Feral Lightning — {3}{R}{R}{R} Sorcery. Three hasty 3/1s that burn out at
/// the end step.
pub fn feral_lightning() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 3,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        colors: vec![Color::Red],
        keywords: vec![Keyword::Haste],
        ..Default::default()
    };
    // One mint-then-schedule pair per token: `AtNextEndStep` binds the token
    // minted in the same step, so all three are exiled.
    let mint = |def: TokenDefinition| {
        Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: def },
            Effect::AtNextEndStep {
                body: Box::new(Effect::Exile { what: Selector::LastCreatedToken }),
            },
        ])
    };
    sorcery(
        "Feral Lightning",
        cost(&[generic(3), r(), r(), r()]),
        Effect::Seq(vec![
            mint(elemental.clone()),
            mint(elemental.clone()),
            mint(elemental),
        ]),
    )
}

/// Freed from the Real — {2}{U} Aura with a tap/untap pair on the host.
pub fn freed_from_the_real() -> CardDefinition {
    CardDefinition {
        name: "Freed from the Real",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![
                ActivatedAbility {
                    mana_cost: cost(&[u()]),
                    effect: Effect::Tap { what: Selector::This },
                    ..Default::default()
                },
                ActivatedAbility {
                    mana_cost: cost(&[u()]),
                    effect: Effect::Untap { what: Selector::This, up_to: None },
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Glitterfang — {R} 1/1 haste that goes home at every end step.
pub fn glitterfang() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..creature("Glitterfang", cost(&[r()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Godo's Irregulars — {R} 1/1 that can shoot whatever is blocking it.
pub fn godos_irregulars() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::DealDamage {
                to: Selector::take(Selector::BlockingCreatures, Value::ONE),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Godo's Irregulars",
            cost(&[r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Blood Clock — {4} Artifact. Every upkeep costs its player a permanent or
/// two life.
pub fn blood_clock() -> CardDefinition {
    CardDefinition {
        name: "Blood Clock",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ActivePlayer,
                cost: crate::card::WardCost::Life(2),
                then: Box::new(Effect::Move {
                    what: Selector::take(
                        Selector::ControlledBy { who: PlayerRef::ActivePlayer, filter: R::Any },
                        Value::ONE,
                    ),
                    to: ZoneDest::Hand(PlayerRef::ActivePlayer),
                }),
                if_paid: None,
            },
        }],
        ..Default::default()
    }
}

/// Evermind — a costless Arcane instant: never castable, but it splices onto
/// Arcane for {1}{U} to draw a card.
pub fn evermind() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Splice(
            cost(&[generic(1), u()]),
            crate::card::SpellSubtype::Arcane,
        )],
        no_mana_cost: true,
        ..arcane_instant(
            "Evermind",
            cost(&[]),
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        )
    }
}

/// Descendant of Masumaro — {2}{G} 1/1 that swings on the hand-size gap each
/// upkeep.
pub fn descendant_of_masumaro() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_upkeep(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HandSizeOf(PlayerRef::You),
            },
            Effect::RemoveCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HandSizeOf(PlayerRef::EachOpponent),
            },
        ]))],
        ..creature(
            "Descendant of Masumaro",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Monk],
            1,
            1,
        )
    }
}
