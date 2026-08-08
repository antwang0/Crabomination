//! Scourge (SCG) — the Dragon/Storm set: Storm, the Decree cycle's
//! cycling triggers, "turned face up" payoffs and the Dragon Auras.
//! Tests in `classic_sets/scg`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, WardCost,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, StaticEffect, Value,
    ZoneDest,
    shortcut::{draw, etb, target_any, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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

fn aura(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

/// "When this creature is turned face up, [effect]." (CR 708.8)
fn on_turn_up(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
        effect,
    }
}

/// "Whenever a permanent is turned face up, [effect]" — the SCG morph-matters
/// payoffs, which watch every player's flips.
fn on_any_turn_up(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::AnyPlayer),
        effect,
    }
}

/// "When you cycle this card, [effect]." — the Decree cycle's may-trigger.
fn on_cycle(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
        effect: Effect::MayDo {
            description: "Use the cycling trigger?".into(),
            body: Box::new(effect),
        },
    }
}

/// The four Dragon Auras share "When a creature with mana value 6 or greater
/// enters, you may return this card from your graveyard to the battlefield
/// attached to that creature."
fn dragon_aura_return() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::ManaValueAtLeast(6)),
            }),
        effect: Effect::MayDo {
            description: "Return this Aura attached to that creature?".into(),
            body: Box::new(Effect::ReturnSelfAttachedToTrigger),
        },
    }
}

/// The greatest mana value among permanents you control — the SCG "domain of
/// fatties" count (Torrent of Fire, Dispersal Shield, Reward the Faithful).
fn greatest_mv() -> Value {
    Value::GreatestManaValueAmongPermanents(PlayerRef::You)
}

// ── White ───────────────────────────────────────────────────────────────────

/// Astral Steel — a Storm-scaled +1/+2.
pub fn astral_steel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..instant(
            "Astral Steel",
            cost(&[generic(2), w()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Aven Farseer — grows off every morph flip on the table.
pub fn aven_farseer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_any_turn_up(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Aven Farseer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Aven Liberator — Morph {3}{W} into a colour-of-your-choice protection trick.
pub fn aven_liberator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(3), w()]))],
        triggered_abilities: vec![on_turn_up(Effect::GrantProtectionFromChosenColor {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Aven Liberator",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Dawn Elemental — a 3/3 flier nothing can damage.
pub fn dawn_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to this creature.",
            effect: StaticEffect::PreventAllDamageToThis,
        }],
        ..creature(
            "Dawn Elemental",
            cost(&[w(), w(), w(), w()]),
            vec![CreatureType::Elemental],
            3,
            3,
        )
    }
}

/// Dragonstalker — a flier the set's Dragons can't touch.
pub fn dragonstalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::ProtectionFromCreatureType(CreatureType::Dragon),
        ],
        ..creature(
            "Dragonstalker",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Exiled Doomsayer — every morph cost on the table goes up {2}.
pub fn exiled_doomsayer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All morph costs cost {2} more.",
            effect: StaticEffect::MorphCostsMore { amount: 2 },
        }],
        ..creature(
            "Exiled Doomsayer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Noble Templar — a vigilant wall that can be cashed in for a Plains.
pub fn noble_templar() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains),
        ],
        ..creature(
            "Noble Templar",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Soldier],
            3,
            6,
        )
    }
}

/// Rain of Blades — one damage to every attacker, for {W}.
pub fn rain_of_blades() -> CardDefinition {
    instant(
        "Rain of Blades",
        cost(&[w()]),
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            amount: Value::ONE,
        },
    )
}

/// Recuperate — six life, or six points of prevention.
pub fn recuperate() -> CardDefinition {
    instant(
        "Recuperate",
        cost(&[generic(3), w()]),
        Effect::ChooseMode(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(6) },
            Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: Value::Const(6),
            },
        ]),
    )
}

