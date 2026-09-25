//! Commander: the cards the **Scrappy Survivors** precon (PIP, Dogmeat, Ever
//! Loyal) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_dogmeat.rs`.
//!
//! Residuals (each also on its card):
//! - **Agility Bobblehead** — the X creatures are your greatest-power ones,
//!   chosen on resolution rather than targeted.
//! - **Brotherhood Outcast**, **Vault 101: Birthday Party** — the Aura or
//!   Equipment card is picked (greatest mana value first) rather than
//!   targeted, and its host is the engine's pick.
//! - **Inventory Management** — every Aura and Equipment you choose moves to
//!   one creature, your greatest-power one.
//! - **Perception Bobblehead** — the rest go to the bottom in the cascade
//!   order, not a random one.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, ConditionalEquipBonus, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, on_you_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, w, Color, ManaCost, SpendRestriction};
use crate::sets::{tap_add_any_color, tap_add_colorless};
use crabomination_base::tokens::{food_token, junk_token, treasure_token};

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

/// An Aura: "Enchant creature" with `bonus` on the enchanted creature.
fn aura(name: &'static str, mana: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
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

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

fn land(name: &'static str, second: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), second],
        ..Default::default()
    }
}

fn aura_card() -> R {
    R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
}

fn equipment_card() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Equipment)
}

fn aura_or_equipment() -> R {
    aura_card().or(equipment_card())
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn make(token: TokenDefinition, n: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(token) }
}

fn junk(n: i32) -> Effect {
    make(junk_token(), Value::Const(n))
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn discard(who: Selector) -> Effect {
    Effect::Discard { who, amount: Value::ONE, random: false }
}

fn plus(what: Selector, n: i32) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::Const(n) }
}

fn rad(who: PlayerRef, n: i32) -> Effect {
    Effect::AddRadCounters { who: Selector::Player(who), amount: Value::Const(n) }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

/// "Whenever you cast an Aura or Equipment spell, `effect`."
fn on_cast_aura_or_equipment(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(aura_or_equipment())),
        effect,
    }
}

/// The dying trigger subject carried at least one `filter` attachment.
fn died_with(filter: R) -> Predicate {
    Predicate::ValueAtLeast(Value::AttachmentsOnDyingSubject { filter }, Value::ONE)
}

fn bobbleheads() -> Value {
    Value::PermanentCountControlledByMatching(PlayerRef::You, R::HasArtifactSubtype(ArtifactSubtype::Bobblehead))
}

fn bobblehead(name: &'static str, second: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Bobblehead], ..Default::default() },
        activated_abilities: vec![tap_add_any_color(), second],
        ..Default::default()
    }
}

/// "Return up to one target Aura or Equipment card from your graveyard to
/// your hand."
fn regrow_attachment() -> Effect {
    Effect::ApplyToTargets {
        max_targets: 1,
        min_targets: 0,
        filter: aura_or_equipment().and(R::InYourGraveyard),
        effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
    }
}

/// "An Aura or Equipment card from your graveyard (or hand) onto the
/// battlefield" — the card and its host are the engine's pick.
fn put_attachment_from(zones: Vec<Zone>, filter: R) -> Effect {
    Effect::PutOntoBattlefieldAttached { zones, filter, host: None, max: Some(Value::ONE), creatures_only: true }
}

/// Acquired Mutation — +2/+2 and goaded; attacking, defending player gets two
/// rad counters.
pub fn acquired_mutation() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is goaded.",
            effect: StaticEffect::AttachedIsGoaded,
        }],
        ..aura(
            "Acquired Mutation",
            cost(&[generic(2), r()]),
            EquipBonus {
                power: 2,
                toughness: 2,
                triggered_abilities: vec![on_attack(rad(PlayerRef::DefendingPlayer, 2))],
                ..Default::default()
            },
        )
    }
}

/// Agility Bobblehead — mana of any color; {3}, {T}: up to X of your
/// creatures gain haste and can't be blocked except by haste creatures.
/// Residual: the X creatures are your greatest-power ones.
pub fn agility_bobblehead() -> CardDefinition {
    let chosen = || Selector::GreatestPowerTopN { filter: R::Creature.and(R::ControlledByYou), count: bobbleheads() };
    bobblehead(
        "Agility Bobblehead",
        ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: chosen(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                Effect::GrantKeyword {
                    what: chosen(),
                    keyword: Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Haste))),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        },
    )
}

