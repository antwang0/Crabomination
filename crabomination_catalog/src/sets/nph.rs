//! New Phyrexia (NPH) — Phyrexian mana, Infect, Living weapon and the Shrine
//! cycle. Tests in `recent_b/nph`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{LookPick, 
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, StaticEffect, Value,
    ZoneDest,
    shortcut::{draw, etb, target_any, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, phyrexian, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn artifact_creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, c, types, p, t)
    }
}

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

/// "Enchant `what`", with the ongoing effect carried as an `EquipBonus`.
fn aura(name: &'static str, c: ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// The permanent this Aura is attached to.
fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// Living weapon (CR 702.92) — mint a 0/0 black Phyrexian Germ and attach.
fn living_weapon() -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Phyrexian Germ".into(),
                card_types: vec![CardType::Creature],
                colors: vec![Color::Black],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Phyrexian],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
    ]))
}

fn equipment(name: &'static str, c: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..artifact(name, c)
    }
}

/// "{cost}: This creature gains `keyword` until end of turn."
fn self_grant(c: ManaCost, keyword: Keyword) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::GrantKeyword {
            what: Selector::This,
            keyword,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// A 1/1 colorless Phyrexian Myr artifact creature token.
fn phyrexian_myr() -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Myr".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Myr],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// A 3/3 colorless Phyrexian Golem artifact creature token.
fn phyrexian_golem() -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Golem".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The Shrine cycle: a charge counter each upkeep and per matching spell, then
/// a sacrifice payoff scaling off the pile.
fn shrine(
    name: &'static str,
    c: ManaCost,
    color: Color,
    payoff: ActivatedAbility,
) -> CardDefinition {
    let tick = Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Charge,
        amount: Value::ONE,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: tick.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasColor(color),
                    }),
                effect: tick,
            },
        ],
        activated_abilities: vec![payoff],
        ..artifact(name, c)
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Inquisitor Exarch — two life for you or two off them.
pub fn inquisitor_exarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
        ]))],
        ..creature(
            "Inquisitor Exarch",
            cost(&[w(), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Lost Leonin — a two-mana infect clock.
pub fn lost_leonin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        ..creature(
            "Lost Leonin",
            cost(&[generic(1), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Cat, CreatureType::Soldier],
            2,
            1,
        )
    }
}

/// Loxodon Convert — a plain 4/2 body.
pub fn loxodon_convert() -> CardDefinition {
    creature(
        "Loxodon Convert",
        cost(&[generic(3), w()]),
        vec![CreatureType::Phyrexian, CreatureType::Elephant, CreatureType::Soldier],
        4,
        2,
    )
}

/// Shriek Raptor — infect in the air.
pub fn shriek_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Infect],
        ..creature(
            "Shriek Raptor",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Bird],
            2,
            3,
        )
    }
}

/// Shattered Angel — every opposing land is three life.
pub fn shattered_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Land,
                }),
            effect: Effect::MayDo {
                description: "Gain 3 life?".into(),
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                }),
            },
        }],
        ..creature(
            "Shattered Angel",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Angel],
            3,
            3,
        )
    }
}

/// Marrow Shards — one damage to every attacker, for two life if you like.
pub fn marrow_shards() -> CardDefinition {
    instant(
        "Marrow Shards",
        cost(&[phyrexian(Color::White)]),
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            amount: Value::ONE,
        },
    )
}

/// War Report — life for the whole board.
pub fn war_report() -> CardDefinition {
    instant(
        "War Report",
        cost(&[generic(3), w()]),
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Sum(vec![
                Value::CountOf(Box::new(Selector::EachPermanent(R::Creature))),
                Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact))),
            ]),
        },
    )
}

/// Remember the Fallen — a creature, an artifact, or both.
pub fn remember_the_fallen() -> CardDefinition {
    sorcery(
        "Remember the Fallen",
        cost(&[generic(2), w()]),
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Artifact.and(R::InGraveyard),
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Due Respect — everything lands tapped for the turn, and you draw.
pub fn due_respect() -> CardDefinition {
    instant(
        "Due Respect",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![Effect::PermanentsEnterTappedThisTurn, draw(1)]),
    )
}


/// Forced Worship — a pacifism that can be picked back up.
pub fn forced_worship() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..aura(
            "Forced Worship",
            cost(&[generic(1), w()]),
            R::Creature,
            EquipBonus { keywords: vec![Keyword::CantAttack], ..Default::default() },
        )
    }
}

/// Auriok Survivors — reanimate an Equipment and strap it on.
pub fn auriok_survivors() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return an Equipment card and attach it?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::InGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::Attach { what: Selector::LastMoved, to: Selector::This },
            ])),
        })],
        ..creature(
            "Auriok Survivors",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            6,
        )
    }
}

/// Norn's Annex — attacking you costs {W} or two life a creature.
pub fn norns_annex() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you or planeswalkers you control unless their \
                          controller pays {W/P} for each of those creatures.",
            effect: StaticEffect::AttackTaxToController {
                amount: Value::ONE,
                protect_planeswalkers: true,
                            filter: None,
            },
        }],
        ..artifact(
            "Norn's Annex",
            cost(&[generic(3), phyrexian(Color::White), phyrexian(Color::White)]),
        )
    }
}

