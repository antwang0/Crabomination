//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, fifth file (`cmdr_edhrec4` is the
//! fourth). Lord of the Nazgûl's Wraiths and Ring spells; Leonardo and
//! Michelangelo's Turtles, Ninjas and Mutagen; Norman Osborn's Villains,
//! connive and mayhem; Reaper King's Scarecrows. Tests in
//! `tests/recent_b/cmdr_edhrec5.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, sneak, target_filtered};
use crate::effect::{ActivatedAbility, DelayedTriggerKind, Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost};
use crate::game::types::TurnStep;
use crabomination_base::tokens::{mutagen_token, treasure_token};
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

fn tempt() -> Effect {
    Effect::RingTempts { who: PlayerRef::You }
}

fn target_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::Target(0), filter }
}

// ── Lord of the Nazgûl (UB) ──────────────────────────────────────────────────

/// Gandalf, Friend of the Shire — {3}{U} 2/4 flash. Your sorceries have
/// flash; the Ring tempting you onto a bearer other than Gandalf draws a card.
pub fn gandalf_friend_of_the_shire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "You may cast sorcery spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: R::HasCardType(CardType::Sorcery) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            // CR 701.54c — the chosen bearer is the event's subject; no
            // creature chosen leaves the player there, which isn't "other".
            event: EventSpec::new(EventKind::RingTempted, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend(
            "Gandalf, Friend of the Shire",
            cost(&[generic(3), u()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            2,
            4,
        )
    }
}

/// Ringwraiths — {4}{B}{B} 5/5. On entry an opponent's creature gets -3/-3
/// and a legendary one's controller loses 3; the Ring tempting you returns it
/// from your graveyard to your hand.
pub fn ringwraiths() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: target_is(R::HasSupertype(Supertype::Legendary)),
                    then: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                        amount: Value::Const(3),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::RingTempted, EventScope::FromYourGraveyard),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            },
        ],
        ..creature(
            "Ringwraiths",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Wraith, CreatureType::Knight],
            5,
            5,
        )
    }
}

/// Sauron, the Necromancer — {3}{B}{B} 4/4 menace. Attacking exiles a creature
/// card from your graveyard for a tapped, attacking 3/3 black menace Wraith
/// copy of it, exiled at the next end step unless Sauron is your Ring-bearer.
pub fn sauron_the_necromancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            // CR 707.2 — the copy reads the card's printed values, so it is
            // minted before the card leaves; the exile follows in the same
            // resolution.
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::Creature.from_your_graveyard()),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: Some((3, 3)),
                override_colors: Some(vec![Color::Black]),
                enters_tapped: true,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![Keyword::Menace],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
            Effect::Exile { what: Selector::Target(0) },
            // CR 707.9b — "a Wraith instead of its other creature types".
            Effect::AmendCopiableValues {
                what: Selector::LastCreatedToken,
                name: None,
                set_creature_types: Some(vec![CreatureType::Wraith]),
                add_creature_types: vec![],
                legendary: false,
                add_colors: vec![],
                set_card_types: None,
            },
            Effect::JoinCombatAttackingChosen {
                cleanup: crate::effect::AttackingTokenCleanup::None,
                what: Selector::LastCreatedToken,
            },
            Effect::DelayUntilWithCapture {
                kind: DelayedTriggerKind::NextEndStep,
                capture: Selector::LastCreatedToken,
                body: Box::new(Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::This, filter: R::IsRingBearer },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::Exile { what: Selector::Target(0) }),
                }),
            },
        ]))],
        ..legend(
            "Sauron, the Necromancer",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Avatar, CreatureType::Horror],
            4,
            4,
        )
    }
}

/// Witch-king of Angmar — {3}{B}{B} 5/3 flying. Combat damage to you makes
/// each opponent sacrifice a creature that dealt it, and the Ring tempts you;
/// discard a card: indestructible until end of turn, then tap it.
pub fn witch_king_of_angmar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::SelfSource).once_per_batch(),
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Creature.and(R::CombatDamagedSourceControllerThisTurn),
                },
                tempt(),
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Witch-king of Angmar",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Wraith, CreatureType::Noble],
            5,
            3,
        )
    }
}

