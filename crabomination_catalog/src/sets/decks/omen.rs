//! Tarkir: Dragonstorm **Omen** cards (CR 702.183). Each is a Dragon creature
//! whose card may instead be cast from hand as an instant/sorcery "Omen" half
//! for the listed cost (`GameAction::CastOmen`); on resolution or counter the
//! card is shuffled into its owner's library. The creature lives on the parent
//! [`CardDefinition`]; the Omen half lives in `omen`. Tracked in `DECK_FEATURES.md`.

use crate::card::{
    ActivatedAbility, Adventure, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement, StaticAbility,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::card::LandType;
use crate::effect::shortcut::{etb, etb_gain_life, target_filtered};
use crate::effect::{Duration, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

/// Dragon subtype shorthand for the Omen creature faces.
fn dragon() -> Subtypes {
    Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() }
}

/// Marang River Regent — {4}{U}{U} Dragon 6/7, Flying. ETB returns up to two
/// other target nonland permanents to their owners' hands. Omen — Coil and
/// Catch {3}{U}: draw three cards, then discard a card.
pub fn marang_river_regent() -> CardDefinition {
    CardDefinition {
        name: "Marang River Regent",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 6,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::Nonland
                    .and(SelectionRequirement::OtherThanSource),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        }],
        omen: Some(Box::new(Adventure {
            name: "Coil and Catch",
            cost: cost(&[generic(3), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ]),
        })),
        ..Default::default()
    }
}

/// Bloomvine Regent — {3}{G}{G} Dragon 4/5, Flying. Whenever this or another
/// Dragon you control enters, gain 3 life. Omen — Claim Territory {2}{G}:
/// search up to two basic Forests, one to the battlefield tapped, one to hand.
pub fn bloomvine_regent() -> CardDefinition {
    let basic_forest = SelectionRequirement::HasLandType(LandType::Forest)
        .and(SelectionRequirement::HasSupertype(Supertype::Basic));
    CardDefinition {
        name: "Bloomvine Regent",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        }],
        omen: Some(Box::new(Adventure {
            name: "Claim Territory",
            cost: cost(&[generic(2), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: basic_forest.clone(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    count: Value::Const(1),
                },
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: basic_forest,
                    to: ZoneDest::Hand(PlayerRef::You),
                    count: Value::Const(1),
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Scavenger Regent — {3}{B} Dragon 4/4, Flying, Ward—Discard a card. Omen —
/// Exude Toxin {X}{B}{B}: each non-Dragon creature gets -X/-X until end of turn.
pub fn scavenger_regent() -> CardDefinition {
    CardDefinition {
        name: "Scavenger Regent",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Discard(1))],
        omen: Some(Box::new(Adventure {
            name: "Exude Toxin",
            cost: cost(&[x(), b(), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::HasCreatureType(CreatureType::Dragon).negate(),
                    ),
                ),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                    toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                    duration: Duration::EndOfTurn,
                }),
            },
        })),
        ..Default::default()
    }
}

/// Dirgur Island Dragon — {5}{U} Dragon 4/4, Flying, Ward {2}. Omen —
/// Skimming Strike {1}{U}: tap up to one target creature, then draw a card.
pub fn dirgur_island_dragon() -> CardDefinition {
    CardDefinition {
        name: "Dirgur Island Dragon",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::generic(2))],
        omen: Some(Box::new(Adventure {
            name: "Skimming Strike",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 1,
                    filter: SelectionRequirement::Creature,
                    effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
        })),
        ..Default::default()
    }
}

/// Twinmaw Stormbrood — {5}{W} Dragon 5/4, Flying. ETB gain 5 life. Omen —
/// Charring Bite {1}{R}: deal 5 damage to target creature without flying.
pub fn twinmaw_stormbrood() -> CardDefinition {
    CardDefinition {
        name: "Twinmaw Stormbrood",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
        }],
        omen: Some(Box::new(Adventure {
            name: "Charring Bite",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying).negate()),
                ),
                amount: Value::Const(5),
            },
        })),
        ..Default::default()
    }
}