/// Shrine of Loyal Legions — a Myr per charge counter.
pub fn shrine_of_loyal_legions() -> CardDefinition {
    shrine(
        "Shrine of Loyal Legions",
        cost(&[generic(2)]),
        Color::White,
        ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                definition: phyrexian_myr(),
            },
            ..Default::default()
        },
    )
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Argent Mutation — an artifact for a turn, plus a card.
pub fn argent_mutation() -> CardDefinition {
    instant(
        "Argent Mutation",
        cost(&[generic(2), u()]),
        Effect::Seq(vec![
            Effect::AddCardTypeIndefinitely {
                what: target_filtered(R::Permanent),
                card_type: CardType::Artifact,
                until_eot: true,
            },
            draw(1),
        ]),
    )
}

/// Corrupted Resolve — a hard counter, if they're poisoned.
pub fn corrupted_resolve() -> CardDefinition {
    instant(
        "Corrupted Resolve",
        cost(&[generic(1), u()]),
        Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::PoisonCountersOf(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                Value::ONE,
            ),
            then: Box::new(Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) }),
            else_: Box::new(Effect::Noop),
        },
    )
}

/// Psychic Barrier — Mana Leak's poisoned cousin.
pub fn psychic_barrier() -> CardDefinition {
    instant(
        "Psychic Barrier",
        cost(&[u(), u()]),
        Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::Creature)),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ONE,
            },
        ]),
    )
}

/// Mindculling — two for you, two off them.
pub fn mindculling() -> CardDefinition {
    sorcery(
        "Mindculling",
        cost(&[generic(5), u()]),
        Effect::Seq(vec![
            draw(2),
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
            },
        ]),
    )
}

/// Spined Thopter — a flier for {U} or two life.
pub fn spined_thopter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..artifact_creature(
            "Spined Thopter",
            cost(&[generic(2), phyrexian(Color::Blue)]),
            vec![CreatureType::Phyrexian, CreatureType::Thopter],
            2,
            1,
        )
    }
}

/// Spire Monitor — a flash flier.
pub fn spire_monitor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        ..creature(
            "Spire Monitor",
            cost(&[generic(4), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Drake],
            3,
            3,
        )
    }
}

/// Impaler Shrike — trade the bird for three cards.
pub fn impaler_shrike() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MaySacrificeSource {
                description: "Sacrifice this to draw three cards?".into(),
                then: Box::new(draw(3)),
                else_: None,
            },
        }],
        ..creature(
            "Impaler Shrike",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Bird],
            3,
            1,
        )
    }
}

/// Chained Throatseeker — infect that only attacks the already-poisoned.
pub fn chained_throatseeker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        static_abilities: vec![StaticAbility {
            description: "This creature can't attack unless defending player is poisoned.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::CantAttack,
                condition: Predicate::Not(Box::new(Predicate::ValueAtLeast(
                    Value::PoisonCountersOf(PlayerRef::EachOpponent),
                    Value::ONE,
                ))),
            },
        }],
        ..creature(
            "Chained Throatseeker",
            cost(&[generic(5), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            5,
            5,
        )
    }
}

/// Viral Drake — a repeatable proliferate engine with wings.
pub fn viral_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Infect],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Proliferate,
            ..Default::default()
        }],
        ..creature(
            "Viral Drake",
            cost(&[generic(3), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Drake],
            1,
            4,
        )
    }
}

/// Trespassing Souleater — a body that walks through for {U} or two life.
pub fn trespassing_souleater() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_grant(
            cost(&[phyrexian(Color::Blue)]),
            Keyword::Unblockable,
        )],
        ..artifact_creature(
            "Trespassing Souleater",
            cost(&[generic(3)]),
            vec![CreatureType::Phyrexian, CreatureType::Construct],
            2,
            2,
        )
    }
}

/// Xenograft — every creature you control joins the chosen tribe.
pub fn xenograft() -> CardDefinition {
    CardDefinition {
        name: "Xenograft",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::NameCreatureType { what: Selector::This },
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control is the chosen type in addition to its \
                          other types.",
            effect: StaticEffect::CreaturesYouControlAreChosenType,
        }],
        ..Default::default()
    }
}

/// Numbing Dose — a lockdown that bleeds them out.
pub fn numbing_dose() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::EnchantedBySource,
            ),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EnchantedPlayer),
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Enchanted permanent doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: host() },
        }],
        ..aura(
            "Numbing Dose",
            cost(&[generic(3), u(), u()]),
            R::Artifact.or(R::Creature),
            EquipBonus::default(),
        )
    }
}

/// Defensive Stance — a cheap combat swing.
pub fn defensive_stance() -> CardDefinition {
    aura(
        "Defensive Stance",
        cost(&[u()]),
        R::Creature,
        EquipBonus { power: -1, toughness: 1, ..Default::default() },
    )
}

