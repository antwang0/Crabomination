//! Commander: the top 600's last claimed gaps (Ozai, Ardbert, Hinata, Ian
//! Malcolm, Megatron, Maarika, Ivy, Imodane, Anzrag, Saruman of Many Colors).
//! Tests in `tests/recent_b/cmdr_edhrec18.rs`.

use crate::card::{
    AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CumulativeUpkeepCost, CreatureType, EnchantmentSubtype, EquipBonus, EquipScale, EventKind,
    EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::game::types::TurnStep;
use std::sync::Arc;
use crate::effect::shortcut::{etb, heroic, on_dies, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost};

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

fn aura(name: &'static str, mana: ManaCost, host: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(host) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn pump(what: Selector, p: Value, t: Value) -> Effect {
    Effect::PumpPT { what, power: p, toughness: t, duration: Duration::EndOfTurn }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

/// "Target creature must be blocked this turn if able."
fn lure_target() -> Effect {
    grant(target_filtered(R::Creature), Keyword::MustBeBlocked)
}

// ── Anzrag, the Quake-Mole ───────────────────────────────────────────────────

/// General Marhault Elsdragon — {2}{R}{G} 4/4 legendary Elf Warrior. Whenever
/// a creature you control becomes blocked, it gets +3/+3 until end of turn for
/// each creature blocking it (counted on resolution, 2022-09-09 ruling).
pub fn general_marhault_elsdragon() -> CardDefinition {
    let per_blocker = || Value::Times(Box::new(Value::Const(3)), Box::new(Value::BlockersOf(Box::new(Selector::TriggerSource))));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
            effect: pump(Selector::TriggerSource, per_blocker(), per_blocker()),
        }],
        ..legend(
            "General Marhault Elsdragon",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Withstand Death — {G} instant. Target creature gains indestructible until
/// end of turn.
pub fn withstand_death() -> CardDefinition {
    spell(
        "Withstand Death",
        cost(&[g()]),
        CardType::Instant,
        grant(target_filtered(R::Creature), Keyword::Indestructible),
    )
}

/// You Look Upon the Tarrasque — {4}{G} instant. Choose one: prevent all
/// combat damage that would be dealt to you and creatures you control this
/// turn; or target creature gets +5/+5 and indestructible until end of turn,
/// and every creature your opponents control able to block it this turn does.
pub fn you_look_upon_the_tarrasque() -> CardDefinition {
    spell(
        "You Look Upon the Tarrasque",
        cost(&[generic(4), g()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::PreventAllCombatDamageToPlayerThisTurn { who: PlayerRef::You },
                Effect::PreventCombatDamageToMatchingThisTurn { filter: R::Creature.and(R::ControlledByYou) },
            ]),
            Effect::Seq(vec![
                pump(target_filtered(R::Creature), Value::Const(5), Value::Const(5)),
                grant(Selector::Target(0), Keyword::Indestructible),
                // CR 509.1c — a Lure requirement on the creature, so one that
                // enters later this turn is bound too.
                grant(Selector::Target(0), Keyword::AllMustBlock),
            ]),
        ]),
    )
}

/// Compelled Duel — {1}{G} sorcery. Target creature gets +3/+3 until end of
/// turn and must be blocked this turn if able.
pub fn compelled_duel() -> CardDefinition {
    spell(
        "Compelled Duel",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![lure_target(), pump(Selector::Target(0), Value::Const(3), Value::Const(3))]),
    )
}

/// Irresistible Prey — {G} sorcery. Target creature must be blocked this turn
/// if able. Draw a card.
pub fn irresistible_prey() -> CardDefinition {
    spell("Irresistible Prey", cost(&[g()]), CardType::Sorcery, Effect::Seq(vec![lure_target(), draw(1)]))
}

// ── Imodane, the Pyrohammer ──────────────────────────────────────────────────

/// Seismic Strike — {2}{R} instant. Deals damage to target creature equal to
/// the number of Mountains you control.
pub fn seismic_strike() -> CardDefinition {
    spell(
        "Seismic Strike",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::PermanentCountControlledByMatching(
                PlayerRef::You,
                R::Land.and(R::HasLandType(LandType::Mountain)),
            ),
        },
    )
}

/// Shivan Meteor — {3}{R}{R} sorcery. 13 damage to target creature. Suspend
/// 2—{1}{R}{R}.
pub fn shivan_meteor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(2, cost(&[generic(1), r(), r()]))],
        ..spell(
            "Shivan Meteor",
            cost(&[generic(3), r(), r()]),
            CardType::Sorcery,
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(13) },
        )
    }
}

