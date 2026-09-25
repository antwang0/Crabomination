//! Commander: the cards the **Avengers Assemble** precon (MSC, Captain
//! America, Team Leader) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_captain_america.rs`.
//!
//! Residuals (each also on its card):
//! - **Captain Marvel, Apex Avenger** — only +1/+1 counters are copied.
//! - **Heroic Return** — the Hero's two counters are put on as it lands, not
//!   as it enters.
//! - **Heroic Sacrifice** — damage to your noncreature permanents is
//!   redirected too.
//! - **Scarlet Witch, Chaotic Avenger** — the two cards are exiled face up.
//! - **Speed, Young Avenger** — "can't be blocked except by creatures with
//!   haste" is unblockable.
//! - **Winter Soldier, Reborn Avenger** — the Hero's counter is put on as it
//!   lands, not as it enters.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, on_you_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, ManaCost, SpendRestriction, cost, generic, r, u, w, x};
use crate::sets::tap_add_colorless;
use crabomination_base::tokens::treasure_token;
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn artifact_creature(def: CardDefinition) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..def }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn may(description: &str, body: Effect) -> Effect {
    Effect::MayDo { description: description.into(), body: Box::new(body) }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn hero() -> R {
    R::HasCreatureType(CreatureType::Hero)
}

fn plus(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn keyword_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn on_combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

/// "Whenever you draw a card, [effect]."
fn on_draw(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl), effect }
}

/// "Look at the top `n` cards; you may reveal a Hero card from among them
/// and put it into your hand; the rest on the bottom."
fn dig_for_hero(n: i32) -> Effect {
    Effect::LookPickToHand(Box::new(LookPick {
        who: PlayerRef::You,
        count: Value::Const(n),
        pick_filter: Some(hero()),
        rest_bottom_random: true,
        ..Default::default()
    }))
}

// ── Commander ───────────────────────────────────────────────────────────────

/// Captain America, Team Leader — another Hero of yours entering gains
/// vigilance and haste, and it and Captain America each get a +1/+1 counter.
pub fn captain_america_team_leader() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: hero().and(R::Creature) }),
            effect: Effect::Seq(vec![
                keyword_eot(Selector::TriggerSource, Keyword::Vigilance),
                keyword_eot(Selector::TriggerSource, Keyword::Haste),
                plus(Selector::TriggerSource, Value::ONE),
                plus(Selector::This, Value::ONE),
            ]),
        }],
        ..creature(
            "Captain America, Team Leader",
            cost(&[r(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            3,
            3,
        )
    })
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Ant-Man, Elusive Avenger — can't be blocked by bigger creatures (skulk);
/// connecting makes that many Treasures.
pub fn ant_man_elusive_avenger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Skulk],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::TriggerEventAmount,
            definition: Arc::new(treasure_token()),
        })],
        ..creature(
            "Ant-Man, Elusive Avenger",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Hero],
            1,
            2,
        )
    })
}

/// Black Widow, Agile Avenger — menace; an opponent's second draw each turn
/// grows her and draws you a card.
pub fn black_widow_agile_avenger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SecondCardDrawnThisTurn, EventScope::OpponentControl),
            effect: Effect::Seq(vec![plus(Selector::This, Value::ONE), draw(Value::ONE)]),
        }],
        ..creature(
            "Black Widow, Agile Avenger",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            2,
            2,
        )
    })
}

/// Captain America, Living Legend — vigilance; during your turn, a creature
/// of yours becoming tapped for the first time that turn untaps.
pub fn captain_america_living_legend() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                Predicate::TriggerSourceFirstTappedThisTurn,
            ])),
            effect: Effect::Untap { what: Selector::TriggerSource, up_to: None },
        }],
        ..creature(
            "Captain America, Living Legend",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            3,
            4,
        )
    })
}

