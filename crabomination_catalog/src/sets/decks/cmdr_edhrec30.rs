//! Commander: two- and three-card gaps for Eriette, Zo-Zu, Charix, Dion,
//! Gyruda, Redshift and Zirda (`cmdr_edhrec29` is the previous batch). Tests
//! in `tests/recent_b/cmdr_edhrec30.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EntersAsCopy,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{deal, emerge, etb, on_cast, on_dies, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, LookPick, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, r, u, w, Color, ManaCost};
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

fn aura(name: &'static str, mana: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..Default::default()
    }
}

fn each_opponent() -> Selector {
    Selector::Player(PlayerRef::EachOpponent)
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// "Create a token that's a copy of `source`" with the given extra creature
/// types.
fn token_copy(source: Selector, extra_creature_types: Vec<CreatureType>) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types,
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
    }
}

// ── Eriette, the Beguiler ────────────────────────────────────────────────────

/// Indestructibility — {3}{W} Aura (enchant permanent): it has indestructible.
pub fn indestructibility() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Indestructible], ..Default::default() }),
        ..aura("Indestructibility", cost(&[generic(3), w()]), R::Permanent)
    }
}

/// Aura Finesse — {U} instant. Attach target Aura you control to target
/// creature. Draw a card.
pub fn aura_finesse() -> CardDefinition {
    CardDefinition {
        name: "Aura Finesse",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Attach {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).and(R::ControlledByYou),
                },
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

// ── Zo-Zu the Punisher ───────────────────────────────────────────────────────

/// Dingus Egg — {4} artifact. Whenever a land is put into a graveyard from the
/// battlefield, it deals 2 damage to that land's controller.
pub fn dingus_egg() -> CardDefinition {
    CardDefinition {
        name: "Dingus Egg",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: deal(2, Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)))),
        }],
        ..Default::default()
    }
}

/// Tunnel Ignus — {1}{R} 2/1 Elemental. A land entering under an opponent's
/// control, if they had another land enter this turn, deals them 3.
pub fn tunnel_ignus() -> CardDefinition {
    let them = || PlayerRef::ControllerOf(Box::new(Selector::TriggerSource));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
                    Predicate::LandsEnteredThisTurnAtLeast { who: them(), at_least: 2 },
                ]),
            ),
            effect: deal(3, Selector::Player(them())),
        }],
        ..creature("Tunnel Ignus", cost(&[generic(1), r()]), vec![CreatureType::Elemental], 2, 1)
    }
}

/// Wildfire — {4}{R}{R} sorcery. Each player sacrifices four lands; 4 damage
/// to each creature.
pub fn wildfire() -> CardDefinition {
    CardDefinition {
        name: "Wildfire",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice { who: Selector::Player(PlayerRef::EachPlayer), count: Value::Const(4), filter: R::Land },
            deal(4, Selector::EachPermanent(R::Creature)),
        ]),
        ..Default::default()
    }
}

// ── Charix, the Raging Isle ──────────────────────────────────────────────────

/// Fortress Crab — {3}{U} 1/6 Crab.
pub fn fortress_crab() -> CardDefinition {
    creature("Fortress Crab", cost(&[generic(3), u()]), vec![CreatureType::Crab], 1, 6)
}

/// Vexing Scuttler — {8} 4/5 Eldrazi Crab, emerge {6}{U}. Casting it may
/// return an instant or sorcery card from your graveyard to your hand.
pub fn vexing_scuttler() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(emerge(cost(&[generic(6), u()]))),
        triggered_abilities: vec![on_cast(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).from_your_graveyard(),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
        })],
        ..creature("Vexing Scuttler", cost(&[generic(8)]), vec![CreatureType::Eldrazi, CreatureType::Crab], 4, 5)
    }
}

/// Crab Umbra — {U} Aura. {2}{U}: untap enchanted creature. Umbra armor.
pub fn crab_umbra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::UmbraArmor],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Untap { what: enchanted(), up_to: None },
            ..Default::default()
        }],
        ..aura("Crab Umbra", cost(&[u()]), R::Creature)
    }
}

// ── Dion, Bahamut's Dominant ─────────────────────────────────────────────────

/// Cavalier of Dawn — {2}{W}{W}{W} 4/6 Elemental Knight, vigilance. ETB:
/// destroy up to one nonland permanent; its controller makes a 3/3 Golem.
/// Dies: return an artifact or enchantment card from your graveyard.
pub fn cavalier_of_dawn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Permanent.and(R::Nonland),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Destroy { what: Selector::Target(0) },
                    Effect::CreateToken {
                        who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        count: Value::ONE,
                        definition: Arc::new(crabomination_base::tokens::golem_3_3_token()),
                    },
                ])),
            }),
            on_dies(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Artifact.or(R::Enchantment).from_your_graveyard(),
                effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
            }),
        ],
        ..creature(
            "Cavalier of Dawn",
            cost(&[generic(2), w(), w(), w()]),
            vec![CreatureType::Elemental, CreatureType::Knight],
            4,
            6,
        )
    }
}

