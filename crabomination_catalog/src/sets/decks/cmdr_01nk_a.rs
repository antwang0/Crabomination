//! Commander: the 2026-10-20 (`01NkAE5B`) one-gap pair closers. Tests in
//! `tests/recent_b/cmdr_01nk_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{blocks, discard, draw, emerge, etb, on_attack, on_cast, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, Value, ZoneDest, ZoneRef};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};
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

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// Blossombind — {1}{U} Aura. When it enters, tap enchanted creature.
/// Enchanted creature can't become untapped and can't have counters put on it.
pub fn blossombind() -> CardDefinition {
    CardDefinition {
        name: "Blossombind",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Tap { what: enchanted() })],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature can't become untapped.",
                effect: StaticEffect::CantBecomeUntapped { applies_to: enchanted() },
            },
            StaticAbility {
                description: "Enchanted creature can't have counters put on it.",
                effect: StaticEffect::CantHaveCountersPutOn { applies_to: enchanted() },
            },
        ],
        ..Default::default()
    }
}

/// Purity — {3}{W}{W}{W} 6/6 flying Elemental Incarnation. Noncombat damage
/// to you is prevented and you gain that much life; put into a graveyard from
/// anywhere, it shuffles into its owner's library.
pub fn purity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If noncombat damage would be dealt to you, prevent that damage. You gain life equal to the damage prevented this way.",
            effect: StaticEffect::PreventNoncombatDamageToYouGainLife,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::ShuffleSelfIntoLibrary,
        }],
        ..creature(
            "Purity",
            cost(&[generic(3), w(), w(), w()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            6,
            6,
        )
    }
}

/// Creeping Trailblazer — {R}{G} 2/2 Elemental. Other Elementals you control
/// get +1/+0. {2}{R}{G}: +1/+1 until end of turn for each Elemental you control.
pub fn creeping_trailblazer() -> CardDefinition {
    let elementals = || {
        Value::PermanentCountControlledByMatching(PlayerRef::You, R::HasCreatureType(CreatureType::Elemental))
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Elementals you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Elemental).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: elementals(),
                toughness: elementals(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Creeping Trailblazer", cost(&[r(), g()]), vec![CreatureType::Elemental], 2, 2)
    }
}

/// Mantle of Leadership — {1}{W} Aura with flash. Whenever a creature enters,
/// enchanted creature gets +2/+2 until end of turn.
pub fn mantle_of_leadership() -> CardDefinition {
    CardDefinition {
        name: "Mantle of Leadership",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::PumpPT {
                what: enchanted(),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Timeline Inquiry — {3}{U} instant, teamwork 2. Draw three cards, then
/// discard a card unless it was cast using teamwork.
pub fn timeline_inquiry() -> CardDefinition {
    CardDefinition {
        name: "Timeline Inquiry",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower { filter: R::Creature.and(R::ControlledByYou), power: 2 }),
        effect: Effect::Seq(vec![
            draw(3),
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Noop),
                else_: Box::new(discard(Selector::You, 1, false)),
            },
        ]),
        ..Default::default()
    }
}

/// Stormfront Riders — {4}{W} 4/3 flying Human Soldier. When it enters,
/// return two creatures you control to their owner's hand. Whenever a
/// creature is returned to your hand from the battlefield, create a 1/1
/// white Soldier — its own return included (CR 603.10a).
pub fn stormfront_riders() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    let bounce = || Effect::ReturnOneYouControl { filter: R::Creature, keep_best: false };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![bounce(), bounce()])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentReturnedToHand, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(soldier) },
            },
        ],
        ..creature(
            "Stormfront Riders",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            3,
        )
    }
}

