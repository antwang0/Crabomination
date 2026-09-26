//! Commander: the cards the **Most Wanted** precon (OTC, Olivia, Opulent
//! Outlaw) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_olivia.rs`.
//!
//! Residuals (each also on its card):
//! - **Dire Fleet Ravager** — the players lose their thirds one after
//!   another (each reads only its own life, so the totals match).
//! - **Vihaan, Goldwaker** — its vigilance/haste grant reads printed types,
//!   so animated Treasures (outlaws on the layered type line, which Olivia's
//!   trigger does see) don't get them.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, extort, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest, ZoneRef};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, generic, r, w, x};
use crate::game::effects::treasure_token;
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn make(count: Value, definition: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(definition) }
}

fn treasures(n: i32) -> Effect {
    make(Value::Const(n), treasure_token())
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn outlaw_creatures_you_control() -> R {
    R::Creature.and(R::IsOutlaw).and(R::ControlledByYou)
}

/// The OTC Mercenary: a 1/1 red token with a sorcery-speed +1/+0 tap.
fn mercenary_token() -> TokenDefinition {
    TokenDefinition {
        name: "Mercenary".to_string(),
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mercenary], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Olivia, Opulent Outlaw — a Treasure when your outlaws connect (once per
/// combat-damage batch); {3} and two Treasures put two +1/+1 counters on
/// each creature you control.
pub fn olivia_opulent_outlaw() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsOutlaw },
                )
            },
            effect: treasures(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Treasure), 2)),
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..legend(
            "Olivia, Opulent Outlaw",
            cost(&[generic(1), r(), w(), b()]),
            vec![CreatureType::Vampire, CreatureType::Assassin],
            3,
            3,
        )
    }
}

/// Angelic Sell-Sword — a Mercenary whenever it or another nontoken creature
/// of yours enters; draws on attack at power 6 or more.
pub fn angelic_sell_sword() -> CardDefinition {
    let big = || Predicate::EntityMatches { what: Selector::This, filter: R::PowerAtLeast(6) };
    let mut attack = on_attack(Effect::If { cond: big(), then: Box::new(draw(1)), else_: Box::new(Effect::Noop) });
    attack.event = attack.event.with_filter(big());
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::NotToken) },
                ),
                effect: make(Value::ONE, mercenary_token()),
            },
            attack,
        ],
        ..creature(
            "Angelic Sell-Sword",
            cost(&[generic(4), w()]),
            vec![CreatureType::Angel, CreatureType::Mercenary],
            4,
            4,
        )
    }
}

/// Angrath's Marauders — damage from your sources is doubled (CR 616; copies
/// stack multiplicatively).
pub fn angraths_marauders() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "If a source you control would deal damage to a permanent or player, it deals double that damage instead.",
            StaticEffect::MultiplyDamageFromYourSources { factor: 2 },
        )],
        ..creature(
            "Angrath's Marauders",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            4,
            4,
        )
    }
}

/// Back in Town — X outlaw creature cards from your graveyard to the
/// battlefield, targeted.
pub fn back_in_town() -> CardDefinition {
    spell(
        "Back in Town",
        cost(&[x(), generic(2), b()]),
        CardType::Sorcery,
        // CR 601.2c — "X target outlaw creature cards".
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 20,
                min_targets: 0,
                filter: R::Creature.and(R::IsOutlaw).and(R::InYourGraveyard),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            }),
        },
    )
}

/// Bounty Board — mana; a sorcery-speed bounty counter; any bounty creature
/// dying feeds its controller's opponents a card and 2 life.
pub fn bounty_board() -> CardDefinition {
    let their_opponents =
        || PlayerRef::OpponentOf(Box::new(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))));
    CardDefinition {
        name: "Bounty Board",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::Bounty,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::WithCounter(CounterType::Bounty),
            }),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(their_opponents()), amount: Value::ONE },
                Effect::GainLife { who: Selector::Player(their_opponents()), amount: Value::Const(2) },
            ]),
        }],
        ..Default::default()
    }
}

/// Charred Graverobber — returns an outlaw card on entry; escapes with a
/// +1/+1 counter.
pub fn charred_graverobber() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Escape(cost(&[generic(3), b(), b()]), 4)],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::IsOutlaw.from_your_graveyard()),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SourceCastFromEscape),
                then: Box::new(Value::ONE),
                else_: Box::new(Value::Const(0)),
            },
        )),
        ..creature(
            "Charred Graverobber",
            cost(&[generic(2), b()]),
            vec![CreatureType::Skeleton, CreatureType::Mercenary],
            3,
            1,
        )
    }
}

