//! Commander: the cards the **Entropic Uprising** precon (C16, Yidris,
//! Maelstrom Wielder) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_yidris.rs`.
//!
//! Residuals (each also on its card):
//! - **Aeon Chronicler** — no Suspend X: the engine's suspend takes no X and
//!   bots never suspend, so its time-counter draw never comes up.
//! - **Vial Smasher the Fierce** — the random opponent is always dealt the
//!   damage, never one of their planeswalkers.
//! - **Blood Tyrant** — grows by the number of living players, not the life
//!   actually lost (a "can't lose life" effect isn't counted out).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, DynamicPt, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn basic_landcycling() -> Keyword {
    Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

fn wheel(who: PlayerRef) -> Effect {
    Effect::EachPlayerDoes {
        who,
        body: Box::new(Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
            Effect::Draw { who: Selector::You, amount: Value::Const(7) },
        ])),
    }
}

/// Yidris, Maelstrom Wielder — trample; connecting gives the spells you cast
/// from your hand this turn cascade.
pub fn yidris_maelstrom_wielder() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::OnEachSpellCastThisTurn {
                body: Box::new(Effect::If {
                    // The triggering spell's own origin; `Predicate::CastFromHand`
                    // reads the resolving trigger's context instead.
                    cond: Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Not(Box::new(R::SpellNotCastFromHand)),
                    },
                    then: Box::new(Effect::Cascade { max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)) }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        }],
        ..creature(
            "Yidris, Maelstrom Wielder",
            cost(&[u(), b(), r(), g()]),
            vec![CreatureType::Ogre, CreatureType::Wizard],
            5,
            4,
        )
    }
}

/// Aeon Chronicler — power and toughness are the cards in your hand (no
/// Suspend X: see the module residuals).
pub fn aeon_chronicler() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ControllerHandSize),
        ..creature("Aeon Chronicler", cost(&[generic(3), u(), u()]), vec![CreatureType::Avatar], 0, 0)
    }
}

/// Blood Tyrant — flying, trample; each upkeep everyone loses 1 and it grows
/// by the life lost; a player losing the game gives it five counters.
pub fn blood_tyrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![
            upkeep(Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    // One per player who lost the life — every living player
                    // (a life-loss lock isn't counted out).
                    amount: Value::CountOf(Box::new(Selector::Player(PlayerRef::EachPlayer))),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerLeftGame, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(5),
                },
            },
        ],
        ..creature("Blood Tyrant", cost(&[generic(4), u(), b(), r()]), vec![CreatureType::Vampire], 5, 5)
    }
}

/// Cruel Entertainment — two target players each control the other's next
/// turn (CR 723.1).
pub fn cruel_entertainment() -> CardDefinition {
    spell(
        "Cruel Entertainment",
        cost(&[generic(6), b()]),
        CardType::Sorcery,
        // Both slots declare their printed filter ("two target players"):
        // the controller of a player target is that player.
        Effect::PlayersControlEachOthersNextTurn {
            first: PlayerRef::ControllerOf(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: R::Player,
            })),
            second: PlayerRef::ControllerOf(Box::new(Selector::TargetFiltered {
                slot: 1,
                filter: R::Player,
            })),
        },
    )
}

/// Devastation Tide — every nonland permanent to its owner's hand; miracle
/// {1}{U}.
pub fn devastation_tide() -> CardDefinition {
    CardDefinition {
        miracle: Some(cost(&[generic(1), u()])),
        ..spell(
            "Devastation Tide",
            cost(&[generic(3), u(), u()]),
            CardType::Sorcery,
            Effect::Move { what: Selector::EachPermanent(R::Nonland), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
        )
    }
}

/// Frenzied Fugue — enchant permanent; entering and at your upkeep, take it
/// until end of turn, untapped and hasty.
pub fn frenzied_fugue() -> CardDefinition {
    let it = || Selector::AttachedTo(Box::new(Selector::This));
    let borrow = || {
        Effect::Seq(vec![
            Effect::GainControl { what: it(), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: it(), up_to: None },
            Effect::GrantKeyword { what: it(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
        ])
    };
    CardDefinition {
        name: "Frenzied Fugue",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Permanent) },
        equipped_bonus: Some(EquipBonus::default()),
        triggered_abilities: vec![etb(borrow()), upkeep(borrow())],
        ..Default::default()
    }
}

/// Ghastly Conscription — manifest every creature card in target player's
/// graveyard under your control (CR 701.34).
pub fn ghastly_conscription() -> CardDefinition {
    spell(
        "Ghastly Conscription",
        cost(&[generic(5), b(), b()]),
        CardType::Sorcery,
        Effect::ManifestFromGraveyard { who: PlayerRef::Target(0), filter: R::Creature },
    )
}

/// Goblin Spymaster — first strike; at each opponent's end step, that player
/// gets a 1/1 Goblin whose "creatures you control attack each combat if
/// able" binds their board.
pub fn goblin_spymaster() -> CardDefinition {
    let goblin = TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Creatures you control attack each combat if able.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::MustAttack,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::OpponentControl),
            effect: Effect::CreateToken { who: PlayerRef::ActivePlayer, count: Value::ONE, definition: Arc::new(goblin) },
        }],
        ..creature("Goblin Spymaster", cost(&[generic(2), r()]), vec![CreatureType::Goblin, CreatureType::Rogue], 2, 1)
    }
}

