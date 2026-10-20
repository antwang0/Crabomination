//! Commander: the 2026-10-20 (`01DDeUtc`) one-gap pair closers — each card
//! here is the last one missing from an EDHREC partner-pair average deck.
//! Tests in `tests/recent_b/cmdr_01dd_b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
};
use crate::effect::shortcut::{
    etb, gain_life, investigate, renown, revolt_etb, target_filtered,
};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, w, x};
use crate::sets::{enters_tapped, tap_add, tap_add_any_color};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// "At the beginning of the end step, sacrifice this creature" — every end
/// step, not only its controller's.
fn sacrifice_at_end_step() -> crate::card::TriggeredAbility {
    crate::card::TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
        effect: Effect::SacrificeSource,
    }
}

/// Lightning Serpent — {X}{R} 2/1 Elemental Serpent. Trample, haste; enters
/// with X +1/+0 counters; sacrificed at the beginning of the end step.
pub fn lightning_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        enters_with_counters: Some((CounterType::PlusOnePlusZero, Value::XFromCost)),
        triggered_abilities: vec![sacrifice_at_end_step()],
        ..creature("Lightning Serpent", cost(&[x(), r()]), vec![CreatureType::Elemental, CreatureType::Serpent], 2, 1)
    }
}

/// Enthralling Victor — {3}{R} 3/2 Human Warrior. ETB: gain control of target
/// creature an opponent controls with power 2 or less until end of turn,
/// untap it, and it gains haste until end of turn.
pub fn enthralling_victor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::PowerAtMost(2))),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Enthralling Victor", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 2)
    }
}

/// Hall — land. Enters tapped; {T}: {R} or {W}; {4}, {T}: investigate.
pub fn hall() -> CardDefinition {
    CardDefinition {
        name: "Hall",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(Color::Red),
            tap_add(Color::White),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                effect: investigate(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Sculpted Perfection — {2}{W}{B} enchantment. ETB: incubate 2. Phyrexians
/// you control get +1/+1.
pub fn sculpted_perfection() -> CardDefinition {
    CardDefinition {
        name: "Sculpted Perfection",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Incubate { who: PlayerRef::You, amount: Value::Const(2) })],
        static_abilities: vec![StaticAbility {
            description: "Phyrexians you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Phyrexian)).and(R::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Grim Captain's Call — {2}{B} sorcery. Return a Pirate card from your
/// graveyard to your hand, then do the same for Vampire, Dinosaur, and
/// Merfolk. Each is chosen as it resolves — no targets (CR 608.2d).
pub fn grim_captains_call() -> CardDefinition {
    let fetch = |kind: CreatureType| Effect::ChooseOneAmong {
        what: Selector::CardsInZone {
            who: PlayerRef::You,
            zone: crate::card::Zone::Graveyard,
            filter: R::HasCreatureType(kind),
        },
        chooser: PlayerRef::You,
        chosen: Box::new(Effect::Move {
            what: Selector::SeparatedPile { chosen: true },
            to: ZoneDest::Hand(PlayerRef::You),
        }),
        other: Box::new(Effect::Noop),
    };
    spell(
        "Grim Captain's Call",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::Seq(
            [CreatureType::Pirate, CreatureType::Vampire, CreatureType::Dinosaur, CreatureType::Merfolk]
                .into_iter()
                .map(fetch)
                .collect(),
        ),
    )
}

/// Hunter of Eyeblights — {3}{B}{B} 3/3 Elf Assassin. ETB: a +1/+1 counter on
/// target creature you don't control. {2}{B}, {T}: destroy target creature
/// with a counter on it.
pub fn hunter_of_eyeblights() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::WithAnyCounter)) },
            ..Default::default()
        }],
        ..creature("Hunter of Eyeblights", cost(&[generic(3), b(), b()]), vec![CreatureType::Elf, CreatureType::Assassin], 3, 3)
    }
}

