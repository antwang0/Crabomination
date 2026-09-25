//! Commander: the cards the **Mystic Intellect** precon (C19, Sevinne, the
//! Chronoclasm) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_sevinne.rs`.
//!
//! Residuals (each also on its card):
//! - **Wall of Stolen Identity** — the tap-and-lock happens as it enters,
//!   not as a reflexive "when you do" trigger, and the lock lasts while the
//!   Wall is on the battlefield rather than while you control it.
//! - **Mandate of Peace** — a triggered ability that had triggered but was
//!   not yet on the stack still goes on (CR 724.2a says it ceases to exist).
//! - **Elsha of the Infinite** — its flash also covers a top-of-library cast
//!   another permission allowed (Mystic Forge), not only its own.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EntersAsCopy, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, SelectionRequirement as R, Selector, SplitCard, SplitHalf, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{cast_is_noncreature, etb, on_attack, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, ManaCost, cost, generic, r, u, w, x};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, ty: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![ty], effect, ..Default::default() }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// "Whenever you cast a spell from your graveyard, `effect`."
fn on_graveyard_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
            Predicate::CastSpellFromGraveyard,
            Predicate::Not(Box::new(Predicate::CastSpellNotOwnedByYou)),
        ])),
        effect,
    }
}

/// 1/1 white Spirit creature token with flying.
fn white_spirit() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    }
}

/// Sevinne, the Chronoclasm — damage to it is prevented; the first instant or
/// sorcery you cast from your graveyard each turn is copied.
pub fn sevinne_the_chronoclasm() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to Sevinne.",
            effect: StaticEffect::PreventAllDamageToThis,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(instant_or_sorcery()),
                Predicate::FirstInstantOrSorceryCastFromGraveyardThisTurn,
                Predicate::Not(Box::new(Predicate::CastSpellNotOwnedByYou)),
            ])),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
        }],
        ..legendary(creature(
            "Sevinne, the Chronoclasm",
            cost(&[generic(2), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        ))
    }
}

/// Elsha of the Infinite — prowess; look at your top card; cast noncreature
/// spells from the top of your library, with flash.
///
/// ⚠ Residual: the flash also covers a top-of-library cast another permission
/// allowed, not only Elsha's own.
pub fn elsha_of_the_infinite() -> CardDefinition {
    let noncreature = R::HasCardType(CardType::Creature).negate().and(R::HasCardType(CardType::Land).negate());
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may cast noncreature spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: noncreature.clone() },
            },
            StaticAbility {
                description: "If you cast a spell this way, you may cast it as though it had flash.",
                effect: StaticEffect::ControllerSpellsHaveFlash { filter: noncreature.and(R::OnTopOfLibrary) },
            },
        ],
        ..legendary(creature(
            "Elsha of the Infinite",
            cost(&[generic(2), u(), r(), w()]),
            vec![CreatureType::Djinn, CreatureType::Monk],
            3,
            3,
        ))
    }
}

/// Backdraft Hellkite — flying; attacking gives each instant and sorcery in
/// your graveyard flashback at its mana cost until end of turn.
pub fn backdraft_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantMayPlay {
            what: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: instant_or_sorcery() },
            duration: MayPlayDuration::EndOfThisTurn,
            to_owner: false,
            exile_after: true,
            pay_own_cost: true,
            any_color: false,
        })],
        ..creature("Backdraft Hellkite", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Burning Vengeance — each spell cast from your graveyard deals 2 to any
/// target.
pub fn burning_vengeance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_graveyard_cast(Effect::DealDamage { to: target_any(), amount: Value::Const(2) })],
        ..spell("Burning Vengeance", cost(&[generic(2), r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Secrets of the Dead — each spell cast from your graveyard draws a card.
pub fn secrets_of_the_dead() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_graveyard_cast(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..spell("Secrets of the Dead", cost(&[generic(2), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Thalia's Geistcaller — lifelink; a 1/1 flying Spirit per spell cast from
/// your graveyard; sacrifice a Spirit: indestructible until end of turn.
pub fn thalias_geistcaller() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![on_graveyard_cast(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(white_spirit()),
        })],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Spirit), 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Thalia's Geistcaller",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            3,
            1,
        )
    }
}

/// Clever Impersonator — may enter as a copy of any nonland permanent.
pub fn clever_impersonator() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy { filter: R::Nonland, ..Default::default() }),
        ..creature("Clever Impersonator", cost(&[generic(2), u(), u()]), vec![CreatureType::Shapeshifter], 0, 0)
    }
}

/// Wall of Stolen Identity — enters as a copy of a creature that is also a
/// Wall with defender; the copied creature is tapped and stays tapped.
///
/// ⚠ Residual: the lock is installed as it enters (not a reflexive trigger)
/// and holds while the Wall is on the battlefield, not while you control it.
pub fn wall_of_stolen_identity() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_creature_types: vec![CreatureType::Wall],
            extra_keywords: vec![Keyword::Defender],
            lock_copied: true,
            ..Default::default()
        }),
        ..creature(
            "Wall of Stolen Identity",
            cost(&[generic(3), u()]),
            vec![CreatureType::Shapeshifter, CreatureType::Wall],
            0,
            0,
        )
    }
}

/// Devil's Play — X damage to any target; flashback {X}{R}{R}{R}.
pub fn devils_play() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[x(), r(), r(), r()]))],
        ..spell(
            "Devil's Play",
            cost(&[x(), r()]),
            CardType::Sorcery,
            Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
        )
    }
}

