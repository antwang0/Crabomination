//! Commander: the cards **The Fantastic Four** precon (MSC, Invisible Woman)
//! needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_invisible_woman.rs`.
//!
//! Residuals (each also on its card):
//! - **Mister Fantastic** — the copies keep the original's targets, and an
//!   activated ability of yours is a legal target too.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{cascade, etb, on_you_attack, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, u, w, x};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// A rebound spell (CR 702.88).
fn rebound(def: CardDefinition) -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Rebound], ..def }
}

fn token(name: &str, colors: Vec<Color>, card_types: Vec<CardType>, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.to_string(),
        power: p,
        toughness: t,
        card_types,
        colors,
        keywords,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

fn make(definition: Arc<TokenDefinition>, n: i32) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition }
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

/// {R}{G}{W}{U} — the Fantastic Four's shared rider cost.
fn rgwu() -> ManaCost {
    cost(&[r(), g(), w(), u()])
}

/// "If you've cast a noncreature spell this turn" (intervening, CR 603.4).
fn cast_noncreature_this_turn() -> Predicate {
    Predicate::NoncreatureSpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE }
}

/// "At the beginning of combat on your turn, if you've cast a noncreature
/// spell this turn, [effect]."
fn combat_if_cast_noncreature(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl)
            .with_filter(cast_noncreature_this_turn()),
        effect,
    }
}

/// "Whenever you cast a noncreature spell, [effect]."
fn on_cast_noncreature(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
        effect,
    }
}

fn keyword_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

// ── Commander ───────────────────────────────────────────────────────────────

/// Invisible Woman — a 0/3 Wall each combat after a noncreature spell;
/// whenever you attack, {R}{G}{W}{U} makes one creature unblockable and +1/+0
/// per creature you control.
pub fn invisible_woman() -> CardDefinition {
    let wall = token("Wall", vec![], vec![CardType::Creature], vec![CreatureType::Wall], 0, 3, vec![
        Keyword::Defender,
        Keyword::Reach,
    ]);
    legendary(CardDefinition {
        triggered_abilities: vec![
            combat_if_cast_noncreature(make(wall, 1)),
            on_you_attack(Effect::MayPay {
                description: "Pay {R}{G}{W}{U}?".into(),
                mana_cost: rgwu(),
                // CR 603.7 — "when you do" targets after the payment.
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::Seq(vec![
                        Effect::PumpPT {
                            what: target_filtered(R::Creature),
                            power: Value::count(Selector::EachPermanent(yours(R::Creature))),
                            toughness: Value::Const(0),
                            duration: Duration::EndOfTurn,
                        },
                        keyword_eot(Selector::Target(0), Keyword::Unblockable),
                    ])),
                }),
                else_: None,
            }),
        ],
        ..creature("Invisible Woman", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3)
    })
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Alicia Masters, Skilled Sculptor — a Treasure each combat after a
/// noncreature spell; at your end step every creature goes home to its owner.
pub fn alicia_masters_skilled_sculptor() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![
            combat_if_cast_noncreature(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(treasure_token()),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::OwnersGainControlOf { filter: R::Creature },
            },
        ],
        ..creature(
            "Alicia Masters, Skilled Sculptor",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            0,
            4,
        )
    })
}

/// Black Bolt, Inhuman King — flying; noncreature spells pump it; Lethal
/// Voice: an opponent targeting it loses a nonland permanent.
pub fn black_bolt_inhuman_king() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_cast_noncreature(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec { actor_is_opponent: true, ..EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource) },
                effect: Effect::Destroy {
                    what: target_filtered(R::Nonland.and(R::ControlledByTriggerPlayer)),
                },
            },
        ],
        ..creature(
            "Black Bolt, Inhuman King",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Inhuman, CreatureType::Noble, CreatureType::Hero],
            3,
            3,
        )
    })
}

/// Council of Reeds — your creatures ignore the legend rule; each combat
/// after a noncreature spell it copies itself.
pub fn council_of_reeds() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "The \"legend rule\" doesn't apply to creatures you control.",
            effect: StaticEffect::LegendRuleDoesntApplyToYourMatching(R::Creature),
        }],
        triggered_abilities: vec![combat_if_cast_noncreature(Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::ONE,
            source: Selector::This,
            extra_creature_types: vec![],
            extra_card_types: vec![],
            override_pt: None,
            override_colors: None,
            enters_tapped: false,
            non_legendary: false,
            legendary: false,
            extra_keywords: vec![],
        })],
        ..creature(
            "Council of Reeds",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            2,
            2,
        )
    })
}

