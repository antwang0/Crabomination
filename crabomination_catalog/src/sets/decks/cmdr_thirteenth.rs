//! Commander: the cards the **Paradox Power** precon (WHO, The Thirteenth
//! Doctor + Yasmin Khan) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_thirteenth.rs`.
//!
//! Residuals (each also on its card):
//! - **Become the Pilot** — no "can't be blocked unless it's attacking its
//!   owner" clause.
//! - **Bigger on the Inside** — the mana and the cascade are yours (no
//!   target player).
//! - **Bill Potts** — only spells are copied, not activated abilities.
//! - **Clara Oswald** — "Impossible Girl" (a chosen color as commander) is
//!   not modeled.
//! - **Last Night Together** — any creature may attack in the extra combat.
//! - **Lunar Hatchling** — escape doesn't also exile a land you control.
//! - **Me, the Immortal** — its counters don't stay with it across zones.
//! - **Psychic Paper** — no chosen name and creature type.
//! - **River Song's Diary** — every resolving instant and sorcery is exiled
//!   (not only those cast from a hand).
//! - **Ryan Sinclair** — cards with mana value above its power are skipped
//!   rather than ending the reveal.
//! - **Strax, Sontaran Nurse** — the creature it fights is picked, not
//!   targeted.
//! - **The Fugitive Doctor** — the flashback cost is the card's mana cost,
//!   not {2}{R}{G}.
//! - **Truth or Consequences** — each consequences vote picks its own random
//!   opponent.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, AdditionalCastCost, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{
    battle_cry, etb, investigate, mentor, on_attack, on_cast, on_dies, target_any, target_filtered, training,
};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, VoteOption, VoteTally, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, u, Color, ManaCost, SpendRestriction};
use crate::sets::{tap_add_any_color, tap_add_colorless};
use crabomination_base::tokens::{clue_token, food_token, treasure_token};

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

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

/// A legendary Doctor's companion (CR 702.124m).
fn companion(mut def: CardDefinition) -> CardDefinition {
    def.keywords.push(Keyword::DoctorsCompanion);
    legend(def)
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: ManaCost, host: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter: host } },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn plus(what: Selector, n: i32) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::Const(n) }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn make(t: TokenDefinition, n: i32) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(t) }
}

fn token(name: &str, color: Color, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

/// Paradox — "Whenever you cast a spell from anywhere other than your hand,
/// `effect`."
fn paradox(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(R::SpellNotCastFromHand)),
        effect,
    }
}

fn not_from_hand() -> Value {
    Value::SpellsCastNotFromHandThisTurn(PlayerRef::You)
}

fn surveil(n: i32) -> Effect {
    Effect::Surveil { who: PlayerRef::You, amount: Value::Const(n) }
}

fn may_planeswalk() -> Effect {
    Effect::MayDo { description: "Planeswalk?".into(), body: Box::new(Effect::Planeswalk { who: PlayerRef::You }) }
}

fn cascade_next_spell() -> Effect {
    Effect::OnYourNextSpellCastThisTurn { body: Box::new(Effect::Cascade { max_mv: Value::TriggerEventAmount, filter: None }) }
}

fn doctors() -> R {
    R::HasCreatureType(CreatureType::Doctor)
}

/// Become the Pilot — enchant noncommander creature; you control it; +2/+2.
/// Residual: no "can't be blocked unless attacking its owner" clause.
pub fn become_the_pilot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::attached_to(Selector::This),
        })],
        ..aura(
            "Become the Pilot",
            cost(&[generic(3), u(), u()]),
            R::Creature.and(R::Not(Box::new(R::IsCommander))),
            EquipBonus { power: 2, toughness: 2, ..Default::default() },
        )
    }
}

/// Bigger on the Inside — enchant artifact or land; it has "{T}: two mana of
/// any one color; the next spell this turn has cascade".
/// Residual: the mana and the cascade are yours (no target player).
pub fn bigger_on_the_inside() -> CardDefinition {
    aura(
        "Bigger on the Inside",
        cost(&[generic(3), r(), g()]),
        R::Artifact.or(R::Land),
        EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(2)) },
                    cascade_next_spell(),
                ]),
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Bill Potts — an instant or sorcery of yours targeting only Bill is copied
/// (once each turn).
/// Residual: an activated ability targeting only Bill isn't copied.
pub fn bill_potts() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(trigger_is(R::SpellTargetsOnlySource.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))))
                .once_per_turn(),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
        }],
        ..companion(creature("Bill Potts", cost(&[generic(3), r()]), vec![CreatureType::Human], 2, 4))
    }
}