/// Shrine of Piercing Vision — dig as deep as the pile is tall.
pub fn shrine_of_piercing_vision() -> CardDefinition {
    shrine(
        "Shrine of Piercing Vision",
        cost(&[generic(2)]),
        Color::Blue,
        ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
    ..Default::default()
})),
            ..Default::default()
        },
    )
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Caress of Phyrexia — three cards for three life and three poison.
pub fn caress_of_phyrexia() -> CardDefinition {
    sorcery(
        "Caress of Phyrexia",
        cost(&[generic(3), b(), b()]),
        Effect::Seq(vec![
            Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(3) },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(3),
            },
            Effect::AddPoison {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(3),
            },
        ]),
    )
}

/// Toxic Nim — a regenerating infect body.
pub fn toxic_nim() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Toxic Nim",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie],
            4,
            1,
        )
    }
}

/// Dementia Bat — a flier that cashes out for two discards.
pub fn dementia_bat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            sac_cost: true,
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Dementia Bat",
            cost(&[generic(4), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Bat],
            2,
            2,
        )
    }
}

/// Blind Zealot — connect and trade it for their best creature.
pub fn blind_zealot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Intimidate],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MaySacrificeSource {
                description: "Sacrifice this to destroy that creature?".into(),
                then: Box::new(Effect::Destroy { what: target_filtered(R::Creature) }),
                else_: None,
            },
        }],
        ..creature(
            "Blind Zealot",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Reaper of Sheoldred — poison anything that dares hit it.
pub fn reaper_of_sheoldred() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::AddPoison {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Reaper of Sheoldred",
            cost(&[generic(4), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            2,
            5,
        )
    }
}

/// Pith Driller — a -1/-1 counter stapled to a body.
pub fn pith_driller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::MinusOneMinusOne,
            amount: Value::ONE,
        })],
        ..artifact_creature(
            "Pith Driller",
            cost(&[generic(4), phyrexian(Color::Black)]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            2,
            4,
        )
    }
}

/// Mortis Dogs — hits hard, then drains for its size.
pub fn mortis_dogs() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            },
        ],
        ..creature(
            "Mortis Dogs",
            cost(&[generic(3), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Dog],
            2,
            2,
        )
    }
}

/// Mindcrank — every point of life they lose is a card off the top.
pub fn mindcrank() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..artifact("Mindcrank", cost(&[generic(2)]))
    }
}

/// Life's Finale — a wrath that also strips their library.
pub fn lifes_finale() -> CardDefinition {
    sorcery(
        "Life's Finale",
        cost(&[generic(4), b(), b()]),
        Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
            Effect::SearchUpToN {
                who: PlayerRef::Target(0),
                filter: R::Creature,
                to: ZoneDest::Graveyard,
                count: Value::Const(3),
            },
        ]),
    )
}

/// Glistening Oil — infect that eats the creature it enchants.
pub fn glistening_oil() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::AddCounter {
                    what: host(),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            },
        ],
        ..aura(
            "Glistening Oil",
            cost(&[b(), b()]),
            R::Creature,
            EquipBonus { keywords: vec![Keyword::Infect], ..Default::default() },
        )
    }
}

/// Shrine of Limitless Power — a discard per charge counter.
pub fn shrine_of_limitless_power() -> CardDefinition {
    shrine(
        "Shrine of Limitless Power",
        cost(&[generic(3)]),
        Color::Black,
        ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                },
                random: false,
            },
            ..Default::default()
        },
    )
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Artillerize — throw a permanent at any target for five.
pub fn artillerize() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Artifact.or(R::Creature),
            count: 1,
        }],
        ..instant(
            "Artillerize",
            cost(&[generic(3), r()]),
            Effect::DealDamage { to: target_any(), amount: Value::Const(5) },
        )
    }
}

/// Furnace Scamp — a one-drop that trades itself for three.
pub fn furnace_scamp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MaySacrificeSource {
                description: "Sacrifice this to deal 3 damage?".into(),
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::Const(3),
                }),
                else_: None,
            },
        }],
        ..creature(
            "Furnace Scamp",
            cost(&[r()]),
            vec![CreatureType::Phyrexian, CreatureType::Beast],
            1,
            1,
        )
    }
}

/// Geosurge — seven red mana for artifacts and creatures.
pub fn geosurge() -> CardDefinition {
    sorcery(
        "Geosurge",
        cost(&[r(), r(), r(), r()]),
        Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![Color::Red; 7]),
        },
    )
}

/// Whipflare — a Pyroclasm your artifacts walk through.
pub fn whipflare() -> CardDefinition {
    sorcery(
        "Whipflare",
        cost(&[generic(1), r()]),
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::Artifact.negate())),
            amount: Value::Const(2),
        },
    )
}

/// Victorious Destruction — Stone Rain or Shatter, plus a life.
pub fn victorious_destruction() -> CardDefinition {
    sorcery(
        "Victorious Destruction",
        cost(&[generic(4), r()]),
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Land)) },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ONE,
            },
        ]),
    )
}

