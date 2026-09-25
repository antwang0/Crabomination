//! Commander: the cards the **Party Time** precon (CLB, Nalia de'Arnise)
//! needed beyond what the catalog had. Tests in `tests/recent_b/cmdr_nalia.rs`.
//!
//! Residuals (each also on its card):
//! - **Calculating Lich** — an attack on a planeswalker drains its
//!   controller too.
//! - **Glorious Protector** — the exiled creatures are the controller's pick
//!   through the choose-cards prompt (the bot keeps them all home).

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{dash, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, w};
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

fn treasure() -> Arc<TokenDefinition> {
    Arc::new(crabomination_base::tokens::treasure_token())
}

/// A Cleric, Rogue, Warrior or Wizard — the four party roles (CR 700.18).
fn party_role() -> R {
    R::HasCreatureType(CreatureType::Cleric)
        .or(R::HasCreatureType(CreatureType::Rogue))
        .or(R::HasCreatureType(CreatureType::Warrior))
        .or(R::HasCreatureType(CreatureType::Wizard))
}

fn full_party() -> Predicate {
    Predicate::ValueAtLeast(Value::PartyCount, Value::Const(4))
}

fn your_combat() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer)
}

fn you_control(t: CreatureType) -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(t)).and(R::ControlledByYou)),
        n: Value::ONE,
    }
}

/// Nalia de'Arnise — {1}{W}{B} 3/3 Human Rogue. Look at your top card any
/// time; cast party-role spells from the top; with a full party, your
/// combat puts a +1/+1 counter and deathtouch on each creature you control.
pub fn nalia_dearnise() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.".into(),
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may cast Cleric, Rogue, Warrior, and Wizard spells from the top of your library."
                    .into(),
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Creature.and(party_role()) },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: your_combat().with_filter(full_party()),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: team(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::GrantKeyword { what: team(), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature(
            "Nalia de'Arnise",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Archpriest of Iona — {W} */2 Human Cleric: power is your party's size;
/// with a full party, your combat gives a creature +1/+1 and flying.
pub fn archpriest_of_iona() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Its power is equal to the number of creatures in your party.".into(),
            effect: StaticEffect::SelfBasePtFromValue { power: Value::PartyCount, toughness: Value::Const(2) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_combat().with_filter(full_party()),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Archpriest of Iona", cost(&[w()]), vec![CreatureType::Human, CreatureType::Cleric], 0, 2)
    }
}

/// Burakos, Party Leader — {3}{B} 2/4 Orc, also each party role. Attacking,
/// the defending player loses X and you make X Treasures (X = your party).
/// Choose a Background.
pub fn burakos_party_leader() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        keywords: vec![Keyword::ChooseABackground],
        static_abilities: [CreatureType::Cleric, CreatureType::Rogue, CreatureType::Warrior, CreatureType::Wizard]
            .into_iter()
            .map(|creature_type| StaticAbility {
                description: "Burakos is also a Cleric, Rogue, Warrior, and Wizard.".into(),
                effect: StaticEffect::AddCreatureTypeToMatching { applies_to: Selector::This, creature_type },
            })
            .collect(),
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::LoseLife { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::PartyCount },
            Effect::CreateToken { who: PlayerRef::You, count: Value::PartyCount, definition: treasure() },
        ]))],
        ..creature(
            "Burakos, Party Leader",
            cost(&[generic(3), b()]),
            vec![CreatureType::Orc],
            2,
            4,
        )
    }
}

/// Calculating Lich — {4}{B}{B} 5/5 Zombie Wizard, menace. A creature
/// attacking one of your opponents drains that player 1.
/// Residual: an attack on a planeswalker drains its controller too.
pub fn calculating_lich() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::Player(PlayerRef::DefendingPlayer),
                filter: R::OpponentPlayer,
            }),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE },
        }],
        ..creature(
            "Calculating Lich",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            5,
            5,
        )
    }
}

/// Deep Gnome Terramancer — {1}{W} 2/2 Gnome Wizard, flash. Mold Earth: once
/// a turn, lands entering under an opponent's control without being played
/// (CR 305.1) let you fetch a Plains tapped.
pub fn deep_gnome_terramancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::OpponentControl)
                .with_filter(Predicate::ValueAtMost(Value::TriggerEventAmount, Value::Const(0)))
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "Search your library for a Plains card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasLandType(LandType::Plains),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            },
        }],
        ..creature(
            "Deep Gnome Terramancer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Gnome, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Firja's Retribution — {1}{W}{W}{B} Saga. I: a 4/4 flying, vigilant Angel
/// Warrior. II: Angels you control gain "{T}: destroy target creature with
/// lesser power" this turn. III: Angels you control gain double strike.
pub fn firjas_retribution() -> CardDefinition {
    let angels = || R::Creature.and(R::HasCreatureType(CreatureType::Angel)).and(R::ControlledByYou);
    let angel = TokenDefinition {
        name: "Angel Warrior".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel, CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..Default::default()
    };
    CardDefinition {
        name: "Firja's Retribution",
        cost: cost(&[generic(1), w(), w(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(angel) }),
            (
                2,
                Effect::GrantActivatedAbilityToMatching {
                    filter: angels(),
                    ability: Box::new(ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerLessThanSource)) },
                        ..Default::default()
                    }),
                    duration: Duration::EndOfTurn,
                },
            ),
            (
                3,
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(angels()),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..Default::default()
    }
}