/// Reward the Faithful — your biggest permanent's mana value, as life.
pub fn reward_the_faithful() -> CardDefinition {
    instant(
        "Reward the Faithful",
        cost(&[w()]),
        Effect::GainLife { who: Selector::Player(PlayerRef::Target(0)), amount: greatest_mv() },
    )
}

/// Silver Knight — the red-hosing first striker.
pub fn silver_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Protection(Color::Red)],
        ..creature(
            "Silver Knight",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Wipe Clean — exile an enchantment, or cycle it away.
pub fn wipe_clean() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(3)]))],
        ..instant(
            "Wipe Clean",
            cost(&[generic(1), w()]),
            Effect::Move {
                what: target_filtered(R::Enchantment),
                to: ZoneDest::Exile,
            },
        )
    }
}

/// Zealous Inquisitor — bounces one point of damage onto another creature.
pub fn zealous_inquisitor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::RedirectNextDamage {
                target: Selector::This,
                to: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Zealous Inquisitor",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Aphetto Runecaster — every morph flip is a card.
pub fn aphetto_runecaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_any_turn_up(Effect::MayDo {
            description: "Draw a card?".into(),
            body: Box::new(draw(1)),
        })],
        ..creature(
            "Aphetto Runecaster",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Coast Watcher — a green-proof one-drop flier.
pub fn coast_watcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Green)],
        ..creature(
            "Coast Watcher",
            cost(&[generic(1), u()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Dispersal Shield — counters anything your biggest permanent outweighs.
pub fn dispersal_shield() -> CardDefinition {
    instant(
        "Dispersal Shield",
        cost(&[generic(1), u()]),
        Effect::If {
            cond: Predicate::ValueAtMost(
                Value::ManaValueOf(Box::new(Selector::Target(0))),
                greatest_mv(),
            ),
            then: Box::new(Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) }),
            else_: Box::new(Effect::Noop),
        },
    )
}

/// Dragon Wings — flight now, or a free re-attach when a fatty lands.
pub fn dragon_wings() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(1), u()]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        triggered_abilities: vec![dragon_aura_return()],
        ..aura("Dragon Wings", cost(&[generic(1), u()]))
    }
}

/// Hindering Touch — a Storm-scaled soft counter.
pub fn hindering_touch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..instant(
            "Hindering Touch",
            cost(&[generic(3), u()]),
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(2)]),
                exile: false,
                extra_generic: None,
            },
        )
    }
}

/// Mercurial Kite — its hits stay tapped through the next untap step.
pub fn mercurial_kite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::SkipNextUntap { what: Selector::Target(0) },
            ]),
        }],
        ..creature("Mercurial Kite", cost(&[generic(3), u()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Raven Guild Master — a connect mills ten into exile.
pub fn raven_guild_master() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), u(), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::TopOfLibrary {
                    who: PlayerRef::Target(0),
                    count: Value::Const(10),
                },
                to: ZoneDest::Exile,
            },
        }],
        ..creature(
            "Raven Guild Master",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Mutant],
            1,
            1,
        )
    }
}

/// Riptide Survivor — the flip is a rummage: discard two, draw three.
pub fn riptide_survivor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), u(), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
            draw(3),
        ]))],
        ..creature(
            "Riptide Survivor",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Scornful Egotist — an 8-drop 1/1 whose whole point is a {U} morph.
pub fn scornful_egotist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[u()]))],
        ..creature(
            "Scornful Egotist",
            cost(&[generic(7), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Stabilizer — nobody cycles.
pub fn stabilizer() -> CardDefinition {
    CardDefinition {
        name: "Stabilizer",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Players can't cycle cards.",
            effect: StaticEffect::PlayersCantCycle,
        }],
        ..Default::default()
    }
}

/// Temporal Fissure — a Storm-scaled bounce.
pub fn temporal_fissure() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..sorcery(
            "Temporal Fissure",
            cost(&[generic(4), u()]),
            Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        )
    }
}