// ── Ivy, Gleeful Spellthief ──────────────────────────────────────────────────

/// Spellwild Ouphe — {1}{G} 1/3 Ouphe. Spells that target it cost {2} less to
/// cast — any player's (2007-05-01 ruling).
pub fn spellwild_ouphe() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells that target this creature cost {2} less to cast.",
            effect: StaticEffect::CostReductionForYourSpellsTargetingThis { amount: 2, any_caster: true, opponents_only: false },
        }],
        ..creature("Spellwild Ouphe", cost(&[generic(1), g()]), vec![CreatureType::Ouphe], 1, 3)
    }
}

/// Auramancer's Guise — {2}{U}{U} Aura. Enchanted creature gets +2/+2 for each
/// Aura attached to it (itself included, 2007-02-01 ruling) and has vigilance.
pub fn auramancers_guise() -> CardDefinition {
    aura(
        "Auramancer's Guise",
        cost(&[generic(2), u(), u()]),
        R::Creature,
        EquipBonus {
            keywords: vec![Keyword::Vigilance],
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 2,
                per_toughness: 2,
                count_host_attachments: Some(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
                ..Default::default()
            }),
            ..Default::default()
        },
    )
}

/// Rune of Might — {1}{G} Aura Rune, enchant permanent. When it enters, draw a
/// card. An enchanted creature gets +1/+1 and has trample; on an Equipment,
/// the creature it equips does.
pub fn rune_of_might() -> CardDefinition {
    let mut c = aura(
        "Rune of Might",
        cost(&[generic(1), g()]),
        R::Permanent,
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Trample],
            relays_through_equipment: true,
            ..Default::default()
        },
    );
    c.triggered_abilities = vec![etb(draw(1))];
    c.subtypes.enchantment_subtypes.push(crate::card::EnchantmentSubtype::Rune);
    c
}

/// Sixth Sense — {G} Aura. Enchanted creature has "Whenever this creature
/// deals combat damage to a player, you may draw a card."
pub fn sixth_sense() -> CardDefinition {
    aura(
        "Sixth Sense",
        cost(&[g()]),
        R::Creature,
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo { description: "Draw a card?".into(), body: Box::new(draw(1)) },
            }],
            ..Default::default()
        },
    )
}

/// Scale the Heights — {2}{G} sorcery. Put a +1/+1 counter on up to one target
/// creature. You gain 2 life. You may play an additional land this turn. Draw
/// a card.
pub fn scale_the_heights() -> CardDefinition {
    spell(
        "Scale the Heights",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE },
            draw(1),
        ]),
    )
}

// ── Ardbert, Warrior of Darkness ─────────────────────────────────────────────

/// Abigale, Eloquent First-Year — {W/B}{W/B} 1/1 legendary Bird Bard; flying,
/// first strike, lifelink. When it enters, up to one other target creature
/// loses all abilities (indefinitely, 2025-11-17 ruling), then gets a flying,
/// a first strike and a lifelink counter.
pub fn abigale_eloquent_first_year() -> CardDefinition {
    let counter = |keyword| Effect::AddKeywordCounter { what: Selector::Target(0), keyword, amount: Value::ONE };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::LoseAllAbilities {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    duration: Duration::Permanent,
                },
                counter(Keyword::Flying),
                counter(Keyword::FirstStrike),
                counter(Keyword::Lifelink),
            ])),
        })],
        ..legend(
            "Abigale, Eloquent First-Year",
            cost(&[hybrid(Color::White, Color::Black), hybrid(Color::White, Color::Black)]),
            vec![CreatureType::Bird, CreatureType::Bard],
            1,
            1,
        )
    }
}

/// Brigone, Soldier of Meletis — {1}{W} 2/2 legendary Human Soldier, vigilance.
/// Heroic: a +1/+1 counter on it. {T}, remove a +1/+1 counter from it: draw a
/// card.
pub fn brigone_soldier_of_meletis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: draw(1),
            ..Default::default()
        }],
        ..legend(
            "Brigone, Soldier of Meletis",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

// ── Ian Malcolm, Chaotician ──────────────────────────────────────────────────

/// Noggle Ransacker — {2}{U/R} 2/1 Noggle Rogue. When it enters, each player
/// draws two cards, then discards a card at random.
pub fn noggle_ransacker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
            Effect::Discard { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE, random: true },
        ]))],
        ..creature(
            "Noggle Ransacker",
            cost(&[generic(2), hybrid(Color::Blue, Color::Red)]),
            vec![CreatureType::Noggle, CreatureType::Rogue],
            2,
            1,
        )
    }
}