/// Clara Oswald — your Doctors' triggered abilities trigger an additional
/// time.
/// Residual: "Impossible Girl" (a chosen color as commander) isn't modeled.
pub fn clara_oswald() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a triggered ability of a Doctor you control triggers, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerTriggersOfType { types: vec![CreatureType::Doctor], exclude_source: false },
        }],
        ..companion(creature("Clara Oswald", cost(&[generic(6)]), vec![CreatureType::Human, CreatureType::Advisor], 2, 6))
    }
}

/// Confession Dial — entering, surveil 3; {T}: a legendary creature card in
/// your graveyard gains escape (its cost plus three other cards).
pub fn confession_dial() -> CardDefinition {
    CardDefinition {
        name: "Confession Dial",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(surveil(3))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantEscapeThisTurn {
                what: target_filtered(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::InYourGraveyard),
                ),
                exile_count: 3,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dan Lewis — your noncreature, non-Equipment artifacts are Equipment with
/// +1/+0 and equip {1}.
pub fn dan_lewis() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Noncreature, non-Equipment artifacts you control are Equipment with \"Equipped creature gets +1/+0\" and equip {1}.",
            effect: StaticEffect::MatchingArtifactsAreEquipment {
                filter: yours(R::Artifact),
                equip: cost(&[generic(1)]),
                power: 1,
                filtered_equip: None,
            },
        }],
        ..companion(creature("Dan Lewis", cost(&[generic(1), r()]), vec![CreatureType::Human], 2, 2))
    }
}

/// Danny Pink — mentor; your creatures draw a card the first time each turn
/// counters are put on them.
pub fn danny_pink() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![mentor()],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have \"Whenever one or more counters are put on this creature for the first time each turn, draw a card.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: yours(R::Creature),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::AnyCounterAdded, EventScope::SelfSource).once_per_turn(),
                    effect: draw(Value::ONE),
                }),
            },
        }],
        ..legend(creature(
            "Danny Pink",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Advisor],
            4,
            3,
        ))
    }
}

/// Decaying Time Loop — discard your hand, draw that many; retrace.
pub fn decaying_time_loop() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Retrace],
        ..spell(
            "Decaying Time Loop",
            cost(&[generic(3), r()]),
            CardType::Instant,
            Effect::DiscardHandDrawThatMany { who: Selector::You },
        )
    }
}

/// Flaming Tyrannosaurus — menace; paradox: 3 damage to any target and a
/// +1/+1 counter; dying, damage equal to its power to each opponent.
pub fn flaming_tyrannosaurus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            paradox(Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
                plus(Selector::This, 1),
            ])),
            on_dies(Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::PowerOf(Box::new(Selector::This)),
            }),
        ],
        ..creature("Flaming Tyrannosaurus", cost(&[generic(5), r(), r()]), vec![CreatureType::Dinosaur], 5, 5)
    }
}

/// Flatline — opposing creatures have base 0/1 until end of turn.
pub fn flatline() -> CardDefinition {
    spell(
        "Flatline",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::SetBasePT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(0),
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Frost Fair Lure Fish — entering, two 1/1 Fish and two tapped Treasures;
/// your Fish have haste and can't be blocked by Humans; foretell.
pub fn frost_fair_lure_fish() -> CardDefinition {
    let fish = || Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Fish)));
    let tapped_treasure = TokenDefinition { tapped: true, ..treasure_token() };
    CardDefinition {
        foretell_cost: Some(cost(&[generic(3), u(), r()])),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            make(token("Fish", Color::Blue, vec![CreatureType::Fish], 1, 1, vec![]), 2),
            make(tapped_treasure, 2),
        ]))],
        static_abilities: vec![
            StaticAbility {
                description: "Fish you control have haste.",
                effect: StaticEffect::GrantKeyword { applies_to: fish(), keyword: Keyword::Haste },
            },
            StaticAbility {
                description: "Fish you control can't be blocked by Humans.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: fish(),
                    keyword: Keyword::CantBeBlockedByCreatureType(CreatureType::Human),
                },
            },
        ],
        ..creature("Frost Fair Lure Fish", cost(&[generic(5), u(), r()]), vec![CreatureType::Fish], 7, 7)
    }
}

