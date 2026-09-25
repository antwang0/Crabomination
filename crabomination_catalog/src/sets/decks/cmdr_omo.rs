//! Commander: the cards the **Tricky Terrain** precon (M3C, Omo, Queen of
//! Vesuva) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_omo.rs`.
//!
//! Residuals (each also on its card):
//! - **Omo, Queen of Vesuva** — "every creature type" is a Changeling grant,
//!   so a layer-4 type-line read doesn't see it.
//! - **Horizon of Progress** — "any type a land you control could produce"
//!   reads your lands' basic land types (Reflecting Pool's approximation).
//! - **Desert Warfare** — a Desert card reaches your graveyard "from your hand
//!   or library" only by a discard or a mill here.
//! - **Sunken Palace** — its rider copies a spell, not an activated ability.
//! - **Magus of the Candelabra** — untaps up to X of your lands, untargeted.
//! - **Rampant Frogantua** — every milled land goes onto the battlefield.
//! - **March from Velis Vel** — the land type is chosen as a mode.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EntersAsCopy, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, cost, g, generic, hybrid, u, x};
use crate::sets::{enters_tapped, tap_add, tap_add_colorless, tap_pay_life_add};
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

fn land(name: &'static str, types: Vec<LandType>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn count_yours(filter: R) -> Value {
    Value::count(yours(filter))
}

fn pump(what: Selector, n: Value) -> Effect {
    Effect::PumpPT { what, power: n.clone(), toughness: n, duration: Duration::EndOfTurn }
}

/// Omo, Queen of Vesuva — an everything counter on up to one land and up to
/// one creature as it enters or attacks. Residual: "every creature type" is
/// a Changeling grant.
pub fn omo_queen_of_vesuva() -> CardDefinition {
    let counter_on = |slot: u8, filter: R| Effect::AddCounter {
        what: Selector::TargetFiltered { slot, filter },
        kind: CounterType::Everything,
        amount: Value::ONE,
    };
    let mark = || Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::Seq(vec![counter_on(0, R::Land), counter_on(1, R::Creature)])),
    };
    let marked_lands = || Selector::EachPermanent(R::Land.and(R::WithCounter(CounterType::Everything)));
    let mut statics = vec![static_ab(
        "Each land with an everything counter on it is every land type in addition to its other types.",
        StaticEffect::GrantAllBasicLandTypes { applies_to: marked_lands() },
    )];
    statics.extend(LandType::NONBASIC.iter().map(|&lt| {
        static_ab(
            "Each land with an everything counter on it is every land type in addition to its other types.",
            StaticEffect::LandTypeChanger { applies_to: marked_lands(), land_type: lt, replace: false },
        )
    }));
    statics.push(static_ab(
        "Each nonland creature with an everything counter on it is every creature type.",
        StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(
                R::Creature.and(R::Nonland).and(R::WithCounter(CounterType::Everything)),
            ),
            keyword: Keyword::Changeling,
        },
    ));
    CardDefinition {
        name: "Omo, Queen of Vesuva",
        cost: cost(&[generic(2), hybrid(Color::Green, Color::Blue)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter, CreatureType::Noble],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        triggered_abilities: vec![etb(mark()), on_attack(mark())],
        static_abilities: statics,
        ..Default::default()
    }
}

/// Aggressive Biomancy — X token copies of your creature, each fighting up
/// to one creature you don't control as it enters (the copy's added
/// "when this token enters" ability, one trigger per token).
pub fn aggressive_biomancy() -> CardDefinition {
    spell(
        "Aggressive Biomancy",
        cost(&[x(), x(), g(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::XFromCost,
                source: target_filtered(R::Creature.and(R::ControlledByYou)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
            Effect::EachPushesTrigger {
                what: Selector::LastCreatedTokens,
                body: Box::new(Effect::Fight {
                    attacker: Selector::This,
                    defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                }),
            },
        ]),
    )
}

/// Basilisk Gate — {2}, {T}: +X/+X where X is your Gates.
pub fn basilisk_gate() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: pump(target_filtered(R::Creature), count_yours(R::HasLandType(LandType::Gate))),
                ..Default::default()
            },
        ],
        ..land("Basilisk Gate", vec![LandType::Gate])
    }
}