/// Scrapyard Salvo — the artifact graveyard as a burn spell.
pub fn scrapyard_salvo() -> CardDefinition {
    sorcery(
        "Scrapyard Salvo",
        cost(&[generic(1), r(), r()]),
        Effect::DealDamage {
            to: target_filtered(R::Player.or(R::Planeswalker)),
            amount: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Artifact },
        },
    )
}

/// Ruthless Invasion — only artifact creatures may block.
pub fn ruthless_invasion() -> CardDefinition {
    sorcery(
        "Ruthless Invasion",
        cost(&[generic(3), phyrexian(Color::Red)]),
        Effect::MatchingCantBlockThisTurn { filter: R::Creature.and(R::Artifact.negate()) },
    )
}

/// Slag Fiend — as big as the artifact graveyards are deep.
pub fn slag_fiend() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::CardTypeInAllGraveyards(CardType::Artifact)),
        ..creature(
            "Slag Fiend",
            cost(&[r()]),
            vec![CreatureType::Phyrexian, CreatureType::Construct],
            0,
            0,
        )
    }
}

/// Slash Panther — a hasty four-power body for {R} or two life.
pub fn slash_panther() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        ..artifact_creature(
            "Slash Panther",
            cost(&[generic(4), phyrexian(Color::Red)]),
            vec![CreatureType::Phyrexian, CreatureType::Cat],
            4,
            2,
        )
    }
}

/// Razor Swine — first strike plus infect.
pub fn razor_swine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Infect],
        ..creature(
            "Razor Swine",
            cost(&[generic(2), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Boar],
            2,
            1,
        )
    }
}

/// Immolating Souleater — pump for {R} or two life, all day.
pub fn immolating_souleater() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[phyrexian(Color::Red)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Immolating Souleater",
            cost(&[generic(2)]),
            vec![CreatureType::Phyrexian, CreatureType::Dog],
            1,
            1,
        )
    }
}

/// Moltensteel Dragon — a life-fuelled dragon.
pub fn moltensteel_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[phyrexian(Color::Red)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Moltensteel Dragon",
            cost(&[generic(4), phyrexian(Color::Red), phyrexian(Color::Red)]),
            vec![CreatureType::Phyrexian, CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Kiln Walker — a wall that hits like a truck.
pub fn kiln_walker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..artifact_creature(
            "Kiln Walker",
            cost(&[generic(3)]),
            vec![CreatureType::Phyrexian, CreatureType::Construct],
            0,
            3,
        )
    }
}

/// Priest of Urabrask — three red mana on arrival.
pub fn priest_of_urabrask() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![Color::Red; 3]),
        })],
        ..creature(
            "Priest of Urabrask",
            cost(&[generic(2), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Cleric],
            2,
            1,
        )
    }
}

/// Vulshok Refugee — a red-proof red beater.
pub fn vulshok_refugee() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red)],
        ..creature(
            "Vulshok Refugee",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            2,
        )
    }
}

/// Flameborn Viron — a plain 6/4.
pub fn flameborn_viron() -> CardDefinition {
    creature(
        "Flameborn Viron",
        cost(&[generic(4), r(), r()]),
        vec![CreatureType::Phyrexian, CreatureType::Insect],
        6,
        4,
    )
}

/// Ogre Menial — a pumpable infect wall.
pub fn ogre_menial() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Ogre Menial",
            cost(&[generic(3), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Ogre],
            0,
            4,
        )
    }
}

/// Fallen Ferromancer — a repeatable infect ping.
pub fn fallen_ferromancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Fallen Ferromancer",
            cost(&[generic(3), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Shrine of Burning Rage — damage equal to the pile.
pub fn shrine_of_burning_rage() -> CardDefinition {
    shrine(
        "Shrine of Burning Rage",
        cost(&[generic(2)]),
        Color::Red,
        ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
            },
            ..Default::default()
        },
    )
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Leeching Bite — a pump and a shrink in one instant.
pub fn leeching_bite() -> CardDefinition {
    instant(
        "Leeching Bite",
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Glissa's Scorn — Shatter with a life drain.
pub fn glissas_scorn() -> CardDefinition {
    instant(
        "Glissa's Scorn",
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ONE,
            },
        ]),
    )
}

/// Corrosive Gale — an X-sized sweeper for fliers.
pub fn corrosive_gale() -> CardDefinition {
    sorcery(
        "Corrosive Gale",
        cost(&[x(), phyrexian(Color::Green)]),
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            amount: Value::XFromCost,
        },
    )
}

/// Fresh Meat — a 3/3 for every creature you have lost this turn.
pub fn fresh_meat() -> CardDefinition {
    instant(
        "Fresh Meat",
        cost(&[generic(3), g()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ControllerCreaturesDiedThisTurn,
            definition: TokenDefinition {
                name: "Beast".into(),
                power: 3,
                toughness: 3,
                colors: vec![Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Beast],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    )
}

/// Viridian Harvest — six life when the artifact goes.
pub fn viridian_harvest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::EnchantedBySource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(6) },
        }],
        ..aura("Viridian Harvest", cost(&[g()]), R::Artifact, EquipBonus::default())
    }
}