/// Witch-king, Bringer of Ruin — {4}{B}{B} 5/3 flying. Attacking makes the
/// defending player sacrifice a creature with the least power they control.
pub fn witch_king_bringer_of_ruin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::DefendingPlayer),
            count: Value::ONE,
            filter: R::Creature.and(R::HasLeastPowerAmongControlled(Box::new(R::Creature))),
        })],
        ..legend(
            "Witch-king, Bringer of Ruin",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Wraith, CreatureType::Noble],
            5,
            3,
        )
    }
}

/// Glorious Gale — {1}{U} instant. Counter target creature spell; a legendary
/// one tempts you with the Ring.
pub fn glorious_gale() -> CardDefinition {
    spell(
        "Glorious Gale",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Creature))),
            },
            Effect::If {
                cond: target_is(R::HasSupertype(Supertype::Legendary)),
                then: Box::new(tempt()),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Sauron's Ransom — {1}{U}{B} instant. An opponent splits your top four into
/// a pile for your hand and one for your graveyard; the Ring tempts you.
pub fn saurons_ransom() -> CardDefinition {
    spell(
        "Sauron's Ransom",
        cost(&[generic(1), u(), b()]),
        CardType::Instant,
        Effect::Seq(vec![Effect::FactOrFiction { count: Value::Const(4), to_bottom: false }, tempt()]),
    )
}

/// Ringsight — {1}{U}{B} sorcery. The Ring tempts you, then search for a card
/// sharing a color with a legendary creature you control.
pub fn ringsight() -> CardDefinition {
    spell(
        "Ringsight",
        cost(&[generic(1), u(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            tempt(),
            Effect::Search {
                who: PlayerRef::You,
                filter: R::SharesColorWithYourPermanentMatching(Box::new(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                )),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
    )
}

/// Sam's Desperate Rescue — {B} sorcery. Return a creature card from your
/// graveyard to your hand; the Ring tempts you.
pub fn sams_desperate_rescue() -> CardDefinition {
    spell(
        "Sam's Desperate Rescue",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            tempt(),
        ]),
    )
}

// ── Leonardo, the Balance // Michelangelo, the Heart (WG) ───────────────────

/// A Mutant, Ninja or Turtle.
fn tmnt() -> R {
    R::HasCreatureType(CreatureType::Mutant)
        .or(R::HasCreatureType(CreatureType::Ninja))
        .or(R::HasCreatureType(CreatureType::Turtle))
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn mutagens(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(mutagen_token()) }
}

fn grant(description: &'static str, filter: R, keyword: Keyword) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::GrantKeyword { applies_to: Selector::EachPermanent(filter), keyword },
    }
}

fn turtle(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    legend(name, mana, vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle], p, t)
}

/// The Ooze — {2} legendary artifact. A creature of yours leaving with
/// +1/+1 counters leaves that many Mutagens; {T}: exile a card from a
/// graveyard and make a Mutagen.
pub fn the_ooze() -> CardDefinition {
    CardDefinition {
        name: "The Ooze",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.10a — the counters are read as it last existed.
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne)),
                },
            ),
            effect: mutagens(Value::CountersOn {
                what: Box::new(Selector::TriggerSource),
                kind: CounterType::PlusOnePlusOne,
            }),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::Any.from_any_graveyard()), to: ZoneDest::Exile },
                mutagens(Value::ONE),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dark Leo & Shredder — {W}{B} 1/3, sneak {W}{B}. Attacking Ninjas you
