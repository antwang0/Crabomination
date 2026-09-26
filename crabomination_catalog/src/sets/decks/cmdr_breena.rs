//! Commander: the cards the **Silverquill Statement** precon (C21, Breena,
//! the Demagogue) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_breena.rs`.
//!
//! Residuals (each also on its card):
//! - **Breena, the Demagogue** — the two counters go on your greatest-power
//!   creature (the engine's pick).
//! - **Author of Shadows** — the castable card is the first nonland card
//!   exiled, not a chosen one.
//! - **Bloodthirsty Blade** / **Parasitic Impetus** — the goad is renewed at
//!   the beginning of each combat by a trigger, not a static.
//! - **Bold Plagiarist** — copies +1/+1 counters only; an opponent's own
//!   Plagiarist is never the creature it copies from.
//! - **Guardian Archon** — "protection from the chosen player" is hexproof
//!   and indestructible for the permanent, and your life can't drop this turn
//!   is not modeled; the chosen player is the engine's most hostile opponent.
//! - **Inkshield** — the Inklings count the power of the unblocked creatures
//!   attacking you as it resolves, not the damage actually prevented.
//! - **Nils, Discipline Enforcer** — the counters go on each player's first
//!   creature, chosen rather than targeted.
//! - **Tragic Arrogance** — each player keeps their own highest-mana-value
//!   permanent of each type; the caster doesn't choose for them.
//! - **Victory Chimes** — you are always the player who adds the mana.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, hybrid, w, Color, ManaCost};

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

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn inkling() -> TokenDefinition {
    TokenDefinition {
        name: "Inkling".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Inkling], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

fn treasure() -> Arc<TokenDefinition> {
    Arc::new(crabomination_base::tokens::treasure_token())
}

/// "Whenever a player attacks one of your opponents" — CR 508.1.
fn opponent_attacked() -> EventSpec {
    EventSpec::new(EventKind::Attacks, EventScope::OpponentOfYoursAttacked)
}

/// A goad renewed at each beginning of combat on the attached creature —
/// the "equipped / enchanted creature is goaded" static, approximated.
fn your_end_step() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
}

/// Breena, the Demagogue — flying; whenever a player attacks one of your
/// opponents who has more life than another of your opponents, that player
/// draws and you put two +1/+1 counters on a creature you control.
pub fn breena_the_demagogue() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: opponent_attacked().with_filter(Predicate::ValueAtLeast(
                Value::LifeOf(PlayerRef::Triggerer),
                Value::Sum(vec![Value::LowestOpponentLife, Value::ONE]),
            )),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
                Effect::AddCounter {
                    what: Selector::GreatestPowerControlledMatching(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..creature(
            "Breena, the Demagogue",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Bird, CreatureType::Warlock],
            1,
            3,
        )
    }
}