/// Kydele, Chosen of Kruphix — partner; {T}: {C} for each card you've drawn
/// this turn.
pub fn kydele_chosen_of_kruphix() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Partner],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::CardsDrawnThisTurn(PlayerRef::You)),
            },
            ..Default::default()
        }],
        ..creature(
            "Kydele, Chosen of Kruphix",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Nath of the Gilt-Leaf — at your upkeep, a target opponent may be made to
/// discard at random; an opponent discarding may make you an Elf Warrior.
pub fn nath_of_the_gilt_leaf() -> CardDefinition {
    let elf = TokenDefinition {
        name: "Elf Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            upkeep(Effect::MayDo {
                description: "Have target opponent discard a card at random?".into(),
                body: Box::new(Effect::Discard {
                    who: target_filtered(R::Player.and(R::ControlledByOpponent)),
                    amount: Value::ONE,
                    random: true,
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
                effect: Effect::MayDo {
                    description: "Create a 1/1 Elf Warrior?".into(),
                    body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(elf) }),
                },
            },
        ],
        ..creature(
            "Nath of the Gilt-Leaf",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Runehorn Hellkite — flying; {5}{R}, exile it from your graveyard: every
/// player discards their hand and draws seven.
pub fn runehorn_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), r()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: wheel(PlayerRef::EachPlayer),
            ..Default::default()
        }],
        ..creature("Runehorn Hellkite", cost(&[generic(5), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Spelltwine — exile an instant or sorcery card from your graveyard and one
/// from an opponent's; cast copies of both free. Exile Spelltwine.
pub fn spelltwine() -> CardDefinition {
    let copy = |slot: u8| Effect::CastWithoutPayingImmediate {
        what: Selector::Target(slot),
        source_zone: Zone::Exile,
        exile_after: false,
        copy: true,
        reduce_generic: 0,
        pay_own_cost: false,
    };
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Spelltwine",
            cost(&[generic(5), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move { what: target_filtered(instant_or_sorcery().from_your_graveyard()), to: ZoneDest::Exile },
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: instant_or_sorcery().from_any_graveyard().and(R::Not(Box::new(R::OwnedByYou))),
                    },
                    to: ZoneDest::Exile,
                },
                copy(0),
                copy(1),
            ]),
        )
    }
}

/// Thrasios, Triton Hero — partner; {4}: scry 1, then a land on top goes onto
/// the battlefield tapped, anything else is drawn.
pub fn thrasios_triton_hero() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Partner],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::EntityMatches { what: top(), filter: R::Land },
                    then: Box::new(Effect::Move {
                        what: top(),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    }),
                    else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Thrasios, Triton Hero",
            cost(&[g(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Treacherous Terrain — each opponent takes damage equal to the lands they
/// control; basic landcycling {2}.
pub fn treacherous_terrain() -> CardDefinition {
    CardDefinition {
        keywords: vec![basic_landcycling()],
        ..spell(
            "Treacherous Terrain",
            cost(&[generic(6), r(), g()]),
            CardType::Sorcery,
            Effect::ForEachOpponent {
                body: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::CountOf(Box::new(Selector::ControlledBy {
                        who: PlayerRef::Triggerer,
                        filter: R::Land,
                    })),
                }),
            },
        )
    }
}

/// Vial Smasher the Fierce — partner; your first spell each turn deals its
/// mana value to a random opponent (never their planeswalker).
pub fn vial_smasher_the_fierce() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::ONE }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::RandomOpponent),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..creature(
            "Vial Smasher the Fierce",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Goblin, CreatureType::Berserker],
            2,
            3,
        )
    }
}

/// Volcanic Vision — an instant or sorcery back to hand, and its mana value
/// in damage to each creature your opponents control. Exile it.
pub fn volcanic_vision() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Volcanic Vision",
            cost(&[generic(5), r(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                },
                Effect::Move {
                    what: target_filtered(instant_or_sorcery().from_your_graveyard()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
        )
    }
}

/// Worm Harvest — a 1/1 Worm for each land card in your graveyard; retrace.
pub fn worm_harvest() -> CardDefinition {
    let bg = || hybrid(Color::Black, Color::Green);
    CardDefinition {
        keywords: vec![Keyword::Retrace],
        ..spell(
            "Worm Harvest",
            cost(&[generic(2), bg(), bg(), bg()]),
            CardType::Sorcery,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Land,
                })),
                definition: Arc::new(TokenDefinition {
                    name: "Worm".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black, Color::Green],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Worm], ..Default::default() },
                    ..Default::default()
                }),
            },
        )
    }
}