/// Crystal, Inhuman Princess — flying; noncreature spells burn each opponent
/// for their colors; {T}: add {R}, {G}, {W} or {U}.
pub fn crystal_inhuman_princess() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_cast_noncreature(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ColorCountOf(Box::new(Selector::TriggerSource)),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Red, Color::Green, Color::White, Color::Blue], Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Crystal, Inhuman Princess",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Inhuman, CreatureType::Noble, CreatureType::Hero],
            2,
            3,
        )
    })
}

/// Dragon Man, Reformed Robot — flying; power is the greatest mana value
/// among your noncreature permanents and graveyard cards; castable from the
/// graveyard by discarding a card.
pub fn dragon_man_reformed_robot() -> CardDefinition {
    legendary(CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::GraveyardCast],
        flashback_additional_cost: vec![AdditionalCastCost::Discard { count: 1, filter: None }],
        dynamic_pt: Some(DynamicPt::GreatestNoncreatureManaValueYoursAndGraveyard { toughness: 5 }),
        ..creature(
            "Dragon Man, Reformed Robot",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Dragon, CreatureType::Robot],
            0,
            5,
        )
    })
}

/// Franklin Richards, Ascendant — discover 6 each combat after a
/// noncreature spell.
pub fn franklin_richards_ascendant() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![combat_if_cast_noncreature(Effect::Discover { n: Value::Const(6), filter: None })],
        ..creature(
            "Franklin Richards, Ascendant",
            cost(&[generic(5), r()]),
            vec![CreatureType::Mutant, CreatureType::Hero],
            6,
            6,
        )
    })
}

/// Galactus, Devourer of Worlds — flying, trample, indestructible; exiles a
/// permanent on entry; attacks the opponent with the most life each combat
/// unless you control Silver Surfer.
pub fn galactus_devourer_of_worlds() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Indestructible],
        static_abilities: vec![StaticAbility {
            description: "Galactus attacks an opponent with the most life among your opponents each combat if able \
                          unless you control a creature named Silver Surfer, Galactus's Herald.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
                    yours(R::Creature).and(R::HasName("Silver Surfer, Galactus's Herald".into())),
                )))),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::MustAttackChosenPlayer],
            },
        }],
        triggered_abilities: vec![
            etb(Effect::Move { what: target_filtered(R::Permanent), to: ZoneDest::Exile }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::RememberPlayerOnSource { who: PlayerRef::HighestLifeOpponent },
            },
        ],
        ..creature(
            "Galactus, Devourer of Worlds",
            cost(&[generic(10)]),
            vec![CreatureType::Elder, CreatureType::Alien],
            12,
            12,
        )
    })
}

/// H.E.R.B.I.E., Lovable Robot — flying; surveil 1 each combat after a
/// noncreature spell; {T}: {C}; {1}, {T}: any color.
pub fn herbie_lovable_robot() -> CardDefinition {
    legendary(CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![combat_if_cast_noncreature(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::ONE,
        })],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
        ],
        ..creature(
            "H.E.R.B.I.E., Lovable Robot",
            cost(&[generic(2)]),
            vec![CreatureType::Robot, CreatureType::Scout],
            1,
            1,
        )
    })
}

/// Human Torch — flies, double strikes and hastes each combat after a
/// noncreature spell; attacking, {R}{G}{W}{U} spreads his combat damage to
/// each other opponent.
pub fn human_torch() -> CardDefinition {
    let spread = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
            amount: Value::TriggerEventAmount,
        },
    };
    legendary(CardDefinition {
        triggered_abilities: vec![
            combat_if_cast_noncreature(Effect::Seq(vec![
                keyword_eot(Selector::This, Keyword::Flying),
                keyword_eot(Selector::This, Keyword::DoubleStrike),
                keyword_eot(Selector::This, Keyword::Haste),
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayPay {
                    description: "Pay {R}{G}{W}{U}?".into(),
                    mana_cost: rgwu(),
                    body: Box::new(Effect::GrantTriggeredAbility {
                        what: Selector::This,
                        trigger: Box::new(spread),
                        duration: Duration::EndOfTurn,
                    }),
                    else_: None,
                },
            },
        ],
        ..creature("Human Torch", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Hero], 3, 2)
    })
}

