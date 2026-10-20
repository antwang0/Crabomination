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