/// Copy Land — may enter as a copy of any land, an enchantment too.
pub fn copy_land() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Land,
            extra_card_types: vec![CardType::Enchantment],
            ..Default::default()
        }),
        ..spell("Copy Land", cost(&[generic(2), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Desert Warfare — sacrificed / discarded / milled Deserts come back at
/// your next end step; five or more Deserts make that many hasty Sand
/// Warriors each combat. Residual: other hand/library routes to the
/// graveyard aren't watched.
pub fn desert_warfare() -> CardDefinition {
    let desert = || Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasLandType(LandType::Desert) };
    let comes_back = || Effect::DelayUntilWithCapture {
        kind: DelayedTriggerKind::YourNextEndStep,
        capture: Selector::TriggerSource,
        body: Box::new(Effect::Move {
            what: Selector::Target(0),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        }),
    };
    let watch = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::YourControl).with_filter(desert()),
        effect: comes_back(),
    };
    let warrior = TokenDefinition {
        name: "Sand Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::Green, Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sand, CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    };
    let deserts = || yours(R::HasLandType(LandType::Desert));
    CardDefinition {
        triggered_abilities: vec![
            watch(EventKind::PermanentSacrificed),
            watch(EventKind::CardDiscarded),
            watch(EventKind::CardMilled),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl)
                    .with_filter(Predicate::SelectorCountAtLeast { sel: deserts(), n: Value::Const(5) }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::count(deserts()),
                    definition: Arc::new(warrior),
                },
            },
        ],
        ..spell("Desert Warfare", cost(&[generic(3), g()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Desert of the Indomitable — enters tapped, {G}, cycling {1}{G}.
pub fn desert_of_the_indomitable() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(1), g()]))],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(Color::Green)],
        ..land("Desert of the Indomitable", vec![LandType::Desert])
    }
}

/// Floriferous Vinewall — defender; look at six, may take a land.
pub fn floriferous_vinewall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(6),
            pick_filter: Some(R::Land),
            optional: true,
            ..Default::default()
        })))],
        ..creature(
            "Floriferous Vinewall",
            cost(&[generic(1), g()]),
            vec![CreatureType::Plant, CreatureType::Wall],
            0,
            2,
        )
    }
}

/// Hashep Oasis — {C}; {T}, 1 life: {G}; sacrifice a Desert for +3/+3.
pub fn hashep_oasis() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_add_colorless(),
            tap_pay_life_add(Color::Green, 1),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g(), g()]),
                tap_cost: true,
                sorcery_speed: true,
                sac_other_filter: Some((R::HasLandType(LandType::Desert), 1)),
                sac_other_may_be_source: true,
                effect: pump(target_filtered(R::Creature), Value::Const(3)),
                ..Default::default()
            },
        ],
        ..land("Hashep Oasis", vec![LandType::Desert])
    }
}

/// Horizon of Progress — pay 1 life for a type a land of yours could make;
/// {3}: put a land from hand tapped; {1}, sacrifice: draw. Residual: reads
/// basic land types.
pub fn horizon_of_progress() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColorYouCouldProduce },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::PutLandsFromHandOntoBattlefieldTapped { count: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..land("Horizon of Progress", vec![])
    }
}

/// Jyoti, Moag Ancient — a Forest Dryad per commander cast; land creatures
/// you control get +X/+X at each combat, X its power.
pub fn jyoti_moag_ancient() -> CardDefinition {
    let dryad = TokenDefinition {
        name: "Forest Dryad".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Land, CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad],
            land_types: vec![LandType::Forest],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CommanderCastsFromCommandZone(PlayerRef::You),
                definition: Arc::new(dryad),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
                effect: pump(yours(R::Land.and(R::Creature)), Value::PowerOf(Box::new(Selector::This))),
            },
        ],
        ..creature("Jyoti, Moag Ancient", cost(&[generic(2), g(), u()]), vec![CreatureType::Elemental], 2, 4)
    }
}

/// Magus of the Candelabra — {X}, {T}: untap X lands. Residual: up to X of
/// your own lands, untargeted.
pub fn magus_of_the_candelabra() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::Untap { what: yours(R::Land.and(R::Tapped)), up_to: Some(Value::XFromCost) },
            ..Default::default()
        }],
        ..creature("Magus of the Candelabra", cost(&[g()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 2)
    }
}

