//! Remaining single-faced STX (Strixhaven 2021) printed cards not covered by
//! the earlier extras batches. These ride existing primitives:
//! `Effect::CastWithoutPayingImmediate` (graveyard recursion), conditional
//! keyword grants (`Effect::If` + `Predicate::SelectorExists`), and
//! graveyard reanimation. Each ships with a functionality test in
//! `tests::stx`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    MayPlayDuration, Predicate, Selector, SelectionRequirement, Subtypes, TriggeredAbility, Value,
    Zone,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, r, w};

// ── Efreet Flamepainter ──────────────────────────────────────────────────────

/// Efreet Flamepainter — {3}{R}, 1/4 Efreet Shaman with Double strike.
/// Combat damage to a player: you may cast a target instant or sorcery from
/// your graveyard without paying its mana cost (exiled instead of buried).
pub fn efreet_flamepainter() -> CardDefinition {
    CardDefinition {
        name: "Efreet Flamepainter",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Efreet, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CastWithoutPayingImmediate {
                what: target_filtered(
                    SelectionRequirement::InGraveyard.and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
                ),
                source_zone: Zone::Graveyard,
                exile_after: true,
            },
        }],
        ..Default::default()
    }
}

// ── Thunderous Orator ────────────────────────────────────────────────────────

/// "If you control a creature with `kw`, this creature gains `kw` until end of
/// turn." The conditional keyword-share clause Thunderous Orator runs on attack.
fn orator_share(kw: Keyword) -> Effect {
    Effect::If {
        cond: Predicate::SelectorExists(Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::HasKeyword(kw.clone())),
        )),
        then: Box::new(Effect::GrantKeyword {
            what: Selector::This,
            keyword: kw,
            duration: Duration::EndOfTurn,
        }),
        else_: Box::new(Effect::Noop),
    }
}

/// Thunderous Orator — {1}{W}, 2/2 Kor Wizard with Vigilance.
/// On attack, it gains flying / first strike / double strike / deathtouch /
/// indestructible / lifelink / menace / trample for each of those keywords a
/// creature you control has.
pub fn thunderous_orator() -> CardDefinition {
    CardDefinition {
        name: "Thunderous Orator",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            orator_share(Keyword::Flying),
            orator_share(Keyword::FirstStrike),
            orator_share(Keyword::DoubleStrike),
            orator_share(Keyword::Deathtouch),
            orator_share(Keyword::Indestructible),
            orator_share(Keyword::Lifelink),
            orator_share(Keyword::Menace),
            orator_share(Keyword::Trample),
        ]))],
        ..Default::default()
    }
}

// ── Venerable Warsinger ──────────────────────────────────────────────────────

/// Venerable Warsinger — {1}{R}{W}, 3/3 Spirit Cleric with Vigilance, Trample.
/// Combat damage to a player: you may return a creature card with mana value 3
/// or less from your graveyard to the battlefield.
///
/// The printed X equals the damage dealt; with a base 3/3 (and no easy way to
/// read combat damage as a `Value` here) the mana-value gate is fixed at 3.
pub fn venerable_warsinger() -> CardDefinition {
    CardDefinition {
        name: "Venerable Warsinger",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Reanimate a creature (MV 3 or less)".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::InGraveyard
                            .and(SelectionRequirement::HasCardType(CardType::Creature))
                            .and(SelectionRequirement::ManaValueAtMost(3)),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Ardent Dustspeaker ───────────────────────────────────────────────────────

/// Ardent Dustspeaker — {4}{R}, 3/4 Minotaur Shaman.
/// On attack, exile the top two cards of your library; you may play them this
/// turn.
///
/// The printed "put an instant or sorcery from your graveyard on the bottom of
/// your library" enabler is dropped — the impulse draw fires unconditionally.
pub fn ardent_dustspeaker() -> CardDefinition {
    CardDefinition {
        name: "Ardent Dustspeaker",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(2),
            duration: MayPlayDuration::EndOfThisTurn, pay_any_color: false,
            uncast_penalty: None,
        })],
        ..Default::default()
    }
}