/// Nectar Faerie — {1}{B} 1/1 flying Faerie Wizard. {B}, {T}: target Faerie
/// or Elf gains lifelink until end of turn.
pub fn nectar_faerie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    R::Creature.and(R::HasCreatureType(CreatureType::Faerie).or(R::HasCreatureType(CreatureType::Elf))),
                ),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Nectar Faerie", cost(&[generic(1), b()]), vec![CreatureType::Faerie, CreatureType::Wizard], 1, 1)
    }
}

/// Airdrop Aeronauts — {3}{W}{W} 4/3 flying Dwarf Scout. Revolt — ETB, if a
/// permanent you controlled left the battlefield this turn, gain 5 life.
pub fn airdrop_aeronauts() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![revolt_etb(gain_life(5))],
        ..creature("Airdrop Aeronauts", cost(&[generic(3), w(), w()]), vec![CreatureType::Dwarf, CreatureType::Scout], 4, 3)
    }
}

/// Swell of Courage — {3}{W}{W} instant. Creatures you control get +2/+2 until
/// end of turn. Reinforce X—{X}{W}{W}.
pub fn swell_of_courage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), w(), w()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..spell(
            "Swell of Courage",
            cost(&[generic(3), w(), w()]),
            CardType::Instant,
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Savage Alliance — {2}{R} instant, escalate {1}. Choose one or more:
/// creatures target player controls gain trample until end of turn; 2 damage
/// to target creature; 1 damage to each creature target opponent controls.
pub fn savage_alliance() -> CardDefinition {
    let creatures_of_target = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature };
    CardDefinition {
        escalate: Some(crate::card::Escalate::Mana(cost(&[generic(1)]))),
        ..spell(
            "Savage Alliance",
            cost(&[generic(2), r()]),
            CardType::Instant,
            Effect::ChooseModesCast {
                min: 1,
                max: 3,
                allow_repeats: false,
                modes: vec![
                    Effect::TargetPlayerThen {
                        filter: R::Player,
                        then: Box::new(Effect::GrantKeyword {
                            what: creatures_of_target(),
                            keyword: Keyword::Trample,
                            duration: Duration::EndOfTurn,
                        }),
                    },
                    Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) },
                    Effect::TargetPlayerThen {
                        filter: R::OpponentPlayer,
                        then: Box::new(Effect::DealDamage { to: creatures_of_target(), amount: Value::ONE }),
                    },
                ],
            },
        )
    }
}

/// Honored Hierarch — {G} 1/1 Human Druid. Renown 1. While renowned it has
/// vigilance and "{T}: Add one mana of any color."
pub fn honored_hierarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![renown(1)],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is renowned, it has vigilance.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsRenowned,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Vigilance],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            condition: Some(Predicate::SourceIsRenowned),
            ..tap_add_any_color()
        }],
        ..creature("Honored Hierarch", cost(&[g()]), vec![CreatureType::Human, CreatureType::Druid], 1, 1)
    }
}

/// Bogardan Dragonheart — {2}{R} 2/2 Human Shaman. Sacrifice another
/// creature: until end of turn it becomes a Dragon with base power and
/// toughness 4/4, flying, and haste.
pub fn bogardan_dragonheart() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
            effect: Effect::Seq(vec![
                Effect::BecomeCreatureType {
                    what: Selector::This,
                    creature_types: vec![CreatureType::Dragon],
                    duration: Duration::EndOfTurn,
                },
                Effect::SetBasePT {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Bogardan Dragonheart", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Shaman], 2, 2)
    }
}

/// Cleaving Reaper — {3}{B}{B} 5/3 flying, trample Angel Berserker. Pay 3
/// life: return this card from your graveyard to your hand; activate only if
/// an Angel or Berserker entered under your control this turn.
pub fn cleaving_reaper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            life_cost: 3,
            condition: Some(Predicate::CreatureEnteredThisTurnMatching {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Angel).or(R::HasCreatureType(CreatureType::Berserker)),
            }),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature("Cleaving Reaper", cost(&[generic(3), b(), b()]), vec![CreatureType::Angel, CreatureType::Berserker], 5, 3)
    }
}

