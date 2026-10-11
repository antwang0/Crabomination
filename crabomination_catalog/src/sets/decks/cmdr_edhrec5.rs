//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, fifth file (`cmdr_edhrec4` is the
//! fourth). Lord of the Nazgûl's Wraiths and Ring spells; Leonardo and
//! Michelangelo's Turtles, Ninjas and Mutagen; Norman Osborn's Villains,
//! connive and mayhem; Reaper King's Scarecrows; Sauron, the Dark Lord's
//! Orc Armies; Saruman, the White Hand's spells; Kibo's Apes and Monkeys.
//! Tests in
//! `tests/recent_b/cmdr_edhrec5.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, EnchantmentSubtype, CardType, CounterType, CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
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

// ── Sauron, the Dark Lord (UBR) ──────────────────────────────────────────────

/// Amass Orcs `n` (CR 701.47).
fn amass(count: Value) -> Effect {
    Effect::Amass { who: PlayerRef::You, count, extra_type: Some(CreatureType::Orc) }
}

fn orc_kin() -> R {
    R::HasCreatureType(CreatureType::Army)
        .or(R::HasCreatureType(CreatureType::Goblin))
        .or(R::HasCreatureType(CreatureType::Orc))
}

/// Barad-dûr — legendary land: tapped unless you control a legendary
/// creature; {T}: {B}; {X}{X}{B}, {T}: amass Orcs X once a creature died
/// this turn.
pub fn barad_dur() -> CardDefinition {
    CardDefinition {
        name: "Barad-dûr",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "Barad-dûr enters tapped unless you control a legendary creature.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::EachPermanent(yours(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                ))),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::OfColor(Color::Black, Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[x(), x(), b()]),
                tap_cost: true,
                condition: Some(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
                effect: amass(Value::XFromCost),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Gothmog, Morgul Lieutenant — {3}{B} 3/3. Amass Orcs 1 on entry; your
/// creature tokens have deathtouch.
pub fn gothmog_morgul_lieutenant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![grant(
            "Creature tokens you control have deathtouch.",
            yours(R::Creature.and(R::IsToken)),
            Keyword::Deathtouch,
        )],
        triggered_abilities: vec![etb(amass(Value::ONE))],
        ..legend(
            "Gothmog, Morgul Lieutenant",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Mauhúr, Uruk-hai Captain — {B}{R} 2/2 menace. +1/+1 counters on your
/// Armies, Goblins and Orcs come one extra.
pub fn mauhur_uruk_hai_captain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on an Army, Goblin, or Orc you control, that many \
                          plus one +1/+1 counters are put on it instead.",
            effect: StaticEffect::ExtraPlusOneCountersMatching { filter: yours(orc_kin()) },
        }],
        ..legend("Mauhúr, Uruk-hai Captain", cost(&[b(), r()]), vec![CreatureType::Orc, CreatureType::Soldier], 2, 2)
    }
}

/// The Great Goblin — {1}{B/R}{B/R} 3/2. Counters you put on your Goblins,
/// Orcs and Armies deal 2 to an opponent; another of them dying impulses the
/// top card until the end of your next turn.
pub fn the_great_goblin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::AnyCounterAdded, EventScope::YouPutCounters)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: yours(orc_kin()) }),
                effect: Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: orc_kin().and(R::OtherThanSource) },
                ),
                effect: Effect::Seq(vec![
                    Effect::ExileTopOfLibrary {
                        who: Selector::You,
                        amount: Value::ONE,
                        link_to_source: false,
                        face_down: false,
                    },
                    Effect::GrantMayPlay {
                        what: Selector::ExiledThisResolution { filter: R::Any },
                        duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ]),
            },
        ],
        ..legend("The Great Goblin", cost(&[generic(1), br(), br()]), vec![CreatureType::Goblin, CreatureType::Noble], 3, 2)
    }
}

/// Warg Rider — {4}{B} 4/3 menace. Your other Orcs and Goblins have menace;
/// at the beginning of combat on your turn, amass Orcs 2.
pub fn warg_rider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![grant(
            "Other Orcs and Goblins you control have menace.",
            yours(
                R::Creature
                    .and(R::HasCreatureType(CreatureType::Orc).or(R::HasCreatureType(CreatureType::Goblin)))
                    .and(R::OtherThanSource),
            ),
            Keyword::Menace,
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: amass(Value::Const(2)),
        }],
        ..creature("Warg Rider", cost(&[generic(4), b()]), vec![CreatureType::Orc, CreatureType::Warrior], 4, 3)
    }
}