/// Captain Mar-Vell, Space-Born — flying, vigilance; while an opponent has
/// cast a spell this turn, your spells have flash.
pub fn captain_mar_vell_space_born() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "As long as an opponent has cast a spell this turn, you may cast spells as though they had flash.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::AnOpponentCastASpellThisTurn,
                inner: Box::new(StaticEffect::ControllerSpellsHaveFlash { filter: R::Any }),
            },
        }],
        ..creature(
            "Captain Mar-Vell, Space-Born",
            cost(&[generic(4), w()]),
            vec![CreatureType::Kree, CreatureType::Soldier, CreatureType::Hero],
            4,
            4,
        )
    })
}

/// Captain Marvel, Apex Avenger — flying, double strike, indestructible;
/// counters you put on another non-Kree creature may be copied onto her.
///
/// ⚠ Residual: only +1/+1 counters are copied.
pub fn captain_marvel_apex_avenger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike, Keyword::Indestructible],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YouPutCounters).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource).and(R::HasCreatureType(CreatureType::Kree).negate()),
                },
            ),
            effect: may("Put the same counters on Captain Marvel?", plus(Selector::This, Value::TriggerEventAmount)),
        }],
        ..creature(
            "Captain Marvel, Apex Avenger",
            cost(&[generic(5), r(), w()]),
            vec![CreatureType::Human, CreatureType::Kree, CreatureType::Hero],
            4,
            4,
        )
    })
}

/// Director Nick Fury — Hero spells cost {1} less; whenever you attack, dig
/// four for a Hero.
pub fn director_nick_fury() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Hero spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: hero(), amount: 1 },
        }],
        triggered_abilities: vec![on_you_attack(dig_for_hero(4))],
        ..creature(
            "Director Nick Fury",
            cost(&[u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            2,
            4,
        )
    })
}

/// Falcon and Redwing — flying; connecting makes that many 1/1 Birds, then
/// a +1/+1 counter.
pub fn falcon_and_redwing() -> CardDefinition {
    let bird = Arc::new(TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        ..Default::default()
    });
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::TriggerEventAmount, definition: bird },
            plus(Selector::This, Value::ONE),
        ]))],
        ..creature(
            "Falcon and Redwing",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Bird, CreatureType::Hero],
            1,
            2,
        )
    })
}

/// Firebird, Blazing Ranger — flying; attacking, the other attackers get
/// +X/+0, X its power.
pub fn firebird_blazing_ranger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
            power: Value::PowerOf(Box::new(Selector::This)),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Firebird, Blazing Ranger",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Ranger, CreatureType::Hero],
            1,
            3,
        )
    })
}

/// Hawkeye, Avenging Archer — reach; an opponent's creature he damaged this
/// turn dying draws a card; {T}: 1 damage to any target.
pub fn hawkeye_avenging_archer() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::DamagedBySourceThisTurn },
            ),
            effect: draw(Value::ONE),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Hawkeye, Avenging Archer",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Human, CreatureType::Archer, CreatureType::Hero],
            3,
            4,
        )
    })
}

/// Hercules, Olympian Hero — attacking grows him and makes him
/// indestructible; the first damage each turn becomes that many counters.
pub fn hercules_olympian_hero() -> CardDefinition {
    let mut first_hurt = TriggeredAbility {
        event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
        effect: plus(Selector::This, Value::TriggerEventAmount),
    };
    first_hurt.event.once_per_turn = true;
    legendary(CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::Seq(vec![
                plus(Selector::This, Value::ONE),
                keyword_eot(Selector::This, Keyword::Indestructible),
            ])),
            first_hurt,
        ],
        ..creature(
            "Hercules, Olympian Hero",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Demigod, CreatureType::Warrior, CreatureType::Hero],
            3,
            3,
        )
    })
}

/// Iron Man, Armored Avenger — flying; your draws put a counter on a
/// creature; attacking gives your other modified attackers flying.
pub fn iron_man_armored_avenger() -> CardDefinition {
    legendary(artifact_creature(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_draw(plus(target_filtered(R::Creature), Value::ONE)),
            on_attack(keyword_eot(
                Selector::EachPermanent(yours(R::Creature).and(R::IsAttacking).and(R::IsModified).and(R::OtherThanSource)),
                Keyword::Flying,
            )),
        ],
        ..creature(
            "Iron Man, Armored Avenger",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Hero],
            2,
            2,
        )
    }))
}

