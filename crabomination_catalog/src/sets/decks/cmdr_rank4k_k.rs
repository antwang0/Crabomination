//! Commander: gap cards of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), eleventh file. Tests in `tests/recent_b/cmdr_rank4k_k.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword,
    MayPlayDuration,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::game::types::TurnStep;
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{ChainCopyCost, Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

// ── Witch-king, Bringer of Ruin / Interceptor, Shadow's Hound ────────────────

/// Gollum, Scheming Guide — {1}{B} legendary 2/1 Halfling Horror. Whenever it
/// attacks, look at the top two cards of your library and put them back in any
/// order, then choose land or nonland; an opponent guesses whether the top card
/// is the chosen kind, and it's revealed. Right: remove Gollum from combat.
/// Wrong: draw a card and Gollum can't be blocked this turn.
pub fn gollum_scheming_guide() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::RearrangeTop {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::GuessTopCardKind {
                right: Box::new(Effect::RemoveFromCombat {
                    what: Selector::This,
                }),
                wrong: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        ]))],
        ..creature(
            "Gollum, Scheming Guide",
            cost(&[generic(1), b()]),
            vec![CreatureType::Halfling, CreatureType::Horror],
            2,
            1,
        )
    }
}

/// Poison-Blade Mentor — {1}{B} 2/1 deathtouch Human Assassin. Whenever it
/// attacks, another target Assassin you control gains deathtouch until end of
/// turn.
pub fn poison_blade_mentor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(
                R::HasCreatureType(CreatureType::Assassin)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
            ),
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Poison-Blade Mentor",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            2,
            1,
        )
    }
}

// ── Denethor, Stone Seer ─────────────────────────────────────────────────────

/// Mudslide — {2}{R} enchantment. Creatures without flying don't untap during
/// their controllers' untap steps. At the beginning of each player's upkeep,
/// that player may pay {2} for each of their tapped creatures without flying;
/// untap each one paid for.
pub fn mudslide() -> CardDefinition {
    let grounded = || R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))));
    CardDefinition {
        name: "Mudslide",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures without flying don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntapGlobal {
                applies_to: Selector::EachPermanent(grounded()),
                condition: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::ForEach {
                selector: Selector::ControlledBy {
                    who: PlayerRef::ActivePlayer,
                    filter: grounded().and(R::Tapped),
                },
                body: Box::new(Effect::MayPayBy {
                    who: PlayerRef::ActivePlayer,
                    description: "Pay {2} to untap this creature?".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(Effect::Untap {
                        what: Selector::TriggerSource,
                        up_to: None,
                    }),
                    else_: None,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Myojin of Seeing Winds ───────────────────────────────────────────────────

/// String of Disappearances — {U} instant. Return target creature to its
/// owner's hand. Then that creature's controller may pay {U}{U} to copy this
/// spell and choose a new target for the copy.
pub fn string_of_disappearances() -> CardDefinition {
    CardDefinition {
        name: "String of Disappearances",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::MayCopyThisSpell {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                cost: ChainCopyCost::Mana(cost(&[u(), u()])),
            },
        ]),
        ..Default::default()
    }
}

// ── Wiccan, Young Avenger ────────────────────────────────────────────────────

/// Asgardian Inspiration — {R} sorcery. Exile the top card of your library;
/// you may play it this turn. From your graveyard: whenever a source you
/// control deals noncombat damage to an opponent, you may pay {2} to return
/// this card to your hand.
pub fn asgardian_inspiration() -> CardDefinition {
    CardDefinition {
        name: "Asgardian Inspiration",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfThisTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PlayerDealtNoncombatDamage,
                EventScope::FromYourGraveyardSourceDamagedOpponent,
            ),
            effect: Effect::MayPay {
                description: "Pay {2} to return Asgardian Inspiration to your hand?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

// ── Kodama of the North Tree ─────────────────────────────────────────────────

/// Blessing of Frost — {3}{G} snow sorcery. Distribute X +1/+1 counters among
/// any number of creatures you control, where X is the amount of {S} spent to
/// cast it. Then draw a card for each creature you control with power 4 or
/// greater.
pub fn blessing_of_frost() -> CardDefinition {
    CardDefinition {
        name: "Blessing of Frost",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DistributeCountersAmong {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                total: Value::SnowManaSpentToCastSource,
                counter: CounterType::PlusOnePlusOne,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                ))),
            },
        ]),
        ..Default::default()
    }
}

// ── The Duke, Rebel Sentry ───────────────────────────────────────────────────

/// Swallow Whole — {W} sorcery. As an additional cost, tap an untapped
/// creature you control. Exile target tapped creature. Put a +1/+1 counter on
/// the creature tapped to pay the additional cost.
pub fn swallow_whole() -> CardDefinition {
    CardDefinition {
        name: "Swallow Whole",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::TapPermanents {
            filter: R::Creature,
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::Creature.and(R::Tapped)),
            },
            Effect::AddCounter {
                what: Selector::CostTappedPermanentOfSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}