// ── Hinata, Dawn-Crowned ─────────────────────────────────────────────────────

/// Reality Spasm — {X}{U}{U} instant. Choose one: tap X target permanents; or
/// untap X target permanents.
pub fn reality_spasm() -> CardDefinition {
    let x_targets = |each: Effect| Effect::TargetsExactlyX {
        body: Box::new(Effect::ApplyToTargets {
            max_targets: 20,
            min_targets: 0,
            filter: R::Permanent,
            effect: Box::new(each),
        }),
    };
    spell(
        "Reality Spasm",
        cost(&[x(), u(), u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            x_targets(Effect::Tap { what: Selector::Target(0) }),
            x_targets(Effect::Untap { what: Selector::Target(0), up_to: None }),
        ]),
    )
}

/// "Instant or sorcery."
fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// Imodane, the Pyrohammer — {2}{R}{R} 4/4 legendary Human Knight. Whenever an
/// instant or sorcery spell you control that targets only a single creature
/// deals damage to that creature, Imodane deals that much damage to each
/// opponent.
pub fn imodane_the_pyrohammer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::YourControl)
                .with_filter(Predicate::ResolvingInstantOrSorceryTargetsOnlyTriggerTarget),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..legend(
            "Imodane, the Pyrohammer",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Chandra, Hope's Beacon — {4}{R}{R} planeswalker, loyalty 5. Whenever you
/// cast an instant or sorcery spell, copy it (new targets allowed); once each
/// turn. +2: two mana in any combination of colors. +1: exile your top five;
/// until the end of your next turn you may cast an instant or sorcery spell
/// from among them. −X: X damage to each of up to two targets.
pub fn chandra_hopes_beacon() -> CardDefinition {
    let exiled_spells = || Selector::ExiledThisResolution { filter: instant_or_sorcery() };
    let x_damage = |slot| Effect::DealDamage {
        to: Selector::TargetFiltered { slot, filter: R::any_target() },
        amount: Value::XFromCost,
    };
    CardDefinition {
        name: "Chandra, Hope's Beacon",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Chandra], ..Default::default() },
        base_loyalty: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: instant_or_sorcery() })
                .once_per_turn(),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::Const(2)) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(5) },
                        to: ZoneDest::Exile,
                    },
                    Effect::GrantMayPlay {
                        what: exiled_spells(),
                        duration: MayPlayDuration::EndOfControllersNextTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                    Effect::OneCastAmongGranted { what: exiled_spells() },
                ]),
                ..Default::default()
            },
            LoyaltyAbility { condition: None,
                loyalty_cost: 0,
                x_cost: true,
                effect: Effect::OptionalTargets { min: 0, body: Box::new(Effect::Seq(vec![x_damage(0), x_damage(1)])) },
            },
        ],
        ..Default::default()
    }
}

/// Meteor Swarm — {X}{R}{R}{R} sorcery. 8 damage divided as you choose among
/// X target creatures and/or planeswalkers.
pub fn meteor_swarm() -> CardDefinition {
    spell(
        "Meteor Swarm",
        cost(&[x(), r(), r(), r()]),
        CardType::Sorcery,
        Effect::TargetsExactlyX {
            body: Box::new(Effect::DealDamageDivided {
                total: Value::Const(8),
                filter: R::Creature.or(R::Planeswalker),
                max_targets: 8,
                retaliate_to_source: false,
            }),
        },
    )
}

// ── Maarika, Brutal Gladiator ────────────────────────────────────────────────

/// "Creature you don't control."
fn theirs() -> R {
    R::Creature.and(R::ControlledByOpponent)
}

/// Maarika, Brutal Gladiator — {2}{B}{R}{G} 7/4 legendary Human Warrior. It
/// must be blocked if able; indestructible during your turn. Whenever it deals
/// damage to a creature dealt excess damage this turn, that creature's
/// controller sacrifices a noncreature, nonland permanent.
pub fn maarika_brutal_gladiator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustBeBlocked],
        static_abilities: vec![StaticAbility {
            description: "As long as it's your turn, Maarika has indestructible.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Indestructible,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::SelfSource).with_filter(
                Predicate::ValueAtLeast(Value::ExcessDamageDealtThisTurnTo(Box::new(Selector::Target(0))), Value::ONE),
            ),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                count: Value::ONE,
                filter: R::Nonland.and(R::Noncreature),
            },
        }],
        ..legend(
            "Maarika, Brutal Gladiator",
            cost(&[generic(2), b(), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            7,
            4,
        )
    }
}

