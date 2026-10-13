//! Commander: the 2026-10-13 rescan's gaps for Gale, Waterdeep Prodigy (with
//! Scion of Halaster), Jhoira, Ageless Innovator and Cecily, Haunted Mage
//! (with Othelm). Tests in `tests/recent_b/cmdr_edhrec44.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::card::Zone;
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, LookPick, ManaPayload, PlayerRef, VoteOption, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, ManaCost, SpendRestriction};

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

fn friends_forever() -> Keyword {
    Keyword::PartnerLabel("Friends forever".into())
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

// ── Gale, Waterdeep Prodigy + Scion of Halaster ──────────────────────────────

/// "Whenever you cast a `cast` spell from your hand, you may cast up to one
/// target `other` card from your graveyard", exiled instead of returning.
fn gale_trigger(cast: CardType, other: CardType) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
            Predicate::CastSpellMatches(R::HasCardType(cast)),
            Predicate::CastSpellFromHand,
        ])),
        effect: Effect::CastWithoutPayingImmediate {
            what: target_filtered(R::HasCardType(other).and(R::InYourGraveyard)),
            source_zone: Zone::Graveyard,
            exile_after: true,
            copy: false,
            reduce_generic: 0,
            pay_own_cost: true,
        },
    }
}

/// Gale, Waterdeep Prodigy — {2}{U} legendary 1/3 Human Wizard. An instant or
/// sorcery cast from your hand lets you cast a card of the other type from
/// your graveyard (paying for it), exiled afterwards. Choose a Background.
pub fn gale_waterdeep_prodigy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        triggered_abilities: vec![
            gale_trigger(CardType::Instant, CardType::Sorcery),
            gale_trigger(CardType::Sorcery, CardType::Instant),
        ],
        ..legend("Gale, Waterdeep Prodigy", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 3)
    }
}

/// Scion of Halaster — {1}{B} legendary Background. Your commander creatures
/// have "the first time you would draw each turn, instead look at the top two,
/// one into your graveyard and the other back, then draw" — read while you
/// control a commander of yours.
pub fn scion_of_halaster() -> CardDefinition {
    CardDefinition {
        name: "Scion of Halaster",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"The first time you would draw a card each turn, instead \
                          look at the top two cards of your library. Put one of them into your graveyard and the \
                          other back on top of your library. Then draw a card.\"",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::ControlsOwnCommander { who: PlayerRef::You },
                inner: Box::new(StaticEffect::FirstDrawEachTurnMillsOneOfTwo),
            },
        }],
        ..Default::default()
    }
}

/// Font of Magic — {3}{U} enchantment. Your instants and sorceries cost {1}
/// less per commander cast from the command zone this game.
pub fn font_of_magic() -> CardDefinition {
    CardDefinition {
        name: "Font of Magic",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast for each time you've cast a \
                          commander from the command zone this game.",
            effect: StaticEffect::CostReductionByValue {
                filter: instant_or_sorcery(),
                amount: Value::CommanderCastsFromCommandZone(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

// ── Jhoira, Ageless Innovator ────────────────────────────────────────────────

/// Jhoira, Ageless Innovator — {U}{R} legendary 2/3 Human Artificer. {T}: two
/// ingenuity counters on her, then you may put an artifact card with mana
/// value at most her ingenuity counters from your hand onto the battlefield.
pub fn jhoira_ageless_innovator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Ingenuity, amount: Value::Const(2) },
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Artifact.and(R::ManaValueAtMostSourceCounters(CounterType::Ingenuity)),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Jhoira, Ageless Innovator",
            cost(&[u(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            3,
        )
    }
}

/// Sewer-veillance Cam — {U} artifact, flash. Entering or leaving: you may tap
/// or untap target creature. {3}{U}, sacrifice it: draw two.
pub fn sewer_veillance_cam() -> CardDefinition {
    let tap_or_untap = || Effect::PlayerChoosesOne {
        who: PlayerRef::You,
        options: vec![
            VoteOption::new("Neither", Effect::Noop),
            VoteOption::new("Tap it", Effect::Tap { what: target_filtered(R::Creature) }),
            VoteOption::new("Untap it", Effect::Untap { what: target_filtered(R::Creature), up_to: None }),
        ],
    };
    CardDefinition {
        name: "Sewer-veillance Cam",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource), effect: tap_or_untap() },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: tap_or_untap(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Forging the Anchor — {2}{U} sorcery. Look at the top five; any number of
/// artifact cards among them to hand, the rest to the bottom at random.
pub fn forging_the_anchor() -> CardDefinition {
    CardDefinition {
        name: "Forging the Anchor",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(5),
            pick_filter: Some(R::Artifact),
            take: Some(Value::Const(5)),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

// ── Cecily, Haunted Mage + Othelm, Sigardian Outcast ─────────────────────────

/// Cecily, Haunted Mage — {1}{U}{B}{R} legendary 3/5 Human Wizard. Maximum
/// hand size eleven. Attacking: draw and lose 1 life, then with eleven or more
/// cards in hand you may cast an instant or sorcery from it for free.
/// Partner—Friends forever.
pub fn cecily_haunted_mage() -> CardDefinition {
    CardDefinition {
        keywords: vec![friends_forever()],
        static_abilities: vec![StaticAbility {
            description: "Your maximum hand size is eleven.",
            effect: StaticEffect::ControllerMaxHandSize(11),
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::Const(11)),
                    then: Box::new(Effect::MayCastFromHandFreeMatching {
                        filter: instant_or_sorcery(),
                        max_mv: Value::Const(99),
                        else_: Box::new(Effect::Noop),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend(
            "Cecily, Haunted Mage",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            5,
        )
    }
}

/// Othelm, Sigardian Outcast — {1}{G}{W} legendary 2/2 Human. {2}, {T}: return
/// a creature card that went to your graveyard from the battlefield this turn
/// to the battlefield tapped. Partner—Friends forever.
pub fn othelm_sigardian_outcast() -> CardDefinition {
    CardDefinition {
        keywords: vec![friends_forever()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    R::Creature.and(R::InYourGraveyard).and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..legend("Othelm, Sigardian Outcast", cost(&[generic(1), g(), w()]), vec![CreatureType::Human], 2, 2)
    }
}

/// Hargilde, Kindly Runechanter — {2}{W}{U} legendary 2/3 Human. {T}: {C}{C}
/// for artifact spells and artifacts' abilities. Partner—Friends forever.
pub fn hargilde_kindly_runechanter() -> CardDefinition {
    CardDefinition {
        keywords: vec![friends_forever()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::Const(2))),
                    SpendRestriction::ArtifactOnly,
                ),
            },
            ..Default::default()
        }],
        ..legend("Hargilde, Kindly Runechanter", cost(&[generic(2), w(), u()]), vec![CreatureType::Human], 2, 3)
    }
}