/// control have deathtouch; combat damage to a player makes a 1/1 Ninja, then
/// at five Ninjas that player loses half their life, rounded up.
pub fn dark_leo_shredder() -> CardDefinition {
    let ninjas = || yours(R::Creature.and(R::HasCreatureType(CreatureType::Ninja)));
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[w(), b()]))],
        alternative_cost: Some(sneak(cost(&[w(), b()]))),
        static_abilities: vec![grant(
            "Attacking Ninjas you control have deathtouch.",
            ninjas().and(R::IsAttacking),
            Keyword::Deathtouch,
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition {
                        name: "Ninja".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Black],
                        subtypes: Subtypes { creature_types: vec![CreatureType::Ninja], ..Default::default() },
                        ..Default::default()
                    }),
                },
                Effect::If {
                    cond: Predicate::SelectorCountAtLeast { sel: Selector::EachPermanent(ninjas()), n: Value::Const(5) },
                    then: Box::new(Effect::LoseHalfLife { who: Selector::Player(PlayerRef::Target(0)), rounded_up: true }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend(
            "Dark Leo & Shredder",
            cost(&[w(), b()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle, CreatureType::Human],
            1,
            3,
        )
    }
}

/// Michelangelo, Mutant BFF — {2}{G}{G} 4/4. Your creatures with counters
/// can't be blocked by more than one creature; entering or attacking makes a
/// Mutagen.
pub fn michelangelo_mutant_bff() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![grant(
            "Each creature you control with a counter on it can't be blocked by more than one creature.",
            yours(R::Creature.and(R::WithAnyCounter)),
            Keyword::CantBeBlockedByMoreThanOne,
        )],
        triggered_abilities: vec![etb(mutagens(Value::ONE)), on_attack(mutagens(Value::ONE))],
        ..turtle("Michelangelo, Mutant BFF", cost(&[generic(2), g(), g()]), 4, 4)
    }
}

/// Michelangelo, Weirdness to 11 — {1}{G} 1/1. A Mutagen on entry; +1/+1
/// counters on your creatures come one extra (the Hardened Scales rider).
pub fn michelangelo_weirdness_to_11() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a creature you control, that many plus one \
                          +1/+1 counters are put on it instead.",
            effect: StaticEffect::ExtraPlusOneCounters,
        }],
        triggered_abilities: vec![etb(mutagens(Value::ONE))],
        ..turtle("Michelangelo, Weirdness to 11", cost(&[generic(1), g()]), 1, 1)
    }
}

/// Mikey & Don, Party Planners — {2}{G/U}{G/U} 3/3, ward {2}. Look at the top
/// of your library; play lands and cast Mutant, Ninja or Turtle spells from
/// it; a creature cast that way enters with an extra +1/+1 counter.
pub fn mikey_don_party_planners() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may play lands and cast Mutant, Ninja, or Turtle spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Land.or(tmnt()) },
            },
            StaticAbility {
                description: "If you cast a creature spell this way, that creature enters with an additional +1/+1 \
                              counter on it.",
                effect: StaticEffect::MatchingEntersWithExtraCounters {
                    filter: R::Creature.and(tmnt()).and(R::WasCastFromLibrary),
                    kind: CounterType::PlusOnePlusOne,
                    amount: 1,
                },
            },
        ],
        ..turtle(
            "Mikey & Don, Party Planners",
            cost(&[generic(2), hybrid(Color::Green, Color::Blue), hybrid(Color::Green, Color::Blue)]),
            3,
            3,
        )
    }
}

/// Mikey & Leo, Chaos & Order — {G/W}{G/W} 2/2. Putting a counter on a
/// creature you control draws a card, once each turn.
pub fn mikey_leo_chaos_order() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            // CR 122.6 — keyed on who put the counter.
            event: EventSpec::new(EventKind::AnyCounterAdded, EventScope::YouPutCounters)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: yours(R::Creature) })
                .once_per_turn(),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..turtle(
            "Mikey & Leo, Chaos & Order",
            cost(&[hybrid(Color::Green, Color::White), hybrid(Color::Green, Color::White)]),
            2,
            2,
        )
    }
}

/// Mutagen Man, Living Ooze — {X}{G}{G} 2/3 trample. Your artifact tokens'
/// activated abilities cost {1} less; enters with X Mutagens.
pub fn mutagen_man_living_ooze() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Activated abilities of artifact tokens you control cost {1} less to activate.",
            effect: StaticEffect::MatchingActivatedAbilitiesCostLess { filter: R::Artifact.and(R::IsToken), amount: 1 },
        }],
        triggered_abilities: vec![etb(mutagens(Value::XFromCost))],
        ..legend(
            "Mutagen Man, Living Ooze",
            cost(&[x(), g(), g()]),
            vec![CreatureType::Ooze, CreatureType::Mutant],
            2,
            3,
        )
    }
}

