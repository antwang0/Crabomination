//! Commander: 2026-10-24 (`01M2Ffkh`) — the Isu the Abominable, Scarlet Witch,
//! Stegron, Nael, Verazol, Chandra, Fire of Kaladesh, Brigid, Mavren Fein, Johan
//! and Liu Bei decks' closers. Tests in `tests/recent_b/cmdr_01m2_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect};
use crate::mana::{ManaCost, cost, g, generic, r, snow_mana, u, w};

fn snow_creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..Default::default()
    }
}

fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn cumulative_snow() -> Keyword {
    Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[snow_mana()])))
}

/// Isu the Abominable — {3}{U}{U} legendary snow 5/5 Yeti. Look at the top
/// card of your library any time; play snow lands and cast snow spells from
/// it. Another snow permanent of yours entering lets you pay {G}, {W} or {U}
/// for a +1/+1 counter (the colour is a mode, then the may-pay).
pub fn isu_the_abominable() -> CardDefinition {
    let counter = || Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE };
    let pay = |pip| Effect::MayPay {
        description: "Pay for a +1/+1 counter on Isu?".into(),
        mana_cost: cost(&[pip]),
        body: Box::new(counter()),
        else_: None,
    };
    let trigger = Effect::ChooseMode(vec![pay(g()), pay(w()), pay(u())]);
    CardDefinition {
        supertypes: vec![Supertype::Legendary, Supertype::Snow],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may play snow lands and cast snow spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::IsSnow },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSnow }),
            effect: trigger,
        }],
        ..snow_creature("Isu the Abominable", cost(&[generic(3), u(), u()]), vec![CreatureType::Yeti], 5, 5)
    }
}

/// Diamond Faerie — {2}{G}{W}{U} snow 3/3 flying Faerie. {1}{S}: snow
/// creatures you control get +1/+1 until end of turn.
pub fn diamond_faerie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), snow_mana()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsSnow).and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..snow_creature("Diamond Faerie", cost(&[generic(2), g(), w(), u()]), vec![CreatureType::Faerie], 3, 3)
    }
}

/// Frostpeak Yeti — {3}{U} snow 3/3 Yeti. {1}{S}: it can't be blocked this turn.
pub fn frostpeak_yeti() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), snow_mana()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..snow_creature("Frostpeak Yeti", cost(&[generic(3), u()]), vec![CreatureType::Yeti], 3, 3)
    }
}

/// Squall Drifter — {1}{W} snow 1/1 flying Elemental. {W}, {T}: tap target creature.
pub fn squall_drifter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w()]),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..snow_creature("Squall Drifter", cost(&[generic(1), w()]), vec![CreatureType::Elemental], 1, 1)
    }
}

/// Snow Devil — {1}{U} Aura. Enchanted creature has flying, and first strike
/// as long as it's blocking and you control a snow land.
pub fn snow_devil() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has first strike as long as it's blocking and you control a snow land.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::All(vec![
                    Predicate::EntityMatches { what: host(), filter: R::IsBlocking },
                    Predicate::SelectorExists(Selector::EachPermanent(
                        R::Land.and(R::IsSnow).and(R::ControlledByYou),
                    )),
                ]),
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::FirstStrike }),
            },
        }],
        ..aura("Snow Devil", cost(&[generic(1), u()]))
    }
}

/// Glacial Plating — {2}{W}{W} snow Aura. Cumulative upkeep {S}; enchanted
/// creature gets +3/+3 for each age counter on it.
pub fn glacial_plating() -> CardDefinition {
    let x = || Value::Times(
        Box::new(Value::Const(3)),
        Box::new(Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Age }),
    );
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        keywords: vec![cumulative_snow()],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +3/+3 for each age counter on this Aura.",
            effect: StaticEffect::PumpPTByValue { applies_to: host(), power: x(), toughness: x() },
        }],
        ..aura("Glacial Plating", cost(&[generic(2), w(), w()]))
    }
}

/// Cover of Winter — {2}{W} snow enchantment. Cumulative upkeep {S}; X of
/// each creature's combat damage to you or your creatures is prevented, X its
/// age counters; {S}: another age counter.
pub fn cover_of_winter() -> CardDefinition {
    CardDefinition {
        name: "Cover of Winter",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Enchantment],
        keywords: vec![cumulative_snow()],
        static_abilities: vec![StaticAbility {
            description: "If a creature would deal combat damage to you and/or one or more creatures you control, prevent X of that damage, where X is the number of age counters on this enchantment.",
            effect: StaticEffect::ReduceCreatureCombatDamageToYouAndYoursByCounters { counter: CounterType::Age },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[snow_mana()]),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Age, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Scarlet Witch, Wanda Maximoff's deck ────────────────────────────────────

/// A creature body (Scarlet Witch's deck on).
fn plain_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn plain_spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// Barging Sergeant — {4}{R} 4/2 haste Minotaur Soldier with mentor.
pub fn barging_sergeant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::effect::shortcut::mentor()],
        ..plain_creature("Barging Sergeant", cost(&[generic(4), r()]), vec![CreatureType::Minotaur, CreatureType::Soldier], 4, 2)
    }
}

/// Bladeback Sliver — {1}{R} 2/2 Sliver. Hellbent: your Slivers have "{T}:
/// this creature deals 1 damage to target player or planeswalker."
pub fn bladeback_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Hellbent — As long as you have no cards in hand, Sliver creatures you control have \"{T}: This creature deals 1 damage to target player or planeswalker.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Sliver)).and(R::ControlledByYou)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::ONE },
                    ..Default::default()
                },
                condition: Some(Predicate::HellbentActive { who: PlayerRef::You }),
            },
        }],
        ..plain_creature("Bladeback Sliver", cost(&[generic(1), r()]), vec![CreatureType::Sliver], 2, 2)
    }
}

/// Hellkite Whelp — {4}{R} 3/3 flying Dragon. Attacking, it deals 1 damage to
/// target creature defending player controls.
pub fn hellkite_whelp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                amount: Value::ONE,
            },
        }],
        ..plain_creature("Hellkite Whelp", cost(&[generic(4), r()]), vec![CreatureType::Dragon], 3, 3)
    }
}

/// Voldaren Duelist — {3}{R} 3/2 haste Vampire Warrior. ETB: target creature
/// can't block this turn.
pub fn voldaren_duelist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..plain_creature("Voldaren Duelist", cost(&[generic(3), r()]), vec![CreatureType::Vampire, CreatureType::Warrior], 3, 2)
    }
}