/// Almost Perfect — base 9/10 and indestructible.
pub fn almost_perfect() -> CardDefinition {
    aura(
        "Almost Perfect",
        cost(&[generic(4), g(), w()]),
        EquipBonus { set_base_pt: Some((9, 10)), keywords: vec![Keyword::Indestructible], ..Default::default() },
    )
}

/// Animal Friend — enchanted creature attacking makes a 1/1 Squirrel with a
/// +1/+1 counter per other Aura and Equipment on it.
pub fn animal_friend() -> CardDefinition {
    let squirrel = TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Squirrel], ..Default::default() },
        ..Default::default()
    };
    let others = Value::AttachmentsOn {
        what: Box::new(Selector::This),
        filter: aura_or_equipment().and(R::Not(Box::new(R::HasName("Animal Friend".into())))),
    };
    aura(
        "Animal Friend",
        cost(&[generic(1), g()]),
        EquipBonus {
            triggered_abilities: vec![on_attack(Effect::Seq(vec![
                make(squirrel, Value::ONE),
                Effect::AddCounter { what: Selector::LastCreatedTokens, kind: CounterType::PlusOnePlusOne, amount: others },
            ]))],
            ..Default::default()
        },
    )
}

/// Armory Paladin — trample; casting an Aura or Equipment spell exiles your
/// top card, playable until the end of your next turn.
pub fn armory_paladin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_cast_aura_or_equipment(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: false,
            uncast_penalty: None,
        })],
        ..creature("Armory Paladin", cost(&[generic(1), r(), w()]), vec![CreatureType::Human, CreatureType::Knight], 3, 3)
    }
}

/// Bighorner Rancher — vigilance; {T}: {G} per the greatest power among your
/// creatures; sacrifice: life equal to the greatest toughness among the
/// others.
pub fn bighorner_rancher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Green, Value::GreatestPowerControlled { who: PlayerRef::You }),
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ToughnessOf(Box::new(Selector::GreatestToughnessYouControl)),
                },
                ..Default::default()
            },
        ],
        ..creature("Bighorner Rancher", cost(&[generic(4), g()]), vec![CreatureType::Human, CreatureType::Ranger], 2, 5)
    }
}

/// Brass Knuckles — casting it copies it; double strike with two or more
/// Equipment attached.
pub fn brass_knuckles() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::CopySpell { what: Selector::This, count: Value::ONE },
        }],
        ..equipment(
            "Brass Knuckles",
            cost(&[generic(4)]),
            cost(&[generic(1)]),
            EquipBonus {
                conditional: vec![ConditionalEquipBonus {
                    host_filter: R::EquippedByAtLeast(2),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::DoubleStrike],
                    condition: None,
                    set_base_pt: None,
                    activated_abilities: vec![],
                }],
                ..Default::default()
            },
        )
    }
}

/// Brotherhood Outcast — entering: an Aura or Equipment card (mana value 3 or
/// less) back from your graveyard, or a shield counter on target creature.
/// Residual: the card is picked rather than targeted.
pub fn brotherhood_outcast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            put_attachment_from(vec![Zone::Graveyard], aura_or_equipment().and(R::ManaValueAtMost(3))),
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::Shield,
                amount: Value::ONE,
            },
        ]))],
        ..creature("Brotherhood Outcast", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 2)
    }
}

/// Cait, Cage Brawler — indestructible on your turn; attacking, you and the
/// defending player each loot, and the greater discard earns two +1/+1
/// counters.
pub fn cait_cage_brawler() -> CardDefinition {
    let defender = || Selector::Player(PlayerRef::DefendingPlayer);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, Cait has indestructible.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Indestructible }),
            },
        }],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            draw(1),
            Effect::Draw { who: defender(), amount: Value::ONE },
            discard(Selector::You),
            discard(defender()),
            Effect::If {
                cond: Predicate::YouDiscardedGreatestManaValueThisEffect,
                then: Box::new(plus(Selector::This, 2)),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..legend("Cait, Cage Brawler", cost(&[r(), g()]), vec![CreatureType::Human, CreatureType::Warrior], 1, 1)
    }
}