/// Boxing Ring — {1}{G} artifact. Whenever a creature you control enters, it
/// fights up to one target creature you don't control with the same mana
/// value. {T}: create a Treasure; only if you control a creature that fought
/// this turn.
pub fn boxing_ring() -> CardDefinition {
    CardDefinition {
        name: "Boxing Ring",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Fight {
                    attacker: Selector::TriggerSource,
                    defender: target_filtered(theirs().and(R::ManaValueEqualsTriggerSubject)),
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::SelectorExists(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::FoughtThisTurn),
            ))),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hunter's Bow — {1}{G} Equipment. When it enters, attach it to target
/// creature you control; that creature deals damage equal to its power to up
/// to one target creature you don't control. Equipped creature has reach and
/// ward {2}. Equip {1}.
pub fn hunters_bow() -> CardDefinition {
    CardDefinition {
        name: "Hunter's Bow",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Reach, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::AttachSourceTo { host: target_filtered(R::Creature.and(R::ControlledByYou)) },
                Effect::DealDamageEqualToPower {
                    source: Selector::Target(0),
                    target: Selector::TargetFiltered { slot: 1, filter: theirs() },
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Gimli's Reckless Might — {3}{R} enchantment. Creatures you control have
/// haste. Formidable — whenever you attack, if creatures you control have
/// total power 8 or greater, target attacking creature you control fights up
/// to one target creature you don't control.
pub fn gimlis_reckless_might() -> CardDefinition {
    CardDefinition {
        name: "Gimli's Reckless Might",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::FormidableActive { who: PlayerRef::You }),
            effect: Effect::OptionalTargets {
                min: 1,
                body: Box::new(Effect::Fight {
                    attacker: target_filtered(R::Creature.and(R::ControlledByYou).and(R::IsAttacking)),
                    defender: Selector::TargetFiltered { slot: 1, filter: theirs() },
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Ardbert, Warrior of Darkness (continued) ─────────────────────────────────

/// "Legendary."
fn legendary() -> R {
    R::HasSupertype(Supertype::Legendary)
}

/// The Kenriths' Royal Funeral — {2}{W}{B} legendary enchantment. When it
/// enters, exile up to two target legendary creature cards from your
/// graveyard; you draw X and lose X life, X the greatest mana value among
/// them. Legendary spells you cast cost {1} less for each card exiled with it.
pub fn the_kenriths_royal_funeral() -> CardDefinition {
    let exiled = || Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Exile, filter: R::ExiledWithSource };
    let x = || Value::HighestManaValueAmong(Box::new(exiled()));
    CardDefinition {
        name: "The Kenriths' Royal Funeral",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature.and(legendary()).and(R::InYourGraveyard),
                effect: Box::new(Effect::ExileLinked { what: Selector::Target(0) }),
            },
            Effect::Draw { who: Selector::You, amount: x() },
            Effect::LoseLife { who: Selector::You, amount: x() },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Legendary spells you cast cost {1} less to cast for each card exiled with this.",
            effect: StaticEffect::CostReductionByValue {
                filter: legendary(),
                amount: Value::CardsExiledWithSourceCount,
            },
        }],
        ..Default::default()
    }
}

/// Kaya the Inexorable — {3}{W}{B} planeswalker, loyalty 5. +1: up to one
/// target nontoken creature gains "when this dies or is put into exile, return
/// it to its owner's hand and create a 1/1 white flying Spirit", marked by a
/// ghostform counter (a reminder only, 2021-02-05 ruling). −3: exile
/// target nonland permanent. −7: an emblem — at the beginning of your upkeep,
/// you may cast a legendary spell from your hand, graveyard or exile free.
pub fn kaya_the_inexorable() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    let ghostform = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(spirit.clone()) },
        ]),
    };
    let grant = |kind, what| Effect::GrantTriggeredAbility {
        what,
        trigger: Box::new(ghostform(kind)),
        duration: Duration::Permanent,
    };
    CardDefinition {
        name: "Kaya the Inexorable",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Kaya], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::AddCounter {
                            what: target_filtered(R::Creature.and(R::NotToken)),
                            kind: CounterType::Ghostform,
                            amount: Value::ONE,
                        },
                        grant(EventKind::CreatureDied, Selector::Target(0)),
                        grant(EventKind::CardExiled, Selector::Target(0)),
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Move { what: target_filtered(R::Nonland.and(R::Permanent)), to: ZoneDest::Exile },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Kaya the Inexorable".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                        effect: Effect::CastOneFreeFromHandGraveyardOrExile { filter: legendary() },
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Megatron, Tyrant ─────────────────────────────────────────────────────────

