//! Commander: 2026-10-18 (`019Y6J4R`) closers, twentieth batch. Tests in
//! `tests/recent_b/cmdr_019y_t.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{ActivatedAbility, Duration, Effect, PlayerRef, Predicate, Selector, StaticEffect, Value};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, u};

/// Dawnhand Dissident — {B} 1/2 Elf Warlock. {T}, Blight 1: surveil 1.
/// {T}, Blight 2: exile target card from a graveyard (linked to this). During
/// your turn, you may cast creature spells from among cards you own exiled
/// with it by removing three counters from among creatures you control in
/// addition to paying their other costs.
pub fn dawnhand_dissident() -> CardDefinition {
    CardDefinition {
        name: "Dawnhand Dissident",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warlock], ..Default::default() },
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                blight_cost: 1,
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                blight_cost: 2,
                effect: Effect::ExileWithSource { what: target_filtered(R::InGraveyard) },
                ..Default::default()
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "During your turn, you may cast creature spells from among cards you own exiled with this creature by removing three counters from among creatures you control in addition to paying their other costs.",
            effect: StaticEffect::CastLinkedExiledCreaturesForCounters { counters: 3 },
        }],
        ..Default::default()
    }
}

/// Cryptic Spires — Land. As you create your deck, circle two colors. Enters
/// tapped; {T}: add one mana of either circled color
/// (`ManaPayload::CircledColors` — read off the commander identity).
pub fn cryptic_spires() -> CardDefinition {
    CardDefinition {
        name: "Cryptic Spires",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::CircledColors },
            ..Default::default()
        }],
        static_abilities: vec![crate::sets::enters_tapped()],
        ..Default::default()
    }
}

/// Absorbing Man — {1}{G}{U} legendary 4/4 vigilance Human Villain. At the
/// beginning of your first main phase, until your next turn, he becomes a
/// copy of up to one target artifact, non-Aura enchantment, or land, except
/// his name is Absorbing Man, he's a legendary 4/4 Human Villain creature in
/// addition to his other types, and he has vigilance (CR 707.9b).
pub fn absorbing_man() -> CardDefinition {
    CardDefinition {
        name: "Absorbing Man",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Villain], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Artifact
                    .or(R::Enchantment.and(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).negate()))
                    .or(R::Land)
                    .and(R::OtherThanSource),
                effect: Box::new(Effect::Seq(vec![
                    Effect::BecomeCopyOfFor {
                        what: Selector::This,
                        source: Selector::Target(0),
                        duration: Duration::UntilNextTurn,
                        non_legendary: false,
                    },
                    Effect::AmendCopy {
                        what: Selector::This,
                        keep_activated: vec![],
                        keep_triggered: vec![],
                        pt: Some((4, 4)),
                        keep_name: true,
                        legendary: true,
                        keywords: vec![Keyword::Vigilance],
                        card_types: vec![CardType::Creature],
                        creature_types: vec![CreatureType::Human, CreatureType::Villain],
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Hurkyl, Master Wizard — {1}{U}{U} legendary 2/4 Human Wizard Advisor. At
/// the beginning of your end step, if you've cast a noncreature spell this
/// turn, reveal the top five cards of your library; for each card type among
/// noncreature spells you've cast this turn, you may put a card of that type
/// from among them into your hand. The rest go on the bottom in a random order.
pub fn hurkyl_master_wizard() -> CardDefinition {
    CardDefinition {
        name: "Hurkyl, Master Wizard",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::NoncreatureSpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
            ),
            effect: Effect::RevealTopTakeOnePerCastType { who: PlayerRef::You, count: Value::Const(5) },
        }],
        ..Default::default()
    }
}

/// Cloak and Dagger, Entwined — {1}{W}{B} legendary 2/2 deathtouch lifelink
/// Human Hero. When they enter, choose target opponent and up to one target
/// creature they control. That player reveals their hand. You may exile a
/// nonland card from their hand or the chosen creature until Cloak and
/// Dagger leave the battlefield (CR 610.3).
pub fn cloak_and_dagger_entwined() -> CardDefinition {
    use crate::card::ExileReturnZone;
    let creature = || Effect::MayDo {
        description: "Exile the chosen creature until Cloak and Dagger leave the battlefield?".into(),
        body: Box::new(Effect::ExileUntilSourceLeaves {
            what: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::SameControllerAsTargetSlot(0)) },
            return_to: ExileReturnZone::Battlefield,
        }),
    };
    CardDefinition {
        name: "Cloak and Dagger, Entwined",
        cost: cost(&[generic(1), crate::mana::w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Hero], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::RevealHand { who: PlayerRef::Target(0) },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CardsInHandMatching { who: PlayerRef::Target(0), filter: R::Nonland },
                        Value::ONE,
                    ),
                    then: Box::new(Effect::MayDoElse {
                        description: "Exile a nonland card from their hand until Cloak and Dagger leave the battlefield?"
                            .into(),
                        body: Box::new(Effect::ExileChosenUntilSourceLeaves {
                            from: crate::effect::shortcut::target_filtered(R::OpponentPlayer),
                            count: Value::ONE,
                            filter: R::Nonland,
                            return_to: ExileReturnZone::Hand,
                        }),
                        else_: Box::new(creature()),
                    }),
                    else_: Box::new(creature()),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Sculpted Sunburst — {3}{W}{W} sorcery. Choose a creature you control, then
/// each opponent chooses a creature they control with equal or lesser power;
/// if you chose one, exile each creature not chosen this way.
pub fn sculpted_sunburst() -> CardDefinition {
    CardDefinition {
        name: "Sculpted Sunburst",
        cost: cost(&[generic(3), crate::mana::w(), crate::mana::w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseCreaturesByPowerExileRest,
        ..Default::default()
    }
}

/// Fumble — {1}{U} instant. Return target creature to its owner's hand; gain
/// control of all Auras and Equipment that were attached to it, then attach
/// them to another creature.
pub fn fumble() -> CardDefinition {
    CardDefinition {
        name: "Fumble",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::BounceStealAttachmentsReattach { what: crate::effect::shortcut::target_filtered(R::Creature) },
        ..Default::default()
    }
}

/// Threats Undetected — {2}{G} sorcery. Search for up to four creature cards
/// with different powers and reveal them; an opponent chooses two to shuffle
/// back, the rest go to your hand.
pub fn threats_undetected() -> CardDefinition {
    CardDefinition {
        name: "Threats Undetected",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchDistinctPowersOpponentReturnsTwo { max: 4, back: 2 },
        ..Default::default()
    }
}

/// Trickster's Talisman — {U} Equipment. Equipped creature gets +1/+1 and has
/// "Whenever this creature deals combat damage to a player, you may sacrifice
/// Trickster's Talisman. If you do, create a token that's a copy of this
/// creature." Equip {2}.
pub fn tricksters_talisman() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Trickster's Talisman",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Sacrifice Trickster's Talisman to create a token copy of this creature?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSelected { what: Selector::AttachmentGranting },
                        crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, Selector::This),
                    ])),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Urza Assembles the Titans — {3}{W}{W} Saga, read ahead. I: scry 4, then