/// Prophetic Ravings — {R} Aura. Enchanted creature has haste and "{T},
/// Discard a card: Draw a card."
pub fn prophetic_ravings() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Haste],
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Prophetic Ravings", cost(&[r()]))
    }
}

/// Volatile Claws — {2}{R} instant. Your creatures get +2/+0 and gain all
/// creature types until end of turn.
pub fn volatile_claws() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    plain_spell(
        "Volatile Claws",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT { what: yours(), power: Value::Const(2), toughness: Value::ZERO, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: yours(), keyword: Keyword::Changeling, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Precision Bolt — {2}{R} sorcery. 3 damage to any target.
pub fn precision_bolt() -> CardDefinition {
    plain_spell(
        "Precision Bolt",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) },
    )
}

/// Vandalize — {4}{R} sorcery. Choose one or both — destroy target artifact;
/// destroy target land.
pub fn vandalize() -> CardDefinition {
    plain_spell(
        "Vandalize",
        cost(&[generic(4), r()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::Destroy { what: target_filtered(R::Land) },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

// ── Stegron the Dinosaur Man's deck ─────────────────────────────────────────

/// Nest Robber — {1}{R} 2/1 haste Dinosaur.
pub fn nest_robber() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        ..plain_creature("Nest Robber", cost(&[generic(1), r()]), vec![CreatureType::Dinosaur], 2, 1)
    }
}

/// Sun-Collared Raptor — {1}{R} 1/2 trample Dinosaur. {2}{R}: +3/+0 until end of turn.
pub fn sun_collared_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PumpPT { what: Selector::This, power: Value::Const(3), toughness: Value::ZERO, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..plain_creature("Sun-Collared Raptor", cost(&[generic(1), r()]), vec![CreatureType::Dinosaur], 1, 2)
    }
}

/// Stampeding Horncrest — {4}{R} 4/4 Dinosaur; haste while you control another Dinosaur.
pub fn stampeding_horncrest() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has haste as long as you control another Dinosaur.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Haste,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Dinosaur).and(R::OtherThanSource).and(R::ControlledByYou),
                )),
            },
        }],
        ..plain_creature("Stampeding Horncrest", cost(&[generic(4), r()]), vec![CreatureType::Dinosaur], 4, 4)
    }
}

/// Sun-Crowned Hunters — {4}{R}{R} 5/4 Dinosaur. Enrage: 3 damage to target
/// opponent or planeswalker.
pub fn sun_crowned_hunters() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer.or(R::Planeswalker)),
                amount: Value::Const(3),
            },
        }],
        ..plain_creature("Sun-Crowned Hunters", cost(&[generic(4), r(), r()]), vec![CreatureType::Dinosaur], 5, 4)
    }
}

/// War-Trained Slasher — {3}{R} 4/3 menace Wolverine Dinosaur. Attacking a
/// battle doubles its power until end of turn.
pub fn war_trained_slasher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::IsAttackingBattle }),
            effect: Effect::DoublePower { what: Selector::This, times: Value::ONE, duration: Duration::EndOfTurn },
        }],
        ..plain_creature(
            "War-Trained Slasher",
            cost(&[generic(3), r()]),
            vec![CreatureType::Wolverine, CreatureType::Dinosaur],
            4,
            3,
        )
    }
}

/// Tilonalli's Crown — {1}{R} Aura. ETB: 1 damage to the enchanted creature;
/// it gets +3/+0 and has trample.
pub fn tilonallis_crown() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus { power: 3, keywords: vec![Keyword::Trample], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::DealDamage { to: host(), amount: Value::ONE })],
        ..aura("Tilonalli's Crown", cost(&[generic(1), r()]))
    }
}

/// Dinosaur Stampede — {2}{R} instant. Attacking creatures get +2/+0; your
/// Dinosaurs gain trample until end of turn.
pub fn dinosaur_stampede() -> CardDefinition {
    plain_spell(
        "Dinosaur Stampede",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Dinosaur).and(R::ControlledByYou)),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Feed the Flames — {3}{R} instant. 5 damage to target creature; if it would
/// die this turn, exile it instead.
pub fn feed_the_flames() -> CardDefinition {
    plain_spell(
        "Feed the Flames",
        cost(&[generic(3), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: target_filtered(R::Creature) },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) },
        ]),
    )
}

/// Cinder Strike — {R} sorcery. You may blight 1 as you cast it: 2 damage to
/// target creature, 4 if the blight was paid.
pub fn cinder_strike() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(crate::card::AdditionalCastCost::Blight { n: 1 }),
        ..plain_spell(
            "Cinder Strike",
            cost(&[r()]),
            CardType::Sorcery,
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(4)),
                    else_: Box::new(Value::Const(2)),
                },
            },
        )
    }
}

// ── Nael, Avizoa Aeronaut's deck ────────────────────────────────────────────

fn domain() -> Value {
    Value::DomainCount(PlayerRef::You)
}

/// "Look at the top X cards, put up to one of them on top and the rest on the
/// bottom in a random order" with X = domain; the seat picks the one it keeps.
fn domain_dig() -> Effect {
    Effect::LookTopKeepOneRestToGraveyard { count: domain(), who: None, exile_rest: false, rest_bottom_random: true }
}

/// Nael, Avizoa Aeronaut — {2}{G}{U} legendary 2/4 flying Elf Scout. Its
/// combat damage to a player digs domain deep; with all five types, draw.
pub fn nael_avizoa_aeronaut() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                domain_dig(),
                Effect::If {
                    cond: Predicate::ValueAtLeast(domain(), Value::Const(5)),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..plain_creature("Nael, Avizoa Aeronaut", cost(&[generic(2), g(), u()]), vec![CreatureType::Elf, CreatureType::Scout], 2, 4)
    }
}

/// Voda Sea Scavenger — {2}{U} 3/2 Merfolk Rogue. ETB: a domain-deep dig.
pub fn voda_sea_scavenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(domain_dig())],
        ..plain_creature("Voda Sea Scavenger", cost(&[generic(2), u()]), vec![CreatureType::Merfolk, CreatureType::Rogue], 3, 2)
    }
}

/// Territorial Maro — {4}{G} */* Elemental: twice your domain.
pub fn territorial_maro() -> CardDefinition {
    let x = || Value::Times(Box::new(Value::Const(2)), Box::new(domain()));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Territorial Maro's power and toughness are each equal to twice the number of basic land types among lands you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: x(), toughness: x() },
        }],
        ..plain_creature("Territorial Maro", cost(&[generic(4), g()]), vec![CreatureType::Elemental], 0, 0)
    }
}

