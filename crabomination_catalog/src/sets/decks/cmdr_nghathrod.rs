//! Commander: the cards the **Mind Flayarrrs** precon (Commander Legends:
//! Battle for Baldur's Gate, Captain N'ghathrod) needed beyond what the
//! catalog had. Tests in `tests/recent_b/cmdr_nghathrod.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes,
    TriggeredAbility, Value, Zone,
};
use crate::catalog::sets::{enters_tapped, tap_add};
use crate::effect::shortcut::{etb, target_filtered};
use crate::card::{EnchantmentSubtype, StaticAbility, Supertype};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, u};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        ..Default::default()
    }
}

/// Dusk Mangler — {5}{B}{B} Creature — Horror 5/4. As an additional cost to
/// cast this spell, sacrifice a creature, discard a card, or pay 4 life. When
/// this creature enters, each opponent sacrifices a creature of their choice,
/// discards a card, and loses 4 life.
pub fn dusk_mangler() -> CardDefinition {
    let each_opp = || Selector::Player(PlayerRef::EachOpponent);
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::OneOf(vec![
            AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 1 },
            AdditionalCastCost::Discard { count: 1, filter: None },
            AdditionalCastCost::PayLife { amount: 4 },
        ])],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Sacrifice { who: each_opp(), count: Value::ONE, filter: R::Creature },
            Effect::Discard { who: each_opp(), amount: Value::ONE, random: false },
            Effect::LoseLife { who: each_opp(), amount: Value::Const(4) },
        ]))],
        ..creature("Dusk Mangler", cost(&[generic(5), b(), b()]), vec![CreatureType::Horror], 5, 4)
    }
}

/// Port of Karfell — Land. This land enters tapped. {T}: Add {U}. {3}{U}{B}{B},
/// {T}, Sacrifice this land: Mill four cards, then return a creature card from
/// your graveyard to the battlefield tapped.
pub fn port_of_karfell() -> CardDefinition {
    CardDefinition {
        name: "Port of Karfell",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u(), b(), b()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::Const(4) },
                    // "…then return" — picked after the mill, so a milled
                    // creature qualifies.
                    Effect::MoveChosen {
                        from: Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Creature,
                        },
                        filter: None,
                        count: Value::ONE,
                        up_to: false,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Sewer Nemesis — {3}{B} Creature — Horror */*. As this creature enters,
/// choose a player. Its power and toughness are each equal to the number of
/// cards in the chosen player's graveyard. Whenever the chosen player casts a
/// spell, that player mills a card.
pub fn sewer_nemesis() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChoosePlayerForSource { opponent: false }),
        dynamic_pt: Some(crate::card::DynamicPt::ChosenPlayerGraveyardMatching {
            base_p: 0,
            base_t: 0,
            filter: R::Any,
            scales_toughness: true,
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::SamePlayer(PlayerRef::Triggerer, PlayerRef::ChosenPlayerOfSource),
            ),
            effect: Effect::Mill { who: Selector::Player(PlayerRef::Triggerer), amount: Value::ONE },
        }],
        ..creature("Sewer Nemesis", cost(&[generic(3), b()]), vec![CreatureType::Horror], 0, 0)
    }
}

/// Memory Plunder — {U/B}{U/B}{U/B}{U/B} Instant. You may cast target instant
/// or sorcery card from an opponent's graveyard without paying its mana cost.
pub fn memory_plunder() -> CardDefinition {
    let ub = || hybrid(Color::Blue, Color::Black);
    CardDefinition {
        name: "Memory Plunder",
        cost: cost(&[ub(), ub(), ub(), ub()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CastWithoutPayingImmediate {
            what: target_filtered(
                R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::InOpponentGraveyard),
            ),
            source_zone: Zone::Graveyard,
            exile_after: false,
            copy: false,
            reduce_generic: 0,
            pay_own_cost: false,
        },
        ..Default::default()
    }
}

/// Nihilith — {4}{B}{B} Creature — Horror 4/4. Fear. Suspend 7—{1}{B}.
/// Whenever a card is put into an opponent's graveyard from anywhere, if this
/// card is suspended, you may remove a time counter from this card.
pub fn nihilith() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear, Keyword::Suspend(7, cost(&[generic(1), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::OpponentControl)
                .while_suspended(),
            effect: Effect::MayDo {
                description: "Remove a time counter from Nihilith?".into(),
                body: Box::new(Effect::RemoveTimeCounterFromSuspendedSource),
            },
        }],
        ..creature("Nihilith", cost(&[generic(4), b(), b()]), vec![CreatureType::Horror], 4, 4)
    }
}

/// From the Catacombs — {3}{B}{B} Sorcery. Put target creature card from a
/// graveyard onto the battlefield under your control with a corpse counter on
/// it. You take the initiative. If that creature would leave the battlefield,
/// exile it instead of putting it anywhere else. Escape—{3}{B}{B}, Exile five
/// other cards from your graveyard.
///
/// The counter is put on as it lands rather than as it enters, so an
/// "enters with a counter" watcher wouldn't see it (none is in the pod).
pub fn from_the_catacombs() -> CardDefinition {
    CardDefinition {
        name: "From the Catacombs",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Escape(cost(&[generic(3), b(), b()]), 5)],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::Corpse,
                amount: Value::ONE,
            },
            Effect::TakeInitiative { who: PlayerRef::You },
            Effect::ExileIfLeavesBattlefield { what: Selector::LastMoved },
        ]),
        ..Default::default()
    }
}

/// Haunted One — {2}{B} Legendary Enchantment — Background. Commander creatures
/// you own have "Whenever this creature becomes tapped, it and other creatures
/// you control that share a creature type with it each get +2/+0 and gain
/// undying until end of turn."
pub fn haunted_one() -> CardDefinition {
    let crew = || {
        Selector::Both(
            Box::new(Selector::This),
            Box::new(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Creature.and(R::SharesCreatureTypeWithSource),
            }),
        )
    };
    let granted = TriggeredAbility {
        event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: crew(),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: crew(),
                keyword: Keyword::Undying,
                duration: Duration::EndOfTurn,
            },
        ]),
    };
    CardDefinition {
        name: "Haunted One",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Background],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"Whenever this creature becomes \
                          tapped, it and other creatures you control that share a creature type \
                          with it each get +2/+0 and gain undying until end of turn.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
                ability: Box::new(granted),
            },
        }],
        ..Default::default()
    }
}
