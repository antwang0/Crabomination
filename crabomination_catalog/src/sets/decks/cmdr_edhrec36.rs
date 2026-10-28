//! Commander: three-card gaps for Melek (Reforged Researcher), Trazyn,
//! O-Kagachi, Orah and Meria (`cmdr_edhrec34` is the previous batch of this
//! run). Tests in `tests/recent_b/cmdr_edhrec36.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};
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

fn instants_and_sorceries() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

// ── Melek, Reforged Researcher ───────────────────────────────────────────────

/// Soul Read — {3}{U} instant. Choose one: counter target spell unless its
/// controller pays {4}; or draw two cards.
pub fn soul_read() -> CardDefinition {
    spell(
        "Soul Read",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(4)]),
                exile: false,
                extra_generic: None,
                if_paid: None,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
    )
}

/// Gravitic Punch — {3}{R} sorcery. Target creature you control deals damage
/// equal to its power to target player. Jump-start.
pub fn gravitic_punch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::JumpStart],
        ..spell(
            "Gravitic Punch",
            cost(&[generic(3), r()]),
            CardType::Sorcery,
            Effect::DealDamageFrom {
                source: target_filtered(R::Creature.and(R::ControlledByYou)),
                to: Selector::TargetFiltered { slot: 1, filter: R::Player },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        )
    }
}

/// Immolating Gyre — {4}{R}{R} sorcery. X damage to each creature and
/// planeswalker you don't control, X = instant and sorcery cards in your
/// graveyard.
pub fn immolating_gyre() -> CardDefinition {
    spell(
        "Immolating Gyre",
        cost(&[generic(4), r(), r()]),
        CardType::Sorcery,
        Effect::DealDamage {
            to: Selector::EachPermanent(
                R::Creature.or(R::Planeswalker).and(R::Not(Box::new(R::ControlledByYou))),
            ),
            amount: Value::count(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: instants_and_sorceries(),
            }),
        },
    )
}

// ── Trazyn the Infinite ──────────────────────────────────────────────────────

/// Amaranthine Wall — {4} 0/6 artifact creature Wall, defender. {2}: it gains
/// indestructible until end of turn.
pub fn amaranthine_wall() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Amaranthine Wall", cost(&[generic(4)]), vec![CreatureType::Wall], 0, 6)
    }
}

/// Disciples of Gix — {4}{B}{B} 4/4 Phyrexian Human. Enters: up to three
/// artifact cards from your library into your graveyard.
pub fn disciples_of_gix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Artifact,
            to: ZoneDest::Graveyard,
            count: Value::Const(3),
        })],
        ..creature(
            "Disciples of Gix",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Human],
            4,
            4,
        )
    }
}

/// Golem Artisan — {5} 3/3 artifact creature Golem. {2}: target artifact
/// creature gets +1/+1 until end of turn. {2}: it gains your choice of
/// flying, trample or haste until end of turn.
pub fn golem_artisan() -> CardDefinition {
    let artifact_creature = || target_filtered(R::Artifact.and(R::Creature));
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::PumpPT {
                    what: artifact_creature(),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::ChooseMode(
                    [Keyword::Flying, Keyword::Trample, Keyword::Haste]
                        .into_iter()
                        .map(|keyword| Effect::GrantKeyword {
                            what: artifact_creature(),
                            keyword,
                            duration: Duration::EndOfTurn,
                        })
                        .collect(),
                ),
                ..Default::default()
            },
        ],
        ..creature("Golem Artisan", cost(&[generic(5)]), vec![CreatureType::Golem], 3, 3)
    }
}

// ── O-Kagachi, Vengeful Kami ─────────────────────────────────────────────────