/// Jarvis, Earth's Mightiest Butler — your Hero spells draw a card.
pub fn jarvis_earths_mightiest_butler() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(hero())),
            effect: draw(Value::ONE),
        }],
        ..creature(
            "Jarvis, Earth's Mightiest Butler",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            4,
        )
    })
}

/// Jocasta, Automaton Avenger — flying; your commander connecting grows her;
/// from your graveyard, attacking with your commander may bring her back
/// tapped and attacking.
pub fn jocasta_automaton_avenger() -> CardDefinition {
    legendary(artifact_creature(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander },
                ),
                effect: plus(Selector::This, Value::ONE),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::FromYourGraveyard).with_filter(
                    Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: R::IsCommander },
                ),
                effect: may(
                    "Return Jocasta to the battlefield tapped and attacking?",
                    Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::This,
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                        },
                        Effect::JoinCombatAttacking { what: Selector::This },
                    ]),
                ),
            },
        ],
        ..creature(
            "Jocasta, Automaton Avenger",
            cost(&[generic(3)]),
            vec![CreatureType::Robot, CreatureType::Hero],
            2,
            2,
        )
    }))
}

/// Patriot, Shield Wielder — {2}, {T}: another creature of yours gets +2/+0
/// and hexproof until end of turn.
pub fn patriot_shield_wielder() -> CardDefinition {
    legendary(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(yours(R::Creature).and(R::OtherThanSource)),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                keyword_eot(Selector::Target(0), Keyword::Hexproof),
            ]),
            ..Default::default()
        }],
        ..creature("Patriot, Shield Wielder", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Hero], 2, 2)
    })
}

/// Photon, Mighty Marvel — flying; connecting adds that much mana of one
/// color, kept until end of turn.
pub fn photon_mighty_marvel() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::AddManaKeptThisTurnAnyOneColor {
            who: PlayerRef::You,
            amount: Value::TriggerEventAmount,
        })],
        ..creature("Photon, Mighty Marvel", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Hero], 2, 4)
    })
}

/// Professor Hulk — trample; connecting draws that many cards.
pub fn professor_hulk() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_combat_damage_to_player(draw(Value::TriggerEventAmount))],
        ..creature(
            "Professor Hulk",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Gamma, CreatureType::Scientist, CreatureType::Hero],
            6,
            6,
        )
    })
}

/// Quicksilver, Speedster — flash, double strike, haste; while tapped, your
/// spells have flash.
pub fn quicksilver_speedster() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::DoubleStrike, Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "As long as Quicksilver is tapped, you may cast spells as though they had flash.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::Tapped },
                inner: Box::new(StaticEffect::ControllerSpellsHaveFlash { filter: R::Any }),
            },
        }],
        ..creature(
            "Quicksilver, Speedster",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Mutant, CreatureType::Hero],
            3,
            4,
        )
    })
}

/// Rescue, Pepper Potts — flash, flying; on entry, bounce up to one other
/// artifact or creature of yours, growing if it was an artifact.
pub fn rescue_pepper_potts() -> CardDefinition {
    legendary(artifact_creature(CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: yours(R::Artifact.or(R::Creature)).and(R::OtherThanSource),
            effect: Box::new(Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Artifact },
                    then: Box::new(plus(Selector::This, Value::ONE)),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ])),
        })],
        ..creature("Rescue, Pepper Potts", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Hero], 2, 1)
    }))
}

/// Scarlet Witch, Chaotic Avenger — flying; connecting exiles the top two
/// cards, then you may cast a Hero or noncreature spell among the cards
/// exiled with her for free.
///
/// ⚠ Residual: the cards are exiled face up.
pub fn scarlet_witch_chaotic_avenger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::Seq(vec![
            Effect::Move {
                what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(2) },
                to: ZoneDest::ExileWithSourceStamp,
            },
            may(
                "Cast a Hero or noncreature spell exiled with Scarlet Witch without paying its mana cost?",
                Effect::CastWithoutPayingImmediate {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Exile,
                            filter: R::ExiledWithSource.and(hero().or(R::Noncreature)).and(R::Nonland),
                        }),
                        count: Box::new(Value::ONE),
                    },
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
            ),
        ]))],
        ..creature(
            "Scarlet Witch, Chaotic Avenger",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Mutant, CreatureType::Warlock, CreatureType::Hero],
            3,
            3,
        )
    })
}