/// Thundercloud Elemental — sweeps small blockers, or grounds the sky.
pub fn thundercloud_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u()]),
                effect: Effect::Tap {
                    what: Selector::EachPermanent(R::Creature.and(R::ToughnessAtMost(2))),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u()]),
                effect: Effect::LoseKeyword { duration: Duration::EndOfTurn,
                    what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                    keyword: Keyword::Flying,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Thundercloud Elemental",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Elemental],
            3,
            4,
        )
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Bladewing's Thrall — flies while you have a Dragon, and rides one back.
pub fn bladewings_thrall() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has flying as long as you control a Dragon.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Flying,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Dragon).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dragon),
                }),
            effect: Effect::MayDo {
                description: "Return this card to the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature(
            "Bladewing's Thrall",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Zombie],
            3,
            3,
        )
    }
}

/// Bladewing the Risen — reanimates a Dragon and pumps the flight.
pub fn bladewing_the_risen() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return a Dragon permanent card from your graveyard?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(
                    R::PermanentCard
                        .and(R::HasCreatureType(CreatureType::Dragon))
                        .and(R::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), r()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Dragon)),
                ),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Bladewing the Risen",
            cost(&[generic(3), b(), b(), r(), r()]),
            vec![CreatureType::Zombie, CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Cabal Conditioning — discard equal to your biggest permanent's mana value.
pub fn cabal_conditioning() -> CardDefinition {
    sorcery(
        "Cabal Conditioning",
        cost(&[generic(6), b()]),
        Effect::Discard { who: Selector::Player(PlayerRef::Target(0)), amount: greatest_mv(), random: false },
    )
}

/// Cabal Interrogator — {X} strips the best of X revealed cards.
pub fn cabal_interrogator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), b()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::DiscardChosenFromRevealed {
                from: Selector::Player(PlayerRef::Target(0)),
                reveal: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..creature(
            "Cabal Interrogator",
            cost(&[generic(1), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Chill Haunting — exile X from your graveyard for a -X/-X.
pub fn chill_haunting() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::ExileFromGraveyard {
            filter: R::Creature,
            count: 2,
        }],
        ..instant(
            "Chill Haunting",
            cost(&[generic(1), b()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Clutch of Undeath — +3/+3 on a Zombie, -3/-3 on anything else.
pub fn clutch_of_undeath() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            power: -3,
            toughness: -3,
            // +3/+3 on a Zombie: the -3/-3 base plus a +6/+6 swing.
            conditional: vec![crate::card::ConditionalEquipBonus {
                host_filter: R::HasCreatureType(CreatureType::Zombie),
                power: 6,
                toughness: 6,
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Clutch of Undeath", cost(&[generic(3), b(), b()]))
    }
}

/// Consumptive Goo — shrinks a creature and grows itself.
pub fn consumptive_goo() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), b()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Consumptive Goo", cost(&[b(), b()]), vec![CreatureType::Ooze], 1, 1)
    }
}

/// Dragon Shadow — +1/+0 and fear, back from the yard on a fatty.
pub fn dragon_shadow() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            keywords: vec![Keyword::Fear],
            ..Default::default()
        }),
        triggered_abilities: vec![dragon_aura_return()],
        ..aura("Dragon Shadow", cost(&[generic(1), b()]))
    }
}

/// Final Punishment — the turn's damage, charged again as life loss.
pub fn final_punishment() -> CardDefinition {
    sorcery(
        "Final Punishment",
        cost(&[generic(3), b(), b()]),
        Effect::LoseLife {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::DamageTakenThisTurn(PlayerRef::Target(0)),
        },
    )
}

/// Lingering Death — the enchanted creature dies at its controller's end step.
pub fn lingering_death() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::ControllerOf(Box::new(
                Selector::AttachedTo(Box::new(Selector::This)),
            )))),
            effect: Effect::SacrificeSelected {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..aura("Lingering Death", cost(&[generic(1), b()]))
    }
}

/// Nefashu — an attack shrinks up to five blockers.
pub fn nefashu() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 5,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..creature(
            "Nefashu",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Mutant],
            5,
            3,
        )
    }
}

