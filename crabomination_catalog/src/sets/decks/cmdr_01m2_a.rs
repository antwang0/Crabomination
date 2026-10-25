//! Commander: 2026-10-24 (`01M2Ffkh`) — the Isu the Abominable, Scarlet Witch,
//! Stegron, Nael and Verazol decks' closers. Tests in `tests/recent_b/cmdr_01m2_a.rs`.

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