/// March from the Black Gate — {1}{B} enchantment. Amass Orcs 1 on entry and
/// whenever an Army you control attacks.
pub fn march_from_the_black_gate() -> CardDefinition {
    CardDefinition {
        name: "March from the Black Gate",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(amass(Value::ONE)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Army),
                }),
                effect: amass(Value::ONE),
            },
        ],
        ..Default::default()
    }
}

/// One Ring to Rule Them All — {2}{B}{B} Saga. I: the Ring tempts you, then
/// each player mills your Ring-bearer's power. II: destroy all nonlegendary
/// creatures. III: each opponent loses 1 per creature card in their graveyard.
pub fn one_ring_to_rule_them_all() -> CardDefinition {
    CardDefinition {
        name: "One Ring to Rule Them All",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    tempt(),
                    Effect::Mill {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::PowerOf(Box::new(Selector::RingBearerOf(PlayerRef::You))),
                    },
                ]),
            ),
            (
                2,
                Effect::Destroy {
                    what: Selector::EachPermanent(R::Creature.and(R::HasSupertype(Supertype::Legendary).negate())),
                },
            ),
            (
                3,
                // Each opponent counts their own graveyard ("that player's").
                Effect::EachPlayerDoes {
                    who: PlayerRef::EachOpponent,
                    body: Box::new(Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Orcish Medicine — {1}{B} instant. A creature gains lifelink or
/// indestructible (your choice) until end of turn; amass Orcs 1.
pub fn orcish_medicine() -> CardDefinition {
    let gains = |keyword| Effect::GrantKeyword {
        what: target_filtered(R::Creature),
        keyword,
        duration: Duration::EndOfTurn,
    };
    spell(
        "Orcish Medicine",
        cost(&[generic(1), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ChooseMode(vec![gains(Keyword::Lifelink), gains(Keyword::Indestructible)]),
            amass(Value::ONE),
        ]),
    )
}

/// Foray of Orcs — {3}{R} sorcery. Amass Orcs 2, then the Army deals damage
/// equal to its power to a creature an opponent controls.
pub fn foray_of_orcs() -> CardDefinition {
    spell(
        "Foray of Orcs",
        cost(&[generic(3), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            amass(Value::Const(2)),
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                amount: army_power(),
            },
        ]),
    )
}

// ── Saruman, the White Hand (UBR) ────────────────────────────────────────────

/// Amass Goblins `n` (CR 701.47).
fn amass_goblins(count: Value) -> Effect {
    Effect::Amass { who: PlayerRef::You, count, extra_type: Some(CreatureType::Goblin) }
}

/// The amassed Army: the one a player normally controls.
fn army_power() -> Value {
    Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(R::HasCreatureType(CreatureType::Army))))
}

fn team_eot(keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what: Selector::EachPermanent(yours(R::Creature)), keyword, duration: Duration::EndOfTurn }
}

/// Bothersome Noisemaker — {1}{R} 2/2. Each noncreature spell you cast amasses
/// Goblins 1.
pub fn bothersome_noisemaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Creature).negate(),
            }),
            effect: amass_goblins(Value::ONE),
        }],
        ..creature("Bothersome Noisemaker", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Bard], 2, 2)
    }
}