/// Cass, Hand of Vengeance — vigilance; when it or another creature of yours
/// dies enchanted or equipped, its Aura cards return attached to target
/// creature and its Equipment moves there.
pub fn cass_hand_of_vengeance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(died_with(aura_or_equipment())),
            effect: Effect::ReturnDyingSubjectAttachmentsTo { host: target_filtered(R::Creature) },
        }],
        ..legend("Cass, Hand of Vengeance", cost(&[generic(2), r(), w()]), vec![CreatureType::Human, CreatureType::Ranger], 4, 3)
    }
}

/// Codsworth, Handy Helper — your commanders have ward {2}; {T}: {W}{W} for
/// Aura and Equipment spells; {T}: move your Aura or Equipment to a creature
/// of yours (sorcery speed).
pub fn codsworth_handy_helper() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Commanders you control have ward {2}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::IsCommander),
                keyword: Keyword::Ward(WardCost::generic(2)),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::White, Color::White])),
                        SpendRestriction::AuraOrEquipmentSpells,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::Attach {
                    what: Selector::TargetFiltered { slot: 0, filter: aura_or_equipment().and(R::ControlledByYou) },
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) },
                },
                ..Default::default()
            },
        ],
        ..legend("Codsworth, Handy Helper", cost(&[generic(2), w()]), vec![CreatureType::Robot], 2, 3)
    }
}

/// Commander Sofia Daguerre — flash; entering, destroy up to one target
/// legendary permanent, whose controller creates a Junk token.
pub fn commander_sofia_daguerre() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Permanent.and(R::HasSupertype(Supertype::Legendary)),
            effect: Box::new(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    count: Value::ONE,
                    definition: Arc::new(junk_token()),
                },
                Effect::Destroy { what: Selector::Target(0) },
            ])),
        })],
        ..legend("Commander Sofia Daguerre", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Pilot], 1, 3)
    }
}

/// Crimson Caravaneer — double strike, trample; combat damage to a player
/// makes a Junk token.
pub fn crimson_caravaneer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: junk(1),
        }],
        ..creature("Crimson Caravaneer", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Scout], 1, 2)
    }
}

/// Dogmeat, Ever Loyal — entering, mill five and take back an Aura or
/// Equipment card; your enchanted or equipped creatures attacking make Junk.
pub fn dogmeat_ever_loyal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(5) },
                Effect::ReturnGraveyardCardsToHand { filter: aura_or_equipment(), max: Value::ONE },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(trigger_is(R::IsEnchanted.or(R::IsEquipped))),
                effect: junk(1),
            },
        ],
        ..legend("Dogmeat, Ever Loyal", cost(&[r(), g(), w()]), vec![CreatureType::Dog], 3, 3)
    }
}

/// Duchess, Wayward Tavernkeep — your creatures dealing combat damage to a
/// player get quest counters; {1}, remove a quest counter: a Junk token.
pub fn duchess_wayward_tavernkeep() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::TriggerSource, kind: CounterType::Quest, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_among_filter: Some((Some(CounterType::Quest), 1, R::Permanent.and(R::ControlledByYou))),
            effect: junk(1),
            ..Default::default()
        }],
        ..legend("Duchess, Wayward Tavernkeep", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Citizen], 4, 3)
    }
}

/// Grim Reaper's Sprint — morbid {3} cheaper; entering, untap your creatures
/// and (in your main phase) an extra combat; +2/+2 and haste.
pub fn grim_reapers_sprint() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Morbid — This spell costs {3} less to cast if a creature died this turn.",
            effect: StaticEffect::SelfCostReducedIfCreatureDiedThisTurn { amount: 3 },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Untap { what: yours(R::Creature), up_to: None },
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..aura(
            "Grim Reaper's Sprint",
            cost(&[generic(4), r()]),
            EquipBonus { power: 2, toughness: 2, keywords: vec![Keyword::Haste], ..Default::default() },
        )
    }
}