/// Lockjaw, Slobbering Teleporter — vigilance; each combat after a
/// noncreature spell it grows, and when it does, it and one other creature
/// of yours can't be blocked.
pub fn lockjaw_slobbering_teleporter() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![combat_if_cast_noncreature(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::ReflexiveTrigger {
                body: Box::new(Effect::Seq(vec![
                    keyword_eot(Selector::This, Keyword::Unblockable),
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: yours(R::Creature).and(R::OtherThanSource),
                        effect: Box::new(keyword_eot(Selector::Target(0), Keyword::Unblockable)),
                    },
                ])),
            },
        ]))],
        ..creature(
            "Lockjaw, Slobbering Teleporter",
            cost(&[generic(1), u()]),
            vec![CreatureType::Inhuman, CreatureType::Dog, CreatureType::Hero],
            1,
            1,
        )
    })
}

/// Medusa, Inhuman Queen — reach, vigilance; grows whenever anyone casts a
/// noncreature spell.
pub fn medusa_inhuman_queen() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature(
            "Medusa, Inhuman Queen",
            cost(&[generic(2), g()]),
            vec![CreatureType::Inhuman, CreatureType::Noble, CreatureType::Hero],
            2,
            2,
        )
    })
}

/// Mister Fantastic — reach, vigilance; draws each combat after a
/// noncreature spell; {R}{G}{W}{U}, {T}: copy one of your triggered
/// abilities twice.
///
/// ⚠ Residual: the copies keep the original's targets, and an activated
/// ability of yours is a legal target too.
pub fn mister_fantastic() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        triggered_abilities: vec![combat_if_cast_noncreature(draw(Value::ONE))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: rgwu(),
            effect: Effect::CopyAbility {
                what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByYou)),
                times: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Mister Fantastic",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            2,
            4,
        )
    })
}

/// Mister Fantastic, Reed Richards — reach; your tokens entering may draw a
/// card.
pub fn mister_fantastic_reed_richards() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsToken })
                .once_per_batch(),
            effect: may("Draw a card?", draw(Value::ONE)),
        }],
        ..creature(
            "Mister Fantastic, Reed Richards",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            2,
            4,
        )
    })
}

/// Namor, Atlantean King — flying; noncreature spells make Merfolk; attacking
/// a player with more life than you pumps your other attackers on them.
pub fn namor_atlantean_king() -> CardDefinition {
    let merfolk = token("Merfolk", vec![Color::Blue], vec![CardType::Creature], vec![CreatureType::Merfolk], 1, 1, vec![]);
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_cast_noncreature(make(merfolk, 1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::ValueAtLeast(
                    Value::LifeOf(PlayerRef::DefendingPlayer),
                    Value::Sum(vec![Value::LifeOf(PlayerRef::You), Value::ONE]),
                )),
                effect: Effect::PumpOtherAttackersOnSamePlayer { power: Value::Const(2), toughness: Value::Const(0) },
            },
        ],
        ..creature(
            "Namor, Atlantean King",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Mutant, CreatureType::Merfolk, CreatureType::Noble],
            2,
            2,
        )
    })
}

/// Power Pack — flying, vigilance, trample, haste; connecting exiles a
/// random instant or sorcery from your graveyard to cast free next upkeep.
pub fn power_pack() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileRandomFromGraveyardWithSource {
                    who: PlayerRef::You,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::YourNextUpkeep,
                    body: Box::new(may(
                        "Cast the exiled card without paying its mana cost?",
                        Effect::CastWithoutPayingImmediate {
                            what: Selector::CardExiledWithSource,
                            source_zone: Zone::Exile,
                            exile_after: true,
                            copy: false,
                            reduce_generic: 0,
                            pay_own_cost: false,
                        },
                    )),
                },
            ]),
        }],
        ..creature(
            "Power Pack",
            cost(&[generic(1), r(), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Hero],
            4,
            4,
        )
    })
}

/// Silver Surfer, Galactus's Herald — flying; may tutor Galactus on entry;
/// connecting makes a creature attack that player each combat until the end
/// of your next turn.
pub fn silver_surfer_galactuss_herald() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(may(
                "Search your library for Galactus, Devourer of Worlds?",
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasName("Galactus, Devourer of Worlds".into()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            )),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MustAttackPlayerFor {
                    attacker: target_filtered(R::Creature),
                    defender: Selector::Player(PlayerRef::Triggerer),
                    duration: Duration::UntilEndOfYourNextTurn,
                },
            },
        ],
        ..creature(
            "Silver Surfer, Galactus's Herald",
            cost(&[generic(5)]),
            vec![CreatureType::Alien, CreatureType::Hero],
            4,
            5,
        )
    })
}