/// Reaping the Graves — a Storm-scaled raise dead.
pub fn reaping_the_graves() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..instant(
            "Reaping the Graves",
            cost(&[generic(2), b()]),
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        )
    }
}

/// Skulltap — a creature for two cards.
pub fn skulltap() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..sorcery("Skulltap", cost(&[generic(1), b()]), draw(2))
    }
}

/// Unspeakable Symbol — three life buys a +1/+1 counter, any number of times.
pub fn unspeakable_symbol() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 3,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Unspeakable Symbol", cost(&[generic(1), b(), b()]))
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Bonethorn Valesk — every morph flip is a ping.
pub fn bonethorn_valesk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_any_turn_up(Effect::DealDamage {
            to: target_any(),
            amount: Value::ONE,
        })],
        ..creature("Bonethorn Valesk", cost(&[generic(4), r()]), vec![CreatureType::Beast], 4, 2)
    }
}

/// Chartooth Cougar — a firebreathing body, or a Mountain when you're short.
pub fn chartooth_cougar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Mountain)],
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
            "Chartooth Cougar",
            cost(&[generic(5), r()]),
            vec![CreatureType::Cat, CreatureType::Beast],
            4,
            4,
        )
    }
}

/// Dragon Breath — haste and firebreathing, back on a fatty.
pub fn dragon_breath() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::Haste],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![dragon_aura_return()],
        ..aura("Dragon Breath", cost(&[generic(1), r()]))
    }
}

/// Dragonspeaker Shaman — Dragons cost {2} less.
pub fn dragonspeaker_shaman() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Dragon spells you cast cost {2} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::HasCreatureType(CreatureType::Dragon),
                amount: 2,
            },
        }],
        ..creature(
            "Dragonspeaker Shaman",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Dragon Tyrant — a 6/6 double-striker on a {R}{R}{R}{R} lease.
pub fn dragon_tyrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::DoubleStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[r(), r(), r(), r()]) },
        }],
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
            "Dragon Tyrant",
            cost(&[generic(8), r(), r()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Enrage — {X}{R} for +X/+0.
pub fn enrage() -> CardDefinition {
    instant(
        "Enrage",
        cost(&[x(), r()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::XFromCost,
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Extra Arms — the enchanted creature pings on every attack.
pub fn extra_arms() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        }],
        ..aura("Extra Arms", cost(&[generic(4), r()]))
    }
}

/// Goblin Brigand — a 2/2 that has to swing.
pub fn goblin_brigand() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustAttack],
        ..creature(
            "Goblin Brigand",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Misguided Rage — a permanent of their choice, gone.
pub fn misguided_rage() -> CardDefinition {
    sorcery(
        "Misguided Rage",
        cost(&[generic(2), r()]),
        Effect::Sacrifice {
            who: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::Permanent,
        },
    )
}

/// Pyrostatic Pillar — every cheap spell costs its caster two life.
pub fn pyrostatic_pillar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::ManaValueAtMost(3),
                },
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::Const(2),
            },
        }],
        ..enchantment("Pyrostatic Pillar", cost(&[generic(1), r()]))
    }
}

/// Scattershot — a Storm-scaled ping.
pub fn scattershot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..instant(
            "Scattershot",
            cost(&[generic(2), r()]),
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
        )
    }
}