/// Apocalypse Demon — {4}{B}{B} */* flying Demon, P/T each the number of
/// cards in your graveyard. At the beginning of your upkeep, tap it unless you
/// sacrifice another creature.
pub fn apocalypse_demon() -> CardDefinition {
    let graveyard = || Value::GraveyardSizeOf(PlayerRef::You);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Apocalypse Demon's power and toughness are each equal to the number of cards in your graveyard.",
            effect: StaticEffect::SelfBasePtFromValue { power: graveyard(), toughness: graveyard() },
        }],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::MaySacrifice {
                description: "Sacrifice another creature to keep Apocalypse Demon untapped?".into(),
                filter: R::Creature.and(R::OtherThanSource),
                count: Value::ONE,
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::Tap { what: Selector::This })),
            },
        }],
        ..creature("Apocalypse Demon", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 0, 0)
    }
}

/// Wren's Run Vanquisher — {1}{G} 3/3 deathtouch Elf Warrior. As an
/// additional cost, reveal an Elf card from your hand or pay {3}.
pub fn wrens_run_vanquisher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::RevealFromHandOrPay {
            filter: R::HasCreatureType(CreatureType::Elf),
            pay: 3,
        }],
        ..creature("Wren's Run Vanquisher", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 3, 3)
    }
}

/// Dinosaur Hunter — {1}{B} 2/2 Human Pirate. Whenever it deals damage to a
/// Dinosaur, destroy that creature.
pub fn dinosaur_hunter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasCreatureType(CreatureType::Dinosaur),
                },
                then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Dinosaur Hunter", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Pirate], 2, 2)
    }
}

/// Pack Attack — {2}{R} instant. Attacking creatures get +X/+0 until end of
/// turn, where X is the number of players being attacked. Draw a card.
pub fn pack_attack() -> CardDefinition {
    spell(
        "Pack Attack",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: Value::OpponentsAttackedThisCombat,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Return from Extinction — {1}{B} sorcery. Choose one: return target creature
/// card from your graveyard to your hand; or return two target creature cards
/// that share a creature type from your graveyard to your hand.
pub fn return_from_extinction() -> CardDefinition {
    let to_hand = |what: Selector| Effect::Move { what, to: ZoneDest::Hand(PlayerRef::You) };
    spell(
        "Return from Extinction",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            to_hand(target_filtered(R::Creature.and(R::InYourGraveyard))),
            Effect::Seq(vec![
                to_hand(target_filtered(R::Creature.and(R::InYourGraveyard))),
                to_hand(Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::InYourGraveyard).and(R::SharesCreatureTypeWithTargetSlot(0)),
                }),
            ]),
        ]),
    )
}

// ── Batch two ───────────────────────────────────────────────────────────────

/// Caravan Escort — {W} 1/1 Human Knight, level up {2}. LEVEL 1-4: 2/2.
/// LEVEL 5+: 5/5 first strike.
pub fn caravan_escort() -> CardDefinition {
    CardDefinition {
        level_bands: vec![
            crate::card::LevelBand { min: 1, max: Some(4), power: 2, toughness: 2, keywords: vec![] },
            crate::card::LevelBand { min: 5, max: None, power: 5, toughness: 5, keywords: vec![Keyword::FirstStrike] },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sorcery_speed: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Level, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Caravan Escort", cost(&[w()]), vec![CreatureType::Human, CreatureType::Knight], 1, 1)
    }
}

/// Timber Wolves — {G} 1/1 Wolf with banding (CR 702.22).
pub fn timber_wolves() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Banding],
        ..creature("Timber Wolves", cost(&[g()]), vec![CreatureType::Wolf], 1, 1)
    }
}

/// Elegant Entourage — {3}{G} 4/4 Elf Druid. Alliance — whenever another
/// creature you control enters, target creature other than this one gets
/// +1/+1 and gains trample until end of turn.
pub fn elegant_entourage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Elegant Entourage", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Druid], 4, 4)
    }
}