/// Shang-Chi and the Ten Rings — first strike; your draws grow it; the tenth
/// counter draws five and gains 5.
pub fn shang_chi_and_the_ten_rings() -> CardDefinition {
    let counters = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    legendary(CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![
            on_draw(plus(Selector::This, Value::ONE)),
            // "When the tenth +1/+1 counter is put on it": this batch took it
            // from below ten to ten or more.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::SelfSource)
                    .with_filter(Predicate::All(vec![
                        Predicate::ValueAtLeast(counters(), Value::Const(10)),
                        Predicate::ValueAtMost(
                            Value::Diff(Box::new(counters()), Box::new(Value::TriggerEventAmount)),
                            Value::Const(9),
                        ),
                    ])),
                effect: Effect::Seq(vec![
                    draw(Value::Const(5)),
                    Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
                ]),
            },
        ],
        ..creature(
            "Shang-Chi and the Ten Rings",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            2,
            2,
        )
    })
}

/// She-Hulk, Wallbreaker — trample; your other Heroes have trample; a Hero
/// of yours becoming blocked gets a counter per blocker.
pub fn she_hulk_wallbreaker() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Other Heroes you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(hero()).and(R::OtherThanSource)),
                keyword: Keyword::Trample,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: hero() }),
            effect: plus(Selector::TriggerSource, Value::count(Selector::CreaturesBlockingTriggerSource)),
        }],
        ..creature(
            "She-Hulk, Wallbreaker",
            cost(&[generic(5), r()]),
            vec![CreatureType::Gamma, CreatureType::Hero],
            5,
            5,
        )
    })
}

/// Speed, Young Avenger — haste; a noncreature spell lets you pay {1} to make
/// a hasty creature evasive this turn.
///
/// ⚠ Residual: "can't be blocked except by creatures with haste" is
/// unblockable.
pub fn speed_young_avenger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
            effect: Effect::MayPay {
                description: "Pay {1}?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(keyword_eot(
                        target_filtered(R::Creature.and(R::HasKeyword(Keyword::Haste))),
                        Keyword::Unblockable,
                    )),
                }),
                else_: None,
            },
        }],
        ..creature("Speed, Young Avenger", cost(&[generic(1), r()]), vec![CreatureType::Mutant, CreatureType::Hero], 2, 2)
    })
}

/// The Wasp, Winsome Avenger — flash, flying; on entry a Hero gains
/// hexproof; attacking taps a creature of the defending player's.
pub fn the_wasp_winsome_avenger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![
            etb(keyword_eot(target_filtered(hero()), Keyword::Hexproof)),
            on_attack(Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)) }),
        ],
        ..creature(
            "The Wasp, Winsome Avenger",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Hero],
            2,
            1,
        )
    })
}

/// Thor, Asgard's Avenger — flying; your other sources deal 1 more damage to
/// opponents and their permanents.
pub fn thor_asgards_avenger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If another source you control would deal damage to an opponent or a permanent an opponent \
                          controls, it deals that much damage plus 1 instead.",
            effect: StaticEffect::AddDamageToOpponents { source_color: None, amount: 1, other: true },
        }],
        ..creature(
            "Thor, Asgard's Avenger",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::God, CreatureType::Warrior, CreatureType::Hero],
            4,
            4,
        )
    })
}