/// Death-Hood Cobra — reach or deathtouch, as the combat demands.
pub fn death_hood_cobra() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            self_grant(cost(&[generic(1), g()]), Keyword::Reach),
            self_grant(cost(&[generic(1), g()]), Keyword::Deathtouch),
        ],
        ..creature(
            "Death-Hood Cobra",
            cost(&[generic(1), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Snake],
            2,
            2,
        )
    }
}

/// Rotted Hystrix — a plain 3/6 wall of a beast.
pub fn rotted_hystrix() -> CardDefinition {
    creature(
        "Rotted Hystrix",
        cost(&[generic(4), g()]),
        vec![CreatureType::Phyrexian, CreatureType::Beast],
        3,
        6,
    )
}

/// Spinebiter — infect that ignores its blockers.
pub fn spinebiter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect, Keyword::AssignsDamageAsThoughUnblocked],
        ..creature(
            "Spinebiter",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Beast],
            3,
            4,
        )
    }
}

/// Thundering Tanadon — a trampler for {G}{G} or four life.
pub fn thundering_tanadon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..artifact_creature(
            "Thundering Tanadon",
            cost(&[generic(4), phyrexian(Color::Green), phyrexian(Color::Green)]),
            vec![CreatureType::Phyrexian, CreatureType::Beast],
            5,
            4,
        )
    }
}

/// Insatiable Souleater — trample for {G} or two life.
pub fn insatiable_souleater() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_grant(
            cost(&[phyrexian(Color::Green)]),
            Keyword::Trample,
        )],
        ..artifact_creature(
            "Insatiable Souleater",
            cost(&[generic(4)]),
            vec![CreatureType::Phyrexian, CreatureType::Beast],
            5,
            1,
        )
    }
}

/// Vital Splicer — a Golem plus a regeneration engine for it.
pub fn vital_splicer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: phyrexian_golem(),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Regenerate {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Golem).and(R::ControlledByYou),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Vital Splicer",
            cost(&[generic(3), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Artificer],
            1,
            1,
        )
    }
}

/// Brutalizer Exarch — tutor to the top, or bottom a problem permanent.
pub fn brutalizer_exarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature,
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            },
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::Creature.negate())),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::Bottom,
                },
            },
        ]))],
        ..creature(
            "Brutalizer Exarch",
            cost(&[generic(5), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Shrine of Boundless Growth — the pile cashes out as colorless mana.
pub fn shrine_of_boundless_growth() -> CardDefinition {
    shrine(
        "Shrine of Boundless Growth",
        cost(&[generic(3)]),
        Color::Green,
        ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                }),
            },
            ..Default::default()
        },
    )
}

// ── Artifacts and gold ──────────────────────────────────────────────────────

/// Darksteel Relic — an indestructible nothing.
pub fn darksteel_relic() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Indestructible],
        ..artifact("Darksteel Relic", ManaCost::default())
    }
}

/// Hovermyr — a flying, vigilant Myr.
pub fn hovermyr() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..artifact_creature("Hovermyr", cost(&[generic(2)]), vec![CreatureType::Myr], 1, 2)
    }
}

/// Blinding Souleater — a tapper for {W} or two life.
pub fn blinding_souleater() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[phyrexian(Color::White)]),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..artifact_creature(
            "Blinding Souleater",
            cost(&[generic(3)]),
            vec![CreatureType::Phyrexian, CreatureType::Cleric],
            1,
            3,
        )
    }
}

/// Pestilent Souleater — rentable infect.
pub fn pestilent_souleater() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_grant(
            cost(&[phyrexian(Color::Black)]),
            Keyword::Infect,
        )],
        ..artifact_creature(
            "Pestilent Souleater",
            cost(&[generic(5)]),
            vec![CreatureType::Phyrexian, CreatureType::Insect],
            3,
            3,
        )
    }
}

/// Sickleslicer — a living weapon that makes a 2/2.
pub fn sickleslicer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![living_weapon()],
        ..equipment(
            "Sickleslicer",
            cost(&[generic(3)]),
            cost(&[generic(4)]),
            EquipBonus { power: 2, toughness: 2, ..Default::default() },
        )
    }
}

/// Necropouncer — a hasty living weapon.
pub fn necropouncer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![living_weapon()],
        ..equipment(
            "Necropouncer",
            cost(&[generic(6)]),
            cost(&[generic(2)]),
            EquipBonus {
                power: 3,
                toughness: 1,
                keywords: vec![Keyword::Haste],
                ..Default::default()
            },
        )
    }
}

/// Mycosynth Wellspring — a basic land on the way in and on the way out.
pub fn mycosynth_wellspring() -> CardDefinition {
    let fetch = Effect::MayDo {
        description: "Search for a basic land card?".into(),
        body: Box::new(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
        }),
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(fetch.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
                effect: fetch,
            },
        ],
        ..artifact("Mycosynth Wellspring", cost(&[generic(2)]))
    }
}

