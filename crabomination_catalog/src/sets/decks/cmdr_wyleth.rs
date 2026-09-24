//! Commander: the cards the **Arm for Battle** precon (CMR, Wyleth, Soul of
//! Steel) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_wyleth.rs`.
//!
//! No residuals: Dawn Charm's counter mode reads a spell that targets *you*
//! (`SpellTargetsMatching(Player & ControlledByYou)`), and Timely Ward's flash
//! is checked against its declared target (`SelfFlashIfTargets`).

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword,
    SelectionRequirement as R, Selector, SplitCard, SplitHalf, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{renown, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, cost, generic, hybrid, r, w, x};
use crate::sets::{enters_tapped, tap_add};

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

/// An Equipment with an equip cost and a bonus.
fn equipment(name: &'static str, mana: crate::mana::ManaCost, equip: crate::mana::ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// An Aura that attaches to a target matching `enchant` on resolution.
fn aura(name: &'static str, mana: crate::mana::ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn equipped() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn any_target() -> Selector {
    target_filtered(R::Creature.or(R::Player).or(R::Planeswalker))
}

/// Blazing Sunsteel — equipped creature gets +1/+0 per opponent; whenever it
/// is dealt damage, it deals that much damage to any target. Equip {4}.
pub fn blazing_sunsteel() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Equipped creature gets +1/+0 for each opponent you have.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: equipped(),
                power: Value::OpponentCount,
                toughness: Value::Const(0),
            },
        }],
        ..equipment(
            "Blazing Sunsteel",
            cost(&[generic(1), r()]),
            cost(&[generic(4)]),
            EquipBonus {
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                    effect: Effect::DealDamage { to: any_target(), amount: Value::TriggerEventAmount },
                }],
                ..Default::default()
            },
        )
    }
}

/// Dawn Charm — choose one: prevent all combat damage this turn; regenerate
/// target creature; counter target spell that targets you.
pub fn dawn_charm() -> CardDefinition {
    CardDefinition {
        name: "Dawn Charm",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PreventAllCombatDamageThisTurn,
            Effect::Regenerate { what: target_filtered(R::Creature) },
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::SpellTargetsMatching(Box::new(
                    R::Player.and(R::ControlledByYou),
                )))),
            },
        ]),
        ..Default::default()
    }
}

/// Faith Unbroken — enchant creature you control; on entry, exile target
/// creature an opponent controls until this leaves; +2/+2.
pub fn faith_unbroken() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                return_to: ExileReturnZone::Battlefield,
            },
        }],
        ..aura(
            "Faith Unbroken",
            cost(&[generic(3), w()]),
            R::Creature.and(R::ControlledByYou),
            EquipBonus { power: 2, toughness: 2, ..Default::default() },
        )
    }
}

/// Haunted Cloak — vigilance, trample, haste. Equip {1}.
pub fn haunted_cloak() -> CardDefinition {
    equipment(
        "Haunted Cloak",
        cost(&[generic(3)]),
        cost(&[generic(1)]),
        EquipBonus { keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Haste], ..Default::default() },
    )
}

/// Hero's Blade — +3/+2; whenever a legendary creature you control enters,
/// you may attach it to that creature. Equip {4}.
pub fn heros_blade() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                },
            ),
            effect: Effect::MayDo {
                description: "Attach Hero's Blade to the legendary creature?".into(),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
            },
        }],
        ..equipment(
            "Hero's Blade",
            cost(&[generic(2)]),
            cost(&[generic(4)]),
            EquipBonus { power: 3, toughness: 2, ..Default::default() },
        )
    }
}

/// Ironclad Slayer — on entry, you may return an Aura or Equipment card from
/// your graveyard to your hand.
pub fn ironclad_slayer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment))
                        .from_your_graveyard(),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..creature(
            "Ironclad Slayer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            2,
        )
    }
}

/// Jaya's Immolating Inferno — legendary sorcery: X damage to each of up to
/// three targets (CR 205.4e — needs a legendary creature or planeswalker).
pub fn jayas_immolating_inferno() -> CardDefinition {
    CardDefinition {
        name: "Jaya's Immolating Inferno",
        cost: cost(&[x(), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature.or(R::Player).or(R::Planeswalker),
            effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::XFromCost }),
        },
        ..Default::default()
    }
}