/// Might of Alara — {G} instant. Target creature gets +1/+1 per basic land
/// type among your lands until end of turn.
pub fn might_of_alara() -> CardDefinition {
    plain_spell(
        "Might of Alara",
        cost(&[g()]),
        CardType::Instant,
        Effect::PumpPT { what: target_filtered(R::Creature), power: domain(), toughness: domain(), duration: Duration::EndOfTurn },
    )
}

/// Manaforce Mace — {4} Equipment. Equipped creature gets +1/+1 per basic
/// land type among your lands. Equip {3}.
pub fn manaforce_mace() -> CardDefinition {
    CardDefinition {
        name: "Manaforce Mace",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        static_abilities: vec![StaticAbility {
            description: "Domain — Equipped creature gets +1/+1 for each basic land type among lands you control.",
            effect: StaticEffect::PumpPTByValue { applies_to: host(), power: domain(), toughness: domain() },
        }],
        ..Default::default()
    }
}

/// Herd Migration — {6}{G} sorcery: a 3/3 Beast per basic land type among
/// your lands. {1}{G}, discard it: a basic land to hand and 3 life.
pub fn herd_migration() -> CardDefinition {
    let beast = std::sync::Arc::new(crate::card::TokenDefinition {
        name: "Beast".into(),
        colors: vec![crate::mana::Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 3,
        ..Default::default()
    });
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to: crate::effect::ZoneDest::Hand(PlayerRef::You) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ]),
            ..Default::default()
        }],
        ..plain_spell(
            "Herd Migration",
            cost(&[generic(6), g()]),
            CardType::Sorcery,
            Effect::CreateToken { who: PlayerRef::You, count: domain(), definition: beast },
        )
    }
}

/// Convincing Mirage — {1}{U} Aura, enchant land. As it enters, choose a basic
/// land type; the enchanted land is that type.
pub fn convincing_mirage() -> CardDefinition {
    CardDefinition {
        name: "Convincing Mirage",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        as_enters_effect: Some(Effect::ChooseBasicLandTypeForSource),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land is the chosen type.",
            effect: StaticEffect::LandTypeChangerChosen { applies_to: host(), replace: true },
        }],
        ..Default::default()
    }
}

/// "{T}: Target land you control becomes the basic land type of your choice
/// until end of turn."
fn illusionist_ability() -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::LandsBecomeChosenBasicType {
            what: target_filtered(R::Land.and(R::ControlledByYou)),
            duration: Duration::EndOfTurn,
            from_chosen_basic: false,
        },
        ..Default::default()
    }
}

/// Grixis Illusionist — {U} 1/1 Human Wizard with the land-type ability.
pub fn grixis_illusionist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![illusionist_ability()],
        ..plain_creature("Grixis Illusionist", cost(&[u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 1)
    }
}

/// Pixie Illusionist — {U} 1/1 flying Faerie Wizard; kicker {3}{G} for two
/// +1/+1 counters; the land-type ability.
pub fn pixie_illusionist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[generic(3), g()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![illusionist_ability()],
        ..plain_creature("Pixie Illusionist", cost(&[u()]), vec![CreatureType::Faerie, CreatureType::Wizard], 1, 1)
    }
}

// ── Verazol, the Split Current's deck ───────────────────────────────────────

fn kicked_cast() -> EventSpec {
    EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellWasKicked)
}

fn kicked_etb(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::SpellWasKicked),
        effect,
    }
}

fn drake() -> std::sync::Arc<crate::card::TokenDefinition> {
    std::sync::Arc::new(crate::card::TokenDefinition {
        name: "Drake".into(),
        colors: vec![crate::mana::Color::Blue],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drake], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    })
}

/// Verazol, the Split Current — {X}{G}{U} legendary 0/0 Serpent. Enters with a
/// +1/+1 counter per mana spent; your kicked spell may cost it two counters to
/// copy that spell.
pub fn verazol_the_split_current() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::CastSpellManaSpent)),
        triggered_abilities: vec![TriggeredAbility {
            event: kicked_cast(),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                    Value::Const(2),
                ),
                then: Box::new(Effect::MayDo {
                    description: "Remove two +1/+1 counters from Verazol to copy that spell?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                        Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..plain_creature(
            "Verazol, the Split Current",
            cost(&[crate::mana::x(), g(), u()]),
            vec![CreatureType::Serpent],
            0,
            0,
        )
    }
}

/// Coralhelm Chronicler — {2}{U} 2/2 Merfolk Wizard. Your kicked spells loot;
/// ETB: a kicker card from the top five to hand, the rest to the bottom.
pub fn coralhelm_chronicler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: kicked_cast(),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
            },
            etb(Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(R::HasKickerAbility),
                optional: true,
                ..Default::default()
            }))),
        ],
        ..plain_creature("Coralhelm Chronicler", cost(&[generic(2), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 2)
    }
}

/// Elvish Hydromancer — {2}{G} 3/2 Elf Wizard, kicker {3}{U}: kicked, a token
/// copy of target creature you control.
pub fn elvish_hydromancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3), u()]))],
        triggered_abilities: vec![kicked_etb(Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::ONE,
            source: target_filtered(R::Creature.and(R::ControlledByYou)),
            extra_creature_types: vec![],
            extra_card_types: vec![],
            override_pt: None,
            override_colors: None,
            enters_tapped: false,
            non_legendary: false,
            legendary: false,
            extra_keywords: vec![],
            no_mana_cost: false,
            enters_with_counters: None,
            remove_keywords: vec![],
        })],
        ..plain_creature("Elvish Hydromancer", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Wizard], 3, 2)
    }
}

/// Lullmage's Familiar — {1}{G}{U} 2/4 Beast. {T}: add {G} or {U}; your
/// kicked spells gain you 2 life.
pub fn lullmages_familiar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::OfColors(vec![crate::mana::Color::Green, crate::mana::Color::Blue], Value::ONE),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: kicked_cast(),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..plain_creature("Lullmage's Familiar", cost(&[generic(1), g(), u()]), vec![CreatureType::Beast], 2, 4)
    }
}

/// Tazeem Roilmage — {1}{U} 2/1 Merfolk Wizard, kicker {4}: kicked, return
/// target instant or sorcery card from your graveyard to your hand.
pub fn tazeem_roilmage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(4)]))],
        triggered_abilities: vec![kicked_etb(Effect::Move {
            what: target_filtered(
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard),
            ),
            to: crate::effect::ZoneDest::Hand(PlayerRef::You),
        })],
        ..plain_creature("Tazeem Roilmage", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 1)
    }
}