/// A Transformers front face: a legendary artifact creature Robot whose More
/// Than Meets the Eye cost casts it converted onto `back` (CR 702.162).
fn robot(name: &'static str, c: ManaCost, mtmte: ManaCost, p: i32, t: i32, back: CardDefinition) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        alternative_cost: Some(AlternativeCost { mana_cost: mtmte, converted: true, ..Default::default() }),
        back_face: Some(Box::new(back)),
        ..legend(name, c, vec![CreatureType::Robot], p, t)
    }
}

/// A Transformers back face: a legendary artifact Vehicle with living metal
/// (CR 702.161), coloured by its indicator (CR 105.2c).
fn vehicle(name: &'static str, p: i32, t: i32, colors: Vec<Color>, mut keywords: Vec<Keyword>) -> CardDefinition {
    keywords.insert(0, Keyword::LivingMetal);
    CardDefinition {
        name,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: p,
        toughness: t,
        color_indicator: colors,
        keywords,
        ..Default::default()
    }
}

fn convert() -> Effect {
    Effect::Transform { what: Selector::This }
}

/// Megatron, Destructive Force — the back. Attacking, it may sacrifice
/// another artifact; when it does, it deals that artifact's mana value to
/// target creature, any excess to that creature's controller instead, and an
/// excess converts it.
pub fn megatron_destructive_force() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Sacrifice another artifact?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificeAndRemember {
                        who: PlayerRef::You,
                        filter: R::Artifact.and(R::OtherThanSource),
                    },
                    Effect::Reflexive {
                        body: Box::new(Effect::Seq(vec![
                            Effect::DealDamageExcessToController {
                                to: target_filtered(R::Creature),
                                amount: Value::SacrificedManaValue,
                            },
                            Effect::If {
                                cond: Predicate::ExcessDamageDealtThisResolution,
                                then: Box::new(convert()),
                                else_: Box::new(Effect::Noop),
                            },
                        ])),
                    },
                ])),
            },
        }],
        ..vehicle("Megatron, Destructive Force", 4, 5, vec![Color::White, Color::Black, Color::Red], vec![])
    }
}

/// Megatron, Tyrant — {3}{R}{W}{B} 7/5; More Than Meets the Eye {1}{R}{W}{B}.
/// Your opponents can't cast spells during combat. At the beginning of each of
/// your postcombat main phases, you may convert it; if you do, add {C} for
/// each 1 life your opponents have lost this turn.
pub fn megatron_tyrant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't cast spells during combat.",
            effect: StaticEffect::OpponentsCantCastDuringCombat,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Convert Megatron for {C} per life your opponents lost this turn?".into(),
                body: Box::new(Effect::Seq(vec![
                    convert(),
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colorless(Value::TotalLifeLostThisTurn(PlayerRef::EachOpponent)),
                    },
                ])),
            },
        }],
        ..robot(
            "Megatron, Tyrant",
            cost(&[generic(3), r(), w(), b()]),
            cost(&[generic(1), r(), w(), b()]),
            7,
            5,
            megatron_destructive_force(),
        )
    }
}

/// Starscream, Seeker Leader — the back; flying, menace, haste. Dealing combat
/// damage to a player with no monarch makes that player the monarch; when you
/// become the monarch, it converts.
pub fn starscream_seeker_leader() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                    .with_filter(Predicate::Not(Box::new(Predicate::IsMonarch { who: PlayerRef::EachPlayer }))),
                effect: Effect::BecomeMonarch { who: PlayerRef::Target(0) },
            },
            TriggeredAbility { event: EventSpec::new(EventKind::BecameMonarch, EventScope::YourControl), effect: convert() },
        ],
        ..vehicle(
            "Starscream, Seeker Leader",
            2,
            3,
            vec![Color::Black],
            vec![Keyword::Flying, Keyword::Menace, Keyword::Haste],
        )
    }
}

/// Starscream, Power Hungry — {3}{B} 2/3 flying; More Than Meets the Eye
/// {2}{B}. Whenever you draw a card while you're the monarch, target opponent
/// loses 2 life. Whenever one or more creatures deal combat damage to you, it
/// converts.
pub fn starscream_power_hungry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                    .with_filter(Predicate::IsMonarch { who: PlayerRef::You }),
                effect: Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::SelfSource).once_per_batch(),
                effect: convert(),
            },
        ],
        ..robot(
            "Starscream, Power Hungry",
            cost(&[generic(3), b()]),
            cost(&[generic(2), b()]),
            2,
            3,
            starscream_seeker_leader(),
        )
    }
}