/// Jodah's Codex — {5} artifact — Book. Domain — {5}, {T}: draw a card; costs
/// {1} less for each basic land type among lands you control.
pub fn jodahs_codex() -> CardDefinition {
    CardDefinition {
        name: "Jodah's Codex",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Book], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(5)]),
            cost_reduction_value: Some(Value::DomainCount(PlayerRef::You)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Timely Interference — {U} instant, kicker {1}{R}. Target creature gets
/// -1/-0 until end of turn; kicked, it also blocks this turn if able. Draw a
/// card.
pub fn timely_interference() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), r()]))],
        ..spell(
            "Timely Interference",
            cost(&[crate::mana::u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-1),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::MustBlock,
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        )
    }
}

/// Magus of the Jar — {3}{U}{U} 3/3 Human Wizard. {T}, sacrifice it: each
/// player exiles their hand face down and draws seven; at the next end step
/// each discards their hand and gets the exiled cards back (Memory Jar).
pub fn magus_of_the_jar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::EachPlayerExilesHandDrawsSeven,
            ..Default::default()
        }],
        ..creature(
            "Magus of the Jar",
            cost(&[generic(3), crate::mana::u(), crate::mana::u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Grim Giganotosaurus — {5}{B}{G} 10/10 Dinosaur. {10}{B}{G}: monstrosity
/// 10, costing {1} less per creature with power 4 or greater your opponents
/// control. When it becomes monstrous, destroy all other artifacts and
/// creatures.
pub fn grim_giganotosaurus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            cost_reduction_value: Some(Value::count(Selector::EachPermanent(
                R::Creature.and(R::ControlledByOpponent).and(R::PowerAtLeast(4)),
            ))),
            ..crate::effect::shortcut::monstrosity(cost(&[generic(10), b(), g()]), 10)
        }],
        triggered_abilities: vec![crate::effect::shortcut::on_becomes_monstrous(Effect::Destroy {
            what: Selector::EachPermanent(R::Artifact.or(R::Creature).and(R::OtherThanSource)),
        })],
        ..creature("Grim Giganotosaurus", cost(&[generic(5), b(), g()]), vec![CreatureType::Dinosaur], 10, 10)
    }
}

/// Emptiness — {4}{W/B}{W/B} 3/5 Elemental Incarnation, evoke {W/B}{W/B}.
/// ETB, if {W}{W} was spent: return target creature card with mana value 3
/// or less from your graveyard to the battlefield. ETB, if {B}{B} was spent:
/// three -1/-1 counters on up to one target creature.
pub fn emptiness() -> CardDefinition {
    use crate::mana::hybrid;
    let wb = || hybrid(Color::White, Color::Black);
    let spent = |color: Color| Predicate::SourceCastWithColorSpent { color, at_least: 2 };
    CardDefinition {
        alternative_cost: Some(crate::effect::shortcut::evoke(ManaCost::new(vec![wb(), wb()]))),
        triggered_abilities: vec![
            crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(spent(Color::White)),
                effect: Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3))),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
            crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(spent(Color::Black)),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::MinusOneMinusOne,
                        amount: Value::Const(3),
                    }),
                },
            },
        ],
        ..creature(
            "Emptiness",
            ManaCost::new(vec![generic(4), wb(), wb()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            3,
            5,
        )
    }
}

/// Flitting Guerrilla — {2}{B} 2/2 flying Faerie Rogue. When it dies, each
/// player mills two; then you may exile this card, and when you do, put
/// target creature or battle card from your graveyard on top of your library.
pub fn flitting_guerrilla() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Seq(vec![
            Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
            Effect::MayDo {
                description: "Exile Flitting Guerrilla to put a creature or battle card on top of your library?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                    Effect::Reflexive {
                        body: Box::new(Effect::Move {
                            what: target_filtered(R::Creature.or(R::HasCardType(CardType::Battle)).and(R::InYourGraveyard)),
                            to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
                        }),
                    },
                ])),
            },
        ]))],
        ..creature("Flitting Guerrilla", cost(&[generic(2), b()]), vec![CreatureType::Faerie, CreatureType::Rogue], 2, 2)
    }
}