/// Territorial Allosaurus — {2}{G}{G} 5/5 Dinosaur, kicker {2}{G}: kicked, it
/// fights another target creature.
pub fn territorial_allosaurus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2), g()]))],
        triggered_abilities: vec![kicked_etb(Effect::Fight {
            attacker: Selector::This,
            defender: target_filtered(R::Creature.and(R::OtherThanSource)),
        })],
        ..plain_creature("Territorial Allosaurus", cost(&[generic(2), g(), g()]), vec![CreatureType::Dinosaur], 5, 5)
    }
}

/// Bubble Snare — {U} Aura, kicker {2}{U}: kicked, it taps the enchanted
/// creature; that creature doesn't untap during its controller's untap step.
pub fn bubble_snare() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2), u()]))],
        triggered_abilities: vec![kicked_etb(Effect::Tap { what: host() })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: host() },
        }],
        ..aura("Bubble Snare", cost(&[u()]))
    }
}

/// Roost of Drakes — {U} enchantment, kicker {2}{U}: kicked, a 2/2 flying
/// Drake; each kicked spell you cast makes another.
pub fn roost_of_drakes() -> CardDefinition {
    let token = || Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: drake() };
    CardDefinition {
        name: "Roost of Drakes",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Kicker(cost(&[generic(2), u()]))],
        triggered_abilities: vec![kicked_etb(token()), TriggeredAbility { event: kicked_cast(), effect: token() }],
        ..Default::default()
    }
}

/// Wild Onslaught — {3}{G} instant, kicker {4}: a +1/+1 counter on each
/// creature you control, two if kicked.
pub fn wild_onslaught() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(4)]))],
        ..plain_spell(
            "Wild Onslaught",
            cost(&[generic(3), g()]),
            CardType::Instant,
            Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(2)),
                    else_: Box::new(Value::ONE),
                },
            },
        )
    }
}

/// Field Research — {2}{U} sorcery, kicker {2}{U}: draw two, three if kicked.
pub fn field_research() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2), u()]))],
        ..plain_spell(
            "Field Research",
            cost(&[generic(2), u()]),
            CardType::Sorcery,
            Effect::Draw {
                who: Selector::You,
                amount: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(3)),
                    else_: Box::new(Value::Const(2)),
                },
            },
        )
    }
}

// ── Chandra, Fire of Kaladesh's deck ────────────────────────────────────────

fn chandra(name: &'static str, mana: ManaCost, loyalty: u32, abilities: Vec<crate::card::LoyaltyAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Chandra], ..Default::default() },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

fn loyalty(cost: i32, effect: Effect) -> crate::card::LoyaltyAbility {
    crate::card::LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() }
}

fn player_or_walker() -> Selector {
    target_filtered(R::Player.or(R::Planeswalker))
}

/// "…and each creature that player or that planeswalker's controller controls."
fn creatures_of_target_slot0() -> Selector {
    Selector::ControlledBy { who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))), filter: R::Creature }
}

/// "Search your library and/or graveyard for a card named `name`, reveal it,
/// and put it into your hand."
fn fetch_named(name: &str) -> Effect {
    Effect::SearchZones {
        who: PlayerRef::You,
        zones: vec![crate::card::Zone::Library, crate::card::Zone::Graveyard],
        filter: R::HasName(name.into()),
        to: crate::effect::ZoneDest::Hand(PlayerRef::You),
    }
}

fn red_instant_or_sorcery() -> R {
    R::HasColor(crate::mana::Color::Red).and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
}

/// Chandra Nalaar — {3}{R}{R}, loyalty 6. +1: 1 to target player or
/// planeswalker. −X: X to target creature. −8: 10 to target player or
/// planeswalker and each creature that player or its controller controls.
pub fn chandra_nalaar() -> CardDefinition {
    chandra(
        "Chandra Nalaar",
        cost(&[generic(3), r(), r()]),
        6,
        vec![
            loyalty(1, Effect::DealDamage { to: player_or_walker(), amount: Value::ONE }),
            crate::card::LoyaltyAbility {
                x_cost: true,
                effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::XFromCost },
                ..Default::default()
            },
            loyalty(
                -8,
                Effect::Seq(vec![
                    Effect::DealDamage { to: player_or_walker(), amount: Value::Const(10) },
                    Effect::DealDamage { to: creatures_of_target_slot0(), amount: Value::Const(10) },
                ]),
            ),
        ],
    )
}

/// Chandra, Heart of Fire — {3}{R}{R}, loyalty 5. +1: discard your hand,
/// exile the top three, play them this turn. +1: 2 to any target. −9: exile
/// every red instant and sorcery from your graveyard and library, cast them
/// this turn, add {R}{R}{R}{R}{R}{R}.
pub fn chandra_heart_of_fire() -> CardDefinition {
    chandra(
        "Chandra, Heart of Fire",
        cost(&[generic(3), r(), r()]),
        5,
        vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::Const(3),
                        duration: crate::card::MayPlayDuration::EndOfThisTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    },
                ]),
            ),
            loyalty(1, Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(2) }),
            loyalty(
                -9,
                Effect::Seq(vec![
                    // "Any number": every match, the most a seat can take.
                    Effect::Move {
                        what: Selector::Both(
                            Box::new(Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: crate::card::Zone::Graveyard,
                                filter: red_instant_or_sorcery(),
                            }),
                            Box::new(Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: crate::card::Zone::Library,
                                filter: red_instant_or_sorcery(),
                            }),
                        ),
                        to: crate::effect::ZoneDest::Exile,
                    },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: crate::card::MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                    Effect::ShuffleLibrary { who: PlayerRef::You },
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: crate::effect::ManaPayload::OfColor(crate::mana::Color::Red, Value::Const(6)),
                    },
                ]),
            ),
        ],
    )
}

/// Chandra, Pyromaster — {2}{R}{R}, loyalty 4. +1: 1 to target player or
/// planeswalker and 1 to up to one target creature of theirs, which can't
/// block. 0: exile the top card, play it this turn. −7: exile the top ten,
/// copy an instant or sorcery among them three times and cast the copies free.
pub fn chandra_pyromaster() -> CardDefinition {
    let exiled_spell = || {
        Selector::best_of(Selector::MatchingAmong {
            inner: Box::new(Selector::CardExiledWithSource),
            filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
        })
    };
    chandra(
        "Chandra, Pyromaster",
        cost(&[generic(2), r(), r()]),
        4,
        vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::DealDamage { to: player_or_walker(), amount: Value::ONE },
                    Effect::DealDamage {
                        to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                        amount: Value::ONE,
                    },
                    Effect::GrantKeyword { what: Selector::Target(1), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
                ]),
            ),
            loyalty(
                0,
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            ),
            loyalty(
                -7,
                Effect::Seq(vec![
                    Effect::ExileTopOfLibrary { who: Selector::You, amount: Value::Const(10), link_to_source: true, face_down: false },
                    Effect::CopyCardAndCastFree { what: exiled_spell() },
                    Effect::CopyCardAndCastFree { what: exiled_spell() },
                    Effect::CopyCardAndCastFree { what: exiled_spell() },
                ]),
            ),
        ],
    )
}

