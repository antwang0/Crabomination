//! Cards that name a commander (COMMANDER_BACKLOG §3, "Missing"): the
//! Battlebond-style familiars Kediss, Esior and Anara, Lozhan, the Backgrounds
//! Agent of the Iron Throne, Inspiring Leader, Tavern Brawler, Far Traveler
//! and Guild Artisan, Astarion's Thirst, and Mines of Moria (§2).
//!
//! A Background's "Commander creatures you own have …" is a
//! `GrantTriggeredAbility` to `Creature ∧ IsCommander ∧ OwnedByYou` (the Folk
//! Hero shape). Not here: Master Chef (a granted enters-with-counters
//! replacement) and Noble Heritage (each player's optional counters plus
//! protection from a player) want primitives.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, SpellSubtype, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{mint_treasures, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn familiar(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Partner],
        ..Default::default()
    }
}

fn background(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Background],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// "Commander creatures you own have [ability]."
fn grant_to_your_commanders(description: &'static str, ability: TriggeredAbility) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::GrantTriggeredAbility {
            filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
            ability: Box::new(ability),
        },
    }
}

fn your_commanders() -> R {
    R::IsCommander.and(R::ControlledByYou)
}

/// Kediss, Emberclaw Familiar — {1}{R} 1/1 Elemental Lizard, partner. Whenever
/// a commander you control deals combat damage to an opponent, it (the
/// commander) deals that much damage to each other opponent.
pub fn kediss_emberclaw_familiar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander }),
            effect: Effect::DealDamageFrom {
                source: Selector::TriggerSource,
                to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..familiar(
            "Kediss, Emberclaw Familiar",
            cost(&[generic(1), r()]),
            vec![CreatureType::Elemental, CreatureType::Lizard],
            1,
            1,
        )
    }
}

/// Esior, Wardwing Familiar — {1}{U} 1/3 Bird, flying, partner. Spells your
/// opponents cast that target one or more commanders you control cost {3}
/// more.
pub fn esior_wardwing_familiar() -> CardDefinition {
    let mut d = familiar("Esior, Wardwing Familiar", cost(&[generic(1), u()]), vec![CreatureType::Bird], 1, 3);
    d.keywords.push(Keyword::Flying);
    d.static_abilities.push(StaticAbility {
        description: "Spells your opponents cast that target one or more commanders you control cost {3} more to cast.",
        effect: StaticEffect::TaxOpponentSpellsTargeting { target_filter: your_commanders(), amount: 3 },
    });
    d
}

/// Anara, Wolvid Familiar — {3}{G} 4/4 Wolf Beast, partner. During your turn,
/// commanders you control have indestructible.
pub fn anara_wolvid_familiar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, commanders you control have indestructible.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(your_commanders()),
                    keyword: Keyword::Indestructible,
                }),
            },
        }],
        ..familiar(
            "Anara, Wolvid Familiar",
            cost(&[generic(3), g()]),
            vec![CreatureType::Wolf, CreatureType::Beast],
            4,
            4,
        )
    }
}

/// Lozhan, Dragons' Legacy — {3}{U}{R} 4/2 legendary Dragon Shaman, flying.
/// Whenever you cast an Adventure or Dragon spell, Lozhan deals damage equal
/// to that spell's mana value to any target that isn't a commander.
pub fn lozhan_dragons_legacy() -> CardDefinition {
    CardDefinition {
        name: "Lozhan, Dragons' Legacy",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Shaman],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasSpellSubtype(SpellSubtype::Adventure)
                        .or(R::HasCreatureType(CreatureType::Dragon)),
                },
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::any_target().and(R::Not(Box::new(R::IsCommander)))),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Agent of the Iron Throne — {2}{B} Background. Commander creatures you own
/// have "Whenever an artifact or creature you control is put into a graveyard
/// from the battlefield, each opponent loses 1 life."
pub fn agent_of_the_iron_throne() -> CardDefinition {
    let drain = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureOrArtifactDied, EventScope::YourControl),
        effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
    };
    CardDefinition {
        static_abilities: vec![grant_to_your_commanders(
            "Commander creatures you own have \"Whenever an artifact or creature you control is put into a \
             graveyard from the battlefield, each opponent loses 1 life.\"",
            drain,
        )],
        ..background("Agent of the Iron Throne", cost(&[generic(2), b()]))
    }
}

