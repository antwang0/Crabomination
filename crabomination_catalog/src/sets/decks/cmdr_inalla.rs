//! Commander: the cards the **Arcane Wizardry** precon (C17, Inalla,
//! Archmage Ritualist) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_inalla.rs`.
//!
//! Residuals (each also on its card):
//! - **Mairsil, the Pretender** — each borrowed ability can be activated any
//!   number of times a turn, not once; the cage takes the highest-mana-value
//!   artifact or creature card from your hand or graveyard.
//! - **Magus of the Abyss** — "target … of their choice" is a choice, not a
//!   target: a hexproof creature can still be picked.
//! - **Shifting Shadow** — the reveal is from the Aura's controller's library,
//!   which is the creature's controller unless the Aura changed hands; the new
//!   creature enters before the old one is destroyed.
//! - **Vindictive Lich** — the modes are always all three, in the order lose
//!   five / discard two / sacrifice; with fewer opponents than modes the
//!   modes left without a distinct opponent do nothing.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EntersAsCopy, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u, ManaCost};

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

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn wizard() -> R {
    R::HasCreatureType(CreatureType::Wizard)
}

/// "Target opponent" — a player slot the hostile auto-picker aims.
fn target_opponent() -> Selector {
    target_filtered(R::Player.and(R::ControlledByOpponent))
}

fn upkeep(scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), scope)
}

/// Inalla, Archmage Ritualist — eminence: another nontoken Wizard of yours
/// entering may be copied for {1} (a hasty token exiled at the next end step),
/// from the command zone or the battlefield. Tap five Wizards: 7 life.
pub fn inalla_archmage_ritualist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(wizard()).and(R::NotToken),
                })
                .in_command_zone(),
            effect: Effect::MayPay {
                description: "Pay {1} to copy that Wizard?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::CreateTokenCopiesHasteSac {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    exile: true,
                }),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Creature.and(wizard()), 5)),
            effect: Effect::LoseLife { who: target_filtered(R::Player), amount: Value::Const(7) },
            ..Default::default()
        }],
        ..creature(
            "Inalla, Archmage Ritualist",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            5,
        )
    }
}

/// Body Double — enters as a copy of any creature card in a graveyard.
pub fn body_double() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            from_graveyards: true,
            ..Default::default()
        }),
        ..creature("Body Double", cost(&[generic(4), u()]), vec![CreatureType::Shapeshifter], 0, 0)
    }
}

/// Clone Legion — a token copy of each creature target player controls.
pub fn clone_legion() -> CardDefinition {
    CardDefinition {
        name: "Clone Legion",
        cost: cost(&[generic(7), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
            body: Box::new(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::TriggerSource,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            }),
        },
        ..Default::default()
    }
}