/// Chandra, the Firebrand — {3}{R}, loyalty 3. +1: 1 to any target. −2: copy
/// your next instant or sorcery this turn. −6: 6 to each of up to six targets.
pub fn chandra_the_firebrand() -> CardDefinition {
    chandra(
        "Chandra, the Firebrand",
        cost(&[generic(3), r()]),
        3,
        vec![
            loyalty(1, Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE }),
            loyalty(
                -2,
                Effect::OnYourNextInstantSorceryThisTurn {
                    body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
                },
            ),
            loyalty(
                -6,
                Effect::ApplyToTargets {
                    max_targets: 6,
                    min_targets: 0,
                    filter: R::any_target(),
                    effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(6) }),
                },
            ),
        ],
    )
}

/// Keral Keep Disciples — {2}{R}{R} 4/3 Human Monk. Activating a Chandra's
/// loyalty ability deals 1 damage to each opponent.
pub fn keral_keep_disciples() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::AbilityActivatedWith(crate::effect::ActivationTrait::Loyalty),
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasPlaneswalkerType(crate::card::PlaneswalkerSubtype::Chandra),
            }),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        }],
        ..plain_creature("Keral Keep Disciples", cost(&[generic(2), r(), r()]), vec![CreatureType::Human, CreatureType::Monk], 4, 3)
    }
}

/// Chandra's Firemaw — {3}{R}{R} 4/2 haste Hellion. ETB: you may fetch
/// Chandra, Flame's Catalyst from your library and/or graveyard.
pub fn chandras_firemaw() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for Chandra, Flame's Catalyst?".into(),
            body: Box::new(fetch_named("Chandra, Flame's Catalyst")),
        })],
        ..plain_creature("Chandra's Firemaw", cost(&[generic(3), r(), r()]), vec![CreatureType::Hellion], 4, 2)
    }
}

/// Pyromancer's Gauntlet — {5} artifact. Your red instants, sorceries and
/// planeswalkers deal 2 more damage.
pub fn pyromancers_gauntlet() -> CardDefinition {
    CardDefinition {
        name: "Pyromancer's Gauntlet",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If a red instant or sorcery spell you control or a red planeswalker you control would deal damage to a permanent or player, it deals that much damage plus 2 to that permanent or player instead.",
            effect: StaticEffect::ColorSpellsAndPlaneswalkersYouControlDealExtraDamage { color: crate::mana::Color::Red, amount: 2 },
        }],
        ..Default::default()
    }
}

/// Chandra's Fury — {4}{R} instant. 4 to target player or planeswalker and 1
/// to each creature that player or its controller controls.
pub fn chandras_fury() -> CardDefinition {
    plain_spell(
        "Chandra's Fury",
        cost(&[generic(4), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: player_or_walker(), amount: Value::Const(4) },
            Effect::DealDamage { to: creatures_of_target_slot0(), amount: Value::ONE },
        ]),
    )
}

/// Chandra's Outburst — {3}{R}{R} sorcery. 4 to target player or planeswalker;
/// fetch Chandra, Bold Pyromancer from your library and/or graveyard.
pub fn chandras_outburst() -> CardDefinition {
    plain_spell(
        "Chandra's Outburst",
        cost(&[generic(3), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: player_or_walker(), amount: Value::Const(4) },
            fetch_named("Chandra, Bold Pyromancer"),
        ]),
    )
}

/// Chandra's Flame Wave — {3}{R}{R} sorcery. 2 to target player and each
/// creature they control; fetch Chandra, Flame's Fury.
pub fn chandras_flame_wave() -> CardDefinition {
    plain_spell(
        "Chandra's Flame Wave",
        cost(&[generic(3), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(2) },
            Effect::DealDamage { to: creatures_of_target_slot0(), amount: Value::Const(2) },
            fetch_named("Chandra, Flame's Fury"),
        ]),
    )
}

/// Chandra's Revolution — {3}{R} sorcery. 4 to target creature; tap target
/// land, which skips its controller's next untap.
pub fn chandras_revolution() -> CardDefinition {
    plain_spell(
        "Chandra's Revolution",
        cost(&[generic(3), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
            Effect::Tap { what: Selector::TargetFiltered { slot: 1, filter: R::Land } },
            Effect::SkipNextUntap { what: Selector::Target(1) },
        ]),
    )
}

// ── Brigid, Hero of Kinsbaile's deck ────────────────────────────────────────

fn kithkin() -> R {
    R::HasCreatureType(CreatureType::Kithkin)
}

fn white() -> R {
    R::HasColor(crate::mana::Color::White)
}

/// Rustic Clachan — land. Enters tapped unless you reveal a Kithkin card from
/// hand. {T}: add {W}. Reinforce 1—{1}{W}.
pub fn rustic_clachan() -> CardDefinition {
    CardDefinition {
        name: "Rustic Clachan",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::Reinforce(1, cost(&[generic(1), w()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::OfColor(crate::mana::Color::White, Value::ONE),
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "As this land enters, you may reveal a Kithkin card from your hand. If you don't, this land enters tapped.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Hand,
                    filter: kithkin(),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Ballynock Cohort — {2}{W} 2/2 first strike Kithkin Soldier; +1/+1 while you
/// control another white creature.
pub fn ballynock_cohort() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 as long as you control another white creature.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(white()).and(R::OtherThanSource).and(R::ControlledByYou),
                )),
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..plain_creature("Ballynock Cohort", cost(&[generic(2), w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 2, 2)
    }
}

/// Ballynock Trapper — {3}{W} 2/2 Kithkin Soldier. {T}: tap target creature;
/// your white spells may untap it.
pub fn ballynock_trapper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellMatches(white())),
            effect: Effect::MayDo {
                description: "Untap Ballynock Trapper?".into(),
                body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
            },
        }],
        ..plain_creature("Ballynock Trapper", cost(&[generic(3), w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 2, 2)
    }
}

