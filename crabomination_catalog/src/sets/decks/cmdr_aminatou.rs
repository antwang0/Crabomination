//! Commander: the cards the **Subjective Reality** precon (C18, Aminatou,
//! the Fateshifter) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_aminatou.rs`.
//!
//! Residuals (each also on its card):
//! - **Aminatou's Augury** — the one free spell per card type is picked at
//!   resolution (greatest mana value first), not as each is cast.
//! - **Portent** — never has the player shuffle.
//! - **Primordial Mist** — exiling the face-down permanent is the ability's
//!   target, not its cost.
//! - **Sower of Discord** — the two players are the opponents with the least
//!   life (you and your only opponent at two seats).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, PlaneswalkerSubtype, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{cascade, etb, evoke, on_attack, on_cast, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, u, w, x, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, t: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![t], effect, ..Default::default() }
}

/// Aminatou, the Fateshifter — +1 draw and put one back; −1 blink one of
/// yours; −6 rotate every nonland permanent one seat.
pub fn aminatou_the_fateshifter() -> CardDefinition {
    CardDefinition {
        name: "Aminatou, the Fateshifter",
        cost: cost(&[w(), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Aminatou], ..Default::default() },
        base_loyalty: 3,
        can_be_commander: true,
        loyalty_abilities: vec![
            crate::card::LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::PutCardFromHandOnTopOfLibrary { who: Selector::You },
                ]),
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::ExileAndReturnToOwner {
                    what: target_filtered(R::Permanent.and(R::OwnedByYou).and(R::OtherThanSource)),
                },
                ..Default::default()
            },
            crate::card::LoyaltyAbility { loyalty_cost: -6, effect: Effect::RotateNonlandPermanents, ..Default::default() },
        ],
        ..Default::default()
    }
}

/// Aminatou's Augury — exile eight, maybe a land, then one free spell per
/// nonland card type. Residual: the picks are made at resolution.
pub fn aminatous_augury() -> CardDefinition {
    spell(
        "Aminatou's Augury",
        cost(&[generic(6), u(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ExileTopOfLibrary { who: Selector::You, amount: Value::Const(8), link_to_source: false, face_down: false },
            Effect::MayDo {
                description: "Put a land card from among them onto the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::ExiledThisResolution { filter: R::Land }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
            Effect::GrantFreeCastOnePerCardType { what: Selector::ExiledThisResolution { filter: R::Nonland } },
        ]),
    )
}

/// Banishing Stroke — an artifact, creature or enchantment to the bottom;
/// miracle {W}.
pub fn banishing_stroke() -> CardDefinition {
    CardDefinition {
        miracle: Some(cost(&[w()])),
        ..spell(
            "Banishing Stroke",
            cost(&[generic(5), w()]),
            CardType::Instant,
            Effect::Move {
                what: target_filtered(R::Artifact.or(R::Creature).or(R::Enchantment)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Bottom,
                },
            },
        )
    }
}

fn manifest_aura(name: &'static str, mana: ManaCost, keywords: [Keyword; 2]) -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ManifestTopAttachSource)],
        static_abilities: keywords
            .into_iter()
            .map(|k| StaticAbility {
                description: "Enchanted creature has this keyword.",
                effect: StaticEffect::GrantKeyword { applies_to: host(), keyword: k },
            })
            .collect(),
        ..Default::default()
    }
}

/// Cloudform — manifests and enchants: flying and hexproof.
pub fn cloudform() -> CardDefinition {
    manifest_aura("Cloudform", cost(&[generic(1), u(), u()]), [Keyword::Flying, Keyword::Hexproof])
}

/// Lightform — manifests and enchants: flying and lifelink.
pub fn lightform() -> CardDefinition {
    manifest_aura("Lightform", cost(&[generic(1), w(), w()]), [Keyword::Flying, Keyword::Lifelink])
}

/// Djinn of Wishes — flying; three wishes: play the top card free or lose it.
pub fn djinn_of_wishes() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::Wish, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            remove_counter_cost: Some((CounterType::Wish, 1)),
            effect: Effect::PlayTopFreeElseExile,
            ..Default::default()
        }],
        ..creature("Djinn of Wishes", cost(&[generic(3), u(), u()]), vec![CreatureType::Djinn], 4, 4)
    }
}

/// Enigma Sphinx — flying, cascade; dying puts it third from the top.
pub fn enigma_sphinx() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Cascade],
        triggered_abilities: vec![
            cascade(7),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::FromTop(2) },
                },
            },
        ],
        ..creature("Enigma Sphinx", cost(&[generic(4), w(), u(), b()]), vec![CreatureType::Sphinx], 5, 4)
    }
}

/// Entreat the Dead — X creature cards back; miracle {X}{B}{B}.
pub fn entreat_the_dead() -> CardDefinition {
    CardDefinition {
        miracle: Some(cost(&[x(), b(), b()])),
        ..spell(
            "Entreat the Dead",
            cost(&[x(), x(), b(), b(), b()]),
            CardType::Sorcery,
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 20,
                    filter: R::Creature.and(R::InYourGraveyard),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                }),
            },
        )
    }
}