/// Book of Mazarbul — {2}{R} Saga. I: amass Orcs 1. II: amass Orcs 2. III:
/// your creatures get +1/+0 and menace until end of turn.
pub fn book_of_mazarbul() -> CardDefinition {
    CardDefinition {
        name: "Book of Mazarbul",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, amass(Value::ONE)),
            (2, amass(Value::Const(2))),
            (
                3,
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::EachPermanent(yours(R::Creature)),
                        power: Value::ONE,
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    team_eot(Keyword::Menace),
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Fall of Cair Andros — {2}{R} enchantment. Excess noncombat damage to an
/// opponent's creature amasses Orcs that much; {7}{R}: 7 damage to a creature.
pub fn fall_of_cair_andros() -> CardDefinition {
    CardDefinition {
        name: "Fall of Cair Andros",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            // CR 120.10 — the excess is the event's amount.
            event: EventSpec::new(EventKind::DealtExcessNoncombatDamage, EventScope::OpponentControl),
            effect: amass(Value::TriggerEventAmount),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), r()]),
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(7) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Storm of Saruman — {4}{U}{U} enchantment, ward {3}. Your second spell each
/// turn is copied, and the copy isn't legendary.
pub fn storm_of_saruman() -> CardDefinition {
    CardDefinition {
        name: "Storm of Saruman",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Ward(WardCost::generic(3))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(2) }),
            effect: Effect::CopySpellNotLegendary { what: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

/// Soul's Fire — {2}{R} instant. A creature you control deals damage equal to
/// its power to any target.
pub fn souls_fire() -> CardDefinition {
    spell(
        "Soul's Fire",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::DealDamageEqualToPower {
            source: Selector::TargetFiltered { slot: 0, filter: yours(R::Creature) },
            target: Selector::TargetFiltered { slot: 1, filter: R::any_target() },
        },
    )
}

/// Assault on Osgiliath — {X}{R}{R}{R} sorcery. Amass Orcs X, then your
/// Goblins and Orcs gain double strike and haste until end of turn.
pub fn assault_on_osgiliath() -> CardDefinition {
    let kin = || {
        Selector::EachPermanent(yours(
            R::Creature.and(R::HasCreatureType(CreatureType::Goblin).or(R::HasCreatureType(CreatureType::Orc))),
        ))
    };
    spell(
        "Assault on Osgiliath",
        cost(&[x(), r(), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            amass(Value::XFromCost),
            Effect::GrantKeyword { what: kin(), keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: kin(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Rage into the Valley — {2}{B} sorcery. Draw a card, lose 1 life, amass
/// Goblins 2.
pub fn rage_into_the_valley() -> CardDefinition {
    spell(
        "Rage into the Valley",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            amass_goblins(Value::Const(2)),
        ]),
    )
}

/// Surrounded by Orcs — {3}{U} sorcery. Amass Orcs 3, then target player
/// mills the Army's power.
pub fn surrounded_by_orcs() -> CardDefinition {
    spell(
        "Surrounded by Orcs",
        cost(&[generic(3), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            amass(Value::Const(3)),
            Effect::Mill { who: target_filtered(R::Player), amount: army_power() },
        ]),
    )
}

// ── Kibo, Uktabi Prince (RG) ─────────────────────────────────────────────────

fn food_each_player() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::EachPlayer,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::food_token()),
    }
}

fn begin_combat(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl), effect }
}

/// Elephant-Mandrill — {2}{G} 3/2 reach. Each player makes a Food on entry; at
/// the beginning of combat on your turn, +1/+1 per artifact your opponents
/// control.
pub fn elephant_mandrill() -> CardDefinition {
    let n = || Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByOpponent))));
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            etb(food_each_player()),
            begin_combat(Effect::PumpPT { what: Selector::This, power: n(), toughness: n(), duration: Duration::EndOfTurn }),
        ],
        ..creature("Elephant-Mandrill", cost(&[generic(2), g()]), vec![CreatureType::Elephant, CreatureType::Monkey], 3, 2)
    }
}

/// Gorilla Shaman — {R} 1/1. {X}{X}{1}: destroy a noncreature artifact with
/// mana value X.
pub fn gorilla_shaman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), x(), generic(1)]),
            effect: Effect::Destroy {
                what: target_filtered(
                    R::Artifact.and(R::Creature.negate()).and(R::ManaValueExactlyXFromCost),
                ),
            },
            ..Default::default()
        }],
        ..creature("Gorilla Shaman", cost(&[r()]), vec![CreatureType::Ape, CreatureType::Shaman], 1, 1)
    }
}

/// Grunn, the Lonely King — {4}{G}{G} 5/5, kicker {3}: kicked, it enters with
/// five +1/+1 counters; attacking alone doubles its power and toughness.
pub fn grunn_the_lonely_king() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3)]))],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SpellWasKicked),
                then: Box::new(Value::Const(5)),
                else_: Box::new(Value::ZERO),
            },
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::AttackingAlone),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::ToughnessOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend("Grunn, the Lonely King", cost(&[generic(4), g(), g()]), vec![CreatureType::Ape, CreatureType::Warrior], 5, 5)
    }
}