/// Cenn's Heir — {1}{W} 1/1 Kithkin Soldier. Attacking, +1/+1 per other
/// attacking Kithkin until end of turn.
pub fn cenns_heir() -> CardDefinition {
    let n = || Value::CountOf(Box::new(Selector::EachPermanent(kithkin().and(R::IsAttacking).and(R::OtherThanSource))));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT { what: Selector::This, power: n(), toughness: n(), duration: Duration::EndOfTurn },
        }],
        ..plain_creature("Cenn's Heir", cost(&[generic(1), w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 1, 1)
    }
}

/// Goldmeadow Dodger — {W} 1/1 Kithkin Rogue; creatures with power 4 or
/// greater can't block it.
pub fn goldmeadow_dodger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::PowerAtLeast(4)))],
        ..plain_creature("Goldmeadow Dodger", cost(&[w()]), vec![CreatureType::Kithkin, CreatureType::Rogue], 1, 1)
    }
}

/// Goldmeadow Stalwart — {W} 2/2 Kithkin Soldier; reveal a Kithkin card from
/// hand or pay {3} as an additional cost.
pub fn goldmeadow_stalwart() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::RevealFromHandOrPay { filter: kithkin(), pay: 3 }],
        ..plain_creature("Goldmeadow Stalwart", cost(&[w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 2, 2)
    }
}

/// Kithkin Rabble — {3}{W} */* vigilance Kithkin: your white permanents.
pub fn kithkin_rabble() -> CardDefinition {
    let n = || Value::CountOf(Box::new(Selector::EachPermanent(white().and(R::ControlledByYou))));
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Kithkin Rabble's power and toughness are each equal to the number of white permanents you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: n(), toughness: n() },
        }],
        ..plain_creature("Kithkin Rabble", cost(&[generic(3), w()]), vec![CreatureType::Kithkin], 0, 0)
    }
}

/// Kithkin Zephyrnaut — {2}{W} 2/2 Kithkin Soldier. Kinship: a top card that
/// shares a creature type with it gives +2/+2, flying and vigilance this turn.
pub fn kithkin_zephyrnaut() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::MatchingAmong {
                    inner: Box::new(Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE }),
                    filter: kithkin().or(R::HasCreatureType(CreatureType::Soldier)),
                }),
                then: Box::new(Effect::Seq(vec![
                    Effect::PumpPT { what: Selector::This, power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..plain_creature("Kithkin Zephyrnaut", cost(&[generic(2), w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 2, 2)
    }
}

/// Order of the Golden Cricket — {1}{W} 2/2 Kithkin Knight. Attacking, you may
/// pay {W} for flying until end of turn.
pub fn order_of_the_golden_cricket() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {W} for flying?".into(),
                mana_cost: cost(&[w()]),
                body: Box::new(Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn }),
                else_: None,
            },
        }],
        ..plain_creature("Order of the Golden Cricket", cost(&[generic(1), w()]), vec![CreatureType::Kithkin, CreatureType::Knight], 2, 2)
    }
}

/// Thoughtweft Trio — {2}{W}{W} 5/5 first strike, vigilance Kithkin Soldier.
/// Champion a Kithkin; it can block any number of creatures.
pub fn thoughtweft_trio() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance, Keyword::CanBlockAnyNumber],
        triggered_abilities: vec![etb(Effect::Champion { filter: kithkin() })],
        ..plain_creature("Thoughtweft Trio", cost(&[generic(2), w(), w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 5, 5)
    }
}

/// Surge of Thoughtweft — {1}{W} Kindred instant — Kithkin. Your creatures get
/// +1/+1 until end of turn; with a Kithkin, draw a card.
pub fn surge_of_thoughtweft() -> CardDefinition {
    CardDefinition {
        name: "Surge of Thoughtweft",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kithkin], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(kithkin().and(R::ControlledByYou))),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Mavren Fein, Dusk Apostle's deck ────────────────────────────────────────

fn vampire() -> R {
    R::HasCreatureType(CreatureType::Vampire)
}

/// Bishop of Binding — {3}{W} 1/1 Vampire Cleric. ETB: exile target creature
/// an opponent controls until it leaves; attacking, target Vampire gets +X/+X
/// where X is the exiled card's power.
pub fn bishop_of_binding() -> CardDefinition {
    let x = || Value::PowerOf(Box::new(Selector::CardExiledWithSource));
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::ExileUntilSourceLeaves {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                return_to: crate::card::ExileReturnZone::Battlefield,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PumpPT { what: target_filtered(vampire()), power: x(), toughness: x(), duration: Duration::EndOfTurn },
            },
        ],
        ..plain_creature("Bishop of Binding", cost(&[generic(3), w()]), vec![CreatureType::Vampire, CreatureType::Cleric], 1, 1)
    }
}

/// Duskborne Skymarcher — {W} 1/1 flying Vampire Cleric. {W}, {T}: target
/// attacking Vampire gets +1/+1 until end of turn.
pub fn duskborne_skymarcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w()]),
            effect: Effect::PumpPT {
                what: target_filtered(vampire().and(R::IsAttacking)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..plain_creature("Duskborne Skymarcher", cost(&[w()]), vec![CreatureType::Vampire, CreatureType::Cleric], 1, 1)
    }
}

/// Exultant Skymarcher — {1}{W}{W} 2/3 flying Vampire Soldier.
pub fn exultant_skymarcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..plain_creature("Exultant Skymarcher", cost(&[generic(1), w(), w()]), vec![CreatureType::Vampire, CreatureType::Soldier], 2, 3)
    }
}

/// Skyblade of the Legion — {1}{W} 1/3 flying Vampire Soldier.
pub fn skyblade_of_the_legion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..plain_creature("Skyblade of the Legion", cost(&[generic(1), w()]), vec![CreatureType::Vampire, CreatureType::Soldier], 1, 3)
    }
}

/// Famished Paladin — {1}{W} 3/3 Vampire Knight. Doesn't untap in your untap
/// step; gaining life untaps it.
pub fn famished_paladin() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..plain_creature("Famished Paladin", cost(&[generic(1), w()]), vec![CreatureType::Vampire, CreatureType::Knight], 3, 3)
    }
}

/// Glorifier of Dusk — {3}{W}{W} 4/4 Vampire Soldier. Pay 2 life: flying until
/// end of turn. Pay 2 life: vigilance until end of turn.
pub fn glorifier_of_dusk() -> CardDefinition {
    let gain = |keyword| ActivatedAbility {
        life_cost: 2,
        effect: Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![gain(Keyword::Flying), gain(Keyword::Vigilance)],
        ..plain_creature("Glorifier of Dusk", cost(&[generic(3), w(), w()]), vec![CreatureType::Vampire, CreatureType::Soldier], 4, 4)
    }
}