/// Torrent of Fire — damage equal to your biggest permanent's mana value.
pub fn torrent_of_fire() -> CardDefinition {
    sorcery(
        "Torrent of Fire",
        cost(&[generic(3), r(), r()]),
        Effect::DealDamage { to: target_any(), amount: greatest_mv() },
    )
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Accelerated Mutation — +X/+X for your biggest permanent's mana value.
pub fn accelerated_mutation() -> CardDefinition {
    instant(
        "Accelerated Mutation",
        cost(&[generic(3), g(), g()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: greatest_mv(),
            toughness: greatest_mv(),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Ancient Ooze — as big as the rest of your board costs.
pub fn ancient_ooze() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(crate::card::DynamicPt::TotalManaValueOfOtherControlledCreatures),
        ..creature("Ancient Ooze", cost(&[generic(5), g(), g()]), vec![CreatureType::Ooze], 0, 0)
    }
}

/// Break Asunder — artifact/enchantment removal that can be cycled away.
pub fn break_asunder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..sorcery(
            "Break Asunder",
            cost(&[generic(2), g(), g()]),
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        )
    }
}

/// Claws of Wirewood — three to every flier and every player.
pub fn claws_of_wirewood() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..sorcery(
            "Claws of Wirewood",
            cost(&[generic(3), g()]),
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                    amount: Value::Const(3),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(3),
                },
            ]),
        )
    }
}

/// Dragon Fangs — +1/+1 and trample, back on a fatty.
pub fn dragon_fangs() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        triggered_abilities: vec![dragon_aura_return()],
        ..aura("Dragon Fangs", cost(&[generic(1), g()]))
    }
}

/// Dragon Scales — +1/+2 and vigilance, back on a fatty.
pub fn dragon_scales() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 2,
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        }),
        triggered_abilities: vec![dragon_aura_return()],
        ..aura("Dragon Scales", cost(&[generic(1), w()]))
    }
}

/// Elvish Aberration — three green, or a Forest when you're short.
pub fn elvish_aberration() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Forest)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Colors(vec![Color::Green; 3]),
            },
            ..Default::default()
        }],
        ..creature(
            "Elvish Aberration",
            cost(&[generic(5), g()]),
            vec![CreatureType::Elf, CreatureType::Mutant],
            4,
            5,
        )
    }
}

/// Krosan Drover — the set's fatties cost {2} less.
pub fn krosan_drover() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast with mana value 6 or greater cost {2} less.",
            effect: StaticEffect::CostReduction {
                filter: R::Creature.and(R::ManaValueAtLeast(6)),
                amount: 2,
            },
        }],
        ..creature("Krosan Drover", cost(&[generic(3), g()]), vec![CreatureType::Elf], 2, 2)
    }
}

/// Krosan Warchief — cheaper Beasts, and a regeneration outlet.
pub fn krosan_warchief() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Beast spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::HasCreatureType(CreatureType::Beast),
                amount: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate {
                what: target_filtered(R::HasCreatureType(CreatureType::Beast)),
            },
            ..Default::default()
        }],
        ..creature("Krosan Warchief", cost(&[generic(2), g()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Kurgadon — every 6-drop creature spell is three counters.
pub fn kurgadon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ManaValueAtLeast(6)),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
        }],
        ..creature("Kurgadon", cost(&[generic(4), g()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Hunting Pack — a Storm-scaled 4/4.
pub fn hunting_pack() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..instant(
            "Hunting Pack",
            cost(&[generic(5), g(), g()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Beast".into(),
                    power: 4,
                    toughness: 4,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Beast],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        )
    }
}

/// Sprouting Vines — a Storm-scaled basic-land tutor.
pub fn sprouting_vines() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..instant(
            "Sprouting Vines",
            cost(&[generic(2), g()]),
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        )
    }
}

/// Titanic Bulvox — a 7/4 trampler you can sneak in face down.
pub fn titanic_bulvox() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Trample,
            Keyword::Morph(cost(&[generic(4), g(), g(), g()])),
        ],
        ..creature("Titanic Bulvox", cost(&[generic(6), g(), g()]), vec![CreatureType::Beast], 7, 4)
    }
}