/// Raphael, the Nightwatcher — {2}{R}{R} 2/3, sneak {1}{R}{R}. Attacking
/// creatures you control have double strike.
pub fn raphael_the_nightwatcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[generic(1), r(), r()]))],
        alternative_cost: Some(sneak(cost(&[generic(1), r(), r()]))),
        static_abilities: vec![grant(
            "Attacking creatures you control have double strike.",
            yours(R::Creature.and(R::IsAttacking)),
            Keyword::DoubleStrike,
        )],
        ..turtle("Raphael, the Nightwatcher", cost(&[generic(2), r(), r()]), 2, 3)
    }
}

/// Splinter, Radical Rat — {1}{W/B}{W/B} 2/4. Your Ninjas' triggered
/// abilities trigger an additional time; {1}{U}: a Ninja can't be blocked
/// this turn.
pub fn splinter_radical_rat() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a triggered ability of a Ninja creature you control triggers, that ability triggers an \
                          additional time.",
            effect: StaticEffect::DoubleControllerTriggersOfType {
                types: vec![CreatureType::Ninja],
                exclude_source: false,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Ninja))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Splinter, Radical Rat",
            cost(&[generic(1), hybrid(Color::White, Color::Black), hybrid(Color::White, Color::Black)]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Rat],
            2,
            4,
        )
    }
}

// ── Norman Osborn // Green Goblin (UBR) ──────────────────────────────────────

fn villain(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    let mut types = types;
    types.push(CreatureType::Villain);
    legend(name, mana, types, p, t)
}

fn connive(what: Selector) -> Effect {
    Effect::Connive { what, amount: Value::ONE }
}

fn treasure(tapped: bool) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition { tapped, ..treasure_token() }),
    }
}

fn br() -> crate::mana::ManaSymbol {
    hybrid(Color::Black, Color::Red)
}

/// Green Goblin — Norman Osborn's back: flying, menace; spells you cast from
/// your graveyard cost {2} less; every nonland card in your graveyard has
/// mayhem for its mana cost. Private: CR 903.4 reads the whole card.
fn green_goblin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Menace],
        static_abilities: vec![
            StaticAbility {
                description: "Spells you cast from your graveyard cost {2} less to cast.",
                effect: StaticEffect::GraveyardCastCostReduction { amount: 2 },
            },
            StaticAbility {
                description: "Goblin Formula — Each nonland card in your graveyard has mayhem. The mayhem cost is equal \
                              to its mana cost.",
                effect: StaticEffect::GraveyardNonlandHaveMayhem,
            },
        ],
        // CR 712.8 — a transforming back face has no mana cost.
        color_indicator: vec![Color::Blue, Color::Black, Color::Red],
        ..villain("Green Goblin", ManaCost::default(), vec![CreatureType::Goblin, CreatureType::Human], 3, 3)
    }
}

/// Norman Osborn — {1}{U} 1/1, can't be blocked; connives off combat damage
/// to a player; {1}{U}{B}{R}: transform, sorcery speed.
pub fn norman_osborn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        back_face: Some(Box::new(green_goblin())),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: connive(Selector::This),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u(), b(), r()]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        ..villain("Norman Osborn", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Scientist], 1, 1)
    }
}

/// Pumpkin Bombs — {1}{R} artifact. {T}, discard two: draw three, add a fuse
/// counter, deal that much to target opponent, who gains control of it.
pub fn pumpkin_bombs() -> CardDefinition {
    CardDefinition {
        name: "Pumpkin Bombs",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 2)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Fuse, amount: Value::ONE },
                Effect::DealDamage {
                    to: target_filtered(R::OpponentPlayer),
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Fuse },
                },
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::Target(0)),
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rocket-Powered Goblin Glider — {3} Equipment: +2/+0, flying and haste;
/// equip {2}; mayhem {2}; cast from your graveyard, it attaches on entry.
pub fn rocket_powered_goblin_glider() -> CardDefinition {
    CardDefinition {
        name: "Rocket-Powered Goblin Glider",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)])), Keyword::Mayhem(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::WasCastFromGraveyard }),
            effect: Effect::Attach { what: Selector::This, to: target_filtered(yours(R::Creature)) },
        }],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            keywords: vec![Keyword::Flying, Keyword::Haste],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Oscorp Industries — a tapped Grixis tri-land with mayhem; entering from a