/// Scrounging Bandar — {1}{G} 0/0, enters with two +1/+1 counters; at your
/// upkeep it may move its +1/+1 counters onto another creature.
pub fn scrounging_bandar() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            // CR 122.5 — how many (zero declines) is the controller's pick.
            effect: Effect::MoveAnyNumberOfCounters {
                from: Selector::This,
                to: target_filtered(R::Creature.and(R::OtherThanSource)),
            },
        }],
        ..creature("Scrounging Bandar", cost(&[generic(1), g()]), vec![CreatureType::Cat, CreatureType::Monkey], 0, 0)
    }
}

/// Silverback Shaman — {3}{G}{G} 5/4 trample; dies: draw a card.
pub fn silverback_shaman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Silverback Shaman", cost(&[generic(3), g(), g()]), vec![CreatureType::Ape, CreatureType::Shaman], 5, 4)
    }
}

/// Simian Simulacrum — {3} 2/1 artifact Ape: two +1/+1 counters on a creature
/// you control on entry; unearth {2}{G}{G}.
pub fn simian_simulacrum() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(yours(R::Creature)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        activated_abilities: vec![crate::effect::shortcut::unearth(cost(&[generic(2), g(), g()]))],
        ..creature("Simian Simulacrum", cost(&[generic(3)]), vec![CreatureType::Ape], 2, 1)
    }
}

/// Towering Gibbon — {3}{G} */4 reach; its power is the greatest mana value
/// among creatures you control.
pub fn towering_gibbon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Towering Gibbon's power is equal to the greatest mana value among creatures you control.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::ManaValueOf(Box::new(Selector::GreatestManaValueControlledMatching {
                    who: PlayerRef::You,
                    filter: R::Creature,
                })),
                toughness: Value::Const(4),
            },
        }],
        ..creature("Towering Gibbon", cost(&[generic(3), g()]), vec![CreatureType::Ape], 0, 4)
    }
}

/// Tree Monkey — {G} 1/1 reach.
pub fn tree_monkey() -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Reach], ..creature("Tree Monkey", cost(&[g()]), vec![CreatureType::Monkey], 1, 1) }
}

/// Wily Bandar — {G} 1/1. {2}{G}: indestructible until end of turn.
pub fn wily_bandar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature("Wily Bandar", cost(&[g()]), vec![CreatureType::Cat, CreatureType::Monkey], 1, 1)
    }
}

/// Yavimaya Steelcrusher — {1}{R} 2/2, enlist; {1}, sacrifice it: destroy an
/// artifact.
pub fn yavimaya_steelcrusher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::enlist()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..creature("Yavimaya Steelcrusher", cost(&[generic(1), r()]), vec![CreatureType::Ape, CreatureType::Warrior], 2, 2)
    }
}

/// Zodiac Monkey — {1}{G} 2/1 forestwalk.
pub fn zodiac_monkey() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(crate::card::LandType::Forest)],
        ..creature("Zodiac Monkey", cost(&[generic(1), g()]), vec![CreatureType::Monkey], 2, 1)
    }
}

/// Party Dude — {G} Class. Each player makes a Food on entry. L2 ({1}{G}): an
/// opponent's artifact going to a graveyard draws you a card. L3 ({4}{G}):
/// when one or more opponents are attacked, an attacking creature gets +X/+X,
/// X your hand size.
pub fn party_dude() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    let hand = || Value::HandSizeOf(PlayerRef::You);
    CardDefinition {
        name: "Party Dude",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        triggered_abilities: vec![
            etb(food_each_player()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    Predicate::SourceClassLevelAtLeast(2),
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::ControlledByOpponent) },
                ])),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer)
                    .with_filter(Predicate::All(vec![Predicate::SourceClassLevelAtLeast(3), Predicate::AnOpponentIsAttacked])),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::PumpPT {
                        what: target_filtered(R::Creature.and(R::IsAttacking)),
                        power: hand(),
                        toughness: hand(),
                        duration: Duration::EndOfTurn,
                    }),
                },
            },
        ],
        activated_abilities: vec![level_up(cost(&[generic(1), g()]), 1), level_up(cost(&[generic(4), g()]), 2)],
        ..Default::default()
    }
}