/// Folk Hero — {1}{W} Background. Your commander creatures have "whenever
/// you cast a spell sharing a creature type with this, draw a card", once a
/// turn.
pub fn folk_hero() -> CardDefinition {
    let granted = TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::SharesCreatureTypeWithSource,
            })
            .once_per_turn(),
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
    };
    CardDefinition {
        name: "Folk Hero",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"Whenever you cast a spell that shares a creature \
                          type with this creature, draw a card. This ability triggers only once each turn.\""
                .into(),
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
                ability: Box::new(granted),
            },
        }],
        ..Default::default()
    }
}

/// Galepowder Mage — {3}{W} 3/3 Kithkin Wizard, flying. Attacking, it
/// flickers another creature until the next end step.
pub fn galepowder_mage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::ExileReturnToOwnerNextEndStep {
            what: target_filtered(R::Creature.and(R::OtherThanSource)),
            tapped: false,
        })],
        ..creature(
            "Galepowder Mage",
            cost(&[generic(3), w()]),
            vec![CreatureType::Kithkin, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Glorious Protector — {2}{W}{W} 3/4 Angel Cleric, flash, flying, foretell
/// {2}{W}. Entering, you may exile any number of your non-Angel creatures
/// until it leaves (CR 610.3).
/// Residual: the bot's pick is the choose-cards default.
pub fn glorious_protector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        foretell_cost: Some(cost(&[generic(2), w()])),
        triggered_abilities: vec![etb(Effect::ExileAnyNumberUntilSourceLeaves {
            filter: R::Creature
                .and(R::HasCreatureType(CreatureType::Angel).negate())
                .and(R::ControlledByYou)
                .and(R::OtherThanSource),
        })],
        ..creature(
            "Glorious Protector",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Cleric],
            3,
            4,
        )
    }
}

/// Grim Hireling — {3}{B} 3/2 Tiefling Rogue. Your creatures hitting a
/// player make two Treasures; {B}, sacrifice X Treasures: target creature
/// gets -X/-X (sorcery speed).
pub fn grim_hireling() -> CardDefinition {
    let minus_x = Value::Negate(Box::new(Value::XFromCost));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: treasure() },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Treasure), 0)),
            sac_other_x: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: minus_x.clone(),
                toughness: minus_x,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Grim Hireling",
            cost(&[generic(3), b()]),
            vec![CreatureType::Tiefling, CreatureType::Rogue],
            3,
            2,
        )
    }
}

/// Mage's Attendant — {2}{W} 3/2 Cat Rogue. Entering, a 1/1 blue Wizard
/// with "{1}, sacrifice this: counter target noncreature spell unless its
/// controller pays {1}."
pub fn mages_attendant() -> CardDefinition {
    let wizard = TokenDefinition {
        name: "Wizard".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)),
                mana_cost: cost(&[generic(1)]),
                exile: false,
                extra_generic: None,
                if_paid: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(wizard),
        })],
        ..creature("Mage's Attendant", cost(&[generic(2), w()]), vec![CreatureType::Cat, CreatureType::Rogue], 3, 2)
    }
}

/// Malakir Blood-Priest — {1}{B} 2/1 Vampire Cleric. Entering, each opponent
/// loses X and you gain X (X = your party).
pub fn malakir_blood_priest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::PartyCount },
            Effect::GainLife { who: Selector::You, amount: Value::PartyCount },
        ]))],
        ..creature(
            "Malakir Blood-Priest",
            cost(&[generic(1), b()]),
            vec![CreatureType::Vampire, CreatureType::Cleric],
            2,
            1,
        )
    }
}

/// Mardu Strike Leader — {2}{B} 3/2 Human Warrior. Attacking, a 2/1 black
/// Warrior. Dash {3}{B}.
pub fn mardu_strike_leader() -> CardDefinition {
    let warrior = TokenDefinition {
        name: "Warrior".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        alternative_cost: Some(dash(cost(&[generic(3), b()]))),
        triggered_abilities: vec![on_attack(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(warrior),
        })],
        ..creature(
            "Mardu Strike Leader",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            2,
        )
    }
}