/// Dockside Extortionist — a Treasure per artifact and enchantment your
/// opponents control.
pub fn dockside_extortionist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CountOf(Box::new(Selector::EachPermanent(
                R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent),
            ))),
            definition: Arc::new(crate::game::effects::treasure_token()),
        })],
        ..creature(
            "Dockside Extortionist",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Pirate],
            1,
            2,
        )
    }
}

/// Gerrard, Weatherlight Hero — first strike; when it dies, exile it and
/// return every artifact and creature card put into your graveyard from the
/// battlefield this turn.
pub fn gerrard_weatherlight_hero() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::Exile { what: Selector::This },
            Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Artifact.or(R::Creature).and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]))],
        ..legendary(creature(
            "Gerrard, Weatherlight Hero",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        ))
    }
}

/// Jace's Sanctum — instants and sorceries cost {1} less; each one you cast
/// scries 1.
pub fn jaces_sanctum() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: instant_or_sorcery(), amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(instant_or_sorcery())),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..spell("Jace's Sanctum", cost(&[generic(3), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Mandate of Peace — combat only: opponents can't cast spells this turn, and
/// the combat phase ends (CR 724.2).
///
/// ⚠ Residual: a trigger waiting to go on the stack isn't removed (CR 724.2a).
pub fn mandate_of_peace() -> CardDefinition {
    CardDefinition {
        cast_only_during_combat: true,
        ..spell(
            "Mandate of Peace",
            cost(&[generic(1), w()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::SilencePlayersThisTurn { who: PlayerRef::EachOpponent },
                Effect::EndTheCombatPhase,
            ]),
        )
    }
}

/// Mass Diminish — creatures target player controls have base P/T 1/1 until
/// your next turn; flashback {3}{U}.
pub fn mass_diminish() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u()]))],
        ..spell(
            "Mass Diminish",
            cost(&[generic(1), u()]),
            CardType::Sorcery,
            Effect::SetBasePT {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::UntilNextTurn,
            },
        )
    }
}

/// Oona's Grace — target player draws a card; retrace.
pub fn oonas_grace() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Retrace],
        ..spell(
            "Oona's Grace",
            cost(&[generic(2), u()]),
            CardType::Instant,
            Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
        )
    }
}

/// Pramikon, Sky Rampart — flying, defender; each player may attack only the
/// nearest opponent in the direction chosen as it enters (CR 508.1a).
pub fn pramikon_sky_rampart() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Defender],
        as_enters_effect: Some(Effect::ChooseAttackDirection),
        static_abilities: vec![StaticAbility {
            description: "Each player may attack only the nearest opponent in the chosen direction.",
            effect: StaticEffect::AttackOnlyNearestOpponentInChosenDirection,
        }],
        ..legendary(creature("Pramikon, Sky Rampart", cost(&[u(), r(), w()]), vec![CreatureType::Wall], 1, 5))
    }
}

/// Pristine Skywise — flying; each noncreature spell you cast untaps it and
/// gives it protection from a color of your choice until end of turn.
pub fn pristine_skywise() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(cast_is_noncreature()),
            effect: Effect::Seq(vec![
                Effect::Untap { what: Selector::This, up_to: None },
                Effect::GrantProtectionFromChosenColor { what: Selector::This, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Pristine Skywise", cost(&[generic(4), w(), u()]), vec![CreatureType::Dragon], 6, 4)
    }
}

/// Purify the Grave — exile target card from a graveyard; flashback {W}.
pub fn purify_the_grave() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[w()]))],
        ..spell(
            "Purify the Grave",
            cost(&[w()]),
            CardType::Instant,
            Effect::Exile { what: target_filtered(R::InGraveyard) },
        )
    }
}

/// Rolling Temblor — 2 damage to each creature without flying; flashback
/// {4}{R}{R}.
pub fn rolling_temblor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), r(), r()]))],
        ..spell(
            "Rolling Temblor",
            cost(&[generic(2), r()]),
            CardType::Sorcery,
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying).negate())),
                amount: Value::Const(2),
            },
        )
    }
}

/// Runic Repetition — return target exiled card with flashback you own to
/// your hand.
pub fn runic_repetition() -> CardDefinition {
    spell(
        "Runic Repetition",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::InExile.and(R::HasFlashback).and(R::OwnedByYou)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Izzet Locket — taps for {U} or {R}; {U/R}×4, {T}, sacrifice: draw two.
pub fn izzet_locket() -> CardDefinition {
    crate::sets::rna::locket("Izzet Locket", Color::Blue, Color::Red)
}

/// Farm // Market — destroy target attacking or blocking creature // (aftermath)
/// draw two, then discard two.
pub fn farm_market() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(2), u()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
                ]),
            },
            fuse: false,
            aftermath: true,
        })),
        ..spell(
            "Farm // Market",
            cost(&[generic(2), w()]),
            CardType::Instant,
            Effect::Destroy { what: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))) },
        )
    }
}

/// Refuse // Cooperate — damage to target spell's controller equal to its
/// mana value // (aftermath) copy target instant or sorcery spell.
pub fn refuse_cooperate() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(2), u()]),
                card_types: vec![CardType::Instant],
                effect: Effect::CopySpellMayChooseTargets {
                    what: target_filtered(R::IsSpellOnStack.and(instant_or_sorcery())),
                    count: Value::ONE,
                },
            },
            fuse: false,
            aftermath: true,
        })),
        ..spell(
            "Refuse // Cooperate",
            cost(&[generic(3), r()]),
            CardType::Instant,
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TargetFiltered {
                    slot: 0,
                    filter: R::IsSpellOnStack,
                }))),
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
        )
    }
}