/// Impelled Giant — {4}{R}{R} 3/3 trample Giant Warrior. Tap an untapped red
/// creature you control other than this creature: it gets +X/+0 until end of
/// turn, where X is the tapped creature's power.
pub fn impelled_giant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(
                R::Creature.and(R::HasColor(Color::Red)).and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::TappedForCostPower,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Impelled Giant",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Doubling Chant — {5}{G} sorcery. For each creature you control, you may
/// search your library for a creature card with the same name, onto the
/// battlefield, then shuffle.
pub fn doubling_chant() -> CardDefinition {
    CardDefinition {
        name: "Doubling Chant",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            body: Box::new(Effect::SearchSameNameToBattlefield { who: PlayerRef::You, what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Dichotomancy — {7}{U}{U} sorcery, suspend 3—{1}{U}{U}. For each tapped
/// nonland permanent target opponent controls, search their library for a
/// card with the same name and put it onto the battlefield under your
/// control; they shuffle.
pub fn dichotomancy() -> CardDefinition {
    CardDefinition {
        name: "Dichotomancy",
        cost: cost(&[generic(7), u(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Suspend(3, cost(&[generic(1), u(), u()]))],
        effect: Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::ForEach {
                selector: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Tapped.and(R::Nonland) },
                body: Box::new(Effect::SearchOpponentLibraryForSameName { what: Selector::TriggerSource }),
            }),
        },
        ..Default::default()
    }
}

/// Sphere of Annihilation — {X}{B} artifact; enters with X void counters. At
/// the beginning of your upkeep, exile it, every creature and planeswalker
/// with mana value at most its void counters, and every such card in a
/// graveyard.
pub fn sphere_of_annihilation() -> CardDefinition {
    // Per object, against the live counter count (the Sphere goes last).
    let exile_small = |zone: Selector| Effect::ForEach {
        selector: zone,
        body: Box::new(Effect::If {
            cond: Predicate::ValueAtMost(
                Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Void },
            ),
            then: Box::new(Effect::Exile { what: Selector::TriggerSource }),
            else_: Box::new(Effect::Noop),
        }),
    };
    let doomed = || R::Creature.or(R::Planeswalker);
    CardDefinition {
        name: "Sphere of Annihilation",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Void, Value::XFromCost)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                exile_small(Selector::EachPermanent(doomed())),
                exile_small(Selector::EachMatching { zone: ZoneRef::Graveyard(PlayerRef::EachPlayer), filter: doomed() }),
                Effect::Exile { what: Selector::This },
            ]),
        }],
        ..Default::default()
    }
}

/// Tarnation Vista — land. Enters tapped; as it enters, choose a color. {T}:
/// add one mana of the chosen color. {1}, {T}: for each color among
/// monocolored permanents you control, add one mana of that color.
pub fn tarnation_vista() -> CardDefinition {
    CardDefinition {
        name: "Tarnation Vista",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OneOfEachColorAmongYourMonocoloredPermanents,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Acquisitions Expert — {1}{B} 1/2 Human Rogue. When it enters, target
/// opponent reveals cards from their hand equal to your party's size; you
/// choose one of them and they discard it.
pub fn acquisitions_expert() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DiscardChosenFromRevealed {
            from: target_filtered(R::OpponentPlayer),
            reveal: Value::PartyCount,
            pick: Value::ONE,
        })],
        ..creature("Acquisitions Expert", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 2)
    }
}

/// Distended Mindbender — {8} 5/5 Eldrazi Insect, emerge {5}{B}{B}. When you
/// cast it, target opponent reveals their hand; you choose a nonland card
/// with mana value 3 or less and a card with mana value 4 or greater from
/// it, and they discard those cards.
pub fn distended_mindbender() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(emerge(cost(&[generic(5), b(), b()]))),
        triggered_abilities: vec![on_cast(Effect::Seq(vec![
            Effect::DiscardChosen {
                from: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Nonland.and(R::ManaValueAtMost(3)),
            },
            Effect::DiscardChosen { from: Selector::Target(0), count: Value::ONE, filter: R::ManaValueAtLeast(4) },
        ]))],
        ..creature("Distended Mindbender", cost(&[generic(8)]), vec![CreatureType::Eldrazi, CreatureType::Insect], 5, 5)
    }
}