/// Surge Node — six counters that walk onto your other artifacts.
pub fn surge_node() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Charge, Value::Const(6))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::Charge, 1)),
            effect: Effect::AddCounter {
                what: target_filtered(R::Artifact),
                kind: CounterType::Charge,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..artifact("Surge Node", cost(&[generic(1)]))
    }
}

/// Unwinding Clock — your artifacts untap on everyone's turn.
pub fn unwinding_clock() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Untap all artifacts you control during each other player's untap step.",
            effect: StaticEffect::UntapYoursEachUntapStepFiltered(R::Artifact),
        }],
        ..artifact("Unwinding Clock", cost(&[generic(4)]))
    }
}

/// Isolation Cell — their creatures cost two life or {2}.
pub fn isolation_cell() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::MayPayBy {
                who: PlayerRef::TriggerEventPlayer,
                description: "Pay {2} to avoid losing 2 life?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::Const(2),
                })),
            },
        }],
        ..artifact("Isolation Cell", cost(&[generic(4)]))
    }
}

/// Jor Kadeen, the Prevailer — a metalcraft anthem on a first-striker.
pub fn jor_kadeen_the_prevailer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Metalcraft — Creatures you control get +3/+0 as long as you control \
                          three or more artifacts.",
            effect: StaticEffect::AnthemForFilterIf {
                filter: R::Creature.and(R::ControlledByYou),
                power: 3,
                toughness: 0,
                keywords: vec![],
                condition: Predicate::MetalcraftActive { who: PlayerRef::You },
                all_players: false,
            },
        }],
        ..creature(
            "Jor Kadeen, the Prevailer",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            5,
            4,
        )
    }
}

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Tormentor Exarch — a pump or a shrink on arrival.
pub fn tormentor_exarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ZERO,
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Tormentor Exarch",
            cost(&[generic(3), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Entomber Exarch — raise a creature, or strip a noncreature card.
pub fn entomber_exarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Noncreature,
            },
        ]))],
        ..creature(
            "Entomber Exarch",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Act of Aggression — a threaten for {R}{R} or four life.
pub fn act_of_aggression() -> CardDefinition {
    instant(
        "Act of Aggression",
        cost(&[generic(3), phyrexian(Color::Red), phyrexian(Color::Red)]),
        Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Ichor Explosion — a wrath the size of whatever you fed it.
pub fn ichor_explosion() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..sorcery(
            "Ichor Explosion",
            cost(&[generic(5), b(), b()]),
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Negate(Box::new(Value::SacrificedPower)),
                toughness: Value::Negate(Box::new(Value::SacrificedPower)),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Greenhilt Trainee — a pump engine that only turns on once it's big.
pub fn greenhilt_trainee() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            condition: Some(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::PowerAtLeast(4),
            }),
            ..Default::default()
        }],
        ..creature(
            "Greenhilt Trainee",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// "An opponent is poisoned."
fn an_opponent_is_poisoned() -> Predicate {
    Predicate::ValueAtLeast(Value::PoisonCountersOf(PlayerRef::EachOpponent), Value::ONE)
}

/// Viridian Betrayers — infect once the poison starts flowing.
pub fn viridian_betrayers() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has infect as long as an opponent is poisoned.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Infect,
                condition: an_opponent_is_poisoned(),
            },
        }],
        ..creature(
            "Viridian Betrayers",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Elf, CreatureType::Warrior],
            3,
            1,
        )
    }
}

/// Mycosynth Fiend — as big as the poison you have dealt.
pub fn mycosynth_fiend() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each poison counter your opponents have.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::PoisonCountersOf(PlayerRef::EachOpponent),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..creature(
            "Mycosynth Fiend",
            cost(&[generic(2), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            2,
            2,
        )
    }
}

/// Phyrexian Swarmlord — an infect Insect per poison counter, every upkeep.
pub fn phyrexian_swarmlord() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::PoisonCountersOf(PlayerRef::EachOpponent),
                definition: TokenDefinition {
                    name: "Phyrexian Insect".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Green],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Infect],
                    ..Default::default()
                },
            },
        }],
        ..creature(
            "Phyrexian Swarmlord",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Insect, CreatureType::Horror],
            4,
            4,
        )
    }
}

/// Whispering Specter — cash it in to strip a card per poison counter.
pub fn whispering_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Infect],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MaySacrificeSource {
                description: "Sacrifice this to strip their hand?".into(),
                then: Box::new(Effect::Discard {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::PoisonCountersOf(PlayerRef::TriggerEventPlayer),
                    random: false,
                }),
                else_: None,
            },
        }],
        ..creature(
            "Whispering Specter",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Specter],
            1,
            1,
        )
    }
}

/// Caged Sun — an anthem and a mana doubler for the chosen colour.
pub fn caged_sun() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseColorForSelf)],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control of the chosen color get +1/+1.",
                effect: StaticEffect::AnthemForChosenColor { power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Whenever a land's ability causes you to add one or more mana of \
                              the chosen color, add an additional one mana of that color.",
                effect: StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: false,
                    filter: R::Land,
                    extra: crate::effect::ExtraManaKind::ChosenColor,
                    while_monarch: false,
                },
            },
        ],
        ..artifact("Caged Sun", cost(&[generic(6)]))
    }
}