/// Vision, Synthezoid Avenger — flying; a spell cast outside its caster's
/// turn grows Vision or phases him out.
pub fn vision_synthezoid_avenger() -> CardDefinition {
    legendary(artifact_creature(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::Triggerer)))),
            effect: Effect::ChooseMode(vec![
                plus(Selector::This, Value::ONE),
                Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
            ]),
        }],
        ..creature(
            "Vision, Synthezoid Avenger",
            cost(&[generic(4)]),
            vec![CreatureType::Robot, CreatureType::Hero],
            3,
            3,
        )
    }))
}

/// War Machine, Avenging Arsenal — flying; attacking gives your modified
/// attackers double strike.
pub fn war_machine_avenging_arsenal() -> CardDefinition {
    legendary(artifact_creature(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(keyword_eot(
            Selector::EachPermanent(yours(R::Creature).and(R::IsAttacking).and(R::IsModified)),
            Keyword::DoubleStrike,
        ))],
        ..creature(
            "War Machine, Avenging Arsenal",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Hero],
            3,
            5,
        )
    }))
}

/// Winter Soldier, Reborn Avenger — attacking reanimates a creature card
/// with mana value up to his power; a Hero gets an extra counter.
///
/// ⚠ Residual: the Hero's counter is put on as it lands, not as it enters.
pub fn winter_soldier_reborn_avenger() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![on_attack(reanimate_hero_bonus(
            R::Creature.and(R::ManaValueAtMostSourcePower).from_your_graveyard(),
            1,
        ))],
        ..creature(
            "Winter Soldier, Reborn Avenger",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            3,
            5,
        )
    })
}