/// Treetop Scout — only fliers can stop it.
pub fn treetop_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Flying)))],
        ..creature(
            "Treetop Scout",
            cost(&[g()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Woodcloaker — Morph {2}{G}{G} into a trample grant.
pub fn woodcloaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), g(), g()]))],
        triggered_abilities: vec![on_turn_up(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Trample,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Woodcloaker", cost(&[generic(5), g()]), vec![CreatureType::Elf], 3, 3)
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Ark of Blight — a one-shot land kill.
pub fn ark_of_blight() -> CardDefinition {
    CardDefinition {
        name: "Ark of Blight",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Land) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── The Decree cycle (CR 702.29 — cast it, or cycle it for the rider) ────────

/// Decree of Annihilation — the full board, or every land on the cycle.
pub fn decree_of_annihilation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(5), r(), r()]))],
        triggered_abilities: vec![on_cycle(Effect::ForEach {
            selector: Selector::EachPermanent(R::Land),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        })],
        ..sorcery(
            "Decree of Annihilation",
            cost(&[generic(8), r(), r()]),
            Effect::Seq(vec![
                Effect::Move {
                    what: Selector::EachPermanent(R::Artifact.or(R::Creature).or(R::Land)),
                    to: ZoneDest::Exile,
                },
                Effect::Move {
                    what: Selector::CardsInZone {
                        zone: crate::card::Zone::Graveyard,
                        who: PlayerRef::EachPlayer,
                        filter: R::Any,
                    },
                    to: ZoneDest::Exile,
                },
                Effect::Move {
                    what: Selector::CardsInZone {
                        zone: crate::card::Zone::Hand,
                        who: PlayerRef::EachPlayer,
                        filter: R::Any,
                    },
                    to: ZoneDest::Exile,
                },
            ]),
        )
    }
}

/// Decree of Pain — a wrath that refills, or a -2/-2 sweep on the cycle.
pub fn decree_of_pain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(3), b(), b()]))],
        triggered_abilities: vec![on_cycle(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
        ..sorcery(
            "Decree of Pain",
            cost(&[generic(6), b(), b()]),
            Effect::Seq(vec![
                draw_per_creature(),
                Effect::ForEach {
                    selector: Selector::EachPermanent(R::Creature),
                    body: Box::new(Effect::DestroyNoRegen { what: Selector::TriggerSource }),
                },
            ]),
        )
    }
}

/// "Draw a card for each creature destroyed this way" — counted before the
/// sweep, which matches the printed count for every destructible board.
fn draw_per_creature() -> Effect {
    Effect::Draw {
        who: Selector::You,
        amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature))),
    }
}

/// Decree of Savagery — four counters on the team, or on one creature.
pub fn decree_of_savagery() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(4), g(), g()]))],
        triggered_abilities: vec![on_cycle(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(4),
        })],
        ..instant(
            "Decree of Savagery",
            cost(&[generic(7), g(), g()]),
            Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            },
        )
    }
}

/// Decree of Silence — three counterspells on a clock, or one on the cycle.
pub fn decree_of_silence() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(4), u(), u()]))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
                effect: Effect::Seq(vec![
                    Effect::CounterSpell { what: Selector::TriggerSource },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Depletion,
                        amount: Value::ONE,
                    },
                    Effect::If {
                        cond: Predicate::SourceHasCountersAtLeast {
                            counter: CounterType::Depletion,
                            n: 3,
                        },
                        then: Box::new(Effect::SacrificeSource),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
            on_cycle(Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) }),
        ],
        ..enchantment("Decree of Silence", cost(&[generic(6), u(), u()]))
    }
}

// ── Second wave ─────────────────────────────────────────────────────────────

/// Alpha Status — +2/+2 per other creature sharing a type with the host.
pub fn alpha_status() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            scale: Some(crate::card::EquipScale {
                filter: R::Creature,
                per_power: 2,
                per_toughness: 2,
                count_sharing_type_with_host: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..aura("Alpha Status", cost(&[generic(2), g()]))
    }
}