/// Memorial to War — enters tapped; {T}: {R}; {4}{R}, {T}, sacrifice it:
/// destroy target land.
pub fn memorial_to_war() -> CardDefinition {
    CardDefinition {
        name: "Memorial to War",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(Color::Red),
            ActivatedAbility {
                mana_cost: cost(&[generic(4), r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Destroy { what: target_filtered(R::Land) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Odric, Lunarch Marshal — at the beginning of each combat, your creatures
/// share the keywords any one of them has.
pub fn odric_lunarch_marshal() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
            effect: Effect::ShareKeywordsAmongYourCreatures {
                keywords: vec![
                    Keyword::FirstStrike,
                    Keyword::Flying,
                    Keyword::Deathtouch,
                    Keyword::DoubleStrike,
                    Keyword::Haste,
                    Keyword::Hexproof,
                    Keyword::Indestructible,
                    Keyword::Lifelink,
                    Keyword::Menace,
                    Keyword::Reach,
                    Keyword::Skulk,
                    Keyword::Trample,
                    Keyword::Vigilance,
                ],
            },
        }],
        ..creature(
            "Odric, Lunarch Marshal",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// On Serra's Wings — legendary Aura: enchanted creature is legendary, gets
/// +1/+1, and has flying, vigilance and lifelink.
pub fn on_serras_wings() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is legendary.",
            effect: StaticEffect::AttachedIsLegendary,
        }],
        ..aura(
            "On Serra's Wings",
            cost(&[generic(3), w()]),
            R::Creature,
            EquipBonus {
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Lifelink],
                ..Default::default()
            },
        )
    }
}

/// Relic Seeker — renown 1; when it becomes renowned, you may search for an
/// Equipment card.
pub fn relic_seeker() -> CardDefinition {
    let mut trigger = renown(1);
    // Renown is the only way it becomes renowned, so the search rides the
    // renown trigger's "if it isn't renowned" branch.
    if let Effect::If { then, .. } = &mut trigger.effect
        && let Effect::Seq(steps) = then.as_mut()
    {
        steps.push(Effect::MayDo {
            description: "Search for an Equipment card?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        });
    }
    CardDefinition {
        triggered_abilities: vec![trigger],
        ..creature("Relic Seeker", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Response // Resurgence — Response: 5 damage to target attacking or
/// blocking creature. Resurgence: your creatures gain first strike and
/// vigilance until end of turn; an additional combat and main phase.
pub fn response_resurgence() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Response // Resurgence",
        cost: cost(&[hybrid(Color::White, Color::Red), hybrid(Color::White, Color::Red)]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            amount: Value::Const(5),
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), r(), w()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword { what: yours(), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: yours(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                    Effect::AdditionalCombatPhaseAfterMain { count: Value::Const(1) },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// "At the beginning of your upkeep, put a +1/+1 counter on equipped
/// creature if it's [color]."
fn ring_upkeep(color: Color) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect: Effect::If {
            cond: Predicate::EntityMatches { what: equipped(), filter: R::HasColor(color) },
            then: Box::new(Effect::AddCounter {
                what: equipped(),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        },
    }
}

/// Ring of Thune — vigilance; upkeep counter on a white bearer. Equip {1}.
pub fn ring_of_thune() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![ring_upkeep(Color::White)],
        ..equipment(
            "Ring of Thune",
            cost(&[generic(2)]),
            cost(&[generic(1)]),
            EquipBonus { keywords: vec![Keyword::Vigilance], ..Default::default() },
        )
    }
}

/// Ring of Valkas — haste; upkeep counter on a red bearer. Equip {1}.
pub fn ring_of_valkas() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![ring_upkeep(Color::Red)],
        ..equipment(
            "Ring of Valkas",
            cost(&[generic(2)]),
            cost(&[generic(1)]),
            EquipBonus { keywords: vec![Keyword::Haste], ..Default::default() },
        )
    }
}

/// Tiana, Ship's Caretaker — flying, first strike; whenever an Aura or
/// Equipment you control is put into a graveyard from the battlefield, you
/// may return it to its owner's hand at the beginning of the next end step.
pub fn tiana_ships_caretaker() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                },
            ),
            effect: Effect::AtNextEndStep {
                body: Box::new(Effect::MayDo {
                    description: "Return the Aura or Equipment to its owner's hand?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                    }),
                }),
            },
        }],
        ..creature(
            "Tiana, Ship's Caretaker",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Angel, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Timely Ward — enchanted creature has indestructible; flash when it
/// targets a commander.
pub fn timely_ward() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may cast this spell as though it had flash if it targets a commander.",
            effect: StaticEffect::SelfFlashIfTargets { filter: R::IsCommander },
        }],
        ..aura(
            "Timely Ward",
            cost(&[generic(2), w()]),
            R::Creature,
            EquipBonus { keywords: vec![Keyword::Indestructible], ..Default::default() },
        )
    }
}

/// Wild Ricochet — you may choose new targets for target instant or sorcery
/// spell, then copy it; you may choose new targets for the copy.
pub fn wild_ricochet() -> CardDefinition {
    CardDefinition {
        name: "Wild Ricochet",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ChooseNewTargetsForSpell {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
            },
            Effect::CopySpellMayChooseTargets { what: Selector::Target(0), count: Value::Const(1) },
        ]),
        ..Default::default()
    }
}