/// Cathedral Membrane — a wall that takes its blockers with it.
pub fn cathedral_membrane() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::CreaturesBlockedBySourceThisTurn,
                amount: Value::Const(6),
            },
        }],
        ..artifact_creature(
            "Cathedral Membrane",
            cost(&[generic(1), phyrexian(Color::White)]),
            vec![CreatureType::Phyrexian, CreatureType::Wall],
            0,
            3,
        )
    }
}

/// Conversion Chamber — eat artifact cards, spit out Golems.
pub fn conversion_chamber() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Seq(vec![
                    Effect::ExileWithSource {
                        what: target_filtered(R::Artifact.and(R::InGraveyard)),
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                remove_counter_cost: Some((CounterType::Charge, 1)),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: phyrexian_golem(),
                },
                ..Default::default()
            },
        ],
        ..artifact("Conversion Chamber", cost(&[generic(3)]))
    }
}

/// Gremlin Mine — four damage to an artifact creature, or four counters off.
pub fn gremlin_mine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::Artifact)),
                    amount: Value::Const(4),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::RemoveCounter {
                    what: target_filtered(R::Artifact.and(R::Creature.negate())),
                    kind: CounterType::Charge,
                    amount: Value::Const(4),
                },
                ..Default::default()
            },
        ],
        ..artifact("Gremlin Mine", cost(&[generic(1)]))
    }
}

/// Etched Monstrosity — a 10/10 that starts crushed, and draws when freed.
pub fn etched_monstrosity() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::Const(5))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            remove_counter_cost: Some((CounterType::MinusOneMinusOne, 5)),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Etched Monstrosity",
            cost(&[generic(5)]),
            vec![CreatureType::Phyrexian, CreatureType::Golem],
            10,
            10,
        )
    }
}

/// Lashwrithe — a living weapon that scales with your Swamps.
pub fn lashwrithe() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![living_weapon()],
        ..equipment(
            "Lashwrithe",
            cost(&[generic(4)]),
            cost(&[phyrexian(Color::Black), phyrexian(Color::Black)]),
            EquipBonus {
                scale: Some(crate::card::EquipScale {
                    filter: R::HasLandType(crate::card::LandType::Swamp),
                    per_power: 1,
                    per_toughness: 1,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
    }
}

/// Exclusion Ritual — exile it and lock its name out of the game.
pub fn exclusion_ritual() -> CardDefinition {
    CardDefinition {
        name: "Exclusion Ritual",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ExileWithSource {
            what: target_filtered(R::Permanent.and(R::Land.negate())),
        })],
        static_abilities: vec![StaticAbility {
            description: "Players can't cast spells with the same name as the exiled card.",
            effect: StaticEffect::OpponentsCantCastNamesExiledWithSource,
        }],
        ..Default::default()
    }
}

/// Psychic Surgery — mine every shuffle for a card to exile.
pub fn psychic_surgery() -> CardDefinition {
    CardDefinition {
        name: "Psychic Surgery",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LibraryShuffled, EventScope::OpponentControl),
            effect: Effect::MayDo {
                description: "Look at the top two cards and exile one?".into(),
                body: Box::new(Effect::LookTopKeepOneRestToGraveyard {
                    count: Value::Const(2),
                    who: Some(PlayerRef::TriggerEventPlayer),
                    exile_rest: true,
                    rest_bottom_random: false,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Praetor's Grasp — steal a card out of their deck and cast it later.
pub fn praetors_grasp() -> CardDefinition {
    sorcery(
        "Praetor's Grasp",
        cost(&[generic(1), b(), b()]),
        Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::Target(0),
                filter: R::Any,
                to: ZoneDest::ExileWithSourceStamp,
            },
            Effect::GrantMayPlay {
                what: Selector::CardExiledWithSource,
                duration: crate::card::MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ]),
    )
}

/// Parasitic Implant — the host rots away into a Myr for you.
pub fn parasitic_implant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::SacrificeSelected { what: host() },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: phyrexian_myr(),
                },
            ]),
        }],
        ..aura("Parasitic Implant", cost(&[generic(3), b()]), R::Creature, EquipBonus::default())
    }
}

/// Phyrexian Ingester — swallows a creature and wears its stats.
pub fn phyrexian_ingester() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile a nontoken creature?".into(),
            body: Box::new(Effect::ExileWithSource {
                what: target_filtered(R::Creature.and(R::NotToken)),
            }),
        })],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +X/+Y, where X is the exiled creature card's power \
                          and Y is its toughness.",
            effect: StaticEffect::PumpSelfByExiledWithStats,
        }],
        ..creature(
            "Phyrexian Ingester",
            cost(&[generic(6), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Beast],
            3,
            3,
        )
    }
}

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// "You may reveal this from your opening hand" — the Chancellor cycle's
/// first-upkeep payoff.
fn chancellor_reveal(body: Effect) -> crate::effect::OpeningHandEffect {
    crate::effect::OpeningHandEffect::RevealForDelayedTrigger {
        kind: crate::effect::DelayedTriggerKind::YourNextUpkeep,
        body,
    }
}