/// Return target `filter` card to the battlefield; a Hero gets `n` more
/// +1/+1 counters.
fn reanimate_hero_bonus(filter: R, n: i32) -> Effect {
    Effect::Seq(vec![
        Effect::Move {
            what: target_filtered(filter),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        Effect::If {
            cond: Predicate::EntityMatches { what: Selector::Target(0), filter: hero() },
            then: Box::new(plus(Selector::Target(0), Value::Const(n))),
            else_: Box::new(Effect::Noop),
        },
    ])
}

// ── Artifacts and lands ─────────────────────────────────────────────────────

/// Avengers Quinjet — flying Vehicle, crew 3; entering or attacking, put a
/// Hero from your hand onto the battlefield or regrow one.
pub fn avengers_quinjet() -> CardDefinition {
    let modes = || {
        Effect::ChooseMode(vec![
            may(
                "Put a Hero creature card from your hand onto the battlefield?",
                Effect::MoveChosen {
                    from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter: hero().and(R::Creature) },
                    filter: None,
                    count: Value::ONE,
                    up_to: true,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
            Effect::Move {
                what: target_filtered(hero().and(R::Creature).from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ])
    };
    CardDefinition {
        name: "Avengers Quinjet",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(3)],
        triggered_abilities: vec![etb(modes()), on_attack(modes())],
        ..Default::default()
    }
}

/// Avengers Tower — {T}: {C}; {T}: any color for Heroes; {4}, {T}: dig three
/// for a Hero.
pub fn avengers_tower() -> CardDefinition {
    CardDefinition {
        name: "Avengers Tower",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::CreatureOfTypeOrItsAbility(CreatureType::Hero),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                effect: dig_for_hero(3),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hulkbuster Armor — equipped creature is a 9/9 flier; equip Hero {3},
/// equip {6}.
pub fn hulkbuster_armor() -> CardDefinition {
    CardDefinition {
        name: "Hulkbuster Armor",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(6)]))],
        equip_filtered_cost: Some((hero(), cost(&[generic(3)]))),
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Flying],
            set_base_pt: Some((9, 9)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Love on the Battlefield — attacking with exactly two creatures gives them
/// first strike and draws a card; each of them connecting this combat grows.
pub fn love_on_the_battlefield() -> CardDefinition {
    let attackers = || Selector::EachPermanent(yours(R::Creature).and(R::IsAttacking));
    CardDefinition {
        name: "Love on the Battlefield",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource)
                .with_filter(Predicate::ValueEquals(Value::count(attackers()), Value::Const(2))),
            effect: Effect::Seq(vec![
                keyword_eot(attackers(), Keyword::FirstStrike),
                draw(Value::ONE),
                Effect::GrantTriggeredAbility {
                    what: attackers(),
                    trigger: Box::new(on_combat_damage_to_player(plus(Selector::This, Value::ONE))),
                    duration: Duration::EndOfCombat,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Plaza of Heroes — {T}: {C}; {T}: any color for legendary spells; {T}: a
/// color among your legendary permanents; {3}, {T}, exile it: a legendary
/// creature gains hexproof and indestructible.
pub fn plaza_of_heroes() -> CardDefinition {
    CardDefinition {
        name: "Plaza of Heroes",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::LegendarySpell,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColorAmongLegendaries },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                exile_self_cost: true,
                effect: Effect::Seq(vec![
                    keyword_eot(
                        target_filtered(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
                        Keyword::Hexproof,
                    ),
                    keyword_eot(Selector::Target(0), Keyword::Indestructible),
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Instants and sorceries ──────────────────────────────────────────────────

/// Avenge — {2} less after a player attacked you last turn; destroy all
/// creatures, gaining 1 life per creature destroyed.
pub fn avenge() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if: Some((Predicate::APlayerAttackedYouLastTurn, 2)),
        ..spell(
            "Avenge",
            cost(&[generic(4), w(), w()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
                Effect::GainLife { who: Selector::You, amount: Value::PermanentsDestroyedThisResolution },
            ]),
        )
    }
}

/// Heroic Return — {2} less while a creature attacks you; reanimate a
/// creature card, a Hero with two more counters.
///
/// ⚠ Residual: the Hero's counters are put on as it lands, not as it enters.
pub fn heroic_return() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if: Some((
            Predicate::SelectorExists(Selector::EachPermanent(R::Creature.and(R::IsAttackingYou))),
            2,
        )),
        ..spell(
            "Heroic Return",
            cost(&[generic(5), w()]),
            CardType::Instant,
            reanimate_hero_bonus(R::Creature.from_your_graveyard(), 2),
        )
    }
}

/// Heroic Sacrifice — until end of turn one of your creatures takes the
/// damage meant for you and yours; when it dies this turn, its counters go
/// to another creature of yours and you draw.
///
/// ⚠ Residual: damage to your noncreature permanents is redirected too.
pub fn heroic_sacrifice() -> CardDefinition {
    spell(
        "Heroic Sacrifice",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::RedirectYourDamageToChosen { what: target_filtered(yours(R::Creature)) },
            Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::Seq(vec![
                    Effect::Reflexive {
                        body: Box::new(Effect::ApplyToTargets {
                            max_targets: 1,
                            min_targets: 0,
                            filter: yours(R::Creature),
                            effect: Box::new(Effect::PutCountersOf {
                                from: Selector::TriggerSource,
                                to: Selector::Target(0),
                            }),
                        }),
                    },
                    draw(Value::ONE),
                ])),
                slot: 0,
                filter: None,
            },
        ]),
    )
}

/// Methods of the Mighty — choose one or more: destroy an artifact, destroy
/// a tapped creature, a counter on each of your creatures.
pub fn methods_of_the_mighty() -> CardDefinition {
    spell(
        "Methods of the Mighty",
        cost(&[generic(3), w()]),
        CardType::Instant,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) },
                plus(Selector::EachPermanent(yours(R::Creature)), Value::ONE),
            ],
            min: 1,
            max: 3,
            allow_repeats: false,
        },
    )
}

/// West Coast Expansion — draw X; with X of 5 or more, you may cast a Hero
/// spell from your hand free.
pub fn west_coast_expansion() -> CardDefinition {
    spell(
        "West Coast Expansion",
        cost(&[x(), u(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            draw(Value::XFromCost),
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5)),
                then: Box::new(may(
                    "Cast a Hero spell from your hand without paying its mana cost?",
                    Effect::CastWithoutPayingImmediate {
                        what: Selector::Take {
                            inner: Box::new(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter: hero() }),
                            count: Box::new(Value::ONE),
                        },
                        source_zone: Zone::Hand,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    },
                )),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}
