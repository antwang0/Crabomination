//! Commander: two-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), fourth file. Tests in `tests/recent_b/cmdr_rank6k_d.rs`.

use crate::card::{
    AlternativeCost, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{ManaCost, b, cost, generic, u};
use std::sync::Arc;

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

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

// ── Siani, Eye of the Storm // Eligeth, Crossroads Augur ─────────────────────

/// Siani, Eye of the Storm — {3}{U} legendary 3/2 flying Djinn Monk; partner.
/// Whenever she attacks, scry X, X the number of attacking creatures with
/// flying.
pub fn siani_eye_of_the_storm() -> CardDefinition {
    let flyers = Selector::EachPermanent(
        R::Creature
            .and(R::IsAttacking)
            .and(R::HasKeyword(Keyword::Flying)),
    );
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Partner],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::CountOf(Box::new(flyers)),
        })],
        ..legend(
            "Siani, Eye of the Storm",
            cost(&[generic(3), u()]),
            vec![CreatureType::Djinn, CreatureType::Monk],
            3,
            2,
        )
    }
}

/// Eligeth, Crossroads Augur — {4}{U}{U} legendary 5/6 flying Sphinx; partner.
/// If you would scry a number of cards, draw that many cards instead.
pub fn eligeth_crossroads_augur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Partner],
        static_abilities: vec![StaticAbility {
            description: "If you would scry a number of cards, draw that many cards instead.",
            effect: StaticEffect::ScryBecomesDraw,
        }],
        ..legend(
            "Eligeth, Crossroads Augur",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Sphinx],
            5,
            6,
        )
    }
}

// ── Klaw, Master of Sound / Black Widow, Super Spy ───────────────────────────

/// Black Widow, Super Spy — {1}{B} legendary 2/1 menace Human Spy Hero.
/// Whenever she deals combat damage to a player, that player exiles from the
/// top of their library until a nonland card; you may put a +1/+1 counter on
/// her; if you don't, you may cast that card this turn, spending mana as
/// though it were any type.
pub fn black_widow_super_spy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                // The passed-over lands stay exiled; the nonland card is
                // `LastMoved` for the cast grant.
                Effect::RevealUntilFind {
                    who: PlayerRef::TriggerEventPlayer,
                    find: R::Nonland,
                    to: ZoneDest::Exile,
                    cap: Value::Const(250),
                    life_per_revealed: 0,
                    miss_dest: crate::effect::RevealMissDest::Exile,
                },
                // The counter and the cast are exclusive ("if you don't").
                Effect::MayDoElse {
                    description: "Put a +1/+1 counter on Black Widow (instead of the cast)?".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: true,
                    }),
                },
            ]),
        }],
        ..legend(
            "Black Widow, Super Spy",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            2,
            1,
        )
    }
}

/// Petty Larceny — {3}{B} sorcery; freerunning {1}{B}. Look at the top two
/// cards of target opponent's library and exile them face down; you may play
/// them while they stay exiled, spending mana as though it were any type.
/// Create a Treasure. The face-down exile is `ExileFaceDown` (CR 406.3).
pub fn petty_larceny() -> CardDefinition {
    CardDefinition {
        name: "Petty Larceny",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::ExileFaceDown {
                    body: Box::new(Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::Target(0),
                        count: Value::Const(2),
                        duration: MayPlayDuration::WhileExiled,
                        pay_any_color: true,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    }),
                }),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        ]),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), b()]),
            condition: Some(Predicate::FreerunningUnlocked {
                who: PlayerRef::You,
            }),
            freerunning: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}