/// Multiclass Baldric — {1} Equipment, equip {2}. The equipped creature has
/// lifelink / deathtouch / haste / flying while you control a Cleric / Rogue
/// / Warrior / Wizard; with a full party, all damage to it is prevented.
pub fn multiclass_baldric() -> CardDefinition {
    let grant = |t: CreatureType, keyword: Keyword| StaticAbility {
        description: "Equipped creature has a keyword while you control a party role.".into(),
        effect: StaticEffect::WhileCondition {
            condition: you_control(t),
            inner: Box::new(StaticEffect::GrantKeyword {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                keyword,
            }),
        },
    };
    CardDefinition {
        name: "Multiclass Baldric",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        static_abilities: vec![
            grant(CreatureType::Cleric, Keyword::Lifelink),
            grant(CreatureType::Rogue, Keyword::Deathtouch),
            grant(CreatureType::Warrior, Keyword::Haste),
            grant(CreatureType::Wizard, Keyword::Flying),
            StaticAbility {
                description: "As long as you have a full party, prevent all damage that would be dealt to \
                              equipped creature."
                    .into(),
                effect: StaticEffect::WhileCondition {
                    condition: full_party(),
                    inner: Box::new(StaticEffect::PreventAllDamageToEnchanted),
                },
            },
        ],
        equipped_bonus: Some(EquipBonus::default()),
        ..Default::default()
    }
}

/// Order of Whiteclay — {1}{W}{W} 1/4 Kithkin Cleric. {1}{W}{W}, {Q}: return
/// a creature card with mana value 3 or less from your graveyard to the
/// battlefield.
pub fn order_of_whiteclay() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w(), w()]),
            untap_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3)).and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Order of Whiteclay",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Kithkin, CreatureType::Cleric],
            1,
            4,
        )
    }
}

/// Seasoned Dungeoneer — {3}{W} 3/4 Human Warrior. Entering, you take the
/// initiative (CR 726); whenever you attack, an attacking party-role
/// creature gains protection from creatures and explores (CR 701.44).
pub fn seasoned_dungeoneer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::TakeInitiative { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature.and(R::IsAttacking).and(party_role())),
                        keyword: Keyword::ProtectionFromCreatures,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Explore { who: Selector::Target(0) },
                ]),
            },
        ],
        ..creature(
            "Seasoned Dungeoneer",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            4,
        )
    }
}

/// Snowfield Sinkhole — Snow Land — Plains Swamp; enters tapped.
pub fn snowfield_sinkhole() -> CardDefinition {
    CardDefinition {
        name: "Snowfield Sinkhole",
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Plains, LandType::Swamp], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add(Color::White), crate::sets::tap_add(Color::Black)],
        static_abilities: vec![crate::sets::enters_tapped()],
        ..Default::default()
    }
}

/// Solemn Doomguide — {3}{B}{B} 4/5 Tiefling Cleric, flying. Party-role
/// creature cards in your graveyard have unearth {1}{B} (CR 702.84).
pub fn solemn_doomguide() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Each Cleric, Rogue, Warrior, and/or Wizard creature card in your graveyard has unearth {1}{B}."
                .into(),
            effect: StaticEffect::GraveyardCardsHaveUnearth {
                filter: R::Creature.and(party_role()),
                cost: cost(&[generic(1), b()]),
            },
        }],
        ..creature(
            "Solemn Doomguide",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Tiefling, CreatureType::Cleric],
            4,
            5,
        )
    }
}

/// Thwart the Grave — {4}{B}{B} sorcery, {1} less per party member. Return a
/// creature card and up to one party-role creature card from your graveyard
/// to the battlefield.
pub fn thwart_the_grave() -> CardDefinition {
    let reanimate = |what: Selector| Effect::Move {
        what,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    CardDefinition {
        name: "Thwart the Grave",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each creature in your party.".into(),
            effect: StaticEffect::SelfCostReducedByValue { amount: Value::PartyCount },
        }],
        effect: Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                reanimate(target_filtered(R::Creature.and(R::InYourGraveyard))),
                reanimate(Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(party_role()).and(R::InYourGraveyard),
                }),
            ])),
        },
        ..Default::default()
    }
}

/// Valiant Changeling — {5}{W}{W} 3/3 Shapeshifter, changeling, double
/// strike; {1} less per creature type among your creatures (at most {5}).
pub fn valiant_changeling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each creature type among creatures you control."
                .into(),
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::Min(
                    Box::new(Value::DistinctCreatureTypesAmongYourCreatures),
                    Box::new(Value::Const(5)),
                ),
            },
        }],
        ..creature(
            "Valiant Changeling",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Shapeshifter],
            3,
            3,
        )
    }
}