/// Jace's Mindseeker — {4}{U}{U} 4/4 flying Fish Illusion. ETB: target
/// opponent mills five; you may cast an instant or sorcery spell from among
/// them without paying its mana cost.
pub fn jaces_mindseeker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::Mill { who: target_filtered(R::OpponentPlayer), amount: Value::Const(5) },
            Effect::CastAnyOrderWithoutPaying {
                what: Selector::LastMoved,
                source_zone: crate::card::Zone::Graveyard,
                filter: Some(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                cap: Some(Value::ONE),
                total_mana_value: None,
            },
        ]))],
        ..creature(
            "Jace's Mindseeker",
            cost(&[generic(4), crate::mana::u(), crate::mana::u()]),
            vec![CreatureType::Fish, CreatureType::Illusion],
            4,
            4,
        )
    }
}

// ── Batch three: the two new primitives and an alternative cost ─────────────

/// Keen-Eared Sentry — {1}{W} 2/1 Human Soldier. You have hexproof. Each
/// opponent can't venture into the dungeon more than once each turn.
pub fn keen_eared_sentry() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility { description: "You have hexproof.", effect: StaticEffect::ControllerHasHexproof },
            StaticAbility {
                description: "Each opponent can't venture into the dungeon more than once each turn.",
                effect: StaticEffect::OpponentsVentureOncePerTurn,
            },
        ],
        ..creature("Keen-Eared Sentry", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 1)
    }
}

/// Once Upon a Time — {1}{G} instant. Free if it's the first spell you've
/// cast this game. Look at the top five cards of your library; you may put a
/// creature or land card from among them into your hand, the rest on the
/// bottom in a random order.
pub fn once_upon_a_time() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: ManaCost::default(),
            condition: Some(Predicate::FirstSpellYouCastThisGame),
            ..Default::default()
        }),
        ..spell(
            "Once Upon a Time",
            cost(&[generic(1), g()]),
            CardType::Instant,
            Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(R::Creature.or(R::Land)),
                optional: true,
                ..Default::default()
            })),
        )
    }
}

/// Veinfire Borderpost — {1}{B}{R} artifact. You may pay {1} and return a
/// basic land you control to its owner's hand rather than pay its mana cost.
/// Enters tapped; {T}: {B} or {R}.
pub fn veinfire_borderpost() -> CardDefinition {
    CardDefinition {
        name: "Veinfire Borderpost",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Artifact],
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(1)]),
            return_to_hand: Some((R::IsBasicLand, 1)),
            ..Default::default()
        }),
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(Color::Black), tap_add(Color::Red)],
        ..Default::default()
    }
}

// ── Batch four ──────────────────────────────────────────────────────────────

/// Scout's Warning — {W} instant. The next creature card you play this turn
/// can be played as though it had flash. Draw a card.
pub fn scouts_warning() -> CardDefinition {
    spell(
        "Scout's Warning",
        cost(&[w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::NextCreatureSpellHasFlashThisTurn,
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Wolf of Devil's Breach — {3}{R}{R} 5/5 Elemental Wolf. Whenever it
/// attacks, you may pay {1}{R} and discard a card; if you do, it deals damage
/// equal to the discarded card's mana value to target creature or
/// planeswalker.
pub fn wolf_of_devils_breach() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::If {
            cond: Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::ONE),
            then: Box::new(Effect::MayPay {
                description: "Pay {1}{R} and discard a card to deal damage equal to its mana value?".into(),
                mana_cost: cost(&[generic(1), r()]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    Effect::DealDamage {
                        to: target_filtered(R::Creature.or(R::Planeswalker)),
                        amount: Value::ManaValueOf(Box::new(Selector::DiscardedThisResolution { filter: R::Any })),
                    },
                ])),
                else_: None,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Wolf of Devil's Breach",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Wolf],
            5,
            5,
        )
    }
}

/// Realm Razer — {3}{R}{G}{W} 4/2 Beast. ETB: exile all lands; when it leaves
/// the battlefield, the exiled cards return to the battlefield tapped under
/// their owners' control. (Modeled as one "until it leaves" exile, CR
/// 610.3: a Razer gone before its ETB resolves exiles nothing, where the
/// printed two-trigger wording would exile the lands for good.)
pub fn realm_razer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: Selector::EachPermanent(R::Land),
            return_to: crate::card::ExileReturnZone::BattlefieldTapped,
        })],
        ..creature("Realm Razer", cost(&[generic(3), r(), g(), w()]), vec![CreatureType::Beast], 4, 2)
    }
}

