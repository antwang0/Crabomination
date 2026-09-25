//! Commander: the cards the **Quick Draw** precon (OTC, Stella Lee, Wild
//! Card) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_otc.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, DynamicPt,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{cascade, magecraft, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneRef};
use crate::mana::{Color, SpendRestriction, cost, generic, r, u};
use std::sync::Arc;

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

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

fn sorcery(name: &'static str, mana: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

/// "Whenever you cast your second spell each turn" — the cast that brings
/// the turn's count to exactly two.
fn on_second_spell(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(2) },
        ),
        effect,
    }
}

/// A 4/4 red Dragon Elemental with flying and prowess (Eris, Elemental
/// Eruption).
fn dragon_elemental() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Dragon Elemental".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Elemental],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying, Keyword::Prowess],
        ..Default::default()
    })
}

fn make_dragon_elemental() -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: dragon_elemental() }
}

/// Stella Lee, Wild Card — the second spell each turn exiles the top card to
/// play through your next turn; with three spells cast, tap to copy one.
pub fn stella_lee_wild_card() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_second_spell(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::SpellsCastThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(3),
            }),
            effect: Effect::CopySpellMayChooseTargets {
                what: target_filtered(
                    R::IsSpellOnStack.and(instant_or_sorcery()).and(R::ControlledByYou),
                ),
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Stella Lee, Wild Card",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            4,
        )
    }
}

/// Eris, Roar of the Storm — {2} cheaper per distinct mana value among
/// instants and sorceries in your graveyard; the second spell each turn
/// makes a 4/4 Dragon Elemental.
pub fn eris_roar_of_the_storm() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Prowess],
        self_cost_reduction_per: Some((
            Value::DistinctManaValuesInGraveyardMatching {
                who: PlayerRef::You,
                filter: instant_or_sorcery(),
            },
            2,
        )),
        triggered_abilities: vec![on_second_spell(make_dragon_elemental())],
        ..creature(
            "Eris, Roar of the Storm",
            cost(&[generic(8), u(), r()]),
            vec![CreatureType::Elemental, CreatureType::Warlock],
            4,
            4,
        )
    }
}

/// Octavia, Living Thesis — {8} cheaper with eight instants and sorceries in
/// your graveyard; ward {8}; magecraft makes a creature a base 8/8.
pub fn octavia_living_thesis() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Ward(WardCost::generic(8))],
        self_cost_reduction_if: Some((
            Predicate::ValueAtLeast(
                Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: instant_or_sorcery() },
                Value::Const(8),
            ),
            8,
        )),
        triggered_abilities: vec![magecraft(Effect::SetBasePT {
            what: target_filtered(R::Creature),
            power: Value::Const(8),
            toughness: Value::Const(8),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Octavia, Living Thesis",
            cost(&[generic(8), u(), u()]),
            vec![CreatureType::Elemental, CreatureType::Octopus],
            8,
            8,
        )
    }
}

/// Kaza, Roil Chaser — tap: the next instant or sorcery this turn costs {1}
/// less per Wizard you control.
pub fn kaza_roil_chaser() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantNextInstantOrSorceryDiscountThisTurn {
                amount: Value::count(Selector::EachPermanent(
                    R::ControlledByYou.and(R::HasCreatureType(CreatureType::Wizard)),
                )),
            },
            ..Default::default()
        }],
        ..creature(
            "Kaza, Roil Chaser",
            cost(&[u(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Thunderclap Drake — instants and sorceries cost {1} less; sacrifice it to
/// copy the next one per command-zone cast of your commander.
pub fn thunderclap_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: instant_or_sorcery(), amount: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            sac_cost: true,
            effect: Effect::OnYourNextInstantSorceryThisTurn {
                body: Box::new(Effect::CopySpellMayChooseTargets {
                    what: Selector::TriggerSource,
                    count: Value::CommanderCastsFromCommandZone(PlayerRef::You),
                }),
            },
            ..Default::default()
        }],
        ..creature("Thunderclap Drake", cost(&[generic(1), u()]), vec![CreatureType::Drake], 2, 1)
    }
}

/// Crackling Spellslinger — flash; cast, it gives the next instant or
/// sorcery this turn storm. ⚠ The storm count is read as the copy trigger
/// resolves, not as the spell is cast (they differ only if spells are cast in
/// response to that trigger).
pub fn crackling_spellslinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.4 — "if you cast it" is an intervening if.
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceWasCast),
            effect: Effect::OnYourNextInstantSorceryThisTurn {
                // CR 702.40a — a copy for each spell cast before it this turn.
                body: Box::new(Effect::CopySpellMayChooseTargets {
                    what: Selector::TriggerSource,
                    count: Value::Diff(
                        Box::new(Value::SpellsCastThisTurnTotal),
                        Box::new(Value::ONE),
                    ),
                }),
            },
        }],
        ..creature(
            "Crackling Spellslinger",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Elemental Eruption — a 4/4 Dragon Elemental, with storm.
pub fn elemental_eruption() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..sorcery("Elemental Eruption", cost(&[generic(4), r(), r()]), make_dragon_elemental())
    }
}