/// Mishra, Tamer of Mak Fawa — {3}{B}{R} 4/4 legendary Human Artificer.
/// Permanents you control have "Ward—Sacrifice a permanent." Each artifact
/// card in your graveyard has unearth {1}{B}{R}.
pub fn mishra_tamer_of_mak_fawa() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Permanents you control have \"Ward—Sacrifice a permanent.\"",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Permanent.and(R::ControlledByYou)),
                    keyword: Keyword::Ward(WardCost::SacrificePermanents(1)),
                },
            },
            StaticAbility {
                description: "Each artifact card in your graveyard has unearth {1}{B}{R}.",
                effect: StaticEffect::GraveyardCardsHaveUnearth {
                    filter: R::Artifact,
                    cost: cost(&[generic(1), b(), r()]),
                },
            },
        ],
        ..legend(
            "Mishra, Tamer of Mak Fawa",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            4,
            4,
        )
    }
}

/// Su-Chi Cave Guard — {8} 8/8 Construct; vigilance, ward {4}. When it dies,
/// add eight {C}, kept as steps and phases end this turn.
pub fn su_chi_cave_guard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Ward(WardCost::Mana(cost(&[generic(4)])))],
        triggered_abilities: vec![on_dies(Effect::AddColorlessKeptThisTurn {
            who: PlayerRef::You,
            amount: Value::Const(8),
            restriction: None,
        })],
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature("Su-Chi Cave Guard", cost(&[generic(8)]), vec![CreatureType::Construct], 8, 8)
    }
}

// ── Fire Lord Ozai ───────────────────────────────────────────────────────────

/// Avatar Roku, Firebender — {3}{R}{R}{R} 6/6 legendary Human Avatar. Whenever
/// a player attacks, add six {R}, kept until end of combat. {R}{R}{R}: target
/// creature gets +3/+0 until end of turn.
pub fn avatar_roku_firebender() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer),
            effect: Effect::AddRedUntilEndOfCombat { amount: Value::Const(6) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), r(), r()]),
            effect: pump(target_filtered(R::Creature), Value::Const(3), Value::ZERO),
            ..Default::default()
        }],
        ..legend(
            "Avatar Roku, Firebender",
            cost(&[generic(3), r(), r(), r()]),
            vec![CreatureType::Human, CreatureType::Avatar],
            6,
            6,
        )
    }
}

/// Braid of Fire — {1}{R} enchantment. Cumulative upkeep—Add {R}.
pub fn braid_of_fire() -> CardDefinition {
    CardDefinition {
        name: "Braid of Fire",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::AddMana(Color::Red))],
        ..Default::default()
    }
}

/// Fire Lord Sozin — the back: 5/5 legendary Human Noble, menace, firebending
/// 3. Whenever it deals combat damage to a player, you may pay {X}; when you
/// do, put any number of target creature cards with total mana value X or
/// less from that player's graveyard onto the battlefield under your control.
pub fn fire_lord_sozin() -> CardDefinition {
    let reanimate = Effect::WithX {
        x: Value::XFromCost,
        body: Box::new(Effect::ReflexiveTrigger {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature
                    .and(R::InOpponentGraveyard)
                    .and(R::ControlledByTriggerPlayer)
                    .and(R::SlotsTotalManaValueAtMostX),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            }),
        }),
    };
    CardDefinition {
        name: "Fire Lord Sozin",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Noble], ..Default::default() },
        power: 5,
        toughness: 5,
        color_indicator: vec![Color::Black],
        keywords: vec![Keyword::Menace, Keyword::Firebending(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayPayX {
                description: "Pay {X} to steal creature cards with total mana value X or less?".into(),
                body: Box::new(reanimate),
            },
        }],
        ..Default::default()
    }
}

/// The Rise of Sozin — {4}{B}{B} Saga. I: destroy all creatures. II: name a
/// card; exile up to four cards with that name from target opponent's
/// graveyard, hand and library, then they shuffle (Ancient Vendetta's
/// `NameCardThenExileFromZones`). III: exile it, then return it transformed.
pub fn the_rise_of_sozin() -> CardDefinition {
    CardDefinition {
        name: "The Rise of Sozin",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::Destroy { what: Selector::EachPermanent(R::Creature) }),
            (2, Effect::NameCardThenExileFromZones { who: PlayerRef::Target(0), count: 4 }),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(fire_lord_sozin())),
        ..Default::default()
    }
}