/// Fugitive of the Judoon — I: a 1/1 ward-2 Human and a 4/4 Alien Rhino;
/// II: investigate; III: you may exile a Human and an artifact of yours to
/// put a Doctor from your library onto the battlefield.
pub fn fugitive_of_the_judoon() -> CardDefinition {
    let human = R::HasCreatureType(CreatureType::Human);
    let one = |filter: R| Selector::Take {
        inner: Box::new(Selector::EachPermanent(yours(filter))),
        count: Box::new(Value::ONE),
    };
    saga("Fugitive of the Judoon", cost(&[generic(4), g()]), vec![
        (
            1,
            Effect::Seq(vec![
                make(
                    token("Human", Color::White, vec![CreatureType::Human], 1, 1, vec![Keyword::Ward(WardCost::generic(2))]),
                    1,
                ),
                make(token("Alien Rhino", Color::White, vec![CreatureType::Alien, CreatureType::Rhino], 4, 4, vec![]), 1),
            ]),
        ),
        (2, investigate(1)),
        (
            3,
            Effect::If {
                cond: Predicate::All(vec![
                    Predicate::SelectorExists(Selector::EachPermanent(yours(human.clone()))),
                    Predicate::SelectorExists(Selector::EachPermanent(yours(R::Artifact))),
                ]),
                then: Box::new(Effect::MayDo {
                    description: "Exile a Human and an artifact you control to search for a Doctor?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Exile { what: one(human) },
                        Effect::Exile { what: one(R::Artifact) },
                        Effect::Search {
                            who: PlayerRef::You,
                            filter: doctors(),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        ),
    ])
}

/// Gallifrey Council Chamber — entering, surveil 1; {T}: {C}; {T}: any
/// color for Time Lord or Alien spells and abilities.
pub fn gallifrey_council_chamber() -> CardDefinition {
    CardDefinition {
        name: "Gallifrey Council Chamber",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(surveil(1))],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::CreatureOfEitherTypeOrItsAbility(CreatureType::TimeLord, CreatureType::Alien),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Heaven Sent — I, II: investigate; III: 1 damage to each opponent, then
/// draw seven if one is at 0 or less, else exile it and you may cast it this
/// turn.
pub fn heaven_sent() -> CardDefinition {
    saga("Heaven Sent", cost(&[u(), r()]), vec![
        (1, investigate(1)),
        (2, investigate(1)),
        (
            3,
            Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                Effect::If {
                    cond: Predicate::PlayerLifeAtMost { who: PlayerRef::EachOpponent, life: 0 },
                    then: Box::new(draw(Value::Const(7))),
                    else_: Box::new(Effect::Seq(vec![
                        Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                        Effect::GrantMayPlay {
                            what: Selector::This,
                            duration: MayPlayDuration::EndOfThisTurn,
                            to_owner: false,
                            exile_after: false,
                            pay_own_cost: true,
                            any_color: false,
                        },
                    ])),
                },
            ]),
        ),
    ])
}

/// Impending Flux — paradox: 1 + spells cast not from hand damage to each
/// opponent and each creature they control; foretell.
pub fn impending_flux() -> CardDefinition {
    let x = || Value::Sum(vec![Value::ONE, not_from_hand()]);
    CardDefinition {
        foretell_cost: Some(cost(&[generic(1), r(), r()])),
        ..spell(
            "Impending Flux",
            cost(&[generic(2), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: x() },
                Effect::DealDamage { to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)), amount: x() },
            ]),
        )
    }
}

/// Iraxxa, Empress of Mars — trample, battle cry; paradox: a 2/2 Alien
/// Warrior.
pub fn iraxxa_empress_of_mars() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            battle_cry(1),
            paradox(make(
                token("Alien Warrior", Color::Red, vec![CreatureType::Alien, CreatureType::Warrior], 2, 2, vec![]),
                1,
            )),
        ],
        ..legend(creature(
            "Iraxxa, Empress of Mars",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Alien, CreatureType::Warrior],
            5,
            4,
        ))
    }
}