/// Curse of Verbosity — whenever the cursed player is attacked, you draw, and
/// so does each opponent attacking them.
pub fn curse_of_verbosity() -> CardDefinition {
    let draw = |who: PlayerRef| Effect::Draw { who: Selector::Player(who), amount: Value::ONE };
    CardDefinition {
        name: "Curse of Verbosity",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                Predicate::AttackedDefenderWithCountAtLeast {
                    who: PlayerRef::ActivePlayer,
                    defender: PlayerRef::EnchantedPlayer,
                    at_least: 1,
                    include_planeswalkers: false,
                },
            ),
            effect: Effect::Seq(vec![
                draw(PlayerRef::You),
                Effect::If {
                    cond: Predicate::PlayerIsOpponent { who: PlayerRef::ActivePlayer },
                    then: Box::new(draw(PlayerRef::ActivePlayer)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Grixis Panorama — {C}, or sacrifice for a tapped basic Island, Swamp or
/// Mountain.
pub fn grixis_panorama() -> CardDefinition {
    CardDefinition {
        name: "Grixis Panorama",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land.and(R::HasSupertype(Supertype::Basic)).and(
                        R::HasLandType(LandType::Island)
                            .or(R::HasLandType(LandType::Swamp))
                            .or(R::HasLandType(LandType::Mountain)),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Izzet Chemister — hasty; bank instants and sorceries from your graveyard
/// under it, then sacrifice it to cast them all free.
pub fn izzet_chemister() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                tap_cost: true,
                effect: Effect::ExileWithSource {
                    what: target_filtered(instant_or_sorcery().and(R::InYourGraveyard)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::CastAnyOrderWithoutPaying {
                    what: Selector::CardExiledWithSource,
                    source_zone: Zone::Exile,
                    filter: None,
                    cap: None,
                    total_mana_value: None,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Izzet Chemister",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Kess, Dissident Mage — flying; once during each of your turns, an instant
/// or sorcery from your graveyard, exiled after.
pub fn kess_dissident_mage() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Once during each of your turns, you may cast an instant or sorcery spell \
                          from your graveyard. If a spell cast this way would be put into your \
                          graveyard, exile it instead.",
            effect: StaticEffect::GraveyardCastOncePerTurn { filter: instant_or_sorcery(), exile_after: true },
        }],
        ..creature(
            "Kess, Dissident Mage",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Magus of the Abyss — at each player's upkeep, that player loses a
/// nonartifact creature of their choice; no regeneration. Residual: the pick
/// is a choice, so hexproof doesn't stop it.
pub fn magus_of_the_abyss() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::AnyPlayer),
            effect: Effect::PlayerChoosesToDestroy {
                who: PlayerRef::ActivePlayer,
                filter: R::Creature.and(R::Not(Box::new(R::Artifact))),
                no_regen: true,
            },
        }],
        ..creature(
            "Magus of the Abyss",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            3,
        )
    }
}

/// Magus of the Mind — sacrifice it: shuffle, exile one plus this turn's spell
/// count off the top, and play them free this turn.
pub fn magus_of_the_mind() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::ShuffleLibrary { who: PlayerRef::You },
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Sum(vec![Value::ONE, Value::SpellsCastThisTurnTotal]),
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Magus of the Mind",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            5,
        )
    }
}

/// Mairsil, the Pretender — cages an artifact or creature card from your hand
/// or graveyard as it enters, and has the activated abilities of every card
/// you own in exile with a cage counter. Residuals: no once-each-turn limit on
/// a borrowed ability; the cage takes the highest-mana-value card.
pub fn mairsil_the_pretender() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile an artifact or creature card from your hand or graveyard with a \
                          cage counter?"
                .into(),
            body: Box::new(Effect::Seq(vec![
                Effect::ExileChosenFromHandOrGraveyard {
                    who: PlayerRef::You,
                    filter: R::Artifact.or(R::Creature),
                },
                Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Cage, amount: Value::ONE },
            ])),
        })],
        static_abilities: vec![StaticAbility {
            description: "Mairsil has all activated abilities of all cards you own in exile with \
                          cage counters on them.",
            effect: StaticEffect::HasActivatedAbilitiesOfOwnedExiledWithCounter { counter: CounterType::Cage },
        }],
        ..creature(
            "Mairsil, the Pretender",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Mirror of the Forebears — names a creature type; {1}: until end of turn it
/// becomes a copy of your creature of that type, and stays an artifact.
pub fn mirror_of_the_forebears() -> CardDefinition {
    CardDefinition {
        name: "Mirror of the Forebears",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::IsSourceChosenCreatureType),
                    ),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
                Effect::AddCardTypeIndefinitely {
                    what: Selector::This,
                    card_type: CardType::Artifact,
                    until_eot: true,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Portal Mage — flash; entering during declare attackers, it may point an
/// attacker at a different player or permanent.
pub fn portal_mage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::CurrentStepIs(TurnStep::DeclareAttackers)),
            effect: Effect::ReselectAttackTarget { what: target_filtered(R::IsAttacking) },
        }],
        ..creature(
            "Portal Mage",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Serendib Sorcerer — {T}: another creature has base 0/2 until end of turn.
pub fn serendib_sorcerer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::SetBasePT {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                power: Value::Const(0),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Serendib Sorcerer",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Sorcerer],
            1,
            1,
        )
    }
}

/// Shifting Shadow — enchanted creature has haste; at its controller's upkeep
/// it's destroyed, and the Aura moves to the next creature card revealed off
/// the top (the rest go to the bottom at random). The trigger sits on the
/// Aura, so "this creature" is what it enchants. Residuals: the library is the
/// Aura controller's, and the new creature enters before the old one is
/// destroyed.
pub fn shifting_shadow() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Shifting Shadow",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Haste], ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: host(),
                filter: R::ControlledByActivePlayer,
            }),
            // The old host is bound as `TriggerSource` and destroyed last: a
            // destroy runs the state-based check at once, which would bin the
            // Aura before it could move.
            effect: Effect::ForEach {
                selector: host(),
                body: Box::new(Effect::Seq(vec![
                    Effect::RevealUntilOneToBattlefieldRestBottom { filter: R::Creature, damage_controller: false },
                    Effect::Attach { what: Selector::This, to: Selector::LastMoved },
                    Effect::Destroy { what: Selector::TriggerSource },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Taigam, Sidisi's Hand — skip your draw step; at your upkeep keep one of the
/// top three and bin the rest; {B}, {T}, exile X graveyard cards: -X/-X.
pub fn taigam_sidisis_hand() -> CardDefinition {
    let minus_x = || Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Skip your draw step.",
            effect: StaticEffect::ControllerSkipsDrawStep,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: true,
                ..Default::default()
            })),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            exile_other_filter: Some((R::InYourGraveyard, 1)),
            exile_other_x: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: minus_x(),
                toughness: minus_x(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Taigam, Sidisi's Hand",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Vindictive Lich — when it dies, choose one or more, each aimed at a
/// different opponent: sacrifice a creature, discard two, lose five. Residual:
/// all three modes, in a fixed order (lose five first, so a duel's one
/// opponent takes the biggest hit).
pub fn vindictive_lich() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::ChooseN {
            picks: vec![2, 1, 0],
            modes: vec![
                Effect::Sacrifice { who: target_opponent(), count: Value::ONE, filter: R::Creature },
                Effect::Discard { who: target_opponent(), amount: Value::Const(2), random: false },
                Effect::LoseLife { who: target_opponent(), amount: Value::Const(5) },
            ],
        })],
        ..creature(
            "Vindictive Lich",
            cost(&[generic(3), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            4,
            1,
        )
    }
}