/// Dead Before Sunrise — outlaws you control get +1/+0 and a "tap: deal
/// damage equal to power to target creature" ability until end of turn.
pub fn dead_before_sunrise() -> CardDefinition {
    spell(
        "Dead Before Sunrise",
        cost(&[generic(3), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(outlaw_creatures_you_control()),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantActivatedAbilityToMatching {
                filter: outlaw_creatures_you_control(),
                ability: Box::new(ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::DealDamage {
                        to: target_filtered(R::Creature),
                        amount: Value::PowerOf(Box::new(Selector::This)),
                    },
                    ..Default::default()
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Dire Fleet Ravager — each player loses a third of their life, rounded up.
/// Residual: the players lose it one after another.
pub fn dire_fleet_ravager() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::LoseLife {
                who: Selector::You,
                // ceil(L / 3) = floor((L + 2) / 3) for L >= 0.
                amount: Value::DivDown(Box::new(Value::Sum(vec![Value::LifeOf(PlayerRef::You), Value::Const(2)])), 3),
            }),
        })],
        ..creature(
            "Dire Fleet Ravager",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Orc, CreatureType::Pirate, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Discreet Retreat — the enchanted land taps for two mana of one colour for
/// outlaws; your first outlaw spell each turn draws a card for 1 life.
pub fn discreet_retreat() -> CardDefinition {
    CardDefinition {
        name: "Discreet Retreat",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![static_ab(
            "Enchanted land has \"{T}: Add two mana of any one color. Spend this mana only to cast outlaw spells or activate abilities of outlaw sources.\"",
            StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Restricted(
                            Box::new(ManaPayload::AnyOneColor(Value::Const(2))),
                            SpendRestriction::OutlawSpellsOrAbilities,
                        ),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellFirstMatchingThisTurn(R::IsOutlaw)),
            effect: Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]),
        }],
        ..Default::default()
    }
}

/// Glittering Stockpile — a Treasure that stores stash counters as it taps
/// for {R}, then cashes them in as one colour.
pub fn glittering_stockpile() -> CardDefinition {
    CardDefinition {
        name: "Glittering Stockpile",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Treasure], ..Default::default() },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::ONE) },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Stash, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Stash,
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Graywater's Fixer — outlaw creature cards in your graveyard have encore
/// {X}, X their mana value.
pub fn graywaters_fixer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Each outlaw creature card in your graveyard has encore {X}, where X is its mana value.",
            StaticEffect::GraveyardCardsHaveEncore { filter: R::Creature.and(R::IsOutlaw), mana_cost: false },
        )],
        ..creature(
            "Graywater's Fixer",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Lizard, CreatureType::Mercenary],
            4,
            4,
        )
    }
}

/// Life Insurance — extort; each nontoken creature's death costs you 1 life
/// and makes a Treasure.
pub fn life_insurance() -> CardDefinition {
    CardDefinition {
        name: "Life Insurance",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            extort(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
                ),
                effect: Effect::Seq(vec![Effect::LoseLife { who: Selector::You, amount: Value::ONE }, treasures(1)]),
            },
        ],
        ..Default::default()
    }
}

/// Mari, the Killing Quill — exiles opponents' dead creatures with hit
/// counters; your Assassins, Mercenaries and Rogues have deathtouch and cash
/// a hit counter in for a card and two Treasures on combat damage.
pub fn mari_the_killing_quill() -> CardDefinition {
    let hitters = || {
        R::Creature.and(R::ControlledByYou).and(
            R::HasCreatureType(CreatureType::Assassin)
                .or(R::HasCreatureType(CreatureType::Mercenary))
                .or(R::HasCreatureType(CreatureType::Rogue)),
        )
    };
    let marked = || {
        Selector::Take {
            inner: Box::new(Selector::EachMatching {
                zone: ZoneRef::Exile,
                filter: R::WithCounter(CounterType::Hit).and(R::OwnedByDefendingPlayer),
            }),
            count: Box::new(Value::ONE),
        }
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Hit, amount: Value::ONE },
            ]),
        }],
        static_abilities: vec![
            static_ab(
                "Assassins, Mercenaries, and Rogues you control have deathtouch.",
                StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(hitters()),
                    keyword: Keyword::Deathtouch,
                },
            ),
            static_ab(
                "Assassins, Mercenaries, and Rogues you control have \"Whenever this creature deals combat damage to a player, you may remove a hit counter from a card that player owns in exile. If you do, draw a card and create two Treasure tokens.\"",
                StaticEffect::GrantTriggeredAbility {
                    filter: hitters(),
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                        effect: Effect::If {
                            cond: Predicate::SelectorExists(marked()),
                            then: Box::new(Effect::MayDo {
                                description: "Remove a hit counter to draw a card and create two Treasures?".into(),
                                body: Box::new(Effect::Seq(vec![
                                    Effect::RemoveCounter { what: marked(), kind: CounterType::Hit, amount: Value::ONE },
                                    draw(1),
                                    treasures(2),
                                ])),
                            }),
                            else_: Box::new(Effect::Noop),
                        },
                    }),
                },
            ),
        ],
        ..legend(
            "Mari, the Killing Quill",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Assassin],
            3,
            2,
        )
    }
}