/// Jenny Flint — partner with Madame Vastra, first strike, training;
/// sacrificing a Clue or Food puts a +1/+1 counter on another creature.
pub fn jenny_flint() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Madame Vastra".into()), Keyword::FirstStrike],
        triggered_abilities: vec![
            training(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(trigger_is(
                    R::HasArtifactSubtype(ArtifactSubtype::Clue).or(R::HasArtifactSubtype(ArtifactSubtype::Food)),
                )),
                effect: plus(target_filtered(yours(R::Creature).and(R::OtherThanSource)), 1),
            },
        ],
        ..legend(creature(
            "Jenny Flint",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            2,
        ))
    }
}

/// Karvanista, Loyal Lupari — vigilance, trample, haste; attacking, a +1/+1
/// counter on each Human you control. Adventure: Lupari Shield — Humans you
/// control gain indestructible until your next turn.
pub fn karvanista_loyal_lupari() -> CardDefinition {
    let humans = || Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Human)));
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::AddCounter {
            what: humans(),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        adventure: Some(Box::new(Adventure {
            name: "Lupari Shield",
            cost: cost(&[generic(1), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::GrantKeyword { what: humans(), keyword: Keyword::Indestructible, duration: Duration::UntilYourNextUntap },
        })),
        ..legend(creature(
            "Karvanista, Loyal Lupari",
            cost(&[generic(4), g()]),
            vec![CreatureType::Alien, CreatureType::Dog, CreatureType::Soldier],
            5,
            5,
        ))
    }
}

/// Last Night Together — two target creatures untap, get two +1/+1 counters
/// and vigilance, indestructible and haste; an extra combat after this main
/// phase.
/// Residual: any creature may attack in the extra combat.
pub fn last_night_together() -> CardDefinition {
    let grant = |kw: Keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword: kw, duration: Duration::EndOfTurn };
    spell(
        "Last Night Together",
        cost(&[generic(3), r(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::Untap { what: Selector::Target(0), up_to: None },
                    plus(Selector::Target(0), 2),
                    grant(Keyword::Vigilance),
                    grant(Keyword::Indestructible),
                    grant(Keyword::Haste),
                ])),
            },
            Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE },
        ]),
    )
}

/// Lunar Hatchling — flying, trample; basic landcycling {2}; escape.
/// Residual: escape doesn't also exile a land you control.
pub fn lunar_hatchling() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Trample,
            Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand))),
            Keyword::Escape(cost(&[generic(4), g(), u()]), 5),
        ],
        ..creature(
            "Lunar Hatchling",
            cost(&[generic(4), g(), u()]),
            vec![CreatureType::Alien, CreatureType::Beast],
            6,
            6,
        )
    }
}

/// Madame Vastra — partner with Jenny Flint; must be blocked if able; a
/// creature it damaged this turn dying makes a Clue and a Food.
pub fn madame_vastra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Jenny Flint".into()), Keyword::MustBeBlocked],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(trigger_is(R::DamagedBySourceThisTurn)),
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(clue_token()) },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(food_token()) },
            ]),
        }],
        ..legend(creature(
            "Madame Vastra",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Lizard, CreatureType::Detective],
            3,
            3,
        ))
    }
}

/// Me, the Immortal — each combat on your turn, a +1/+1, first strike,
/// vigilance or menace counter; castable from your graveyard by discarding
/// two cards.
/// Residual: its counters don't stay with it across zones.
pub fn me_the_immortal() -> CardDefinition {
    let kw = |k: Keyword| Effect::AddKeywordCounter { what: Selector::This, keyword: k, amount: Value::ONE };
    CardDefinition {
        keywords: vec![Keyword::GraveyardCast],
        flashback_additional_cost: vec![AdditionalCastCost::Discard { count: 2, filter: None }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::ChooseMode(vec![
                plus(Selector::This, 1),
                kw(Keyword::FirstStrike),
                kw(Keyword::Vigilance),
                kw(Keyword::Menace),
            ]),
        }],
        ..legend(creature(
            "Me, the Immortal",
            cost(&[generic(2), g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            3,
        ))
    }
}

/// Memory Worm — paradox: 2 damage to target player, who loots; a +1/+1
/// counter.
pub fn memory_worm() -> CardDefinition {
    let that = || Selector::Player(PlayerRef::Target(0));
    CardDefinition {
        triggered_abilities: vec![paradox(Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(2) },
            Effect::Discard { who: that(), amount: Value::ONE, random: false },
            Effect::Draw { who: that(), amount: Value::ONE },
            plus(Selector::This, 1),
        ]))],
        ..creature("Memory Worm", cost(&[generic(1), r()]), vec![CreatureType::Alien, CreatureType::Worm], 1, 1)
    }
}