/// Volcanic Torrent — cascade; X damage to each creature and planeswalker
/// your opponents control, X = spells you've cast this turn.
pub fn volcanic_torrent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cascade],
        triggered_abilities: vec![cascade(5)],
        ..sorcery(
            "Volcanic Torrent",
            cost(&[generic(4), r()]),
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
                ),
                amount: Value::SpellsCastThisTurn(PlayerRef::You),
            },
        )
    }
}

/// Lock and Load — draw one, plus one per other instant and sorcery you've
/// cast this turn; plot {3}{U}.
pub fn lock_and_load() -> CardDefinition {
    CardDefinition {
        plot_cost: Some(cost(&[generic(3), u()])),
        ..sorcery(
            "Lock and Load",
            cost(&[generic(2), u()]),
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Diff(
                        Box::new(Value::InstantsOrSorceriesCastThisTurn(PlayerRef::You)),
                        Box::new(Value::ONE),
                    ),
                },
            ]),
        )
    }
}

/// Pyretic Charge — discard your hand, draw four, +1/+0 to your creatures
/// per card discarded; plot {3}{R}.
pub fn pyretic_charge() -> CardDefinition {
    CardDefinition {
        plot_cost: Some(cost(&[generic(3), r()])),
        ..sorcery(
            "Pyretic Charge",
            cost(&[generic(4), r()]),
            Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(4) },
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::CardsDiscardedThisEffect,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Rousing Refrain — {R} per card in target opponent's hand, kept until end
/// of turn; then it suspends itself again. Suspend 3—{1}{R}.
pub fn rousing_refrain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(3, cost(&[generic(1), r()]))],
        ..sorcery(
            "Rousing Refrain",
            cost(&[generic(3), r(), r()]),
            Effect::Seq(vec![
                Effect::AddManaKeptThisTurnCount {
                    who: PlayerRef::You,
                    color: Color::Red,
                    amount: Value::HandSizeOf(PlayerRef::Target(0)),
                },
                Effect::ExileSelfSuspended,
            ]),
        )
    }
}

/// Smoldering Stagecoach — power equal to your instants and sorceries in the
/// graveyard; attacking gives the next instant and the next sorcery this
/// turn cascade. Crew 2.
pub fn smoldering_stagecoach() -> CardDefinition {
    let cascade_next = |t: CardType| Effect::OnYourNextSpellOfTypeThisTurn {
        card_type: t,
        // The next-cast dispatch carries the cast spell's mana value as the
        // event amount (CR 702.85a — "less than this spell's mana value").
        body: Box::new(Effect::Cascade { max_mv: Value::TriggerEventAmount }),
    };
    CardDefinition {
        name: "Smoldering Stagecoach",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 0,
        toughness: 5,
        dynamic_pt: Some(DynamicPt::InstantsSorceriesInControllerGraveyard { base_t: 5 }),
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![cascade_next(CardType::Instant), cascade_next(CardType::Sorcery)]),
        }],
        ..Default::default()
    }
}

/// Leyline Dowser — {1}, tap: mill one, keep it if it's an instant or
/// sorcery; tap an untapped legendary creature you control to untap it.
pub fn leyline_dowser() -> CardDefinition {
    CardDefinition {
        name: "Leyline Dowser",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::MillThenToHandN {
                    amount: Value::ONE,
                    filter: instant_or_sorcery(),
                    take: Value::ONE,
                    otherwise: None,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_others_cost: Some((
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasSupertype(Supertype::Legendary)),
                    1,
                )),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Forger's Foundry — {T}: {U} that exiles a cheap instant or sorcery it
/// funds (`SpendRestriction::SmallInstantSorceryExileInstead`); {3}{U}{U},
/// {T}: cast any number of the cards exiled with it free, as a sorcery.
pub fn forgers_foundry() -> CardDefinition {
    CardDefinition {
        name: "Forger's Foundry",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Blue])),
                        SpendRestriction::SmallInstantSorceryExileInstead,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), u(), u()]),
                sorcery_speed: true,
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
        ..Default::default()
    }
}

/// Arcane Bombardment — the first instant or sorcery each turn exiles one at
/// random from your graveyard with it, then offers a free copy of every card
/// exiled with it.
pub fn arcane_bombardment() -> CardDefinition {
    CardDefinition {
        name: "Arcane Bombardment",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: instant_or_sorcery(),
                })
                .once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::ExileLinked {
                    what: Selector::RandomOf(Box::new(Selector::EachMatching {
                        zone: ZoneRef::Graveyard(PlayerRef::You),
                        filter: instant_or_sorcery(),
                    })),
                },
                // CR 707.12 — each copy is cast; the exiled cards stay put.
                Effect::ForEach {
                    selector: Selector::CardExiledWithSource,
                    body: Box::new(Effect::CastWithoutPayingImmediate {
                        what: Selector::TriggerSource,
                        source_zone: Zone::Exile,
                        exile_after: false,
                        copy: true,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}
