//! Commander: 2026-10-18 (`019Y6J4R`) closers, seventeenth batch — each the
//! one missing card of an otherwise-buildable EDHREC average deck. Tests in
//! `tests/recent_b/cmdr_019y_q.rs`.

use crate::card::{
    BattleSubtype, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, SplitCard, SplitHalf, StaticAbility, Subtypes, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{choose_one_then, chosen_one, etb, target_filtered};
use crate::effect::{
    Duration, Effect, PlayerRef, RevealMissDest, Selector, StaticEffect, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
use std::sync::Arc;

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn yours(f: R) -> R {
    f.and(R::ControlledByYou)
}

/// Invasion of Xerex // Vertex Paladin — {2}{W}{U} battle — Siege, defense 4.
/// ETB: return up to one target creature to its owner's hand. Back: a flying
/// Angel Knight whose power and toughness are each the number of creatures
/// you control.
pub fn invasion_of_xerex() -> CardDefinition {
    let creatures = || Value::count(Selector::EachPermanent(yours(R::Creature)));
    let paladin = CardDefinition {
        name: "Vertex Paladin",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::White, Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel, CreatureType::Knight], ..Default::default() },
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Vertex Paladin's power and toughness are each equal to the number of creatures you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: creatures(), toughness: creatures() },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Xerex",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 4,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
        })],
        back_face: Some(Box::new(paladin)),
        ..Default::default()
    }
}

/// Driven // Despair — {1}{G} sorcery: until end of turn, creatures you control
/// gain trample and "whenever this creature deals combat damage to a player,
/// draw a card." Aftermath {1}{B} sorcery: menace and "… that player discards
/// a card."
pub fn driven_despair() -> CardDefinition {
    let team = || Selector::EachPermanent(yours(R::Creature));
    let grant = |kw: Keyword, body: Effect| {
        Effect::Seq(vec![
            Effect::GrantKeyword { what: team(), keyword: kw, duration: Duration::EndOfTurn },
            Effect::GrantTriggeredAbility {
                what: team(),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: body,
                }),
                duration: Duration::EndOfTurn,
            },
        ])
    };
    CardDefinition {
        name: "Driven // Despair",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: grant(Keyword::Trample, Effect::Draw { who: Selector::You, amount: Value::ONE }),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(1), b()]),
                card_types: vec![CardType::Sorcery],
                effect: grant(
                    Keyword::Menace,
                    Effect::Discard { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE, random: false },
                ),
            },
            fuse: false,
            aftermath: true,
        })),
        ..Default::default()
    }
}

/// Sunder the Gateway — {1}{W} sorcery. Choose one: destroy target nontoken
/// artifact or enchantment an opponent controls, then incubate 2; or incubate
/// 2, then transform an Incubator token you control.
pub fn sunder_the_gateway() -> CardDefinition {
    let incubate = || Effect::Incubate { who: PlayerRef::You, amount: Value::Const(2) };
    spell(
        "Sunder the Gateway",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(
                        R::Artifact.or(R::Enchantment).and(R::IsToken.negate()).and(R::ControlledByOpponent),
                    ),
                },
                incubate(),
            ]),
            Effect::Seq(vec![
                incubate(),
                choose_one_then(
                    Selector::EachPermanent(yours(R::IsToken.and(R::HasName("Incubator".into())))),
                    PlayerRef::You,
                    Effect::Transform { what: chosen_one() },
                ),
            ]),
        ]),
    )
}

/// Seize the Storm — {4}{R} sorcery. Create a red Elemental token with
/// trample whose power and toughness are each the number of instant and
/// sorcery cards in your graveyard plus the cards with flashback you own in
/// exile. Flashback {6}{R}.
pub fn seize_the_storm() -> CardDefinition {
    let size = || {
        Value::Sum(vec![
            Value::count(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            }),
            Value::count(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Exile, filter: R::HasFlashback }),
        ])
    };
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This token's power and toughness are each equal to the number of instant and sorcery cards in your graveyard plus the number of cards with flashback you own in exile.",
            effect: StaticEffect::SelfBasePtFromValue { power: size(), toughness: size() },
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(6), r()]))],
        ..spell(
            "Seize the Storm",
            cost(&[generic(4), r()]),
            CardType::Sorcery,
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(elemental) },
        )
    }
}

/// Telemin Performance — {3}{U}{U} sorcery. Target opponent reveals cards from
/// the top of their library until a creature card; the rest go to their
/// graveyard and you put the creature onto the battlefield under your control.
pub fn telemin_performance() -> CardDefinition {
    spell(
        "Telemin Performance",
        cost(&[generic(3), u(), u()]),
        CardType::Sorcery,
        Effect::RevealUntilFind {
            who: PlayerRef::Target(0),
            find: R::Creature,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            cap: Value::LibrarySizeOf(PlayerRef::Target(0)),
            life_per_revealed: 0,
            miss_dest: RevealMissDest::Graveyard,
        },
    )
}

/// Rift Elemental — {R} 1/1 Elemental. {1}{R}, remove a time counter from a
/// permanent you control or suspended card you own: +2/+0 until end of turn.
pub fn rift_elemental() -> CardDefinition {
    CardDefinition {
        name: "Rift Elemental",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![crate::effect::ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            remove_counter_among_filter: Some((Some(crate::card::CounterType::Time), 1, R::Permanent)),
            remove_counter_among_or_suspended: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