/// Soul Separator — {3} artifact. {5}, {T}, sacrifice it: exile target
/// creature card from your graveyard; create a token copy of it that's a 1/1
/// flying Spirit in addition to its other types, and a black Zombie with the
/// card's power and toughness.
pub fn soul_separator() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        dynamic_pt: Some((
            Value::PowerOf(Box::new(Selector::Target(0))),
            Value::ToughnessOf(Box::new(Selector::Target(0))),
        )),
        ..Default::default()
    };
    CardDefinition {
        name: "Soul Separator",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(5)]),
            effect: Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Creature.and(R::InYourGraveyard)) },
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::Target(0),
                    extra_creature_types: vec![CreatureType::Spirit],
                    extra_card_types: vec![],
                    override_pt: Some((1, 1)),
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![Keyword::Flying],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(zombie) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ajani's Last Stand — {2}{W}{W} enchantment. Whenever a creature or
/// planeswalker you control dies, you may sacrifice it; if you do, create a
/// 4/4 white flying Avatar. When an opponent's spell or ability makes you
/// discard it, if you control a Plains, create that Avatar.
pub fn ajanis_last_stand() -> CardDefinition {
    let avatar = || {
        Arc::new(TokenDefinition {
            name: "Avatar".into(),
            power: 4,
            toughness: 4,
            card_types: vec![CardType::Creature],
            colors: vec![Color::White],
            subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
            keywords: vec![Keyword::Flying],
            ..Default::default()
        })
    };
    let mint = move || Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: avatar() };
    CardDefinition {
        name: "Ajani's Last Stand",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.or(R::Planeswalker) },
                ),
                effect: Effect::MaySacrificeSource {
                    description: "Sacrifice Ajani's Last Stand for a 4/4 Avatar?".into(),
                    then: Box::new(mint()),
                    else_: None,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::OpponentCausedYouToDiscard, EventScope::SelfSource).with_filter(
                    Predicate::SelectorExists(Selector::EachPermanent(
                        R::HasLandType(LandType::Plains).and(R::ControlledByYou),
                    )),
                ),
                effect: mint(),
            },
        ],
        ..Default::default()
    }
}

/// Veteran's Armaments — {2} Kindred Artifact — Soldier Equipment. Equipped
/// creature has "whenever this creature attacks or blocks, it gets +1/+1
/// until end of turn for each attacking creature". Whenever a Soldier
/// creature enters, you may attach it to it. Equip {2}.
pub fn veterans_armaments() -> CardDefinition {
    let attackers = || Value::CountOf(Box::new(Selector::EachPermanent(R::IsAttacking)));
    let pump = move || Effect::PumpPT { what: Selector::This, power: attackers(), toughness: attackers(), duration: Duration::EndOfTurn };
    CardDefinition {
        name: "Veteran's Armaments",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Kindred, CardType::Artifact],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Soldier)),
                },
            ),
            effect: Effect::MayDo {
                description: "Attach Veteran's Armaments to it?".into(),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
            },
        }],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_attack(pump()), blocks(pump())],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Jace, Cunning Castaway — {1}{U}{U} legendary planeswalker, loyalty 3.
/// +1: until end of turn, whenever one or more creatures you control deal
/// combat damage to a player, draw a card, then discard a card. −2: create a
/// 2/2 blue Illusion with "when this becomes the target of a spell, sacrifice
/// it". −5: create two tokens that are copies of Jace, except not legendary.
pub fn jace_cunning_castaway() -> CardDefinition {
    let illusion = TokenDefinition {
        name: "Illusion".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Illusion], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource).caused_by(R::IsSpellOnStack),
            effect: Effect::SacrificeSource,
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Jace, Cunning Castaway",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::MatchingCreaturesDealingCombatDamageBatchThisTurn {
                    filter: R::Creature,
                    body: Box::new(Effect::Seq(vec![draw(1), discard(Selector::You, 1, false)])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(illusion) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: true,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Experimental Frenzy — {3}{R} enchantment. You may look at the top card of
/// your library any time, and play lands and cast spells from it; you can't
/// play lands or cast spells from your hand. {3}{R}: destroy this enchantment.
pub fn experimental_frenzy() -> CardDefinition {
    CardDefinition {
        name: "Experimental Frenzy",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may play lands and cast spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Any },
            },
            StaticAbility {
                description: "You can't play lands or cast spells from your hand.",
                effect: StaticEffect::ControllerCantPlayFromHand,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            effect: Effect::Destroy { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nephalia Academy — land. An opponent's spell or ability making you discard
/// a card puts it on top of your library instead (revealed). {T}: add {C}.
pub fn nephalia_academy() -> CardDefinition {
    CardDefinition {
        name: "Nephalia Academy",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "If a spell or ability an opponent controls causes you to discard a card, you may reveal that card and put it on top of your library instead of putting it anywhere else.",
            effect: StaticEffect::OpponentDiscardToLibraryTop,
        }],
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        ..Default::default()
    }
}

fn instant_or_sorcery_cast() -> EventSpec {
    EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
    })
}