/// Inspiring Cleric — {2}{W} 3/2 Vampire Cleric. ETB: gain 4 life.
pub fn inspiring_cleric() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(4) })],
        ..plain_creature("Inspiring Cleric", cost(&[generic(2), w()]), vec![CreatureType::Vampire, CreatureType::Cleric], 3, 2)
    }
}

/// Paladin of Atonement — {1}{W} 1/1 Vampire Knight. Each upkeep, if you lost
/// life last turn, a +1/+1 counter; dying, gain life equal to its toughness.
pub fn paladin_of_atonement() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::AnyPlayer)
                    .with_filter(Predicate::PlayerLostLifeLastTurn { who: PlayerRef::You }),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ToughnessOf(Box::new(Selector::This)) },
            },
        ],
        ..plain_creature("Paladin of Atonement", cost(&[generic(1), w()]), vec![CreatureType::Vampire, CreatureType::Knight], 1, 1)
    }
}

/// Sanguine Glorifier — {3}{W} 3/3 Vampire Cleric. ETB: a +1/+1 counter on
/// another target Vampire you control.
pub fn sanguine_glorifier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(vampire().and(R::ControlledByYou).and(R::OtherThanSource)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..plain_creature("Sanguine Glorifier", cost(&[generic(3), w()]), vec![CreatureType::Vampire, CreatureType::Cleric], 3, 3)
    }
}

/// Queen's Commission — {2}{W} sorcery: two 1/1 white lifelink Vampires.
pub fn queens_commission() -> CardDefinition {
    let token = std::sync::Arc::new(crate::card::TokenDefinition {
        name: "Vampire".into(),
        colors: vec![crate::mana::Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    });
    plain_spell(
        "Queen's Commission",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: token },
    )
}

/// Vampire's Zeal — {W} instant. Target creature gets +2/+2; a Vampire also
/// gains first strike, until end of turn.
pub fn vampires_zeal() -> CardDefinition {
    plain_spell(
        "Vampire's Zeal",
        cost(&[w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: vampire() },
                then: Box::new(Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

// ── Johan's deck ────────────────────────────────────────────────────────────

/// "You may exert this creature as it attacks. When you do, [bonus]."
fn exert_body(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32, bonus: Effect) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Exert],
        triggered_abilities: vec![TriggeredAbility { event: EventSpec::new(EventKind::Exerted, EventScope::SelfSource), effect: bonus }],
        ..plain_creature(name, mana, types, p, t)
    }
}

fn this_until_eot(power: i32, toughness: i32, keywords: Vec<Keyword>) -> Effect {
    let mut seq = vec![Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }];
    seq.extend(keywords.into_iter().map(|keyword| Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn }));
    Effect::Seq(seq)
}

/// "Whenever you exert a creature, [effect]" on an exert creature.
fn exert_payoff(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32, effect: Effect) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Exert],
        triggered_abilities: vec![TriggeredAbility { event: EventSpec::new(EventKind::Exerted, EventScope::YourControl), effect }],
        ..plain_creature(name, mana, types, p, t)
    }
}

/// Bitterblade Warrior — {1}{G} 2/2 Jackal Warrior; exerted, +1/+0 and deathtouch.
pub fn bitterblade_warrior() -> CardDefinition {
    exert_body("Bitterblade Warrior", cost(&[generic(1), g()]), vec![CreatureType::Jackal, CreatureType::Warrior], 2, 2, this_until_eot(1, 0, vec![Keyword::Deathtouch]))
}

/// Champion of Rhonas — {3}{G} 3/3 Jackal Warrior; exerted, you may put a
/// creature card from your hand onto the battlefield.
pub fn champion_of_rhonas() -> CardDefinition {
    exert_body(
        "Champion of Rhonas",
        cost(&[generic(3), g()]),
        vec![CreatureType::Jackal, CreatureType::Warrior],
        3,
        3,
        Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Creature,
            count: Value::ONE,
            tapped: false,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
            then: None,
        },
    )
}