/// Author of Shadows — enters: exile every opponent's graveyard; you may cast
/// a nonland card exiled this way while it stays exiled, with mana of any
/// type.
pub fn author_of_shadows() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExilePlayerGraveyard { who: PlayerRef::EachOpponent, filter: None },
            Effect::GrantMayPlay {
                what: Selector::Take {
                    inner: Box::new(Selector::ExiledThisResolution { filter: R::Nonland }),
                    count: Box::new(Value::ONE),
                },
                duration: crate::card::MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
        ]))],
        ..creature(
            "Author of Shadows",
            cost(&[generic(4), b()]),
            vec![CreatureType::Shade, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Bold Plagiarist — flash; an opponent putting +1/+1 counters on a creature
/// they control puts as many on this. Residual: +1/+1 counters only, and an
/// opponent's own Bold Plagiarist never counts.
pub fn bold_plagiarist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            // The event carries no actor, so "an opponent puts … on a creature
            // they control" is read off the recipient's controller. The
            // counters a Plagiarist is given are put by the opponent on a
            // creature they *don't* control, so another Plagiarist must not see
            // them — two of them fed each other to a million-action cap.
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasName("Bold Plagiarist".into()).negate()),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature(
            "Bold Plagiarist",
            cost(&[generic(3), b()]),
            vec![CreatureType::Vampire, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Boreas Charger — flying; leaving the battlefield fetches as many Plains as
/// the land gap to the opponent with the most lands: one onto the battlefield
/// tapped, the rest to hand.
pub fn boreas_charger() -> CardDefinition {
    let gap = Value::NonNeg(Box::new(Value::Diff(
        Box::new(Value::MostControlledByAnOpponent(R::Land)),
        Box::new(Value::PermanentCountControlledByMatching(PlayerRef::You, R::Land)),
    )));
    let plains = R::HasLandType(LandType::Plains);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(gap.clone(), Value::ONE),
                then: Box::new(Effect::Seq(vec![
                    // To hand first: the gap is re-read per step, and the
                    // Plains onto the battlefield would close it by one.
                    Effect::SearchUpToN {
                        who: PlayerRef::You,
                        filter: plains.clone(),
                        to: ZoneDest::Hand(PlayerRef::You),
                        count: Value::Diff(Box::new(gap), Box::new(Value::ONE)),
                    },
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: plains,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Boreas Charger", cost(&[generic(2), w()]), vec![CreatureType::Pegasus], 2, 1)
    }
}

/// Combat Calligrapher — flying; Inklings can't attack you or your
/// planeswalkers; whenever a player attacks one of your opponents, that player
/// gets a tapped 2/1 flying Inkling attacking that opponent.
pub fn combat_calligrapher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Inklings can't attack you or planeswalkers you control.",
            effect: StaticEffect::CreaturesCantAttackController {
                protect_planeswalkers: true,
                filter: Some(R::HasCreatureType(CreatureType::Inkling)),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: opponent_attacked(),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::Target(0),
                count: Value::ONE,
                definition: Arc::new(inkling()),
                cleanup: Default::default(),
                defender: Some(PlayerRef::Triggerer),
            },
        }],
        ..creature(
            "Combat Calligrapher",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Deathbringer Liege — your other white creatures and your other black
/// creatures each get +1/+1; a white spell may tap a creature, a black spell
/// may destroy a tapped one.
pub fn deathbringer_liege() -> CardDefinition {
    let lord = |c: Color| StaticAbility {
        description: "Other creatures of a color you control get +1/+1.",
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(
                R::Creature.and(R::HasColor(c)).and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            power: 1,
            toughness: 1,
        },
    };
    let on_cast = |c: Color, effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::HasColor(c))),
        effect,
    };
    let wb = || hybrid(Color::White, Color::Black);
    CardDefinition {
        static_abilities: vec![lord(Color::White), lord(Color::Black)],
        triggered_abilities: vec![
            on_cast(
                Color::White,
                Effect::MayDo {
                    description: "Tap target creature?".into(),
                    body: Box::new(Effect::Tap { what: target_filtered(R::Creature) }),
                },
            ),
            on_cast(
                Color::Black,
                Effect::MayDo {
                    description: "Destroy target creature if it's tapped?".into(),
                    // A tapped creature as the target: the untapped one the
                    // printed text could also target would do nothing.
                    body: Box::new(Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) }),
                },
            ),
        ],
        ..creature("Deathbringer Liege", cost(&[generic(2), wb(), wb(), wb()]), vec![CreatureType::Horror], 3, 4)
    }
}

/// Deathbringer Regent — flying; entering from a cast from hand with five or
/// more other creatures around destroys all other creatures.
pub fn deathbringer_regent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(
                Predicate::All(vec![
                    Predicate::CastFromHand,
                    Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                        n: Value::Const(5),
                    },
                ]),
            ),
            effect: Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)) },
        }],
        ..creature("Deathbringer Regent", cost(&[generic(5), b(), b()]), vec![CreatureType::Dragon], 5, 6)
    }
}