/// Gunner Conscript — trample; +1/+1 per Aura and Equipment on it; dying
/// enchanted makes a Junk token, and dying equipped makes another.
pub fn gunner_conscript() -> CardDefinition {
    let per = || Value::AttachmentsOn { what: Box::new(Selector::This), filter: aura_or_equipment() };
    let dies_with = |filter: R| TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(died_with(filter)),
        effect: junk(1),
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each Aura and Equipment attached to it.",
            effect: StaticEffect::PumpPTByValue { applies_to: Selector::This, power: per(), toughness: per() },
        }],
        triggered_abilities: vec![dies_with(aura_card()), dies_with(equipment_card())],
        ..creature("Gunner Conscript", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Mercenary], 2, 2)
    }
}

/// Ian the Reckless — attacking while modified, it may deal damage equal to
/// its power to you and any target.
pub fn ian_the_reckless() -> CardDefinition {
    let power = || Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::IsModified }),
            effect: Effect::MayDo {
                description: "Have Ian deal damage equal to its power to you and any target?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::You, amount: power() },
                    Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: power() },
                ])),
            },
        }],
        ..legend("Ian the Reckless", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 1)
    }
}

/// Idolized — attacking alone, enchanted creature gets +X/+X for each
/// nonland permanent you control.
pub fn idolized() -> CardDefinition {
    let x = || Value::PermanentCountControlledByMatching(PlayerRef::You, R::Nonland);
    aura(
        "Idolized",
        cost(&[generic(1), w()]),
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::AttackingAlone),
                effect: Effect::PumpPT { what: Selector::This, power: x(), toughness: x(), duration: Duration::EndOfTurn },
            }],
            ..Default::default()
        },
    )
}

/// Inventory Management — split second; attach your Auras and Equipment to
/// a creature you control.
/// Residual: everything chosen moves to your greatest-power creature.
pub fn inventory_management() -> CardDefinition {
    CardDefinition {
        name: "Inventory Management",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::SplitSecond],
        effect: Effect::AttachAnyNumberTo {
            what: yours(equipment_card().or(aura_card().and(R::AttachedToCreature))),
            to: Selector::GreatestPowerYouControl,
        },
        ..Default::default()
    }
}

/// Junk Jet — entering, a Junk token; {3}, sacrifice another artifact: double
/// the equipped creature's power.
pub fn junk_jet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(junk(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::DoublePower {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                times: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..equipment("Junk Jet", cost(&[generic(1), r()]), cost(&[generic(1)]), EquipBonus::default())
    }
}

/// Junktown — {T}: {C}; {4}{R}, {T}, sacrifice: three Junk tokens.
pub fn junktown() -> CardDefinition {
    land(
        "Junktown",
        ActivatedAbility {
            mana_cost: cost(&[generic(4), r()]),
            tap_cost: true,
            sac_cost: true,
            effect: junk(3),
            ..Default::default()
        },
    )
}

/// Megaton's Fate — destroy target artifact and make four Treasures, or 8
/// damage to each creature and four rad counters for each player.
pub fn megatons_fate() -> CardDefinition {
    CardDefinition {
        name: "Megaton's Fate",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                make(treasure_token(), Value::Const(4)),
            ]),
            Effect::Seq(vec![
                Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(8) },
                rad(PlayerRef::EachPlayer, 4),
            ]),
        ]),
        ..Default::default()
    }
}

/// Mister Gutsy — a +1/+1 counter per Aura or Equipment spell you cast;
/// dying, a Junk token per +1/+1 counter.
pub fn mister_gutsy() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![
            on_cast_aura_or_equipment(plus(Selector::This, 1)),
            on_dies(make(
                junk_token(),
                Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
            )),
        ],
        ..creature("Mister Gutsy", cost(&[generic(2)]), vec![CreatureType::Robot, CreatureType::Soldier], 1, 1)
    }
}