/// Chancellor of the Dross — three life off each opponent before the game starts.
pub fn chancellor_of_the_dross() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        opening_hand: Some(chancellor_reveal(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(3),
        })),
        ..creature(
            "Chancellor of the Dross",
            cost(&[generic(4), b(), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Vampire],
            6,
            6,
        )
    }
}

/// Chancellor of the Forge — a free hasty Goblin, and a swarm when it lands.
pub fn chancellor_of_the_forge() -> CardDefinition {
    let goblin = TokenDefinition {
        name: "Phyrexian Goblin".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin],
            ..Default::default()
        },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    };
    CardDefinition {
        opening_hand: Some(chancellor_reveal(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: goblin.clone(),
        })),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::PermanentCountControlledByMatching(PlayerRef::You, R::Creature),
            definition: goblin,
        })],
        ..creature(
            "Chancellor of the Forge",
            cost(&[generic(4), r(), r(), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Giant],
            5,
            5,
        )
    }
}

/// Chancellor of the Spires — a pre-game mill, then a free spell off their yard.
pub fn chancellor_of_the_spires() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        opening_hand: Some(chancellor_reveal(Effect::Mill {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(7),
        })),
        triggered_abilities: vec![etb(Effect::CastWithoutPayingImmediate {
            what: target_filtered(
                R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::InGraveyard)
                    .and(R::ControlledByOpponent),
            ),
            source_zone: crate::card::Zone::Graveyard,
            exile_after: false,
            copy: false,
            reduce_generic: 0,
                                pay_own_cost: false,
        })],
        ..creature(
            "Chancellor of the Spires",
            cost(&[generic(4), u(), u(), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Sphinx],
            5,
            7,
        )
    }
}

/// Omen Machine — nobody draws; everybody plays off the top instead.
pub fn omen_machine() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players can't draw cards.",
            effect: StaticEffect::PlayersSkipDraws,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Draw),
                EventScope::AnyPlayer,
            ),
            effect: Effect::ExileTopUntilNonlandMayPlay {
                who: PlayerRef::ActivePlayer,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                free: true,
                hand_unless_mv_below: None,
                grant_to_exiling_player: true,
            },
        }],
        ..artifact("Omen Machine", cost(&[generic(6)]))
    }
}

/// Arm with Aether — your team bounces what it hits this turn.
pub fn arm_with_aether() -> CardDefinition {
    sorcery(
        "Arm with Aether",
        cost(&[generic(2), u()]),
        Effect::GrantTriggeredAbilityThisTurnToMatching {
            filter: R::Creature.and(R::ControlledByYou),
            trigger: Box::new(TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::SelfSource,
                ),
                effect: Effect::MayDo {
                    description: "Bounce a creature that player controls?".into(),
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                },
            }),
        },
    )
}

/// Hex Parasite — eats counters off anything and swells for each one.
pub fn hex_parasite() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), phyrexian(Color::Black)]),
            effect: Effect::Seq(vec![
                Effect::RemoveCountersUpTo {
                    what: target_filtered(R::Permanent),
                    amount: Value::XFromCost,
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::CountersRemovedThisEffect,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..artifact_creature(
            "Hex Parasite",
            cost(&[generic(1)]),
            vec![CreatureType::Phyrexian, CreatureType::Insect],
            1,
            1,
        )
    }
}

/// Rage Extractor — every Phyrexian-mana spell throws its mana value at something.
pub fn rage_extractor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasPhyrexianManaInCost,
                },
            ),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..artifact("Rage Extractor", cost(&[generic(4), phyrexian(Color::Red)]))
    }
}

/// Invader Parasite — imprint a land and burn everyone who plays another.
pub fn invader_parasite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::ExileWithSource { what: target_filtered(R::Land) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Land.and(R::SameNameAsExiledWithSource),
                    }),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(
                        Selector::TriggerSource,
                    ))),
                    amount: Value::Const(2),
                },
            },
        ],
        ..creature(
            "Invader Parasite",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Insect],
            3,
            2,
        )
    }
}

/// Myr Superion — a 5/6 for {2} that only creature-produced mana can cast
/// (CR 106.6b).
pub fn myr_superion() -> CardDefinition {
    CardDefinition {
        spend_only_creature_mana: true,
        ..artifact_creature(
            "Myr Superion",
            cost(&[generic(2)]),
            vec![CreatureType::Myr],
            5,
            6,
        )
    }
}

/// Bludgeon Brawl — every other artifact becomes an Equipment with equip {X}
/// and "Equipped creature gets +X/+0", X being its own mana value.
pub fn bludgeon_brawl() -> CardDefinition {
    CardDefinition {
        name: "Bludgeon Brawl",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Each noncreature, non-Equipment artifact is an Equipment with \
                          equip {X} and \"Equipped creature gets +X/+0,\" where X is that \
                          artifact's mana value.",
            effect: StaticEffect::ArtifactsAreEquipment,
        }],
        ..Default::default()
    }
}