/// Inspiring Leader — {2}{W} Background. Commander creatures you own have
/// "Creature tokens you control get +2/+2." Approximation: the anthem is the
/// Background's controller's while they control a commander creature they
/// own; a commander stolen by another player doesn't pump the thief's tokens.
pub fn inspiring_leader() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"Creature tokens you control get +2/+2.\"",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::IsCommander).and(R::OwnedByYou).and(R::ControlledByYou),
                )),
                inner: Box::new(StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::IsToken).and(R::ControlledByYou)),
                    power: 2,
                    toughness: 2,
                }),
            },
        }],
        ..background("Inspiring Leader", cost(&[generic(2), w()]))
    }
}

/// Tavern Brawler — {2}{R} Background. Commander creatures you own have "At
/// the beginning of your upkeep, exile the top card of your library. This
/// creature gets +X/+0 until end of turn, where X is that card's mana value.
/// You may play that card this turn."
pub fn tavern_brawler() -> CardDefinition {
    let impulse = TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect: Effect::Seq(vec![
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            Effect::PumpPT {
                what: Selector::This,
                power: Value::ManaValueOf(Box::new(Selector::ExiledThisResolution { filter: R::Any })),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]),
    };
    CardDefinition {
        static_abilities: vec![grant_to_your_commanders(
            "Commander creatures you own have \"At the beginning of your upkeep, exile the top card of your \
             library. This creature gets +X/+0 until end of turn, where X is that card's mana value. You may \
             play that card this turn.\"",
            impulse,
        )],
        ..background("Tavern Brawler", cost(&[generic(2), r()]))
    }
}

/// Far Traveler — {2}{W} Background. Commander creatures you own have "At the
/// beginning of your end step, exile up to one target tapped creature you
/// control, then return it to the battlefield under its owner's control."
pub fn far_traveler() -> CardDefinition {
    let blink = TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
        effect: Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Creature.and(R::Tapped).and(R::ControlledByYou)) },
                Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                },
            ])),
        },
    };
    CardDefinition {
        static_abilities: vec![grant_to_your_commanders(
            "Commander creatures you own have \"At the beginning of your end step, exile up to one target \
             tapped creature you control, then return it to the battlefield under its owner's control.\"",
            blink,
        )],
        ..background("Far Traveler", cost(&[generic(2), w()]))
    }
}

/// Guild Artisan — {1}{R} Background. Commander creatures you own have
/// "Whenever this creature attacks a player, if no opponent has more life than
/// that player, you create two Treasure tokens."
pub fn guild_artisan() -> CardDefinition {
    // "No opponent has more life than that player": not (some opponent's life
    // exceeds the defending player's). `ForAnyPlayer` binds each opponent as
    // `Triggerer`.
    let richest = Predicate::Not(Box::new(Predicate::ForAnyPlayer {
        who: PlayerRef::EachOpponent,
        pred: Box::new(Predicate::Not(Box::new(Predicate::ValueAtLeast(
            Value::LifeOf(PlayerRef::DefendingPlayer),
            Value::LifeOf(PlayerRef::Triggerer),
        )))),
    }));
    let treasures = TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::All(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::IsAttackingOpponentPlayer },
            richest,
        ])),
        effect: mint_treasures(2),
    };
    CardDefinition {
        static_abilities: vec![grant_to_your_commanders(
            "Commander creatures you own have \"Whenever this creature attacks a player, if no opponent has \
             more life than that player, you create two Treasure tokens.\"",
            treasures,
        )],
        ..background("Guild Artisan", cost(&[generic(1), r()]))
    }
}

/// Astarion's Thirst — {3}{B} Instant. Exile target creature. Put X +1/+1
/// counters on a commander creature you control, where X is the power of the
/// creature exiled this way (last known, CR 608.2h).
pub fn astarions_thirst() -> CardDefinition {
    CardDefinition {
        name: "Astarion's Thirst",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Creature) },
            Effect::AddCounter {
                what: Selector::take(
                    Selector::EachPermanent(R::Creature.and(your_commanders())),
                    Value::ONE,
                ),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::PowerOf(Box::new(Selector::ExiledThisResolution { filter: R::Creature })),
            },
        ]),
        ..Default::default()
    }
}

/// Mines of Moria — Legendary Land. Enters tapped unless you control a
/// legendary creature. {T}: Add {R}. {3}{R}, {T}, Exile three cards from your
/// graveyard: Create two Treasure tokens.
pub fn mines_of_moria() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Mines of Moria",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control a legendary creature.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
                )),
            },
        }],
        activated_abilities: vec![
            super::super::tap_add(Color::Red),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), r()]),
                exile_other_filter: Some((R::Any, 3)),
                effect: mint_treasures(2),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