/// you may reveal the top card; a planeswalker card goes to your hand.
/// II: you may put a planeswalker card with mana value 6 or less from your
/// hand onto the battlefield. III: loyalty abilities twice this turn.
pub fn urza_assembles_the_titans() -> CardDefinition {
    let walker = || R::HasCardType(CardType::Planeswalker);
    CardDefinition {
        name: "Urza Assembles the Titans",
        cost: cost(&[generic(3), crate::mana::w(), crate::mana::w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        read_ahead: true,
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Scry { who: PlayerRef::You, amount: Value::Const(4) },
                    Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        pick_filter: Some(walker()),
                        optional: true,
                        rest_on_top: true,
                        ..Default::default()
                    })),
                ]),
            ),
            (
                2,
                Effect::MayDo {
                    description: "Put a planeswalker card with mana value 6 or less from your hand onto the battlefield?"
                        .into(),
                    body: Box::new(Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: walker().and(R::ManaValueAtMost(6)),
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    }),
                },
            ),
            (3, Effect::LoyaltyAbilitiesTwiceThisTurn),
        ],
        ..Default::default()
    }
}

/// Musician — {2}{U} 1/3 Human Wizard. Cumulative upkeep {1}. {T}: put a
/// music counter on target creature; if it doesn't have "At the beginning of
/// your upkeep, destroy this creature unless you pay {1} for each music
/// counter on it," it gains that ability.
pub fn musician() -> CardDefinition {
    CardDefinition {
        name: "Musician",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::CumulativeUpkeep(crate::card::CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMusicCounterGrantUpkeepToll {
                what: crate::effect::shortcut::target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Balduvian Warlord — {3}{R} 3/2 Human Barbarian. {T}: remove target
/// blocking creature from combat; creatures it was blocking that no other
/// creature blocks become unblocked, then it blocks an attacking creature of
/// your choice. Activate only during the declare blockers step.
pub fn balduvian_warlord() -> CardDefinition {
    CardDefinition {
        name: "Balduvian Warlord",
        cost: cost(&[generic(3), crate::mana::r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Barbarian], ..Default::default() },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::CurrentStepIs(TurnStep::DeclareBlockers)),
            effect: Effect::RemoveBlockerThenReblock {
                what: crate::effect::shortcut::target_filtered(R::Creature.and(R::IsBlocking)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Beamsplitter Mage — {U}{R} 2/2 Vedalken Wizard. Whenever you cast an
/// instant or sorcery spell that targets only this creature, if you control
/// another creature it could target, choose one; copy the spell, the copy
/// targeting the chosen creature (CR 707.10).
pub fn beamsplitter_mage() -> CardDefinition {
    CardDefinition {
        name: "Beamsplitter Mage",
        cost: cost(&[u(), crate::mana::r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_instant_or_sorcery()),
            effect: Effect::CopyForChosenOtherTargetableCreature,
        }],
        ..Default::default()
    }
}
