//! Commander: the 1500 rescan's three-card gaps for Niv-Mizzet Reborn,
//! Niv-Mizzet Supreme, Trelasarra, Shanna, Ramses, Varolz and Horde of Notions
//! (`cmdr_edhrec33` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec34.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, evoke, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost};
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

fn ward(n: u32) -> Keyword {
    Keyword::Ward(WardCost::Mana(cost(&[generic(n)])))
}

fn gain(n: i32) -> Effect {
    Effect::GainLife { who: Selector::You, amount: Value::Const(n) }
}

/// "Whenever you gain life, [effect]. This ability triggers only once each
/// turn."
fn once_a_turn_life_gain(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl).once_per_turn(), effect }
}

/// CR 105.4 — exactly two colors: one of the ten pairs and none of the rest.
fn exactly_two_colors() -> R {
    let all = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    let mut out: Option<R> = None;
    for i in 0..5 {
        for j in i + 1..5 {
            let mut pair = R::HasColor(all[i]).and(R::HasColor(all[j]));
            for (k, c) in all.iter().enumerate() {
                if k != i && k != j {
                    pair = pair.and(R::Not(Box::new(R::HasColor(*c))));
                }
            }
            out = Some(match out {
                Some(o) => o.or(pair),
                None => pair,
            });
        }
    }
    out.expect("ten pairs")
}

// ── Niv-Mizzet Reborn / Niv-Mizzet Supreme ───────────────────────────────────

/// Niv-Mizzet, Supreme — {W}{U}{B}{R}{G} legendary 5/5 Dragon Avatar, flying,
/// hexproof from monocolored. Each instant and sorcery card in your graveyard
/// that's exactly two colors has jump-start.
pub fn niv_mizzet_supreme() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::HexproofFromMonocolored],
        static_abilities: vec![StaticAbility {
            description: "Each instant and sorcery card in your graveyard that's exactly two colors has jump-start.",
            effect: StaticEffect::GraveyardCardsHaveJumpStart {
                filter: R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(exactly_two_colors()),
            },
        }],
        ..legend(
            "Niv-Mizzet, Supreme",
            cost(&[w(), u(), b(), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::Avatar],
            5,
            5,
        )
    }
}

/// Turn to Mist — {1}{W/U} instant. Exile target creature; return it under
/// its owner's control at the beginning of the next end step.
pub fn turn_to_mist() -> CardDefinition {
    spell(
        "Turn to Mist",
        cost(&[generic(1), hybrid(Color::White, Color::Blue)]),
        CardType::Instant,
        Effect::ExileReturnNextEndStep { what: target_filtered(R::Creature) },
    )
}

/// Reborn Hope — {G}{W} sorcery. Return target multicolored card from your
/// graveyard to your hand.
pub fn reborn_hope() -> CardDefinition {
    spell(
        "Reborn Hope",
        cost(&[g(), w()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Multicolored.and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Cosmic Rebirth — {1}{G}{W} instant. Choose target permanent card in your
/// graveyard: mana value 3 or less, it may go onto the battlefield; otherwise
/// to your hand. You gain 3 life.
pub fn cosmic_rebirth() -> CardDefinition {
    let card = || target_filtered(R::Permanent.and(R::InYourGraveyard));
    spell(
        "Cosmic Rebirth",
        cost(&[generic(1), g(), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::ManaValueAtMost(3) },
                then: Box::new(Effect::MayDoElse {
                    description: "Put it onto the battlefield (instead of into your hand)?".into(),
                    body: Box::new(Effect::Move {
                        what: card(),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Box::new(Effect::Move { what: card(), to: ZoneDest::Hand(PlayerRef::You) }),
                }),
                else_: Box::new(Effect::Move { what: card(), to: ZoneDest::Hand(PlayerRef::You) }),
            },
            gain(3),
        ]),
    )
}

// ── Trelasarra, Moon Dancer / Shanna, Purifying Blade ────────────────────────

/// Bloombrute — {2}{G}{W} 4/4 Plant Elemental. Your first life gain each turn
/// draws a card. {4}{G}{W}: target creature gains trample and lifelink until
/// end of turn.
pub fn bloombrute() -> CardDefinition {
    let grant = |keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn };
    CardDefinition {
        triggered_abilities: vec![once_a_turn_life_gain(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), w()]),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                grant(Keyword::Lifelink),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Bloombrute",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Plant, CreatureType::Elemental],
            4,
            4,
        )
    }
}

/// Vigorbloom Charm — {G}{W} instant. Choose one: a permanent you control gains
/// hexproof and indestructible until end of turn; or draw a card and gain 3;
/// or a +1/+1 counter on your creature, then it fights an opponent's.
pub fn vigorbloom_charm() -> CardDefinition {
    let yours = || target_filtered(R::Permanent.and(R::ControlledByYou));
    spell(
        "Vigorbloom Charm",
        cost(&[g(), w()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::GrantKeyword { what: yours(), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::Seq(vec![Effect::Draw { who: Selector::You, amount: Value::ONE }, gain(3)]),
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Fight {
                    attacker: Selector::Target(0),
                    defender: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                },
            ]),
        ]),
    )
}