/// Nardole, Resourceful Cyborg — {T}: {U} per counter on it for noncreature
/// spells; undying.
pub fn nardole_resourceful_cyborg() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::OfColor(Color::Blue, Value::TotalCountersOn { what: Box::new(Selector::This) })),
                    SpendRestriction::NoncreatureSpellsOnly,
                ),
            },
            ..Default::default()
        }],
        keywords: vec![Keyword::Undying, Keyword::DoctorsCompanion],
        supertypes: vec![Supertype::Legendary],
        ..creature("Nardole, Resourceful Cyborg", cost(&[generic(1), u()]), vec![CreatureType::Scientist], 1, 2)
    }
}

/// Ominous Cemetery — {T}: {C}; {5}, {T}, exile it: target creature's owner
/// shuffles it into their library.
pub fn ominous_cemetery() -> CardDefinition {
    CardDefinition {
        name: "Ominous Cemetery",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                exile_self_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        pos: LibraryPosition::Shuffled,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Osgood, Operation Double — casting it makes a nonlegendary token copy;
/// {T}: {C} for artifacts; paradox: investigate.
pub fn osgood_operation_double() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_cast(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: true,
                legendary: false,
                extra_keywords: vec![],
            }),
            paradox(investigate(1)),
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(Box::new(ManaPayload::Colorless(Value::ONE)), SpendRestriction::ArtifactOnly),
            },
            ..Default::default()
        }],
        ..legend(creature(
            "Osgood, Operation Double",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Alien, CreatureType::Shapeshifter],
            2,
            2,
        ))
    }
}

/// Psychic Paper — equipped creature has ward {1} and can't be blocked.
/// Residual: no chosen name and creature type.
pub fn psychic_paper() -> CardDefinition {
    CardDefinition {
        name: "Psychic Paper",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Ward(WardCost::generic(1)), Keyword::Unblockable],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Return the Past — during your turn, instants and sorceries in your
/// graveyard have flashback equal to their mana cost.
pub fn return_the_past() -> CardDefinition {
    CardDefinition {
        name: "Return the Past",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "During your turn, each instant and sorcery card in your graveyard has flashback equal to its mana cost.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GraveyardInstantsSorceriesHaveFlashback),
            },
        }],
        ..Default::default()
    }
}

/// River Song — you draw from the bottom of your library; an opponent
/// scrying, surveilling or searching grows it, and it hits them for its
/// power.
pub fn river_song() -> CardDefinition {
    let spoilers = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::OpponentControl),
        effect: Effect::Seq(vec![
            plus(Selector::This, 1),
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
        ]),
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You draw cards from the bottom of your library rather than the top.",
            effect: StaticEffect::ControllerDrawsFromBottom,
        }],
        triggered_abilities: vec![spoilers(EventKind::ScriedOrSurveiled), spoilers(EventKind::PlayerSearchedLibrary)],
        ..legend(creature(
            "River Song",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human, CreatureType::TimeLord, CreatureType::Rogue],
            2,
            2,
        ))
    }
}

/// River Song's Diary — resolving instants and sorceries are exiled with it;
/// with four or more there at your upkeep, cast one at random for free.
/// Residual: every resolving instant and sorcery is exiled, not only those
/// cast from a hand.
pub fn river_songs_diary() -> CardDefinition {
    CardDefinition {
        name: "River Song's Diary",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Book], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Whenever a player casts an instant or sorcery spell from their hand, exile it instead of putting it into a graveyard as it resolves.",
            effect: StaticEffect::ExileResolvingInstantsAndSorceries,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(Value::CardsExiledWithSourceCount, Value::Const(4)),
            ),
            effect: Effect::MayDo {
                description: "Cast a random card exiled with River Song's Diary for free?".into(),
                body: Box::new(Effect::CastWithoutPayingImmediate {
                    what: Selector::TakeRandom {
                        inner: Box::new(Selector::CardExiledWithSource),
                        count: Box::new(Value::ONE),
                    },
                    source_zone: crate::card::Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Ryan Sinclair — attacking, reveal to a nonland card of mana value up to
/// its power and cast it free.
/// Residual: higher-mana-value cards are skipped rather than ending the
/// reveal.
pub fn ryan_sinclair() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Discover {
            n: Value::PowerOf(Box::new(Selector::This)),
            filter: None,
        })],
        ..companion(creature("Ryan Sinclair", cost(&[generic(2), r()]), vec![CreatureType::Human], 2, 2))
    }
}