/// Smoldering Egg // Ashmouth Dragon — {1}{R} 0/4 defender Dragon Egg.
/// Whenever you cast an instant or sorcery spell, put ember counters on it
/// equal to the mana spent to cast that spell; then at seven or more, remove
/// them and transform it. Back: a 4/4 flying Dragon — whenever you cast an
/// instant or sorcery spell, it deals 2 damage to any target.
pub fn smoldering_egg() -> CardDefinition {
    let dragon = CardDefinition {
        color_indicator: vec![Color::Red],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: instant_or_sorcery_cast(),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(2) },
        }],
        ..creature("Ashmouth Dragon", ManaCost::default(), vec![CreatureType::Dragon], 4, 4)
    };
    let embers = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Ember };
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: instant_or_sorcery_cast(),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Ember, amount: Value::CastSpellManaSpent },
                Effect::If {
                    cond: Predicate::ValueAtLeast(embers(), Value::Const(7)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Ember, amount: embers() },
                        Effect::Transform { what: Selector::This },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(dragon)),
        ..creature("Smoldering Egg", cost(&[generic(1), r()]), vec![CreatureType::Dragon, CreatureType::Egg], 0, 4)
    }
}

/// Visions of Dread — {2}{B} sorcery. Target opponent puts a creature card of
/// their choice from their graveyard onto the battlefield under your control.
/// Flashback {8}{B}{B}, {X} less where X is the greatest mana value of a
/// commander you own on the battlefield or in the command zone.
pub fn visions_of_dread() -> CardDefinition {
    CardDefinition {
        name: "Visions of Dread",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(8), b(), b()]))],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast this way, where X is the greatest mana value of a commander you own on the battlefield or in the command zone.",
            effect: StaticEffect::SelfFlashbackCostsLess {
                amount: Value::GreatestCommanderManaValueInPlayOrCommandZone(PlayerRef::You),
            },
        }],
        effect: Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::PlayerChoosesFromGraveyard {
                who: PlayerRef::Target(0),
                filter: R::Creature,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        },
        ..Default::default()
    }
}

/// Uncage the Menagerie — {X}{G}{G} sorcery. Search your library for up to X
/// creature cards with different names that each have mana value X, reveal
/// them, put them into your hand, then shuffle.
pub fn uncage_the_menagerie() -> CardDefinition {
    CardDefinition {
        name: "Uncage the Menagerie",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Creature.and(R::ManaValueExactlyXFromCost).and(R::NameDiffersFromLastMoved),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::XFromCost,
            },
        ]),
        ..Default::default()
    }
}

/// Storm of Memories — {2}{R}{R}{R} sorcery, storm. Exile an instant or
/// sorcery card with mana value 3 or less from your graveyard at random; you
/// may cast it without paying its mana cost, and it's exiled instead of going
/// to a graveyard.
pub fn storm_of_memories() -> CardDefinition {
    CardDefinition {
        name: "Storm of Memories",
        cost: cost(&[generic(2), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Storm],
        effect: Effect::Seq(vec![
            Effect::ExileRandomFromGraveyardWithSource {
                who: PlayerRef::You,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::ManaValueAtMost(3)),
            },
            // The free cast asks its own "may".
            Effect::CastWithoutPayingImmediate {
                what: Selector::CardExiledWithSource,
                source_zone: Zone::Exile,
                exile_after: true,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: false,
            },
        ]),
        ..Default::default()
    }
}

/// Cast Through Time — {4}{U}{U}{U} enchantment. Instant and sorcery spells
/// you control have rebound.
pub fn cast_through_time() -> CardDefinition {
    CardDefinition {
        name: "Cast Through Time",
        cost: cost(&[generic(4), u(), u(), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you control have rebound.",
            effect: StaticEffect::YourInstantsAndSorceriesHaveRebound,
        }],
        ..Default::default()
    }
}