/// Daru Spiritualist — targeting a Cleric of yours toughens it.
pub fn daru_spiritualist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Cleric)),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::ZERO,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Daru Spiritualist",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Edgewalker — Clerics cost {W}{B} less (colored pips only).
pub fn edgewalker() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Cleric spells you cast cost {W}{B} less to cast.",
            effect: StaticEffect::ColoredCostReduction {
                filter: R::HasCreatureType(CreatureType::Cleric),
                less: cost(&[w(), b()]),
            },
        }],
        ..creature(
            "Edgewalker",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Eternal Dragon — recurs itself in upkeep, or fetches a Plains.
pub fn eternal_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), w()]),
            from_graveyard: true,
            condition: Some(Predicate::CurrentStepIs(crate::game::TurnStep::Upkeep)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature(
            "Eternal Dragon",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Dragon, CreatureType::Spirit],
            5,
            5,
        )
    }
}

/// Frontline Strategist — the flip fogs every non-Soldier.
pub fn frontline_strategist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[w()]))],
        triggered_abilities: vec![on_turn_up(
            Effect::PreventAllCombatDamageByMatchingThisTurn {
                filter: R::Creature.and(R::Not(Box::new(R::HasCreatureType(
                    CreatureType::Soldier,
                )))),
            },
        )],
        ..creature(
            "Frontline Strategist",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Guilty Conscience — the enchanted creature hits itself just as hard.
pub fn guilty_conscience() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::EnchantedBySource),
            effect: Effect::DealDamage {
                to: Selector::AttachedTo(Box::new(Selector::This)),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..aura("Guilty Conscience", cost(&[w()]))
    }
}



// ── Wave 2: the non-mana morphs (CR 702.36b) and friends ────────────────────

/// Putrid Raptor — Morph—Discard a Zombie card.
pub fn putrid_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MorphCost(Box::new(WardCost::DiscardMatching(
            Box::new(R::HasCreatureType(CreatureType::Zombie)),
            1,
        )))],
        ..creature(
            "Putrid Raptor",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Dinosaur, CreatureType::Beast],
            4,
            4,
        )
    }
}

/// Zombie Cutthroat — Morph—Pay 5 life.
pub fn zombie_cutthroat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MorphCost(Box::new(WardCost::Life(5)))],
        ..creature(
            "Zombie Cutthroat",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Zombie],
            3,
            4,
        )
    }
}

/// Raven Guild Initiate — Morph—Return a Bird you control to its owner's hand.
pub fn raven_guild_initiate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MorphCost(Box::new(WardCost::ReturnMatchingToHand(
            Box::new(R::HasCreatureType(CreatureType::Bird)),
            1,
        )))],
        ..creature(
            "Raven Guild Initiate",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            4,
        )
    }
}

/// Skirk Volcanist — Morph—Sacrifice two Mountains; the flip is 3 damage split.
pub fn skirk_volcanist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MorphCost(Box::new(WardCost::SacrificeMatchingN(
            Box::new(R::HasLandType(LandType::Mountain)),
            2,
        )))],
        triggered_abilities: vec![on_turn_up(Effect::DealDamageDivided {
            total: Value::Const(3),
            filter: R::Creature,
            max_targets: 3,
            retaliate_to_source: false,
        })],
        ..creature("Skirk Volcanist", cost(&[generic(3), r()]), vec![CreatureType::Goblin], 3, 1)
    }
}

// ── Wave 2: the rest ────────────────────────────────────────────────────────

/// Frozen Solid — the enchanted creature stays tapped and dies to any damage.
pub fn frozen_solid() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::EnchantedBySource),
            effect: Effect::Destroy {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..aura("Frozen Solid", cost(&[generic(1), u(), u()]))
    }
}

/// One with Nature — the enchanted creature's hits fetch a basic.
pub fn one_with_nature() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Search for a basic land?".into(),
                    body: Box::new(Effect::Search {
                        who: PlayerRef::You,
                        filter: R::IsBasicLand,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    }),
                },
            }],
            ..Default::default()
        }),
        ..aura("One with Nature", cost(&[g()]))
    }
}

/// Consumptive Goo's sibling in blue-black removal shells: Soul Collector
/// reanimates whatever it kills.
pub fn soul_collector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[b(), b(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::DamagedBySourceThisTurn,
                },
            ),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..creature(
            "Soul Collector",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Vampire],
            3,
            4,
        )
    }
}