/// graveyard costs you 2 life.
pub fn oscorp_industries() -> CardDefinition {
    CardDefinition {
        // CR 702.187 — a land's mayhem lets it be played, so it has no cost.
        keywords: vec![Keyword::Mayhem(ManaCost::default())],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches { what: Selector::This, filter: R::EnteredFromGraveyardThisTurn },
            ),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..crate::sets::tri_land("Oscorp Industries", Color::Blue, Color::Black, Color::Red)
    }
}

/// Doctor Octopus, Master Planner — {5}{U}{B} 4/8. Other Villains you control
/// get +2/+2; maximum hand size eight; your end step refills to eight.
pub fn doctor_octopus_master_planner() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other Villains you control get +2/+2.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        yours(R::Creature.and(R::HasCreatureType(CreatureType::Villain))).and(R::OtherThanSource),
                    ),
                    power: 2,
                    toughness: 2,
                },
            },
            StaticAbility { description: "Your maximum hand size is eight.", effect: StaticEffect::ControllerMaxHandSize(8) },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::Const(8))))),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::Const(8)),
                    Box::new(Value::HandSizeOf(PlayerRef::You)),
                ))),
            },
        }],
        ..villain(
            "Doctor Octopus, Master Planner",
            cost(&[generic(5), u(), b()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            4,
            8,
        )
    }
}

fn on_discard(filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

/// Green Goblin, Nemesis — {2}{B}{R} 3/3 flying. Discarding a nonland card
/// grows a Goblin of yours; discarding a land makes a tapped Treasure.
pub fn green_goblin_nemesis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_discard(
                R::Nonland,
                Effect::AddCounter {
                    what: target_filtered(yours(R::Creature.and(R::HasCreatureType(CreatureType::Goblin)))),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ),
            on_discard(R::Land, treasure(true)),
        ],
        ..villain(
            "Green Goblin, Nemesis",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Goblin, CreatureType::Human],
            3,
            3,
        )
    }
}

/// Green Goblin, Revenant — {3}{B}{R} 3/3 flying, deathtouch. Attacking:
/// discard a card, then draw one for each card you've discarded this turn.
pub fn green_goblin_revenant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisTurn(PlayerRef::You) },
        ]))],
        ..villain(
            "Green Goblin, Revenant",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Goblin, CreatureType::Human],
            3,
            3,
        )
    }
}

/// Leader, Super-Genius — {2}{U}{U} 1/3. Your creatures' connives each draw a
/// card first; at the beginning of combat on your turn, a creature of yours
/// connives.
pub fn leader_super_genius() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a creature you control would connive, instead you draw a card, then that creature connives.",
            effect: StaticEffect::ConniveDrawsFirst,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: connive(target_filtered(yours(R::Creature))),
        }],
        ..villain("Leader, Super-Genius", cost(&[generic(2), u(), u()]), vec![CreatureType::Gamma, CreatureType::Scientist], 1, 3)
    }
}

/// Ultimate Green Goblin — {1}{B/R}{B/R} 5/4. Your upkeep: discard a card,
/// then a Treasure; mayhem {2}{B/R}.
pub fn ultimate_green_goblin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Mayhem(cost(&[generic(2), br()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }, treasure(false)]),
        }],
        ..villain("Ultimate Green Goblin", cost(&[generic(1), br(), br()]), vec![CreatureType::Goblin], 5, 4)
    }
}

/// Unstable Experiment — {1}{U} instant. Target player draws a card, then up
/// to one target creature you control connives.
pub fn unstable_experiment() -> CardDefinition {
    spell(
        "Unstable Experiment",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::Draw { who: target_filtered(R::Player), amount: Value::ONE },
                connive(Selector::TargetFiltered { slot: 1, filter: yours(R::Creature) }),
            ])),
        },
    )
}

// ── Reaper King (WUBRG) ──────────────────────────────────────────────────────

fn scarecrow(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, mana, vec![CreatureType::Scarecrow], p, t)
    }
}

fn shapeshifter(p: i32, t: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Shapeshifter".into(),
            power: p,
            toughness: t,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
            keywords: vec![Keyword::Changeling],
            ..Default::default()
        }),
    }
}

fn any_color_mana() -> Effect {
    Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE) }
}

fn while_you_control(color: Color, inner: StaticEffect) -> StaticEffect {
    StaticEffect::WhileCondition {
        condition: Predicate::SelectorExists(Selector::EachPermanent(yours(R::Creature.and(R::HasColor(color))))),
        inner: Box::new(inner),
    }
}