/// Guardian Archon — flying; as it enters, secretly choose an opponent; once:
/// you and target permanent you control gain protection from them this turn.
/// Residual: the permanent gains hexproof and indestructible; your own
/// protection isn't modeled.
pub fn guardian_archon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(Effect::ChoosePlayerForSource { opponent: true }),
        activated_abilities: vec![ActivatedAbility {
            activate_once: true,
            effect: Effect::GrantKeywords {
                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                keywords: vec![Keyword::Hexproof, Keyword::Indestructible],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Guardian Archon", cost(&[generic(4), w(), w()]), vec![CreatureType::Archon], 5, 5)
    }
}

/// Inkshield — prevent all combat damage to you this turn and make a 2/1
/// flying Inkling per point. Residual: the count is the power of the
/// unblocked creatures attacking you as it resolves.
pub fn inkshield() -> CardDefinition {
    CardDefinition {
        name: "Inkshield",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::EachPermanent(R::IsAttackingYou.and(R::IsUnblocked)))),
                definition: Arc::new(inkling()),
            },
            Effect::PreventAllCombatDamageToPlayerThisTurn { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

/// Keen Duelist — your upkeep: you and target opponent reveal your top cards,
/// each lose life equal to the other's card's mana value, and each take it.
pub fn keen_duelist() -> CardDefinition {
    let top = |who: PlayerRef| Selector::TopOfLibrary { who, count: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: target_filtered(R::Player.and(R::ControlledByOpponent)),
                    amount: Value::ManaValueOf(Box::new(top(PlayerRef::You))),
                },
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::ManaValueOf(Box::new(top(PlayerRef::Target(0)))),
                },
                Effect::Move { what: top(PlayerRef::You), to: ZoneDest::Hand(PlayerRef::You) },
                Effect::Move { what: top(PlayerRef::Target(0)), to: ZoneDest::Hand(PlayerRef::Target(0)) },
            ]),
        }],
        ..creature("Keen Duelist", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Nils, Discipline Enforcer — your end step puts a +1/+1 counter on a
/// creature of each player's; creatures with counters can't attack you or your
/// planeswalkers unless their controller pays {X}, X their counters.
/// Residual: each player's first creature, chosen rather than targeted.
pub fn nils_discipline_enforcer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Each creature with counters on it can't attack you or planeswalkers you control unless its controller pays {X}, X its counters.",
            effect: StaticEffect::AttackTaxToController {
                amount: Value::TotalCountersOn { what: Box::new(Selector::TriggerSource) },
                protect_planeswalkers: true,
                filter: Some(R::WithAnyCounter),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_end_step(),
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::AddCounter {
                    what: Selector::Take {
                        inner: Box::new(Selector::ControlledBy { who: PlayerRef::Triggerer, filter: R::Creature }),
                        count: Box::new(Value::ONE),
                    },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature(
            "Nils, Discipline Enforcer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Parasitic Impetus — enchanted creature gets +2/+2 and is goaded; when it
/// attacks, its controller loses 2 and you gain 2.
pub fn parasitic_impetus() -> CardDefinition {
    CardDefinition {
        name: "Parasitic Impetus",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +2/+2.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                    power: 2,
                    toughness: 2,
                },
            },
            StaticAbility { description: "Enchanted creature is goaded.", effect: StaticEffect::AttachedIsGoaded },
        ],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::AttachedTo(Box::new(
                        Selector::This,
                    ))))),
                    to: Selector::You,
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Pendant of Prosperity — enters under an opponent's control; {2}, {T}: its
/// controller and then its owner each draw and may put a land from hand onto
/// the battlefield.
pub fn pendant_of_prosperity() -> CardDefinition {
    let draw_and_land = |who: PlayerRef| {
        Effect::Seq(vec![
            Effect::Draw { who: Selector::Player(who.clone()), amount: Value::ONE },
            Effect::PutFromHandOntoBattlefield {
                who,
                filter: R::Land,
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        ])
    };
    let owner = PlayerRef::OwnerOf(Box::new(Selector::This));
    CardDefinition {
        enters_under_opponent_control: true,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                draw_and_land(PlayerRef::You),
                Effect::AsPlayer { who: owner.clone(), body: Box::new(draw_and_land(PlayerRef::You)) },
            ]),
            ..Default::default()
        }],
        ..artifact("Pendant of Prosperity", cost(&[generic(3)]))
    }
}

/// Stinging Study — draw X and lose X, X the mana value of a commander you
/// own on the battlefield or in the command zone.
pub fn stinging_study() -> CardDefinition {
    let mv = Value::GreatestCommanderManaValueInPlayOrCommandZone(PlayerRef::You);
    CardDefinition {
        name: "Stinging Study",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: mv.clone() },
            Effect::LoseLife { who: Selector::You, amount: mv },
        ]),
        ..Default::default()
    }
}

/// Tempting Contract — your upkeep: each opponent may create a Treasure, and
/// you create one for each who does.
pub fn tempting_contract() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::PlayersMayAccept {
                who: PlayerRef::EachOpponent,
                description: "Create a Treasure token?".into(),
                on_accept: Box::new(Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::Target(0), count: Value::ONE, definition: treasure() },
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: treasure() },
                ])),
                if_any: Box::new(Effect::Noop),
                otherwise: Box::new(Effect::Noop),
            },
        }],
        ..artifact("Tempting Contract", cost(&[generic(4)]))
    }
}

/// Tragic Arrogance — for each player an artifact, a creature, an enchantment
/// and a planeswalker are kept; everything else nonland is sacrificed.
/// Residual: the engine picks — the caster's own best of each type, each
/// opponent's weakest.
pub fn tragic_arrogance() -> CardDefinition {
    CardDefinition {
        name: "Tragic Arrogance",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        // The caster chooses for every player: its own best of each type, an
        // opponent's weakest (`SacrificeAllButOnePerTypeYouChoose`).
        effect: Effect::SacrificeAllButOnePerTypeYouChoose { who: Selector::Player(PlayerRef::EachPlayer) },
        ..Default::default()
    }
}

/// Victory Chimes — untaps during each other player's untap step; {T}: a player
/// of your choice adds {C}. Residual: you always add it.
pub fn victory_chimes() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Untap this artifact during each other player's untap step.",
            effect: StaticEffect::UntapSelfEachOtherUntapStep,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
            ..Default::default()
        }],
        ..artifact("Victory Chimes", cost(&[generic(3)]))
    }
}