/// Helm of Obedience — {4} artifact. {X}, {T}: target opponent mills a card,
/// then repeats until a creature card or X cards have been put into their
/// graveyard this way. If a creature card was, sacrifice this artifact and
/// put it onto the battlefield under your control. X can't be 0.
pub fn helm_of_obedience() -> CardDefinition {
    CardDefinition {
        name: "Helm of Obedience",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[x()]),
            x_nonzero: true,
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::MillUntilMatching {
                        who: Selector::Player(PlayerRef::Target(0)),
                        filter: R::Creature,
                        max: Value::XFromCost,
                    },
                    // Milling stops at the first creature card, so at most one.
                    Effect::ForEach {
                        selector: Selector::LastMoved,
                        body: Box::new(Effect::If {
                            cond: Predicate::EntityMatches {
                                what: Selector::TriggerSource,
                                filter: R::Creature.and(R::InGraveyard),
                            },
                            then: Box::new(Effect::Seq(vec![
                                Effect::SacrificeSource,
                                Effect::Move {
                                    what: Selector::TriggerSource,
                                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                                },
                            ])),
                            else_: Box::new(Effect::Noop),
                        }),
                    },
                ])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Territory Forge — {4}{R} artifact. When it enters, if you cast it, exile
/// target artifact or land. It has all activated abilities of the exiled card.
pub fn territory_forge() -> CardDefinition {
    CardDefinition {
        name: "Territory Forge",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::SourceWasCast),
            effect: Effect::ExileWithSource { what: target_filtered(R::Artifact.or(R::Land)) },
        }],
        static_abilities: vec![StaticAbility {
            description: "This artifact has all activated abilities of the exiled card.",
            effect: StaticEffect::HasActivatedAbilitiesOfExiledWithSelf,
        }],
        ..Default::default()
    }
}

/// Animate Artifact — {3}{U} Aura, enchant artifact. As long as enchanted
/// artifact isn't a creature, it's an artifact creature with power and
/// toughness each equal to its mana value.
pub fn animate_artifact() -> CardDefinition {
    CardDefinition {
        name: "Animate Artifact",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Artifact) },
        equipped_bonus: Some(EquipBonus { animate_noncreature_mv: true, ..Default::default() }),
        ..Default::default()
    }
}

/// Throne of Makindi — land. {T}: add {C}. {1}, {T}: put a charge counter on
/// it. {T}, remove a charge counter: add two mana of any one color, spendable
/// only on kicked spells.
pub fn throne_of_makindi() -> CardDefinition {
    CardDefinition {
        name: "Throne of Makindi",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 1)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::Const(2))),
                        crate::mana::SpendRestriction::KickedSpellsOnly,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cheering Fanatic — {1}{R} 2/2 Goblin. Whenever it attacks, choose a card
/// name; spells with the chosen name cost {1} less to cast this turn. (The
/// name is chosen among the nonland cards in your hand — see
/// `Effect::NameSpellsCostLessThisTurn`.)
pub fn cheering_fanatic() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::NameSpellsCostLessThisTurn { amount: 1 })],
        ..creature("Cheering Fanatic", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 2, 2)
    }
}

/// Evolving Door — {2}{G} artifact. {1}, {T}, sacrifice a creature: search
/// your library for a creature card exactly one more color than the
/// sacrificed creature, exile it, then shuffle; you may cast it (paying its
/// costs). Activate only as a sorcery.
pub fn evolving_door() -> CardDefinition {
    CardDefinition {
        name: "Evolving Door",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::ColorCountEqualsSacrificedPlus(1)),
                    to: ZoneDest::ExileWithSourceStamp,
                },
                Effect::CastWithoutPayingImmediate {
                    what: Selector::CardExiledWithSource,
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: true,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kaslem's Stonetree // Kaslem's Strider — {2}{G} artifact. When it enters,
/// look at the top six cards of your library; you may put a land card from
/// among them onto the battlefield tapped, the rest on the bottom in a random
/// order. Craft with Cave {5}{G}. Back: a 5/5 Golem artifact creature.
pub fn kaslems_stonetree() -> CardDefinition {
    let strider = CardDefinition {
        name: "Kaslem's Strider",
        color_indicator: vec![Color::Green],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 5,
        toughness: 5,
        ..Default::default()
    };
    CardDefinition {
        name: "Kaslem's Stonetree",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::LookTopPutMatchingOntoBattlefield {
            count: Value::Const(6),
            filter: R::Land,
            then: None,
            max: Some(1),
            tapped: true,
            exile_rest: false,
            rest_to_graveyard: false,
            mandatory: false,
        })],
        activated_abilities: vec![crate::effect::shortcut::craft(cost(&[generic(5), g()]), R::HasLandType(LandType::Cave), 1)],
        back_face: Some(Box::new(strider)),
        ..Default::default()
    }
}