/// Moira Brown, Guide Author — entering, the Wasteland Survival Guide (a
/// Book Equipment: +1/+1 per quest counter among your permanents, equip
/// {1}); whenever you attack, a quest counter on target nonland permanent
/// you control.
pub fn moira_brown_guide_author() -> CardDefinition {
    let quests = || Value::CountersOn { what: Box::new(yours(R::Permanent)), kind: CounterType::Quest };
    let guide = TokenDefinition {
        name: "Wasteland Survival Guide".into(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book, ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        static_abilities: vec![StaticAbility {
            description: "Equipped creature gets +1/+1 for each quest counter among permanents you control.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: quests(),
                toughness: quests(),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(make(guide, Value::ONE)),
            on_you_attack(Effect::AddCounter {
                what: target_filtered(R::Nonland.and(R::ControlledByYou)),
                kind: CounterType::Quest,
                amount: Value::ONE,
            }),
        ],
        ..legend("Moira Brown, Guide Author", cost(&[generic(1), r(), w()]), vec![CreatureType::Human, CreatureType::Citizen], 2, 3)
    }
}

/// Perception Bobblehead — mana of any color; {3}, {T}: look at the top X
/// (X = your Bobbleheads) and cast one with mana value 3 or less free.
/// Residual: the rest go to the bottom in the cascade order.
pub fn perception_bobblehead() -> CardDefinition {
    bobblehead(
        "Perception Bobblehead",
        ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::RevealTopMayCastOneFree { count: bobbleheads(), max_mv: Value::Const(3), filter: None },
            ..Default::default()
        },
    )
}

/// Pip-Boy 3000 — equipped creature attacking: loot, a +1/+1 counter on it,
/// or untap up to two target lands.
pub fn pip_boy_3000() -> CardDefinition {
    equipment(
        "Pip-Boy 3000",
        cost(&[generic(1)]),
        cost(&[generic(2)]),
        EquipBonus {
            triggered_abilities: vec![on_attack(Effect::ChooseMode(vec![
                Effect::Seq(vec![draw(1), discard(Selector::You)]),
                plus(Selector::This, 1),
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Land,
                    effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
                },
            ]))],
            ..Default::default()
        },
    )
}

/// Pre-War Formalwear — entering, a creature card (mana value 3 or less)
/// back from your graveyard wearing it; +2/+2 and vigilance.
pub fn pre_war_formalwear() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::Attach { what: Selector::This, to: Selector::Target(0) },
        ]))],
        ..equipment(
            "Pre-War Formalwear",
            cost(&[generic(2), w()]),
            cost(&[generic(3)]),
            EquipBonus { power: 2, toughness: 2, keywords: vec![Keyword::Vigilance], ..Default::default() },
        )
    }
}

/// Preston Garvey, Minuteman — each combat on your turn, a Settlement Aura
/// token on up to one of your lands ("{T}: Add one mana of any color");
/// attacking, untap your enchanted permanents.
pub fn preston_garvey_minuteman() -> CardDefinition {
    let settlement = TokenDefinition {
        name: "Settlement".into(),
        card_types: vec![CardType::Enchantment],
        colors: vec![Color::Green],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        equipped_bonus: Some(EquipBonus { activated_abilities: vec![tap_add_any_color()], ..Default::default() }),
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Land.and(R::ControlledByYou),
                    effect: Box::new(Effect::CreateTokenAttachedTo {
                        target: Selector::Target(0),
                        definition: Arc::new(settlement),
                    }),
                },
            },
            on_attack(Effect::Untap { what: yours(R::IsEnchanted), up_to: None }),
        ],
        ..legend(
            "Preston Garvey, Minuteman",
            cost(&[generic(2), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Roadside Reliquary — {T}: {C}; {2}, {T}, sacrifice: a card if you control
/// an artifact, and a card if you control an enchantment.
pub fn roadside_reliquary() -> CardDefinition {
    let if_control = |filter: R| Effect::If {
        cond: Predicate::SelectorExists(yours(filter)),
        then: Box::new(draw(1)),
        else_: Box::new(Effect::Noop),
    };
    land(
        "Roadside Reliquary",
        ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![if_control(R::Artifact), if_control(R::Enchantment)]),
            ..Default::default()
        },
    )
}

/// Silver Shroud Costume — flash; entering, it attaches to target creature
/// you control, which gains shroud until end of turn; unblockable.
pub fn silver_shroud_costume() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Shroud, duration: Duration::EndOfTurn },
        ]))],
        ..equipment(
            "Silver Shroud Costume",
            cost(&[generic(2)]),
            cost(&[generic(3)]),
            EquipBonus { keywords: vec![Keyword::Unblockable], ..Default::default() },
        )
    }
}

