//! Tarkir: Dragonstorm (TDM) — assorted non-Omen cards. The set's Omen Dragon
//! cycle lives in `decks::omen`; this module collects the straightforward
//! spells/creatures that ride existing primitives. Tracked in `DECK_FEATURES.md`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement, Selector, StaticAbility, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, mobilize, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticEffect};
use crate::mana::{b, cost, g, generic, mono_hybrid, r, u, w, Color};

/// Sarkhan's Resolve — {1}{G} Instant. Choose one — target creature gets +3/+3
/// until end of turn; or destroy target creature with flying.
pub fn sarkhans_resolve() -> CardDefinition {
    CardDefinition {
        name: "Sarkhan's Resolve",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Dragonback Lancer — {3}{W} 3/3 Human Soldier with Flying and Mobilize 1.
pub fn dragonback_lancer() -> CardDefinition {
    CardDefinition {
        name: "Dragonback Lancer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![mobilize(1)],
        ..Default::default()
    }
}

/// Sibsig Appraiser — {2}{U} 2/1 Zombie Advisor. ETB: look at the top two cards,
/// put one into your hand and the other into your graveyard.
pub fn sibsig_appraiser() -> CardDefinition {
    CardDefinition {
        name: "Sibsig Appraiser",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(2),
            rest_to_graveyard: true,
            pick_filter: None,
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Defibrillating Current — {2/R}{2/W}{2/B} Sorcery. Deal 4 damage to target
/// creature or planeswalker and gain 2 life.
pub fn defibrillating_current() -> CardDefinition {
    CardDefinition {
        name: "Defibrillating Current",
        cost: cost(&[
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::Black),
        ]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Mardu Devotee — {W} 1/2 Human Scout. ETB: scry 2. `{1}: Add {R}, {W}, or
/// {B}. Activate only once each turn.`
pub fn mardu_devotee() -> CardDefinition {
    CardDefinition {
        name: "Mardu Devotee",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            once_per_turn: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Red, Color::White, Color::Black], Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sibsig Host — {4}{B} 2/6 Zombie. ETB: each player mills three cards.
pub fn sibsig_host() -> CardDefinition {
    CardDefinition {
        name: "Sibsig Host",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 6,
        triggered_abilities: vec![etb(Effect::Mill {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(3),
        })],
        ..Default::default()
    }
}

/// Stormscale Scion — {4}{R}{R} 4/4 Dragon with Flying and Storm. Other Dragons
/// you control get +1/+1.
pub fn stormscale_scion() -> CardDefinition {
    CardDefinition {
        name: "Stormscale Scion",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Storm],
        static_abilities: vec![StaticAbility {
            description: "Other Dragons you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Roilmage's Trick — {3}{U} Instant. Converge — creatures your opponents
/// control get -X/-0 until end of turn, where X is the number of colors of mana
/// spent to cast this spell. Draw a card.
pub fn roilmages_trick() -> CardDefinition {
    CardDefinition {
        name: "Roilmage's Trick",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::ConvergedValue)),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Kishla Skimmer — {G}{U} 2/2 Bird Scout with Flying. Whenever a card leaves
/// your graveyard during your turn, draw a card (only once each turn).
pub fn kishla_skimmer() -> CardDefinition {
    CardDefinition {
        name: "Kishla Skimmer",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                kind: EventKind::CardLeftGraveyard,
                scope: EventScope::YourControl,
                filter: Some(Predicate::IsTurnOf(PlayerRef::You)),
                once_per_turn: true,
                per_subject_cap: None,
                actor_is_opponent: false,
            },
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Inevitable Defeat — {1}{R}{W}{B} Instant that can't be countered. Exile
/// target nonland permanent; its controller loses 3 life and you gain 3 life.
pub fn inevitable_defeat() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Inevitable Defeat",
        cost: cost(&[generic(1), r(), w(), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Nonland),
                to: ZoneDest::Exile,
            },
        ]),
        ..Default::default()
    }
}

/// Magmatic Hellkite — {2}{R}{R} 4/5 Dragon with Flying. ETB: destroy target
/// nonbasic land an opponent controls; its controller searches for a basic land
/// and puts it onto the battlefield tapped with a stun counter on it.
pub fn magmatic_hellkite() -> CardDefinition {
    use crate::card::{CounterType, Supertype};
    use crate::effect::ZoneDest;
    let opp_land = SelectionRequirement::Land
        .and(SelectionRequirement::HasSupertype(Supertype::Basic).negate())
        .and(SelectionRequirement::ControlledByOpponent);
    let basic = SelectionRequirement::Land.and(SelectionRequirement::HasSupertype(Supertype::Basic));
    CardDefinition {
        name: "Magmatic Hellkite",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            // The land's controller ramps a stunned basic before the land is
            // destroyed, so `ControllerOf(target)` still resolves to them; the
            // net effect matches the printed "destroy, then its controller…".
            Effect::SearchUpToN {
                who: PlayerRef::ControllerOf(Box::new(target_filtered(opp_land.clone()))),
                filter: basic,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::ControllerOf(Box::new(target_filtered(opp_land.clone()))),
                    tapped: true,
                },
                count: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
            Effect::Destroy { what: target_filtered(opp_land) },
        ]))],
        ..Default::default()
    }
}

/// Hardened Tactician — {1}{W}{B} 2/4 Human Warrior. `{1}, Sacrifice a token:
/// Draw a card.`
pub fn hardened_tactician() -> CardDefinition {
    CardDefinition {
        name: "Hardened Tactician",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((SelectionRequirement::IsToken, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}