/// Kura, the Boundless Sky — {3}{G}{G} legendary 4/4 Dragon Spirit, flying,
/// deathtouch. Dies: choose one — up to three land cards to hand; or an X/X
/// green Spirit, X = lands you control.
pub fn kura_the_boundless_sky() -> CardDefinition {
    let lands = || Value::count(Selector::ControlledBy { who: PlayerRef::You, filter: R::Land });
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        dynamic_pt: Some((lands(), lands())),
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![on_dies(Effect::ChooseMode(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Land,
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(3),
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(spirit) },
        ]))],
        ..legend(
            "Kura, the Boundless Sky",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Dragon, CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Kyodai, Soul of Kamigawa — {3}{W} legendary 3/3 Dragon Spirit, flash,
/// flying. Enters: another target permanent gains indestructible for as long
/// as you control Kyodai. {W}{U}{B}{R}{G}: +5/+5 until end of turn.
pub fn kyodai_soul_of_kamigawa() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Permanent.and(R::OtherThanSource)),
            keyword: Keyword::Indestructible,
            duration: Duration::WhileYouControlSource,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(5),
                toughness: Value::Const(5),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Kyodai, Soul of Kamigawa",
            cost(&[generic(3), w()]),
            vec![CreatureType::Dragon, CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Kairi, the Swirling Sky — {4}{U}{U} legendary 6/6 Dragon Spirit, flying,
/// ward {3}. Dies: choose one — return any number of target nonland
/// permanents with total mana value 6 or less to their owners' hands; or mill
/// six, then return up to two instant and/or sorcery cards from your
/// graveyard to your hand.
pub fn kairi_the_swirling_sky() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(3)])))],
        triggered_abilities: vec![on_dies(Effect::ChooseMode(vec![
            Effect::ApplyToTargets {
                max_targets: 6,
                min_targets: 0,
                filter: R::Permanent.and(R::Nonland).and(R::SlotsTotalManaValueAtMost(6)),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
            Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(6) },
                Effect::ChooseSomeAmong {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: instants_and_sorceries(),
                    },
                    chooser: PlayerRef::You,
                    count: Value::Const(2),
                    up_to: true,
                    chosen: Box::new(Effect::Move {
                        what: Selector::SeparatedPile { chosen: true },
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    other: Box::new(Effect::Noop),
                },
            ]),
        ]))],
        ..legend(
            "Kairi, the Swirling Sky",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Dragon, CreatureType::Spirit],
            6,
            6,
        )
    }
}

// ── Orah, Skyclave Hierophant ────────────────────────────────────────────────

/// Orah, Skyclave Hierophant — {2}{W}{B} legendary 3/3 Kor Cleric, lifelink.
/// Whenever Orah or another Cleric you control dies, return target Cleric
/// card with lesser mana value from your graveyard to the battlefield.
pub fn orah_skyclave_hierophant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Cleric),
                },
            ),
            effect: Effect::Move {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Cleric)
                        .and(R::InYourGraveyard)
                        .and(R::ManaValueLessThanEventAmount),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..legend(
            "Orah, Skyclave Hierophant",
            cost(&[generic(2), w(), b()]),
            vec![CreatureType::Kor, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Pyre of Heroes — {2} artifact. {2}, {T}, sacrifice a creature: search for
/// a creature card sharing a creature type with it, with mana value one more,
/// onto the battlefield. Sorcery speed.
pub fn pyre_of_heroes() -> CardDefinition {
    CardDefinition {
        name: "Pyre of Heroes",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            sorcery_speed: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature
                    .and(R::SharesCreatureTypeWithSacrificed)
                    .and(R::ManaValueEqualsSacrificedPlus(1)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cleric of Life's Bond — {W}{B} 2/2 Vampire Cleric. Another Cleric entering
/// gains you 1 life; your first life gain each turn puts a +1/+1 counter on
/// it.
pub fn cleric_of_lifes_bond() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Cleric),
                    },
                ),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl).once_per_turn(),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature(
            "Cleric of Life's Bond",
            cost(&[w(), b()]),
            vec![CreatureType::Vampire, CreatureType::Cleric],
            2,
            2,
        )
    }
}

// ── Meria, Scholar of Antiquity ──────────────────────────────────────────────

/// Meria, Scholar of Antiquity — {1}{R}{G} legendary 3/3 Elf Artificer. Tap an
/// untapped nontoken artifact you control: add {G}. Tap two: exile the top
/// card of your library; you may play it this turn.
pub fn meria_scholar_of_antiquity() -> CardDefinition {
    let nontoken_artifact = || R::Artifact.and(R::Not(Box::new(R::IsToken)));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_other_filter: Some(nontoken_artifact()),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green]) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_n_filter: Some((nontoken_artifact(), 2)),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Meria, Scholar of Antiquity",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Elf, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Herbal Poultice — {0} artifact. {3}, sacrifice it: regenerate target
/// creature.
pub fn herbal_poultice() -> CardDefinition {
    CardDefinition {
        name: "Herbal Poultice",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_cost: true,
            effect: Effect::Regenerate { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Traxos, Scourge Eternal — {4} legendary 5/4 artifact creature Dragon
/// Construct, trample. It doesn't untap during your untap step; casting an
/// artifact or creature spell untaps it.
pub fn traxos_scourge_eternal() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Traxos doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Artifact.or(R::Creature))),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..legend(
            "Traxos, Scourge Eternal",
            cost(&[generic(4)]),
            vec![CreatureType::Dragon, CreatureType::Construct],
            5,
            4,
        )
    }
}