/// Isolated Watchtower — {T}: {C}; behind on lands, {2}, {T}: scry into a
/// basic.
pub fn isolated_watchtower() -> CardDefinition {
    CardDefinition {
        name: "Isolated Watchtower",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                condition: Some(Predicate::OpponentControlsAtLeastMoreLands(2)),
                effect: Effect::Seq(vec![
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                            filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
                        },
                        then: Box::new(Effect::Move {
                            what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Jeskai Infiltrator — unblockable alone; connecting re-manifests it with
/// the top card.
pub fn jeskai_infiltrator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Can't be blocked as long as you control no other creatures.",
            effect: StaticEffect::GrantKeywordWhileControllerControlsAtMost {
                filter: R::IsSource,
                keyword: Keyword::Unblockable,
                count_filter: R::Creature.and(R::OtherThanSource),
                max: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ExileSourceAndTopThenManifest,
        }],
        ..creature("Jeskai Infiltrator", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Monk], 2, 3)
    }
}

/// Magus of the Balance — {4}{W}, {T}, sacrifice: Balance.
pub fn magus_of_the_balance() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::Balance,
            ..Default::default()
        }],
        ..creature("Magus of the Balance", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Night Incarnate — deathtouch; leaving shrinks every creature -3/-3;
/// evoke {3}{B}.
pub fn night_incarnate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        alternative_cost: Some(evoke(cost(&[generic(3), b()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Night Incarnate", cost(&[generic(4), b()]), vec![CreatureType::Elemental], 3, 4)
    }
}

/// Portent — order a player's top three; draw at the next upkeep. Residual:
/// never shuffles.
pub fn portent() -> CardDefinition {
    spell(
        "Portent",
        cost(&[u()]),
        CardType::Sorcery,
        Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(Effect::Seq(vec![
                Effect::RearrangeTop { who: PlayerRef::Target(0), amount: Value::Const(3) },
                Effect::AtNextTurnsUpkeep {
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ])),
        },
    )
}

/// Primordial Mist — manifest each end step; a face-down permanent may be
/// exiled to play it this turn. Residual: the exile is the ability's target.
pub fn primordial_mist() -> CardDefinition {
    CardDefinition {
        name: "Primordial Mist",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Manifest the top card of your library?".into(),
                body: Box::new(Effect::Manifest { who: PlayerRef::You, amount: Value::ONE }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::FaceDown.and(R::ControlledByYou)), to: ZoneDest::Exile },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Silent-Blade Oni — ninjutsu; connecting casts a spell from their hand.
pub fn silent_blade_oni() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(4), u(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LookAtHandCastFree { who: Selector::Player(PlayerRef::TriggerEventPlayer) },
        }],
        ..creature(
            "Silent-Blade Oni",
            cost(&[generic(3), u(), u(), b(), b()]),
            vec![CreatureType::Demon, CreatureType::Ninja],
            6,
            5,
        )
    }
}

/// Skull Storm — copied per commander cast; each opponent sacrifices a
/// creature or loses half their life.
pub fn skull_storm() -> CardDefinition {
    let body = Effect::ForEachOpponent {
        body: Box::new(Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CountOf(Box::new(Selector::ControlledBy { who: PlayerRef::Triggerer, filter: R::Creature })),
                Value::ONE,
            ),
            then: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Triggerer),
                count: Value::ONE,
                filter: R::Creature,
            }),
            else_: Box::new(Effect::LoseHalfLife { who: Selector::Player(PlayerRef::Triggerer), rounded_up: true }),
        }),
    };
    CardDefinition {
        triggered_abilities: vec![on_cast(Effect::CopySpell {
            what: Selector::This,
            count: Value::CommanderCastsFromCommandZone(PlayerRef::You),
        })],
        ..spell("Skull Storm", cost(&[generic(7), b(), b()]), CardType::Sorcery, body)
    }
}

/// Sower of Discord — flying; as it enters, choose two players; damage to
/// one drains the other.
pub fn sower_of_discord() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(Effect::ChooseTwoPlayersForSource),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::AnyPlayer),
            effect: Effect::OtherChosenPlayerLosesLife { amount: Value::TriggerEventAmount },
        }],
        ..creature("Sower of Discord", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 6, 6)
    }
}

/// Varina, Lich Queen — Zombie attacks loot and gain life; {2} and two
/// graveyard cards make a tapped Zombie.
pub fn varina_lich_queen() -> CardDefinition {
    let zombies = || {
        Value::CountOf(Box::new(Selector::EachPermanent(
            R::HasCreatureType(CreatureType::Zombie).and(R::IsAttacking).and(R::ControlledByYou),
        )))
    };
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(zombies(), Value::ONE)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: zombies() },
                Effect::Discard { who: Selector::You, amount: zombies(), random: false },
                Effect::GainLife { who: Selector::You, amount: zombies() },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            exile_other_filter: Some((R::InYourGraveyard, 2)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(zombie),
            },
            ..Default::default()
        }],
        ..creature(
            "Varina, Lich Queen",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Yennett, Cryptic Sovereign — flying, vigilance, menace; attacking casts
/// an odd top card free or draws.
pub fn yennett_cryptic_sovereign() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::CastTopFreeIfElseDraw { filter: R::ManaValueParity { odd: true } })],
        ..creature(
            "Yennett, Cryptic Sovereign",
            cost(&[generic(2), w(), u(), b()]),
            vec![CreatureType::Sphinx],
            3,
            5,
        )
    }
}