/// Disruptive Stormbrood — {4}{G} Dragon 3/3, Flying. ETB destroy up to one
/// target artifact or enchantment. Omen — Petty Revenge {1}{B}: destroy target
/// creature with power 3 or less.
pub fn disruptive_stormbrood() -> CardDefinition {
    CardDefinition {
        name: "Disruptive Stormbrood",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                filter: SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
        }],
        omen: Some(Box::new(Adventure {
            name: "Petty Revenge",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(3)),
                ),
            },
        })),
        ..Default::default()
    }
}

/// Omenpath to Naya — Land — Omenpath. Vanishing 4. `{T}: Add {R}, {G}, or {W}.`
pub fn omenpath_to_naya() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::ManaPayload;
    use crate::mana::Color;
    CardDefinition {
        name: "Omenpath to Naya",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Omenpath], ..Default::default() },
        keywords: vec![Keyword::Vanishing(4)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(
                    vec![Color::Red, Color::Green, Color::White],
                    Value::Const(1),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sagu Wildling — {4}{G} Dragon 3/3, Flying. ETB gain 3 life. Omen — Roost Seek
/// {G}: search your library for a basic land, put it into your hand, shuffle.
pub fn sagu_wildling() -> CardDefinition {
    let basic = SelectionRequirement::Land.and(SelectionRequirement::HasSupertype(Supertype::Basic));
    CardDefinition {
        name: "Sagu Wildling",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb_gain_life(3)],
        omen: Some(Box::new(Adventure {
            name: "Roost Seek",
            cost: cost(&[g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: basic,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        })),
        ..Default::default()
    }
}

/// Riling Dawnbreaker — {4}{W} Dragon 3/4, Flying, Vigilance. At the beginning
/// of combat on your turn, another target creature you control gets +1/+0.
/// Omen — Signaling Roar {1}{W}: create a 2/2 white Soldier creature token.
pub fn riling_dawnbreaker() -> CardDefinition {
    CardDefinition {
        name: "Riling Dawnbreaker",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        omen: Some(Box::new(Adventure {
            name: "Signaling Roar",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Soldier".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Soldier],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        })),
        ..Default::default()
    }
}

/// Feral Deathgorger — {5}{B} Dragon 3/5, Flying, Deathtouch. ETB exile up to
/// two target cards from a single graveyard. Omen — Dusk Sight {1}{B}: put a
/// +1/+1 counter on up to one target creature, then draw a card.
pub fn feral_deathgorger() -> CardDefinition {
    CardDefinition {
        name: "Feral Deathgorger",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            // "From a single graveyard" isn't enforced — any graveyard cards.
            filter: SelectionRequirement::InGraveyard,
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
        })],
        omen: Some(Box::new(Adventure {
            name: "Dusk Sight",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 1,
                    filter: SelectionRequirement::Creature,
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
        })),
        ..Default::default()
    }
}

/// Purging Stormbrood — {4}{B} Dragon 4/4, Flying, Ward—Pay 2 life. ETB remove
/// all counters from up to one target creature. Omen — Absorb Essence {1}{W}:
/// target creature gets +2/+2 and gains lifelink and hexproof until end of turn.
pub fn purging_stormbrood() -> CardDefinition {
    CardDefinition {
        name: "Purging Stormbrood",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Life(2))],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::RemoveAllCounters { what: Selector::Target(0) }),
        })],
        omen: Some(Box::new(Adventure {
            name: "Absorb Essence",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Runescale Stormbrood — {3}{R} Dragon 2/4, Flying. Whenever you cast a
/// noncreature spell or a Dragon spell, this gets +2/+0 until end of turn.
/// Omen — Chilling Screech {1}{U}: counter target spell with mana value 2 or less.
pub fn runescale_stormbrood() -> CardDefinition {
    CardDefinition {
        name: "Runescale Stormbrood",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        omen: Some(Box::new(Adventure {
            name: "Chilling Screech",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::Any.and(SelectionRequirement::ManaValueAtMost(2))),
            },
        })),
        ..Default::default()
    }
}

/// Stonehide Ancient — {3}{G}{U}{R} Dragon 6/6, Flying, Vigilance. ETB return
/// all non-Dragon creatures to their owners' hands. Omen — Warning Tremor
/// {2}{R}: deal 2 damage to any target; your Dragon spells cost {2} less this turn.
pub fn stonehide_ancient() -> CardDefinition {
    CardDefinition {
        name: "Stonehide Ancient",
        cost: cost(&[generic(3), g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon).negate()),
            ),
            body: Box::new(Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        omen: Some(Box::new(Adventure {
            name: "Warning Tremor",
            cost: cost(&[generic(2), r()]),
            card_types: vec![CardType::Sorcery],
            // "The next Dragon spell you cast costs {2} less" is approximated as
            // a turn-scoped discount on Dragon spells.
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Any),
                    amount: Value::Const(2),
                },
                Effect::SpellsCostLessThisTurn {
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                    amount: 2,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Stormshriek Feral — {4}{R} Dragon 3/3, Flying, Haste. `{1}{R}: +1/+0 until
/// end of turn.` Omen — Flush Out {1}{R}: discard a card; if you do, draw two.
pub fn stormshriek_feral() -> CardDefinition {
    CardDefinition {
        name: "Stormshriek Feral",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        omen: Some(Box::new(Adventure {
            name: "Flush Out",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ]),
        })),
        ..Default::default()
    }
}

/// Whirlwing Stormbrood — {4}{U} Dragon 4/3, Flash, Flying. You may cast sorcery
/// and Dragon spells as though they had flash. Omen — Dynamic Soar {2}{G}: put
/// three +1/+1 counters on target creature you control.
pub fn whirlwing_stormbrood() -> CardDefinition {
    CardDefinition {
        name: "Whirlwing Stormbrood",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You may cast sorcery and Dragon spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash {
                filter: SelectionRequirement::HasCardType(CardType::Sorcery)
                    .or(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
            },
        }],
        omen: Some(Box::new(Adventure {
            name: "Dynamic Soar",
            cost: cost(&[generic(2), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
        })),
        ..Default::default()
    }
}

/// Pearl Lake Warden — {3}{U} Dragon 4/5, Flying. (Its "you may look at and cast
/// this while it's the top card of your library" static is approximated by a
/// play-from-top permission.) Omen — Nesting Instinct {2}{G}: seek a land card
/// and put it onto the battlefield.
pub fn pearl_lake_warden() -> CardDefinition {
    CardDefinition {
        name: "Pearl Lake Warden",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You may look at the top card of your library any time.",
            effect: StaticEffect::TopOfLibraryRevealed,
        }],
        omen: Some(Box::new(Adventure {
            name: "Nesting Instinct",
            cost: cost(&[generic(2), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seek {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                count: Value::Const(1),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        })),
        ..Default::default()
    }
}

/// Cunning Azurescale — {5}{U} Dragon 5/5, Flash, Flying. ETB: secretly choose
/// land or nonland, then seek two cards of the chosen kind. Omen — Divining Dive
/// {1}{U}: secretly choose land or nonland, then seek one card of that kind.
pub fn cunning_azurescale() -> CardDefinition {
    // "Secretly choose land or nonland" is modeled as a ChooseMode between
    // seeking that many lands and seeking that many nonlands.
    let seek_mode = |count: i32, to: ZoneDest| {
        Effect::ChooseMode(vec![
            Effect::Seek {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                count: Value::Const(count),
                to: to.clone(),
            },
            Effect::Seek {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land.negate(),
                count: Value::Const(count),
                to,
            },
        ])
    };
    CardDefinition {
        name: "Cunning Azurescale",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(seek_mode(2, ZoneDest::Hand(PlayerRef::You)))],
        omen: Some(Box::new(Adventure {
            name: "Divining Dive",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: seek_mode(1, ZoneDest::Hand(PlayerRef::You)),
        })),
        ..Default::default()
    }
}