/// The Thing — trample; four counters each combat after a noncreature
/// spell; attacking, {R}{G}{W}{U} doubles each kind of counter on your
/// permanents.
pub fn the_thing() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            combat_if_cast_noncreature(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayPay {
                    description: "Pay {R}{G}{W}{U}?".into(),
                    mana_cost: rgwu(),
                    body: Box::new(Effect::ReflexiveTrigger {
                        body: Box::new(Effect::ApplyToTargets {
                            max_targets: 8,
                            min_targets: 0,
                            filter: yours(R::Permanent).and(R::WithAnyCounter),
                            effect: Box::new(Effect::DoubleAllCountersOn { what: Selector::Target(0) }),
                        }),
                    }),
                    else_: None,
                },
            },
        ],
        ..creature("The Thing", cost(&[generic(5), g()]), vec![CreatureType::Human, CreatureType::Hero], 5, 5)
    })
}

/// Valeria Richards, Precocious — noncreature spells cost {1} less; your
/// first noncreature spell each turn draws a card.
pub fn valeria_richards_precocious() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Noncreature, amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(R::Noncreature),
                Predicate::ValueEquals(Value::NoncreatureSpellsCastThisTurn(PlayerRef::You), Value::ONE),
            ])),
            effect: draw(Value::ONE),
        }],
        ..creature(
            "Valeria Richards, Precocious",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            3,
            3,
        )
    })
}

/// Willie Lumpkin, Postman — can't be blocked; connecting draws you a card
/// and offers that player one, whose price is not attacking you next turn.
pub fn willie_lumpkin_postman() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                draw(Value::ONE),
                Effect::PlayersMayAccept {
                    who: PlayerRef::Triggerer,
                    description: "Draw a card? (You can't attack them or their permanents during your next turn.)"
                        .into(),
                    on_accept: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
                        // "During their next turn": their next turn comes
                        // before yours, so until your next untap covers it.
                        Effect::GrantCantAttackYou {
                            what: Selector::EachPermanent(R::Creature.and(R::ControlledByTriggerPlayer)),
                            duration: Duration::UntilYourNextUntap,
                        },
                    ])),
                    if_any: Box::new(Effect::Noop),
                    otherwise: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..creature(
            "Willie Lumpkin, Postman",
            cost(&[w(), u()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            1,
            3,
        )
    })
}

// ── Artifacts, lands and enchantments ──────────────────────────────────────

/// Baxter Building — {T}: {C}; {4}, {T}: four mana in any colors; {4}, {T}:
/// draw, with a toughness-4 creature.
pub fn baxter_building() -> CardDefinition {
    CardDefinition {
        name: "Baxter Building",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::Const(4)) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                condition: Some(Predicate::SelectorExists(Selector::EachPermanent(
                    yours(R::Creature).and(R::ToughnessAtLeast(4)),
                ))),
                effect: draw(Value::ONE),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cosmic Crucible — four mana at your first main phase; once a turn, copy a
/// noncreature spell you cast.
pub fn cosmic_crucible() -> CardDefinition {
    let mut copy = on_cast_noncreature(may("Copy that spell?", Effect::CopySpell {
        what: Selector::TriggerSource,
        count: Value::ONE,
    }));
    copy.event.once_per_turn = true;
    CardDefinition {
        name: "Cosmic Crucible",
        cost: cost(&[generic(4), g(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::Const(4)) },
            },
            copy,
        ],
        ..Default::default()
    }
}

/// Negative Zone Portal — {2}, {T}: exile a card from an opponent's
/// graveyard, drawing for a creature; with four creature cards under it, a
/// lost upkeep flip sacrifices it and hands one back.
pub fn negative_zone_portal() -> CardDefinition {
    CardDefinition {
        name: "Negative Zone Portal",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                    then: Box::new(draw(Value::ONE)),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Move {
                    what: target_filtered(R::Any.and(R::InOpponentGraveyard)),
                    to: ZoneDest::ExileWithSourceStamp,
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(Value::CardsExiledWithSourceMatching(R::Creature), Value::Const(4)),
            ),
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Noop),
                on_tails: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::IsSource },
                    Effect::ReturnRandomExiledWithSource,
                ])),
            },
        }],
        ..Default::default()
    }
}