/// Mass Mutiny — for each opponent, threaten up to one of their creatures.
pub fn mass_mutiny() -> CardDefinition {
    spell(
        "Mass Mutiny",
        cost(&[generic(3), r(), r()]),
        CardType::Sorcery,
        Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 15,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Seq(vec![
                    Effect::GainControl { what: Selector::Target(0), to: None, duration: Duration::EndOfTurn },
                    Effect::Untap { what: Selector::Target(0), up_to: None },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
        },
    )
}

/// Misfortune Teller — on entry and on combat damage to a player, exile a
/// card from a graveyard: a creature card makes a 2/2 Rogue, a land card a
/// Treasure (both, for Dryad Arbor), anything else 3 life.
pub fn misfortune_teller() -> CardDefinition {
    let rogue = TokenDefinition {
        name: "Rogue".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rogue], ..Default::default() },
        ..Default::default()
    };
    let was = |filter: R| Predicate::EntityMatches { what: Selector::Target(0), filter };
    let body = Effect::Seq(vec![
        Effect::Move { what: target_filtered(R::Any.from_any_graveyard()), to: ZoneDest::Exile },
        Effect::If { cond: was(R::Creature), then: Box::new(make(Value::ONE, rogue)), else_: Box::new(Effect::Noop) },
        Effect::If { cond: was(R::Land), then: Box::new(treasures(1)), else_: Box::new(Effect::Noop) },
        Effect::If {
            cond: Predicate::Not(Box::new(was(R::Creature.or(R::Land)))),
            then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
            else_: Box::new(Effect::Noop),
        },
    ]);
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            etb(body.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: body,
            },
        ],
        ..creature(
            "Misfortune Teller",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            3,
            1,
        )
    }
}

/// Mistmeadow Skulk — lifelink, protection from mana value 3 or greater.
pub fn mistmeadow_skulk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink, Keyword::ProtectionFromMatching(Box::new(R::ManaValueAtLeast(3)))],
        ..creature(
            "Mistmeadow Skulk",
            cost(&[generic(1), w()]),
            vec![CreatureType::Kithkin, CreatureType::Rogue],
            1,
            1,
        )
    }
}

/// Vihaan, Goldwaker — other outlaws have vigilance and haste; at your
/// combat, your Treasures may become 3/3 Construct Assassins (so outlaws).
/// Residual: the grant's outlaw filter reads printed types, so the animated
/// Treasures don't get vigilance and haste.
pub fn vihaan_goldwaker() -> CardDefinition {
    let other_outlaws = || Selector::EachPermanent(R::IsOutlaw.and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "Other outlaws you control have vigilance.",
                StaticEffect::GrantKeyword { applies_to: other_outlaws(), keyword: Keyword::Vigilance },
            ),
            static_ab(
                "Other outlaws you control have haste.",
                StaticEffect::GrantKeyword { applies_to: other_outlaws(), keyword: Keyword::Haste },
            ),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Have your Treasures become 3/3 Construct Assassin artifact creatures?".into(),
                body: Box::new(Effect::BecomeCreature {
                    what: Selector::EachPermanent(
                        R::HasArtifactSubtype(ArtifactSubtype::Treasure).and(R::ControlledByYou),
                    ),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Construct, CreatureType::Assassin],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..legend(
            "Vihaan, Goldwaker",
            cost(&[r(), w(), b()]),
            vec![CreatureType::Dwarf, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// We Ride at Dawn — legendary creature spells have convoke; a Mercenary
/// whenever your commander attacks (whoever controls it).
pub fn we_ride_at_dawn() -> CardDefinition {
    CardDefinition {
        name: "We Ride at Dawn",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![static_ab(
            "Legendary creature spells you cast have convoke.",
            StaticEffect::GrantConvokeToSpells { filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)) },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::IsCommander.and(R::OwnedByYou),
            }),
            effect: make(Value::ONE, mercenary_token()),
        }],
        ..Default::default()
    }
}