/// Sisterhood of Karn — enters with a +1/+1 counter; paradox doubles them.
pub fn sisterhood_of_karn() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        triggered_abilities: vec![paradox(Effect::DoubleCountersOnEach {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
        })],
        ..creature("Sisterhood of Karn", cost(&[generic(1), g()]), vec![CreatureType::Cleric], 0, 0)
    }
}

/// Sonic Screwdriver — any color; untap another artifact; scry 1; target
/// creature can't be blocked this turn.
pub fn sonic_screwdriver() -> CardDefinition {
    let paid = |n: u32, effect: Effect| ActivatedAbility {
        mana_cost: cost(&[generic(n)]),
        tap_cost: true,
        effect,
        ..Default::default()
    };
    CardDefinition {
        name: "Sonic Screwdriver",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            tap_add_any_color(),
            paid(1, Effect::Untap { what: target_filtered(R::Artifact.and(R::OtherThanSource)), up_to: None }),
            paid(2, Effect::Scry { who: PlayerRef::You, amount: Value::ONE }),
            paid(
                3,
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..Default::default()
    }
}

/// Start the TARDIS — surveil 2, draw, you may planeswalk; jump-start.
pub fn start_the_tardis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::JumpStart],
        ..spell(
            "Start the TARDIS",
            cost(&[generic(1), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![surveil(2), draw(Value::ONE), may_planeswalk()]),
        )
    }
}

/// Strax, Sontaran Nurse — vigilance, trample; {2}, {T}, sacrifice an
/// artifact: a random player's creature fights it; damaging a creature grows
/// it.
/// Residual: the creature it fights is picked, not targeted.
pub fn strax_sontaran_nurse() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Seq(vec![
                Effect::RememberPlayerOnSource { who: PlayerRef::RandomPlayer },
                Effect::Fight {
                    attacker: Selector::This,
                    defender: Selector::Take {
                        inner: Box::new(Selector::ControlledBy {
                            who: PlayerRef::ChosenPlayerOfSource,
                            filter: R::Creature.and(R::OtherThanSource),
                        }),
                        count: Box::new(Value::ONE),
                    },
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::SelfSource),
            effect: plus(Selector::This, 1),
        }],
        ..legend(creature(
            "Strax, Sontaran Nurse",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Alien, CreatureType::Cleric],
            5,
            5,
        ))
    }
}

/// Surge of Brilliance — paradox: a card per spell cast not from hand this
/// turn; foretell.
pub fn surge_of_brilliance() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[generic(1), u()])),
        ..spell("Surge of Brilliance", cost(&[generic(1), u()]), CardType::Instant, draw(not_from_hand()))
    }
}

/// TARDIS — flying Vehicle, crew 2; attacking with a Time Lord, the next
/// spell this turn has cascade and you may planeswalk.
pub fn tardis() -> CardDefinition {
    CardDefinition {
        name: "TARDIS",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::TimeLord)))),
            then: Box::new(Effect::Seq(vec![cascade_next_spell(), may_planeswalk()])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// The Flux — I: 4 damage to an opposing creature; II–V: impulse the top
/// card; VI: add {R}{R}{R}{R}{R}{R}.
pub fn the_flux() -> CardDefinition {
    let impulse = || Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::ONE,
        duration: MayPlayDuration::EndOfThisTurn,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: false,
        uncast_penalty: None,
    };
    saga("The Flux", cost(&[generic(2), r(), r()]), vec![
        (
            1,
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::Const(4),
            },
        ),
        (2, impulse()),
        (3, impulse()),
        (4, impulse()),
        (5, impulse()),
        (6, Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::Const(6)) }),
    ])
}

/// The Foretold Soldier — must be blocked, one blocker at most; dealing
/// damage exiles it foretold; foretell.
pub fn the_foretold_soldier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustBeBlocked, Keyword::CantBeBlockedByMoreThanOne],
        foretell_cost: Some(cost(&[generic(1), g()])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource).once_per_batch(),
            effect: Effect::ExileSelfForetold,
        }],
        ..creature(
            "The Foretold Soldier",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Alien, CreatureType::Zombie, CreatureType::Soldier],
            6,
            6,
        )
    }
}