/// Strong Back — equip abilities and Aura spells targeting the enchanted
/// creature cost {3} less; +2/+2 per Aura and Equipment on it.
pub fn strong_back() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Equip abilities you activate and Aura spells you cast that target enchanted creature cost {3} less.",
            effect: StaticEffect::CostReductionTargetingHost { amount: 3 },
        }],
        ..aura(
            "Strong Back",
            cost(&[generic(2), g()]),
            EquipBonus {
                scale: Some(EquipScale {
                    per_power: 2,
                    per_toughness: 2,
                    count_host_attachments: Some(aura_or_equipment()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
    }
}

/// Super Mutant Scavenger — trample; entering or dying, up to one target
/// Aura or Equipment card back to hand.
pub fn super_mutant_scavenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(regrow_attachment()), on_dies(regrow_attachment())],
        ..creature("Super Mutant Scavenger", cost(&[generic(4), g()]), vec![CreatureType::Mutant, CreatureType::Warrior], 5, 5)
    }
}

/// Three Dog, Galaxy News DJ — whenever you attack, you may pay {2} and
/// sacrifice an Aura attached to it; a copy of that Aura then goes onto each
/// other attacking creature of yours.
pub fn three_dog_galaxy_news_dj() -> CardDefinition {
    let on_me = || aura_card().and(R::AttachedToSource);
    CardDefinition {
        triggered_abilities: vec![on_you_attack(Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(on_me())),
            then: Box::new(Effect::MayPay {
                description: "Pay {2} and sacrifice an Aura attached to Three Dog?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: on_me() },
                    Effect::CreateTokenCopyOfAttachedToEach {
                        source: Selector::SacrificedThisResolution { filter: aura_card() },
                        hosts: yours(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
                    },
                ])),
                else_: None,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..legend("Three Dog, Galaxy News DJ", cost(&[generic(1), r(), w()]), vec![CreatureType::Human, CreatureType::Bard], 1, 5)
    }
}

/// Vault 101: Birthday Party — I: a 1/1 Human Soldier and a Food; II, III: an
/// Aura or Equipment card from your hand or graveyard onto the battlefield.
/// Residual: the card and its host are the engine's pick.
pub fn vault_101_birthday_party() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    let gift = || put_attachment_from(vec![Zone::Hand, Zone::Graveyard], aura_or_equipment());
    saga("Vault 101: Birthday Party", cost(&[generic(3), w()]), vec![
        (1, Effect::Seq(vec![make(soldier, Value::ONE), make(food_token(), Value::ONE)])),
        (2, gift()),
        (3, gift()),
    ])
}

/// Vault 21: House Gambit — I, II: discard, then draw; III: reveal up to five
/// nonland cards, a Treasure per card sharing a mana value with another.
pub fn vault_21_house_gambit() -> CardDefinition {
    let rummage = || Effect::Seq(vec![discard(Selector::You), draw(1)]);
    saga("Vault 21: House Gambit", cost(&[generic(1), r()]), vec![
        (1, rummage()),
        (2, rummage()),
        (3, Effect::TreasurePerPairedManaValueInHand { max: 5 }),
    ])
}

/// Veronica, Dissident Scribe — menace; attacking, you may discard to draw;
/// your first nonland discard each turn makes a Junk token.
pub fn veronica_dissident_scribe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            on_attack(Effect::If {
                cond: Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::ONE),
                then: Box::new(Effect::MayDo {
                    description: "Discard a card to draw a card?".into(),
                    body: Box::new(Effect::Seq(vec![discard(Selector::You), draw(1)])),
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl)
                    .with_filter(trigger_is(R::Nonland))
                    .once_per_turn(),
                effect: junk(1),
            },
        ],
        ..legend(
            "Veronica, Dissident Scribe",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Well Rested — enchanted creature untapping (once each turn): two +1/+1
/// counters, 2 life and a card.
pub fn well_rested() -> CardDefinition {
    aura(
        "Well Rested",
        cost(&[generic(1), g()]),
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource).once_per_turn(),
                effect: Effect::Seq(vec![
                    plus(Selector::This, 2),
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    draw(1),
                ]),
            }],
            ..Default::default()
        },
    )
}
