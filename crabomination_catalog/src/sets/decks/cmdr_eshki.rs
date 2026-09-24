//! Commander: the cards the **Temur Roar** precon (TDC, Eshki, Temur's Roar)
//! needed beyond what the catalog had. Tests in `tests/recent_b/cmdr_eshki.rs`.
//!
//! Residuals (each also on its card):
//! - **Deceptive Frostkite** — the copy isn't optional when a creature with
//!   power 4 or greater is there to copy.
//! - **Will of the Temur** — "if you control a commander as you cast" is read
//!   as it resolves, as the other Will cards read it.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EntersAsCopy, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, cost, g, generic, r, u, x};
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

fn dragon(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature(name, mana, vec![CreatureType::Dragon], p, t)
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn cast_power_at_least(n: i32) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::PowerAtLeast(n) }
}

/// "Mana value less than or equal to" the trigger's event amount.
fn mv_at_most_event() -> R {
    R::ManaValueLessThanEventAmount.or(R::ManaValueEqualsTriggerAmount)
}

/// Eshki, Temur's Roar — each creature spell you cast grows Eshki; power 4+
/// draws, power 6+ burns each opponent for Eshki's power.
pub fn eshki_temurs_roar() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::If {
                    cond: cast_power_at_least(4),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: cast_power_at_least(6),
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::PowerOf(Box::new(Selector::This)),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..creature(
            "Eshki, Temur's Roar",
            cost(&[g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Become the Avalanche — a card per creature you control with power 4+, then
/// your creatures get +X/+X for your hand size.
pub fn become_the_avalanche() -> CardDefinition {
    let hand = || Value::HandSizeOf(PlayerRef::You);
    spell(
        "Become the Avalanche",
        cost(&[generic(4), g(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(yours(R::Creature.and(R::PowerAtLeast(4))))),
            },
            Effect::PumpPT {
                what: yours(R::Creature),
                power: hand(),
                toughness: hand(),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Broodcaller Scourge — flying; when your Dragons connect, you may put a
/// permanent card with mana value up to that damage from hand onto the
/// battlefield.
pub fn broodcaller_scourge() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dragon),
                })
                .once_per_batch_summing_damage(),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Permanent.and(mv_at_most_event()),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        }],
        ..dragon("Broodcaller Scourge", cost(&[generic(5), g(), g()]), 5, 7)
    }
}

/// Deceptive Frostkite — flying; enters as a copy of your creature with power
/// 4 or greater, a flying Dragon too. ⚠ The copy isn't optional.
pub fn deceptive_frostkite() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
            extra_creature_types: vec![CreatureType::Dragon],
            extra_keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        ..dragon("Deceptive Frostkite", cost(&[u(), u()]), 1, 1)
    }
}

/// Draconic Lore — {2} less with a Dragon; draw three.
pub fn draconic_lore() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if you control a Dragon.",
            effect: StaticEffect::SelfCostReducedIfControlEach {
                filters: vec![R::HasCreatureType(CreatureType::Dragon)],
                amount: 2,
            },
        }],
        ..spell(
            "Draconic Lore",
            cost(&[generic(5), u()]),
            CardType::Instant,
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        )
    }
}

/// Gadrak, the Crown-Scourge — flying; attacks only with four artifacts; your
/// end step makes a Treasure per nontoken creature that died this turn.
pub fn gadrak_the_crown_scourge() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![
            Keyword::Flying,
            Keyword::CantAttackOrBlockUnlessYouControlCount {
                filter: Box::new(R::Artifact),
                min: 4,
                attack_only: true,
                block_only: false,
                exclude_self: false,
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CreatureDeathsThisTurnMatching { filter: R::NotToken },
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        }],
        ..creature(
            "Gadrak, the Crown-Scourge",
            cost(&[generic(2), r()]),
            vec![CreatureType::Dragon],
            5,
            4,
        )
    }
}

/// Hammerhead Tyrant — flying; each spell you cast may bounce an opponent's
/// nonland permanent with mana value up to the spell's.
pub fn hammerhead_tyrant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Return a permanent with mana value up to that spell's?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        R::Permanent.and(R::Nonland).and(R::ControlledByOpponent).and(mv_at_most_event()),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
        }],
        ..dragon("Hammerhead Tyrant", cost(&[generic(4), u(), u()]), 6, 6)
    }
}