/// Fraying Line — {4} artifact. When it enters, put a rope counter on target
/// creature you control. At the beginning of each player's upkeep, that
/// player may pay {2}; if they do, they put a rope counter on a creature they
/// control. Otherwise, exile this artifact and each creature without a rope
/// counter on it, then remove all rope counters from all creatures.
pub fn fraying_line() -> CardDefinition {
    let roped = || R::WithCounter(CounterType::Rope);
    CardDefinition {
        name: "Fraying Line",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::Rope,
                amount: Value::ONE,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
                effect: Effect::MayPayBy {
                    who: PlayerRef::ActivePlayer,
                    description: "Pay {2} to put a rope counter on a creature you control?".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(Effect::ChooseOneAmong {
                        what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                        chooser: PlayerRef::You,
                        chosen: Box::new(Effect::AddCounter {
                            what: Selector::SeparatedPile { chosen: true },
                            kind: CounterType::Rope,
                            amount: Value::ONE,
                        }),
                        other: Box::new(Effect::Noop),
                    }),
                    else_: Some(Box::new(Effect::Seq(vec![
                        Effect::Exile { what: Selector::This },
                        Effect::Exile { what: Selector::EachPermanent(R::Creature.and(roped().negate())) },
                        Effect::RemoveCounter {
                            what: Selector::EachPermanent(R::Creature.and(roped())),
                            kind: CounterType::Rope,
                            amount: Value::Const(i32::MAX),
                        },
                    ]))),
                },
            },
        ],
        ..Default::default()
    }
}

/// Forcefield — {3} artifact. {1}: the next time an unblocked creature of
/// your choice would deal combat damage to you this turn, prevent all but 1
/// of that damage. (The shield reads any damage from the chosen creature
/// this turn — in practice its combat damage.)
pub fn forcefield() -> CardDefinition {
    CardDefinition {
        name: "Forcefield",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PreventAllButOneFromChosenSource { filter: R::Creature.and(R::IsUnblocked) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Secret Invasion — {1}{U}{U} Aura, enchant creature you control. When it
/// enters, exile up to one target creature other than enchanted creature
/// until this Aura leaves the battlefield; enchanted creature becomes a copy
/// of it for as long as this Aura remains. Enchanted creature has ward {2}.
pub fn secret_invasion() -> CardDefinition {
    CardDefinition {
        name: "Secret Invasion",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature.and(R::IsHostOfSource.negate()),
            effect: Box::new(Effect::Seq(vec![
                Effect::ExileUntilSourceLeaves { what: Selector::Target(0), return_to: crate::card::ExileReturnZone::Battlefield },
                Effect::BecomeCopyOfFor {
                    what: enchanted(),
                    source: Selector::CardExiledWithSource,
                    // While attached = until the Aura leaves (it can't move).
                    duration: Duration::WhileSourceAttached,
                    non_legendary: false,
                },
            ])),
        })],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(2)])))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Shivan Wumpus — {3}{R} 6/6 trample Beast. When it enters, any player may
/// sacrifice a land of their choice; if a player does, put it on top of its
/// owner's library.
pub fn shivan_wumpus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::AnyPlayerMaySacrificeToTopdeckSource { filter: R::Land })],
        ..creature("Shivan Wumpus", cost(&[generic(3), r()]), vec![CreatureType::Beast], 6, 6)
    }
}

/// Flux — {2}{U} sorcery. Each player discards any number of cards, then
/// draws that many cards. Draw a card.
pub fn flux() -> CardDefinition {
    CardDefinition {
        name: "Flux",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![Effect::EachPlayerDiscardsAnyThenDrawsThatMany, draw(1)]),
        ..Default::default()
    }
}