/// The Fugitive Doctor — entering, investigate; attacking, you may sacrifice
/// a Clue so an instant or sorcery in your graveyard gains flashback.
/// Residual: the flashback cost is the card's mana cost, not {2}{R}{G}.
pub fn the_fugitive_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(investigate(1)),
            on_attack(Effect::MaySacrifice {
                description: "Sacrifice a Clue to give an instant or sorcery flashback?".into(),
                filter: R::HasArtifactSubtype(ArtifactSubtype::Clue),
                count: Value::ONE,
                then: Box::new(Effect::GrantFlashbackThisTurn {
                    what: target_filtered(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard)),
                }),
                else_: None,
            }),
        ],
        ..legend(creature(
            "The Fugitive Doctor",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::TimeLord, CreatureType::Doctor],
            4,
            4,
        ))
    }
}

/// The Thirteenth Doctor — paradox: a +1/+1 counter on target creature;
/// your end step untaps your creatures with counters.
pub fn the_thirteenth_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            paradox(plus(target_filtered(R::Creature), 1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::Untap {
                    what: Selector::EachPermanent(yours(R::Creature).and(R::WithAnyCounter)),
                    up_to: None,
                },
            },
        ],
        ..legend(creature(
            "The Thirteenth Doctor",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::TimeLord, CreatureType::Doctor],
            2,
            2,
        ))
    }
}

/// The Twelfth Doctor — the first spell each turn you cast from anywhere but
/// your hand has demonstrate; copying a spell grows it.
pub fn the_twelfth_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(trigger_is(R::SpellNotCastFromHand))
                    .once_per_turn(),
                effect: Effect::Demonstrate,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCopied, EventScope::YourControl),
                effect: plus(Selector::This, 1),
            },
        ],
        ..legend(creature(
            "The Twelfth Doctor",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::TimeLord, CreatureType::Doctor],
            4,
            4,
        ))
    }
}

/// Thijarian Witness — flash; another creature that dies attacking or
/// blocking alone is exiled, and you investigate.
pub fn thijarian_witness() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf)),
                trigger_is(R::IsAttackingAlone.or(R::IsBlockingAlone)),
            ])),
            effect: Effect::Seq(vec![Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile }, investigate(1)]),
        }],
        ..creature(
            "Thijarian Witness",
            cost(&[generic(1), g()]),
            vec![CreatureType::Alien, CreatureType::Cleric],
            0,
            4,
        )
    }
}

/// Truth or Consequences — secret council: a card per truth vote, 3 damage
/// to a random opponent per consequences vote.
/// Residual: each consequences vote picks its own random opponent.
pub fn truth_or_consequences() -> CardDefinition {
    spell(
        "Truth or Consequences",
        cost(&[generic(2), u(), r()]),
        CardType::Sorcery,
        Effect::Vote {
            tally: VoteTally::PerVote,
            options: vec![
                VoteOption::new("truth", draw(Value::ONE)),
                VoteOption::new(
                    "consequences",
                    Effect::DealDamage { to: Selector::Player(PlayerRef::RandomOpponent), amount: Value::Const(3) },
                ),
            ],
        },
    )
}

/// Twice Upon a Time — with two or more Doctors, take an extra turn (then
/// exile it). Adventure: Unlikely Meeting — tutor a Doctor to hand.
pub fn twice_upon_a_time() -> CardDefinition {
    CardDefinition {
        cast_condition: Some(Predicate::ValueAtLeast(
            Value::PermanentCountControlledByMatching(PlayerRef::You, R::Creature.and(doctors())),
            Value::Const(2),
        )),
        exile_on_resolve: true,
        adventure: Some(Box::new(Adventure {
            name: "Unlikely Meeting",
            cost: cost(&[generic(2), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Search { who: PlayerRef::You, filter: doctors(), to: ZoneDest::Hand(PlayerRef::You) },
        })),
        ..spell(
            "Twice Upon a Time",
            cost(&[generic(4), u(), u()]),
            CardType::Sorcery,
            Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
        )
    }
}

/// Yasmin Khan — {T}: exile the top card; play it until your next end step.
pub fn yasmin_khan() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::UntilYourNextEndStep,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: false,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..companion(creature(
            "Yasmin Khan",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Detective],
            3,
            3,
        ))
    }
}