/// March from Velis Vel — your lands of a chosen nonbasic type become copies
/// of your creature with haste until end of turn; flashback {4}{U}.
/// Residual: the type is chosen as a mode.
pub fn march_from_velis_vel() -> CardDefinition {
    let modes = LandType::NONBASIC
        .iter()
        .map(|&lt| {
            let lands = || yours(R::Land.and(R::HasLandType(lt)));
            // Haste first: once copied the lands no longer have the type.
            Effect::Seq(vec![
                Effect::GrantKeyword { what: lands(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                Effect::BecomeCopyOfFor {
                    what: lands(),
                    source: target_filtered(R::Creature.and(R::ControlledByYou)),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
            ])
        })
        .collect();
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), u()]))],
        ..spell("March from Velis Vel", cost(&[generic(2), u()]), CardType::Sorcery, Effect::ChooseMode(modes))
    }
}

/// Rampant Frogantua — +10/+10 per player who has lost; on combat damage,
/// may mill that many and put the lands onto the battlefield. Residual: every
/// milled land is put onto the battlefield.
pub fn rampant_frogantua() -> CardDefinition {
    let lost = |n| {
        static_ab(
            "This creature gets +10/+10 for each player who has lost the game.",
            StaticEffect::PumpSelfIf { condition: Predicate::PlayersLostAtLeast(n), power: 10, toughness: 10, keywords: vec![] },
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![lost(1), lost(2), lost(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Mill that many cards and put the lands onto the battlefield?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::TriggerEventAmount },
                    Effect::Move {
                        what: Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter: R::Land },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                ])),
            },
        }],
        ..creature("Rampant Frogantua", cost(&[generic(2), g()]), vec![CreatureType::Frog], 3, 3)
    }
}

/// Sage of the Maze — two mana; animates a land by its Gates; untaps by
/// tapping a Gate.
pub fn sage_of_the_maze() -> CardDefinition {
    let gates = || count_yours(R::HasLandType(LandType::Gate));
    let x2 = || Value::Times(Box::new(Value::Const(2)), Box::new(gates()));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::Const(2)) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::BecomeCreature {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                    power: x2(),
                    toughness: x2(),
                    creature_types: vec![CreatureType::Citizen],
                    keywords: vec![Keyword::Haste],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_other_filter: Some(R::HasLandType(LandType::Gate)),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
        ],
        ..creature("Sage of the Maze", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Wizard], 1, 3)
    }
}

/// Summary Dismissal — exile all other spells, counter all abilities.
pub fn summary_dismissal() -> CardDefinition {
    spell(
        "Summary Dismissal",
        cost(&[generic(2), u(), u()]),
        CardType::Instant,
        Effect::ExileAllOtherSpellsCounterAllAbilities,
    )
}

/// Sunken Palace — enters tapped, {U}; {1}{U}, {T}, exile seven graveyard
/// cards: {U} that copies the spell it pays for. Residual: an ability it
/// pays for isn't copied.
pub fn sunken_palace() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                tap_cost: true,
                exile_other_filter: Some((R::Any, 7)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::OfColor(Color::Blue, Value::ONE)),
                        SpendRestriction::SpellOrAbilityCopy,
                    ),
                },
                ..Default::default()
            },
        ],
        ..land("Sunken Palace", vec![LandType::Cave])
    }
}

/// Ulvenwald Hydra — P/T equal to your lands; reach; may fetch a land tapped.
pub fn ulvenwald_hydra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        dynamic_pt: Some(DynamicPt::LandsControlled { base: 0 }),
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a land card?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
        })],
        ..creature("Ulvenwald Hydra", cost(&[generic(4), g(), g()]), vec![CreatureType::Hydra], 0, 0)
    }
}

/// Wonderscape Sage — {T}, return a land: draw, then discard unless that land
/// had a nonbasic land type (read as it last existed on the battlefield).
pub fn wonderscape_sage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            bounce_other_filter: Some((R::Land, 1)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::CostReturnedHadNonbasicLandType,
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Wonderscape Sage", cost(&[generic(1), u()]), vec![CreatureType::Moonfolk, CreatureType::Wizard], 1, 3)
    }
}