/// The Fantasticar — flying Vehicle; noncreature spells may animate it; your
/// fourth noncreature spell each turn may trade it for four 4/4 Constructs.
pub fn the_fantasticar() -> CardDefinition {
    let construct = token(
        "Construct",
        vec![],
        vec![CardType::Artifact, CardType::Creature],
        vec![CreatureType::Construct],
        4,
        4,
        vec![Keyword::Flying, Keyword::Haste],
    );
    CardDefinition {
        name: "The Fantasticar",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_cast_noncreature(may(
                "Have The Fantasticar become an artifact creature until end of turn?",
                Effect::AnimateAsCreature { what: Selector::This, duration: Duration::EndOfTurn },
            )),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                    Predicate::CastSpellMatches(R::Noncreature),
                    Predicate::ValueEquals(Value::NoncreatureSpellsCastThisTurn(PlayerRef::You), Value::Const(4)),
                ])),
                effect: Effect::MaySacrifice {
                    description: "Sacrifice The Fantasticar for four 4/4 Constructs?".into(),
                    filter: R::IsSource,
                    count: Value::ONE,
                    then: Box::new(make(construct, 4)),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Unstable Molecule Suit — +2/+2 and indestructible; equip commander {2},
/// equip {4}.
pub fn unstable_molecule_suit() -> CardDefinition {
    CardDefinition {
        name: "Unstable Molecule Suit",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equip_filtered_cost: Some((R::IsCommander, cost(&[generic(2)]))),
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Indestructible],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Instants and sorceries ──────────────────────────────────────────────────

/// Fantastic Elasticity — bounce a nonland permanent or regrow an instant or
/// sorcery; rebound.
pub fn fantastic_elasticity() -> CardDefinition {
    rebound(spell(
        "Fantastic Elasticity",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Nonland.and(R::Permanent)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::Move {
                what: target_filtered(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).from_your_graveyard(),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
    ))
}

/// First Family — draw X and gain X, X the colors among your permanents and
/// the spells you cast this turn.
pub fn first_family() -> CardDefinition {
    let x = || Value::ColorsAmongYoursAndSpellsCastThisTurn;
    spell(
        "First Family",
        cost(&[generic(2), g(), u()]),
        CardType::Instant,
        Effect::Seq(vec![draw(x()), Effect::GainLife { who: Selector::You, amount: x() }]),
    )
}

/// Flame On! — +1/+1 counters per noncreature, nonland card in your
/// graveyard and flying until end of turn; rebound.
pub fn flame_on() -> CardDefinition {
    rebound(spell(
        "Flame On!",
        cost(&[generic(4), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Noncreature.and(R::Nonland),
                }),
            },
            keyword_eot(Selector::Target(0), Keyword::Flying),
        ]),
    ))
}

/// Into the Time Vortex — cascade; rebound.
pub fn into_the_time_vortex() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![cascade(5)],
        ..rebound(spell("Into the Time Vortex", cost(&[generic(4), r()]), CardType::Sorcery, Effect::Noop))
    }
}

/// Invisible Force Field — up to four of your permanents gain indestructible
/// until end of turn; rebound.
pub fn invisible_force_field() -> CardDefinition {
    rebound(spell(
        "Invisible Force Field",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: yours(R::Permanent),
            effect: Box::new(keyword_eot(Selector::Target(0), Keyword::Indestructible)),
        },
    ))
}

/// It's Clobberin' Time! — one of your creatures bites an opponent's, or
/// destroy an artifact or enchantment; rebound.
pub fn its_clobberin_time() -> CardDefinition {
    rebound(spell(
        "It's Clobberin' Time!",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::DealDamageEqualToPower {
                source: target_filtered(yours(R::Creature)),
                target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            },
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        ]),
    ))
}

/// Nova Flame — X counters on one of your creatures, then it deals its power
/// to each other creature.
pub fn nova_flame() -> CardDefinition {
    spell(
        "Nova Flame",
        cost(&[x(), generic(2), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(yours(R::Creature)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            },
            Effect::DealDamageEqualToPowerToEach {
                source: Selector::Target(0),
                targets: Selector::EachPermanent(R::Creature),
                each_opponent: false,
            },
        ]),
    )
}

/// Recurring Insight — draw a card per card in an opponent's hand; rebound.
pub fn recurring_insight() -> CardDefinition {
    rebound(spell(
        "Recurring Insight",
        cost(&[generic(4), u(), u()]),
        CardType::Sorcery,
        Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(draw(Value::HandSizeOf(PlayerRef::Target(0)))),
        },
    ))
}

/// Ultimate Nullification — sacrifice a legendary creature: exile every
/// creature and graveyard, then it goes to the bottom of its owner's library.
pub fn ultimate_nullification() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: yours(R::Creature).and(R::HasSupertype(Supertype::Legendary)),
            count: 1,
        }],
        library_bottom_on_resolve: true,
        ..spell(
            "Ultimate Nullification",
            cost(&[generic(4), w()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move { what: Selector::EachPermanent(R::Creature), to: ZoneDest::Exile },
                Effect::ExileAllGraveyards { filter: None, opponents_only: false },
            ]),
        )
    }
}