// ── Hinata, Dawn-Crowned (continued) ─────────────────────────────────────────

/// Hinata, Dawn-Crowned — {1}{U}{R}{W} 4/4 legendary Kirin Spirit; flying,
/// trample. Spells you cast cost {1} less for each target; spells your
/// opponents cast cost {1} more for each target.
pub fn hinata_dawn_crowned() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast cost {1} less to cast for each target. Spells your opponents cast cost {1} more to cast for each target.",
            effect: StaticEffect::PerTargetSpellCost { yours_less: 1, opponents_more: 1 },
        }],
        ..legend(
            "Hinata, Dawn-Crowned",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Kirin, CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Soulfire Grand Master — {1}{W} 2/2 Human Monk, lifelink. Instant and
/// sorcery spells you control have lifelink. {2}{U/R}{U/R}: the next instant
/// or sorcery spell you cast this turn returns to your hand as it resolves
/// (from any zone, not only your hand — INCOMPLETE_CARDS).
pub fn soulfire_grand_master() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you control have lifelink.",
            effect: StaticEffect::YourInstantSorcerySpellsHaveLifelink,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), ur(), ur()]),
            effect: Effect::OnYourNextInstantSorceryThisTurn {
                body: Box::new(Effect::SpellReturnsToHandOnResolve { what: Selector::TriggerSource }),
            },
            ..Default::default()
        }],
        ..creature(
            "Soulfire Grand Master",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Monk],
            2,
            2,
        )
    }
}

/// March of Swirling Mist — {X}{U} instant. As an additional cost you may
/// exile any number of blue cards from your hand, {2} less each. Up to X
/// target creatures phase out.
pub fn march_of_swirling_mist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As an additional cost to cast this spell, you may exile any number of blue cards from your hand. This spell costs {2} less to cast for each card exiled this way.",
            effect: StaticEffect::ExileFromHandCostReduction { per: 2, filter: R::HasColor(Color::Blue) },
        }],
        ..spell(
            "March of Swirling Mist",
            cost(&[x(), u()]),
            CardType::Instant,
            Effect::CapTargetsAt {
                amount: Value::XFromCost,
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 20,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::PhaseOut { what: Selector::Target(0), until_source_leaves: false }),
                }),
            },
        )
    }
}

// ── Ian Malcolm, Chaotician (continued) ──────────────────────────────────────

/// Ian Malcolm's permission, handed to the active player: they may cast the
/// cards exiled with Ian that they don't own this turn, mana of any type.
fn ian_turn_grant() -> Effect {
    let theirs = || Selector::CardsInZone { who: PlayerRef::EachOpponent, zone: Zone::Exile, filter: R::ExiledWithSource };
    Effect::EachPlayerDoes {
        who: PlayerRef::ActivePlayer,
        body: Box::new(Effect::Seq(vec![
            Effect::GrantMayPlay {
                what: theirs(),
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
            Effect::GrantedCastOnly { what: theirs() },
        ])),
    }
}

/// Ian Malcolm, Chaotician — {1}{U}{R} 2/2 legendary Human Scientist.
/// Whenever a player draws their second card each turn, they exile their top
/// card with Ian. During each player's turn, that player may cast spells from
/// among the cards exiled with Ian they don't own, mana of any type. The
/// permission is stamped at each upkeep and after each exile, for the turn.
pub fn ian_malcolm_chaotician() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::AnyPlayer),
                effect: Effect::Seq(vec![
                    Effect::ExileLinked {
                        what: Selector::TopOfLibrary { who: PlayerRef::Triggerer, count: Value::ONE },
                    },
                    ian_turn_grant(),
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
                effect: ian_turn_grant(),
            },
        ],
        ..legend(
            "Ian Malcolm, Chaotician",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            2,
            2,
        )
    }
}

/// Hive Mind — {5}{U} enchantment. Whenever a player casts an instant or
/// sorcery spell, each other player copies it and may choose new targets.
pub fn hive_mind() -> CardDefinition {
    CardDefinition {
        name: "Hive Mind",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: instant_or_sorcery() }),
            effect: Effect::CopySpellForPlayer {
                what: Selector::TriggerSource,
                who: PlayerRef::EachPlayerExceptControllerOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Scrambleverse — {6}{R}{R} sorcery. For each nonland permanent, choose a
/// player at random; each player gains control of each permanent they were
/// chosen for. Untap those permanents.
pub fn scrambleverse() -> CardDefinition {
    let nonland = || R::Permanent.and(R::Nonland);
    spell(
        "Scrambleverse",
        cost(&[generic(6), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(nonland()),
                body: Box::new(Effect::WithRandomPlayer {
                    body: Box::new(Effect::GainControl {
                        what: Selector::TriggerSource,
                        to: Some(PlayerRef::ChosenPlayerOfSource),
                        duration: Duration::Permanent,
                    }),
                }),
            },
            Effect::Untap { what: Selector::EachPermanent(nonland()), up_to: None },
        ]),
    )
}