/// Opportunistic Dragon — flying; on entry, take an opponent's Human or
/// artifact for as long as the Dragon stays; it loses all abilities and can't
/// attack or block.
pub fn opportunistic_dragon() -> CardDefinition {
    let held = |keyword| Effect::GrantKeyword {
        what: Selector::Target(0),
        keyword,
        duration: Duration::WhileSourceOnBattlefield,
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControlWhileSourceRemains {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Human).or(R::Artifact).and(R::ControlledByOpponent),
                ),
            },
            Effect::LoseAllAbilities { what: Selector::Target(0), duration: Duration::WhileSourceOnBattlefield },
            held(Keyword::CantAttack),
            held(Keyword::CantBlock),
        ]))],
        ..dragon("Opportunistic Dragon", cost(&[generic(2), r(), r()]), 4, 3)
    }
}

/// Temple of the Dragon Queen — enters tapped unless you reveal a Dragon from
/// hand or control one; taps for the color chosen as it enters.
pub fn temple_of_the_dragon_queen() -> CardDefinition {
    let dragon = || R::HasCreatureType(CreatureType::Dragon);
    CardDefinition {
        name: "Temple of the Dragon Queen",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "Enters tapped unless you reveal a Dragon card from your hand or control a Dragon.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::Any(vec![
                    Predicate::SelectorExists(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: dragon(),
                    }),
                    Predicate::SelectorExists(yours(dragon())),
                ]),
            },
        }],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Territorial Hellkite — flying, haste; each of your combats it must attack
/// a random opponent it didn't attack last combat, and taps when there's none.
pub fn territorial_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::MustAttackChosenPlayer],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::ChooseRandomOpponentNotAttackedLastCombat,
        }],
        ..dragon("Territorial Hellkite", cost(&[generic(2), r(), r()]), 6, 5)
    }
}

/// Ureni of the Unwritten — flying, trample; entering or attacking, look at
/// the top eight and you may put a Dragon creature card from them onto the
/// battlefield.
pub fn ureni_of_the_unwritten() -> CardDefinition {
    let dig = || Effect::LookTopPutMatchingOntoBattlefield {
        count: Value::Const(8),
        filter: R::Creature.and(R::HasCreatureType(CreatureType::Dragon)),
        then: None,
        max: Some(1),
        tapped: false,
        exile_rest: false,
        rest_to_graveyard: false,
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![etb(dig()), on_attack(dig())],
        ..creature(
            "Ureni of the Unwritten",
            cost(&[generic(4), g(), u(), r()]),
            vec![CreatureType::Spirit, CreatureType::Dragon],
            7,
            7,
        )
    }
}

/// Will of the Temur — a 4/4 flying Dragon token copy of target permanent,
/// and/or a player draws your greatest mana value; both with a commander.
pub fn will_of_the_temur() -> CardDefinition {
    let modes = || {
        vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::Permanent),
                extra_creature_types: vec![CreatureType::Dragon],
                extra_card_types: vec![CardType::Creature],
                override_pt: Some((4, 4)),
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![Keyword::Flying],
            },
            Effect::Draw {
                who: target_filtered(R::Player),
                amount: Value::GreatestManaValueAmongPermanents(PlayerRef::You),
            },
        ]
    };
    spell(
        "Will of the Temur",
        cost(&[generic(5), u()]),
        CardType::Sorcery,
        Effect::If {
            cond: Predicate::YouControlACommander,
            then: Box::new(Effect::ChooseN { picks: vec![0, 1], modes: modes() }),
            else_: Box::new(Effect::ChooseMode(modes())),
        },
    )
}

/// Zenith Festival — exile the top X; play them until the end of your next
/// turn. Harmonize {X}{R}{R}.
pub fn zenith_festival() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Harmonize(cost(&[x(), r(), r()]))],
        ..spell(
            "Zenith Festival",
            cost(&[x(), r(), r()]),
            CardType::Sorcery,
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::XFromCost,
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        )
    }
}