/// Out of Bounds — {3}{U} instant. Assist. Counter target spell. (Assist —
/// another player paying up to {3} of the cost — is not modeled.)
pub fn out_of_bounds() -> CardDefinition {
    spell(
        "Out of Bounds",
        cost(&[generic(3), crate::mana::u()]),
        CardType::Instant,
        Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
    )
}

/// Consecrate Land — {W} Aura, enchant land. Enchanted land has
/// indestructible. (Its "can't be enchanted by other Auras" is not modeled.)
pub fn consecrate_land() -> CardDefinition {
    CardDefinition {
        name: "Consecrate Land",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(crate::card::EquipBonus { keywords: vec![Keyword::Indestructible], ..Default::default() }),
        ..Default::default()
    }
}

// ── Batch five: craft with one of each ──────────────────────────────────────

/// Throne of the Grim Captain // The Grim Captain — {2} legendary artifact.
/// {T}: mill two. Craft with a Dinosaur, a Merfolk, a Pirate, and a Vampire
/// {4}. Back: 7/7 legendary Skeleton Spirit Pirate with menace, trample,
/// lifelink, hexproof; whenever it attacks, each opponent sacrifices a
/// nonland permanent of their choice, then you may put an exiled creature
/// card used to craft it onto the battlefield under your control tapped and
/// attacking.
pub fn throne_of_the_grim_captain() -> CardDefinition {
    let kinds = [CreatureType::Dinosaur, CreatureType::Merfolk, CreatureType::Pirate, CreatureType::Vampire];
    let captain = CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        color_indicator: vec![Color::Black],
        keywords: vec![Keyword::Menace, Keyword::Trample, Keyword::Lifelink, Keyword::Hexproof],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Permanent.and(R::Not(Box::new(R::Land))),
            },
            Effect::MayDo {
                description: "Put an exiled creature card used to craft The Grim Captain onto the battlefield tapped and attacking?".into(),
                body: Box::new(Effect::ChooseOneAmong {
                    what: Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Exile,
                        filter: R::Creature.and(R::ExiledWithSource),
                    },
                    chooser: PlayerRef::You,
                    chosen: Box::new(Effect::Seq(vec![
                        Effect::ClearLastMoved,
                        Effect::Move {
                            what: Selector::SeparatedPile { chosen: true },
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                        },
                        Effect::JoinCombatAttacking { what: Selector::LastMoved },
                    ])),
                    other: Box::new(Effect::Noop),
                }),
            },
        ]))],
        ..creature(
            "The Grim Captain",
            ManaCost::default(),
            vec![CreatureType::Skeleton, CreatureType::Spirit, CreatureType::Pirate],
            7,
            7,
        )
    };
    CardDefinition {
        name: "Throne of the Grim Captain",
        cost: cost(&[generic(2)]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
            crate::effect::shortcut::craft_each(
                cost(&[generic(4)]),
                kinds.into_iter().map(R::HasCreatureType).collect(),
            ),
        ],
        back_face: Some(Box::new(captain)),
        ..Default::default()
    }
}