/// Birthing Boughs — {3} artifact. {4}, {T}: a 2/2 Shapeshifter with
/// changeling.
pub fn birthing_boughs() -> CardDefinition {
    CardDefinition {
        name: "Birthing Boughs",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: shapeshifter(2, 2),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Heap Doll — {1} 1/1. Sacrifice it: exile target card from a graveyard.
pub fn heap_doll() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Move { what: target_filtered(R::Any.from_any_graveyard()), to: ZoneDest::Exile },
            ..Default::default()
        }],
        ..scarecrow("Heap Doll", cost(&[generic(1)]), 1, 1)
    }
}

/// Jawbone Skulkin — {1} 1/1. {2}: a red creature gains haste until end of
/// turn.
pub fn jawbone_skulkin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Red))),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..scarecrow("Jawbone Skulkin", cost(&[generic(1)]), 1, 1)
    }
}

/// One-Eyed Scarecrow — {3} 2/3 defender. Opponents' fliers get -1/-0.
pub fn one_eyed_scarecrow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        static_abilities: vec![StaticAbility {
            description: "Creatures with flying your opponents control get -1/-0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasKeyword(Keyword::Flying)).and(R::ControlledByOpponent),
                ),
                power: -1,
                toughness: 0,
            },
        }],
        ..scarecrow("One-Eyed Scarecrow", cost(&[generic(3)]), 2, 3)
    }
}

/// Scarecrone — {3} 1/2. {1}, sacrifice a Scarecrow: draw; {4}, {T}: return
/// an artifact creature card from your graveyard to the battlefield.
pub fn scarecrone() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_other_filter: Some((R::Creature.and(R::HasCreatureType(CreatureType::Scarecrow)), 1)),
                sac_other_may_be_source: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Artifact.and(R::Creature).from_your_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..scarecrow("Scarecrone", cost(&[generic(3)]), 1, 2)
    }
}

/// Scorn Effigy — {3} 2/3, foretell {0}.
pub fn scorn_effigy() -> CardDefinition {
    CardDefinition { foretell_cost: Some(ManaCost::default()), ..scarecrow("Scorn Effigy", cost(&[generic(3)]), 2, 3) }
}

/// Scuttlemutt — {3} 2/2. {T}: any color; {T}: a creature becomes the color
/// of your choice until end of turn.
pub fn scuttlemutt() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: any_color_mana(), ..Default::default() },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::BecomeChosenColor { what: target_filtered(R::Creature), duration: Duration::EndOfTurn },
                ..Default::default()
            },
        ],
        ..scarecrow("Scuttlemutt", cost(&[generic(3)]), 2, 2)
    }
}

/// Shire Scarecrow — {2} 0/3 defender. {1}: any color, once each turn.
pub fn shire_scarecrow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            once_per_turn: true,
            effect: any_color_mana(),
            ..Default::default()
        }],
        ..scarecrow("Shire Scarecrow", cost(&[generic(2)]), 0, 3)
    }
}

/// Wild-Field Scarecrow — {3} 1/4 defender. {2}, sacrifice it: up to two
/// basic lands to your hand.
pub fn wild_field_scarecrow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(2),
            },
            ..Default::default()
        }],
        ..scarecrow("Wild-Field Scarecrow", cost(&[generic(3)]), 1, 4)
    }
}

/// Wingrattle Scarecrow — {3} 2/2. Flying while you control a blue creature,
/// persist while you control a black one.
pub fn wingrattle_scarecrow() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "This creature has flying as long as you control a blue creature.",
                effect: while_you_control(
                    Color::Blue,
                    StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Flying },
                ),
            },
            StaticAbility {
                description: "This creature has persist as long as you control a black creature.",
                effect: while_you_control(
                    Color::Black,
                    StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Persist },
                ),
            },
        ],
        ..scarecrow("Wingrattle Scarecrow", cost(&[generic(3)]), 2, 2)
    }
}

/// Personify — {1}{W} instant. Blink a creature you control back under its
/// owner's control, and make a 1/1 Shapeshifter with changeling.
pub fn personify() -> CardDefinition {
    spell(
        "Personify",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Exile { what: target_filtered(yours(R::Creature)) },
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
            },
            shapeshifter(1, 1),
        ]),
    )
}