/// Diamond Knight — {3} 1/1 artifact Knight, vigilance. As it enters, choose
/// a color; your spells of that color grow it.
pub fn diamond_knight() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Vigilance],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellSharesChosenColorOfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Diamond Knight", cost(&[generic(3)]), vec![CreatureType::Knight], 1, 1)
    }
}

/// Fiendslayer Paladin — {1}{W}{W} 2/2 Human Knight, first strike, lifelink;
/// black or red spells your opponents control can't target it.
pub fn fiendslayer_paladin() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Lifelink,
            Keyword::CantBeTargetedByOpponentSpellsMatching(Box::new(
                R::HasColor(Color::Black).or(R::HasColor(Color::Red)),
            )),
        ],
        ..creature("Fiendslayer Paladin", cost(&[generic(1), w(), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

// ── Gyruda, Doom of Depths ───────────────────────────────────────────────────

/// Mirrorhall Mimic // Ghastly Mimicry — {3}{U} 0/0 Spirit that may enter as a
/// copy of any creature (a Spirit too). Disturb {3}{U}{U}: an Aura that each
/// upkeep makes a token copy of the enchanted creature (a Spirit too).
pub fn mirrorhall_mimic() -> CardDefinition {
    let ghastly = CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: token_copy(enchanted(), vec![CreatureType::Spirit]),
        }],
        color_indicator: vec![Color::Blue],
        ..aura("Ghastly Mimicry", ManaCost::default(), R::Creature)
    };
    CardDefinition {
        keywords: vec![Keyword::Disturb(cost(&[generic(3), u(), u()]))],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        }),
        back_face: Some(Box::new(ghastly)),
        ..creature("Mirrorhall Mimic", cost(&[generic(3), u()]), vec![CreatureType::Spirit], 0, 0)
    }
}

/// Undercover Operative — {2}{U}{U} 0/0 Shapeshifter Rogue that may enter as a
/// copy of any creature, with a shield counter if you control that creature.
pub fn undercover_operative() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            copied_counters: vec![(R::ControlledByYou, CounterType::Shield, 1)],
            ..Default::default()
        }),
        ..creature(
            "Undercover Operative",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Shapeshifter, CreatureType::Rogue],
            0,
            0,
        )
    }
}

// ── Redshift, Rocketeer Chief / Zirda, the Dawnwaker ─────────────────────────

/// Bhaal's Invoker — {2}{R} 4/2 Dragon Shaman. {8}: 4 damage to each opponent.
pub fn bhaals_invoker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: deal(4, each_opponent()),
            ..Default::default()
        }],
        ..creature("Bhaal's Invoker", cost(&[generic(2), r()]), vec![CreatureType::Dragon, CreatureType::Shaman], 4, 2)
    }
}

/// Reckless Handling — {1}{R} sorcery. Tutor an artifact card, then discard
/// a card at random; an artifact discarded deals 2 to each opponent.
pub fn reckless_handling() -> CardDefinition {
    CardDefinition {
        name: "Reckless Handling",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search { who: PlayerRef::You, filter: R::Artifact, to: ZoneDest::Hand(PlayerRef::You) },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::DiscardedThisResolution { filter: R::Artifact }),
                then: Box::new(deal(2, each_opponent())),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Cogwork Assembler — {3} 2/3 artifact Assembly-Worker. {7}: a hasty token
/// copy of target artifact, exiled at the next end step.
pub fn cogwork_assembler() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7)]),
            effect: Effect::Seq(vec![
                token_copy(target_filtered(R::Artifact), vec![]),
                Effect::GrantKeyword {
                    what: Selector::LastCreatedToken,
                    keyword: Keyword::Haste,
                    duration: Duration::Permanent,
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Move { what: Selector::LastCreatedToken, to: ZoneDest::Exile }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Cogwork Assembler", cost(&[generic(3)]), vec![CreatureType::AssemblyWorker], 2, 3)
    }
}

/// Thran Spider — {3} 2/4 artifact Spider, reach. ETB: you and target
/// opponent each make a tapped Powerstone. {7}: look at four, take an
/// artifact.
pub fn thran_spider() -> CardDefinition {
    let powerstone = || {
        Arc::new(crate::card::TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() })
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 1,
            filter: R::OpponentPlayer,
            effect: Box::new(Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: powerstone() },
                Effect::CreateToken { who: PlayerRef::Target(0), count: Value::ONE, definition: powerstone() },
            ])),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7)]),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(4),
                pick_filter: Some(R::Artifact),
                optional: true,
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..creature("Thran Spider", cost(&[generic(3)]), vec![CreatureType::Spider], 2, 4)
    }
}