/// Devoted Crop-Mate — {2}{W} 3/2 Human Warrior; exerted, return target
/// creature card with mana value 2 or less from your graveyard to the battlefield.
pub fn devoted_crop_mate() -> CardDefinition {
    exert_body(
        "Devoted Crop-Mate",
        cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        3,
        2,
        Effect::Move {
            what: target_filtered(R::Creature.and(R::ManaValueAtMost(2)).and(R::InYourGraveyard)),
            to: crate::effect::ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Emberhorn Minotaur — {3}{R} 4/3 Minotaur Warrior; exerted, +1/+1 and menace.
pub fn emberhorn_minotaur() -> CardDefinition {
    exert_body("Emberhorn Minotaur", cost(&[generic(3), r()]), vec![CreatureType::Minotaur, CreatureType::Warrior], 4, 3, this_until_eot(1, 1, vec![Keyword::Menace]))
}

/// Gust Walker — {1}{W} 2/2 Human Wizard; exerted, +1/+1 and flying.
pub fn gust_walker() -> CardDefinition {
    exert_body("Gust Walker", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2, this_until_eot(1, 1, vec![Keyword::Flying]))
}

/// Oketra's Avenger — {1}{W} 3/1 Human Warrior; exerted, prevent all combat
/// damage that would be dealt to it this turn.
pub fn oketras_avenger() -> CardDefinition {
    exert_body(
        "Oketra's Avenger",
        cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        3,
        1,
        Effect::PreventCombatDamageToTargetThisTurn { target: Selector::This },
    )
}

/// Rhonas's Stalwart — {1}{G} 2/2 Human Warrior; exerted, +1/+1 and creatures
/// with power 2 or less can't block it this turn.
pub fn rhonass_stalwart() -> CardDefinition {
    exert_body(
        "Rhonas's Stalwart",
        cost(&[generic(1), g()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        2,
        2,
        this_until_eot(1, 1, vec![Keyword::CantBeBlockedBy(Box::new(R::PowerAtMost(2)))]),
    )
}

/// Watchful Naga — {2}{G} 2/2 Snake Wizard; exerted, draw a card.
pub fn watchful_naga() -> CardDefinition {
    exert_body(
        "Watchful Naga",
        cost(&[generic(2), g()]),
        vec![CreatureType::Snake, CreatureType::Wizard],
        2,
        2,
        Effect::Draw { who: Selector::You, amount: Value::ONE },
    )
}

/// Vizier of the True — {3}{W} 3/2 Human Cleric with exert; whenever you exert
/// a creature, tap target creature an opponent controls.
pub fn vizier_of_the_true() -> CardDefinition {
    exert_payoff(
        "Vizier of the True",
        cost(&[generic(3), w()]),
        vec![CreatureType::Human, CreatureType::Cleric],
        3,
        2,
        Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
    )
}

/// Resolute Survivors — {1}{R}{W} 3/3 Human Warrior with exert; whenever you
/// exert a creature, 1 damage to each opponent and you gain 1 life.
pub fn resolute_survivors() -> CardDefinition {
    exert_payoff(
        "Resolute Survivors",
        cost(&[generic(1), r(), w()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        3,
        3,
        Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Rohirrim Chargers — {2}{R}{W} 4/4 Human Knight with exert; whenever you
/// exert a creature, reveal until an Equipment and put it onto the
/// battlefield attached.
pub fn rohirrim_chargers() -> CardDefinition {
    exert_payoff(
        "Rohirrim Chargers",
        cost(&[generic(2), r(), w()]),
        vec![CreatureType::Human, CreatureType::Knight],
        4,
        4,
        Effect::RevealUntilPutAttachedElseHand { filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment) },
    )
}

/// Knotvine Paladin — {G}{W} 2/2 Human Knight. Attacking, +1/+1 per untapped
/// creature you control.
pub fn knotvine_paladin() -> CardDefinition {
    let n = || Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::Untapped).and(R::ControlledByYou))));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT { what: Selector::This, power: n(), toughness: n(), duration: Duration::EndOfTurn },
        }],
        ..plain_creature("Knotvine Paladin", cost(&[g(), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

// ── Liu Bei, Lord of Shu's deck ─────────────────────────────────────────────

fn shu(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    plain_creature(name, mana, types, p, t)
}

/// "Cast this spell only during the declare attackers step and only if you've
/// been attacked this step."
fn when_attacked() -> Predicate {
    Predicate::All(vec![
        Predicate::CurrentStepIs(crate::game::types::TurnStep::DeclareAttackers),
        Predicate::SelectorExists(Selector::EachPermanent(R::IsAttackingYouOrYourPlaneswalker)),
    ])
}

fn ambush_spell(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { cast_condition: Some(when_attacked()), ..plain_spell(name, mana, CardType::Instant, effect) }
}

/// Alert Shu Infantry — {2}{W} 2/2 vigilance Human Soldier.
pub fn alert_shu_infantry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..shu("Alert Shu Infantry", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Flanking Troops — {2}{W}{W} 2/2 Human Soldier. Attacking, you may tap
/// target creature.
pub fn flanking_troops() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Tap target creature?".into(),
                body: Box::new(Effect::Tap { what: target_filtered(R::Creature) }),
            },
        }],
        ..shu("Flanking Troops", cost(&[generic(2), w(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Kongming's Contraptions — {3}{W} 2/4 Human Soldier. {T}: 2 damage to target
/// attacking creature, only in the declare attackers step you were attacked in.
pub fn kongmings_contraptions() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(when_attacked()),
            effect: Effect::DealDamage { to: target_filtered(R::Creature.and(R::IsAttacking)), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..shu("Kongming's Contraptions", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 4)
    }
}

/// Shu Defender — {2}{W} 2/2 Human Soldier; blocking, +0/+2 until end of turn.
pub fn shu_defender() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::PumpPT { what: Selector::This, power: Value::ZERO, toughness: Value::Const(2), duration: Duration::EndOfTurn },
        }],
        ..shu("Shu Defender", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Shu Elite Infantry — {3}{W} 3/3 Human Soldier.
pub fn shu_elite_infantry() -> CardDefinition {
    shu("Shu Elite Infantry", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 3)
}

/// Shu Farmer — {1}{W} 1/1 Human. {T}: gain 1 life, only during your turn
/// before attackers are declared.
pub fn shu_farmer() -> CardDefinition {
    use crate::game::types::TurnStep as S;
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Any(
                    [S::Upkeep, S::Draw, S::PreCombatMain, S::BeginCombat]
                        .into_iter()
                        .map(Predicate::CurrentStepIs)
                        .collect(),
                ),
            ])),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..shu("Shu Farmer", cost(&[generic(1), w()]), vec![CreatureType::Human], 1, 1)
    }
}

/// Shu Soldier-Farmers — {4}{W} 2/4 Human Soldier. ETB: gain 4 life.
pub fn shu_soldier_farmers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(4) })],
        ..shu("Shu Soldier-Farmers", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 4)
    }
}

/// Eightfold Maze — {2}{W} instant, only in a declare attackers step you were
/// attacked in: destroy target attacking creature.
pub fn eightfold_maze() -> CardDefinition {
    ambush_spell(
        "Eightfold Maze",
        cost(&[generic(2), w()]),
        Effect::Destroy { what: target_filtered(R::Creature.and(R::IsAttacking)) },
    )
}

/// Rally the Troops — {W} instant, only in a declare attackers step you were
/// attacked in: untap all creatures you control.
pub fn rally_the_troops() -> CardDefinition {
    ambush_spell(
        "Rally the Troops",
        cost(&[w()]),
        Effect::Untap { what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)), up_to: None },
    )
}

/// Warrior's Stand — {1}{W} instant, only in a declare attackers step you were
/// attacked in: your creatures get +2/+2 until end of turn.
pub fn warriors_stand() -> CardDefinition {
    ambush_spell(
        "Warrior's Stand",
        cost(&[generic(1), w()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Empty City Ruse — {W} sorcery. Target opponent skips their next combat
/// phase (a turn with extra combats keeps the later ones).
pub fn empty_city_ruse() -> CardDefinition {
    plain_spell(
        "Empty City Ruse",
        cost(&[w()]),
        CardType::Sorcery,
        Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::SkipNextCombatPhase { who: PlayerRef::Target(0) }),
        },
    )
}

/// Misfortune's Gain — {3}{W} sorcery. Destroy target creature; its owner
/// gains 4 life.
pub fn misfortunes_gain() -> CardDefinition {
    plain_spell(
        "Misfortune's Gain",
        cost(&[generic(3), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::GainLife { who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))), amount: Value::Const(4) },
        ]),
    )
}

/// Peach Garden Oath — {W} sorcery. Gain 2 life per creature you control.
pub fn peach_garden_oath() -> CardDefinition {
    plain_spell(
        "Peach Garden Oath",
        cost(&[w()]),
        CardType::Sorcery,
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))))),
            ),
        },
    )
}

/// Virtuous Charge — {2}{W} sorcery. Your creatures get +1/+1 until end of turn.
pub fn virtuous_charge() -> CardDefinition {
    plain_spell(
        "Virtuous Charge",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
    )
}