// ── Batch six ───────────────────────────────────────────────────────────────

/// Narset Transcendent — {2}{W}{U} planeswalker, loyalty 6. +1: look at the
/// top card of your library; a noncreature, nonland card may be revealed and
/// put into your hand. −2: when you next cast an instant or sorcery spell
/// from your hand this turn, it gains rebound. −9: an emblem with "Your
/// opponents can't cast noncreature spells."
pub fn narset_transcendent() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    let loyalty = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    let spell_kind = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Narset Transcendent",
        cost: cost(&[generic(2), w(), crate::mana::u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Narset], ..Default::default() },
        base_loyalty: 6,
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: top(),
                        filter: R::Not(Box::new(R::Creature)).and(R::Not(Box::new(R::Land))),
                    },
                    then: Box::new(Effect::MayDo {
                        description: "Reveal it and put it into your hand?".into(),
                        body: Box::new(Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ),
            loyalty(
                -2,
                Effect::OnYourNextSpellMatchingThisTurn {
                    filter: spell_kind().and(R::WasCastFromHand),
                    body: Box::new(Effect::GrantReboundToSpell { what: Selector::TriggerSource }),
                },
            ),
            loyalty(
                -9,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Narset Transcendent".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "Your opponents can't cast noncreature spells.",
                        effect: StaticEffect::OpponentsCantCastMatching { filter: R::Not(Box::new(R::Creature)) },
                    }],
                },
            ),
        ],
        ..Default::default()
    }
}

/// Elite Arcanist — {3}{U} 1/1 Human Wizard. ETB: you may exile an instant
/// card from your hand. {X}, {T}: copy the exiled card and you may cast the
/// copy without paying its mana cost; X is the exiled card's mana value.
pub fn elite_arcanist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseOneAmong {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Hand,
                filter: R::HasCardType(CardType::Instant),
            },
            chooser: PlayerRef::You,
            chosen: Box::new(Effect::Move {
                what: Selector::SeparatedPile { chosen: true },
                to: ZoneDest::ExileWithSourceStamp,
            }),
            other: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost_increase: Some(Value::ManaValueOf(Box::new(Selector::CardExiledWithSource))),
            condition: Some(Predicate::SelectorCountAtLeast { sel: Selector::CardExiledWithSource, n: Value::ONE }),
            effect: Effect::CastWithoutPayingImmediate {
                what: Selector::CardExiledWithSource,
                source_zone: crate::card::Zone::Exile,
                exile_after: false,
                copy: true,
                reduce_generic: 0,
                pay_own_cost: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Elite Arcanist",
            cost(&[generic(3), crate::mana::u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Minion Reflector — {5} artifact. Whenever a nontoken creature you control
/// enters, you may pay {2}; if you do, create a token copy of it with haste,
/// sacrificed at the beginning of the next end step.
pub fn minion_reflector() -> CardDefinition {
    CardDefinition {
        name: "Minion Reflector",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::Not(Box::new(R::IsToken))),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {2} to copy it?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::CreateTokenCopiesHasteSac {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    exile: false,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Phyrexian Devourer — {6} 1/1 artifact creature — Phyrexian Construct.
/// When its power is 7 or greater, sacrifice it (CR 603.8). Exile the top
/// card of your library: put X +1/+1 counters on it, where X is the exiled
/// card's mana value.
pub fn phyrexian_devourer() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        state_trigger: Some(crate::card::StateTriggeredAbility {
            condition: Predicate::ValueAtLeast(Value::PowerOf(Box::new(Selector::This)), Value::Const(7)),
            effect: Effect::SacrificeSource,
        }),
        activated_abilities: vec![ActivatedAbility {
            exile_top_cost: 1,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ExiledForCostManaValue,
            },
            ..Default::default()
        }],
        ..creature("Phyrexian Devourer", cost(&[generic(6)]), vec![CreatureType::Phyrexian, CreatureType::Construct], 1, 1)
    }
}