/// Shanna, Purifying Blade — {G}{W}{U} legendary 3/3 Human Warrior, lifelink.
/// At your end step, you may pay {X} to draw X cards, X at most the life you
/// gained this turn.
pub fn shanna_purifying_blade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::MayPayXUpTo {
                description: "Pay {X} to draw X cards".into(),
                cap: Value::LifeGainedThisTurn(PlayerRef::You),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::XFromCost }),
            },
        }],
        ..legend(
            "Shanna, Purifying Blade",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Kwia Vigorbloom — {3}{G}{W}{W} legendary 6/6 Elder Sphinx, flying,
/// vigilance, lifelink, ward {2}. Your first life gain each turn makes a Lotus
/// ("{T}, sacrifice: add three mana of any one color").
pub fn kwia_vigorbloom() -> CardDefinition {
    let lotus = TokenDefinition {
        name: "Lotus".into(),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(3)) },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Lifelink, ward(2)],
        triggered_abilities: vec![once_a_turn_life_gain(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(lotus),
        })],
        ..legend(
            "Kwia Vigorbloom",
            cost(&[generic(3), g(), w(), w()]),
            vec![CreatureType::Elder, CreatureType::Sphinx],
            6,
            6,
        )
    }
}

// ── Ramses, Assassin Lord ────────────────────────────────────────────────────

/// Brotherhood Spy — {1}{U} 1/3 Human Assassin. At the beginning of combat on
/// your turn, if you control a legendary Assassin, it gets +1/+0 and can't be
/// blocked this turn.
pub fn brotherhood_spy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl).with_filter(
                Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Assassin).and(R::HasSupertype(Supertype::Legendary)),
                }),
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Brotherhood Spy", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Assassin], 1, 3)
    }
}

/// Hired Poisoner — {B} 1/1 Human Assassin, deathtouch.
pub fn hired_poisoner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        ..creature("Hired Poisoner", cost(&[b()]), vec![CreatureType::Human, CreatureType::Assassin], 1, 1)
    }
}

/// Midnight Assassin — {2}{B} 1/2 Vampire Assassin, flying, deathtouch.
pub fn midnight_assassin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        ..creature(
            "Midnight Assassin",
            cost(&[generic(2), b()]),
            vec![CreatureType::Vampire, CreatureType::Assassin],
            1,
            2,
        )
    }
}

// ── Varolz, the Scar-Striped ─────────────────────────────────────────────────

/// Bristlebane Battler — {1}{G} 6/6 Kithkin Soldier, trample, ward {2}. Enters
/// with five -1/-1 counters; another creature of yours entering while it has
/// one removes one.
pub fn bristlebane_battler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, ward(2)],
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::Const(5))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(Predicate::All(
                vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                    Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::WithCounterAtLeast(CounterType::MinusOneMinusOne, 1),
                    },
                ],
            )),
            effect: Effect::RemoveCounter {
                what: Selector::This,
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Bristlebane Battler",
            cost(&[generic(1), g()]),
            vec![CreatureType::Kithkin, CreatureType::Soldier],
            6,
            6,
        )
    }
}

/// Force of Savagery — {2}{G} 8/0 Elemental, trample.
pub fn force_of_savagery() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Force of Savagery", cost(&[generic(2), g()]), vec![CreatureType::Elemental], 8, 0)
    }
}

/// Nyxathid — {1}{B}{B} 7/7 Elemental. As it enters, choose an opponent; it
/// gets -1/-1 for each card in that player's hand.
pub fn nyxathid() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::RememberPlayerOnSource { who: PlayerRef::HostileOpponent }),
        static_abilities: vec![StaticAbility {
            description: "This creature gets -1/-1 for each card in the chosen player's hand.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::HandSizeOf(PlayerRef::ChosenPlayerOfSource),
                per_power: -1,
                per_toughness: -1,
            },
        }],
        ..creature("Nyxathid", cost(&[generic(1), b(), b()]), vec![CreatureType::Elemental], 7, 7)
    }
}

// ── Horde of Notions ─────────────────────────────────────────────────────────

/// Brighthearth Banneret — {1}{R} 1/1 Elemental Warrior. Elemental and
/// Warrior spells you cast cost {1} less. Reinforce 1—{1}{R}.
pub fn brighthearth_banneret() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reinforce(1, cost(&[generic(1), r()]))],
        static_abilities: vec![StaticAbility {
            description: "Elemental spells and Warrior spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::HasCreatureType(CreatureType::Elemental).or(R::HasCreatureType(CreatureType::Warrior)),
                amount: 1,
            },
        }],
        ..creature(
            "Brighthearth Banneret",
            cost(&[generic(1), r()]),
            vec![CreatureType::Elemental, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Torrent Elemental — {4}{U} 3/5 Elemental, flying. Attacking taps every
/// creature the defending player controls. {3}{B/G}{B/G}: put it from exile
/// onto the battlefield tapped, as a sorcery.
pub fn torrent_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Tap { what: Selector::EachPermanent(R::Creature.and(R::ControlledByDefendingPlayer)) },
        }],
        activated_abilities: vec![ActivatedAbility {
            from_exile: true,
            sorcery_speed: true,
            mana_cost: cost(&[generic(3), hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green)]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature("Torrent Elemental", cost(&[generic(4), u()]), vec![CreatureType::Elemental], 3, 5)
    }
}

/// Wispmare — {2}{W} 1/3 Elemental, flying. Enters: destroy target
/// enchantment. Evoke {W}.
pub fn wispmare() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Destroy { what: target_filtered(R::Enchantment) })],
        alternative_cost: Some(evoke(cost(&[w()]))),
        ..creature("Wispmare", cost(&[generic(2), w()]), vec![CreatureType::Elemental], 1, 3)
    }
}
