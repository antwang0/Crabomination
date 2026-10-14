//! Commander: gap cards of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), eleventh file. Tests in `tests/recent_b/cmdr_rank4k_k.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::game::types::TurnStep;
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{ChainCopyCost, Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{ManaCost, b, cost, generic, r, u};

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