// ── Saruman of Many Colors ───────────────────────────────────────────────────

/// "Enchantment, instant, or sorcery."
fn enchantment_instant_or_sorcery() -> R {
    R::HasCardType(CardType::Enchantment).or(instant_or_sorcery())
}

/// "Whenever you cast your second spell each turn."
fn second_spell(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(2) }),
        effect,
    }
}

/// Saruman of Many Colors — {3}{W}{U}{B} 5/4 legendary Avatar Wizard.
/// Ward—Discard an enchantment, instant, or sorcery card. Whenever you cast
/// your second spell each turn, each opponent mills two; then exile target
/// enchantment, instant, or sorcery card with mana value at most that spell's
/// from an opponent's graveyard, copy it, and you may cast the copy free.
pub fn saruman_of_many_colors() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::DiscardMatching(Box::new(enchantment_instant_or_sorcery()), 1))],
        triggered_abilities: vec![second_spell(Effect::Seq(vec![
            Effect::Mill { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            Effect::WithX {
                x: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::Seq(vec![
                        Effect::ExileWithSource {
                            what: target_filtered(
                                enchantment_instant_or_sorcery()
                                    .and(R::InOpponentGraveyard)
                                    .and(R::ManaValueAtMostXFromCost),
                            ),
                        },
                        Effect::CopyCardAndCastFree { what: Selector::Target(0) },
                    ])),
                }),
            },
        ]))],
        ..legend(
            "Saruman of Many Colors",
            cost(&[generic(3), w(), u(), b()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            5,
            4,
        )
    }
}

/// Saruman the White — {4}{U} 4/4 legendary Avatar Wizard, ward {2}. Whenever
/// you cast your second spell each turn, amass Orcs 2.
pub fn saruman_the_white() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![second_spell(Effect::Amass {
            who: PlayerRef::You,
            count: Value::Const(2),
            extra_type: Some(CreatureType::Orc),
        })],
        ..legend("Saruman the White", cost(&[generic(4), u()]), vec![CreatureType::Avatar, CreatureType::Wizard], 4, 4)
    }
}

/// Press the Enemy — {2}{U}{U} instant. Return target spell or nonland
/// permanent an opponent controls to its owner's hand; you may cast an
/// instant or sorcery spell with equal or lesser mana value from your hand
/// free. (The two kinds of target are offered as two modes.)
pub fn press_the_enemy() -> CardDefinition {
    let free = || Effect::MayCastFromHandFreeMatching {
        filter: instant_or_sorcery(),
        max_mv: Value::ManaValueOf(Box::new(Selector::Target(0))),
        else_: Box::new(Effect::Noop),
    };
    spell(
        "Press the Enemy",
        cost(&[generic(2), u(), u()]),
        CardType::Instant,
        // One slot holds either kind: `Move` lifts a spell off the stack.
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::Permanent.and(R::Nonland).or(R::IsSpellOnStack).and(R::ControlledByOpponent),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            free(),
        ]),
    )
}

/// Sorcerous Squall — {6}{U}{U}{U} sorcery, delve. Target opponent mills
/// nine; then you may cast an instant or sorcery spell from that player's
/// graveyard without paying its mana cost, exiled instead of binned.
pub fn sorcerous_squall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Delve],
        ..spell(
            "Sorcerous Squall",
            cost(&[generic(6), u(), u(), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Mill { who: target_filtered(R::OpponentPlayer), amount: Value::Const(9) },
                // Not targeted: the spell is chosen as this resolves.
                Effect::CastAnyOrderWithoutPaying {
                    what: Selector::CardsInZone {
                        who: PlayerRef::Target(0),
                        zone: crate::card::Zone::Graveyard,
                        filter: instant_or_sorcery(),
                    },
                    source_zone: crate::card::Zone::Graveyard,
                    filter: None,
                    cap: Some(Value::ONE),
                    total_mana_value: None,
                    exile_after: true,
                },
            ]),
        )
    }
}
