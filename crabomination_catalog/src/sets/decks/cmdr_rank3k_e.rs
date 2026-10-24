//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), fifth file. Tests in `tests/recent_b/cmdr_rank3k_e.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, on_attack, sneak, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, ManaCost};
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

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn counters(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(def) }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn powerstone() -> TokenDefinition {
    TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() }
}

fn opponents_turn() -> Predicate {
    Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))
}

fn your_commanders() -> R {
    R::Creature.and(R::IsCommander).and(R::OwnedByYou)
}

// ── Harold and Bob, First Numens ─────────────────────────────────────────────

/// Hope Tender — {1}{G} 2/2 Human Druid. {1}, {T}: untap target land. {1},
/// {T}, exert it: untap two target lands. (Residual: the exert lands as the
/// ability resolves — the engine has no exert activation cost.)
pub fn hope_tender() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Untap { what: target_filtered(R::Land), up_to: None },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::SkipNextUntap { what: Selector::This },
                    Effect::ApplyToTargets {
                        max_targets: 2,
                        min_targets: 1,
                        filter: R::Land,
                        effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Hope Tender", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Druid], 2, 2)
    }
}

/// Sculptor of Winter — {1}{G} snow 2/2 Elf Rogue. {T}: untap target snow land.
pub fn sculptor_of_winter() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::Land.and(R::HasSupertype(Supertype::Snow))), up_to: None },
            ..Default::default()
        }],
        ..creature("Sculptor of Winter", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Rogue], 2, 2)
    }
}

// ── Aphemia, the Cacophony ───────────────────────────────────────────────────

/// Blightcaster — {3}{B} 2/3 Human Wizard. Whenever you cast an enchantment
/// spell, you may have target creature get -2/-2 until end of turn.
pub fn blightcaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(trigger_is(R::HasCardType(CardType::Enchantment))),
            effect: Effect::MayDo {
                description: "Give target creature -2/-2?".into(),
                body: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..creature("Blightcaster", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

/// Sinister Concoction — {B} enchantment. {B}, pay 1 life, mill a card, discard
/// a card, sacrifice it: destroy target creature.
pub fn sinister_concoction() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            life_cost: 1,
            mill_cost: 1,
            discard_cost: Some((R::Any, 1)),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..spell("Sinister Concoction", cost(&[b()]), CardType::Enchantment, Effect::Noop)
    }
}

// ── Casey Jones, Back Alley Brute ────────────────────────────────────────────

/// Casey Jones, Jury-Rig Justiciar — {1}{R} legendary 2/1 haste. ETB: look at
/// the top four; you may reveal an artifact card and put it into your hand;
/// the rest on the bottom in a random order.
pub fn casey_jones_jury_rig_justiciar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(R::Artifact),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })))],
        ..legend(
            "Casey Jones, Jury-Rig Justiciar",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            2,
            1,
        )
    }
}

/// Curse of Stalked Prey — {1}{R} Aura Curse. Whenever a creature deals combat
/// damage to enchanted player, a +1/+1 counter on that creature.
pub fn curse_of_stalked_prey() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer).with_filter(
                Predicate::SamePlayer(PlayerRef::TriggerEventPlayer, PlayerRef::EnchantedPlayer),
            ),
            effect: counters(Selector::TriggerSource, Value::ONE),
        }],
        ..spell("Curse of Stalked Prey", cost(&[generic(1), r()]), CardType::Enchantment, Effect::Noop)
    }
}

// ── Savanti Romero / Black Widow, Deadly Hunter ──────────────────────────────

/// Savanti Romero, Time's Exile — {3}{B}{B} legendary 4/4 trample. Beginning of
/// combat on your turn: a +1/+1 counter on it, then draw X and lose X life, X
/// its counters.
pub fn savanti_romero_times_exile() -> CardDefinition {
    let x = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                counters(Selector::This, Value::ONE),
                draw(x()),
                Effect::LoseLife { who: Selector::You, amount: x() },
            ]),
        }],
        ..legend(
            "Savanti Romero, Time's Exile",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Demon, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Madame Null, Power Broker — {2}{B} legendary 1/3 deathtouch. Another creature
/// you control enters: you may pay life equal to its power; if you do, that
/// many +1/+1 counters on it.
pub fn madame_null_power_broker() -> CardDefinition {
    let power = || Value::PowerOf(Box::new(Selector::TriggerSource));
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(Predicate::All(vec![
                trigger_is(R::Creature),
                Predicate::ValueAtLeast(power(), Value::ONE),
            ])),
            effect: Effect::MayPayLife {
                description: "Pay life equal to its power for that many +1/+1 counters?".into(),
                amount: power(),
                body: Box::new(counters(Selector::TriggerSource, power())),
                else_: None,
            },
        }],
        ..legend(
            "Madame Null, Power Broker",
            cost(&[generic(2), b()]),
            vec![CreatureType::Demon, CreatureType::Advisor],
            1,
            3,
        )
    }
}

/// Origin of Black Widow — {2}{B} Saga. I: each opponent sacrifices a creature.
/// II: creatures you control gain deathtouch until end of turn. III: each
/// opponent loses 1 life per creature card in their graveyard.
pub fn origin_of_black_widow() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Creature,
                },
            ),
            (2, grant(yours(R::Creature), Keyword::Deathtouch)),
            (
                3,
                Effect::EachPlayerDoes {
                    who: PlayerRef::EachOpponent,
                    body: Box::new(Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
                    }),
                },
            ),
        ],
        ..spell("Origin of Black Widow", cost(&[generic(2), b()]), CardType::Enchantment, Effect::Noop)
    }
}

// ── Cyan, Vengeful Samurai ───────────────────────────────────────────────────

/// Wand of Vertebrae — {1} artifact. {T}: mill a card. {2}, {T}, exile it:
/// shuffle up to five target cards from your graveyard into your library.
pub fn wand_of_vertebrae() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Mill { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                exile_self_cost: true,
                effect: Effect::ApplyToTargets {
                    max_targets: 5,
                    min_targets: 0,
                    filter: R::InYourGraveyard,
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Shuffled },
                    }),
                },
                ..Default::default()
            },
        ],
        ..spell("Wand of Vertebrae", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// Airlift Chaplain — {2}{W} 1/1 flying Human Cleric. ETB: mill three; you may
/// put a Plains or a creature card with mana value 3 or less from among them
/// into your hand; if you don't, a +1/+1 counter on it.
pub fn airlift_chaplain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MillThenToHand {
            amount: Value::Const(3),
            filter: R::HasLandType(crate::card::LandType::Plains).or(R::Creature.and(R::ManaValueAtMost(3))),
            otherwise: Some(Box::new(counters(Selector::This, Value::ONE))),
        })],
        ..creature("Airlift Chaplain", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 1)
    }
}

// ── Immortus, Master of Eternity ─────────────────────────────────────────────

/// Immortus, Master of Eternity — {2}{U}{U} legendary 2/2. {T}: add {U} per
/// card you've drawn this turn, spent only on noncreature spells. Power-up
/// {5}{U}{U}: each player shuffles hand and graveyard into library and draws
/// seven; a +1/+1 counter on Immortus.
pub fn immortus_master_of_eternity() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::OfColor(Color::Blue, Value::CardsDrawnThisTurn(PlayerRef::You))),
                        crate::mana::SpendRestriction::NoncreatureSpellsOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5), u(), u()]),
                power_up: true,
                effect: Effect::Seq(vec![
                    Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
                    Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(7) },
                    counters(Selector::This, Value::ONE),
                ]),
                ..Default::default()
            },
        ],
        ..legend(
            "Immortus, Master of Eternity",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Hero, CreatureType::Villain],
            2,
            2,
        )
    }
}

/// Refocus — {1}{U} instant: untap target creature; draw a card.
pub fn refocus() -> CardDefinition {
    spell(
        "Refocus",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![Effect::Untap { what: target_filtered(R::Creature), up_to: None }, draw(Value::ONE)]),
    )
}

// ── Kongming, "Sleeping Dragon" ──────────────────────────────────────────────

/// Kabira Vindicator — {3}{W} 2/4 Human Knight. Level up {2}{W}. LEVEL 2-4: 3/6,
/// other creatures you control get +1/+1. LEVEL 5+: 4/8, +2/+2 instead.
pub fn kabira_vindicator() -> CardDefinition {
    use crate::card::LevelBand;
    let anthem = |n: u32| StaticAbility {
        description: "Other creatures you control get +1/+1 (level 2+; +2/+2 at level 5+).",
        effect: StaticEffect::WhileCountersAtLeast {
            kind: CounterType::Level,
            n,
            inner: Box::new(StaticEffect::PumpPT {
                applies_to: yours(R::Creature.and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            }),
        },
    };
    CardDefinition {
        level_bands: vec![
            LevelBand { min: 2, max: Some(4), power: 3, toughness: 6, keywords: vec![] },
            LevelBand { min: 5, max: None, power: 4, toughness: 8, keywords: vec![] },
        ],
        // Level 5+ stacks the second +1/+1 on the first.
        static_abilities: vec![anthem(2), anthem(5)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            sorcery_speed: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Level, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Kabira Vindicator", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 4)
    }
}

/// In the Trenches — {1}{W}{W} enchantment: creatures you control get +1/+1.
/// {5}{W}: exile target nonland permanent you don't control until this leaves
/// the battlefield; sorcery speed, only once.
pub fn in_the_trenches() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: yours(R::Creature), power: 1, toughness: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w()]),
            sorcery_speed: true,
            activate_once: true,
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(R::Permanent.and(R::Land.negate()).and(R::ControlledByYou.negate())),
                return_to: crate::card::ExileReturnZone::Battlefield,
            },
            ..Default::default()
        }],
        ..spell("In the Trenches", cost(&[generic(1), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

// ── Heidegger, Shinra Executive ──────────────────────────────────────────────

/// Benalish Commander — {3}{W} */* Human Soldier; P/T each the Soldiers you
/// control. Suspend X—{X}{W}{W}, X can't be 0. Whenever a time counter is
/// removed from it while it's exiled, a 1/1 white Soldier.
pub fn benalish_commander() -> CardDefinition {
    let soldiers = || Value::count(yours(R::Creature.and(R::HasCreatureType(CreatureType::Soldier))));
    CardDefinition {
        keywords: vec![Keyword::Suspend(0, cost(&[x(), w(), w()]))],
        static_abilities: vec![StaticAbility {
            description: "Benalish Commander's power and toughness are each equal to the number of Soldiers you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: soldiers(), toughness: soldiers() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::Time), EventScope::SelfSource),
            effect: make(
                Value::ONE,
                TokenDefinition {
                    name: "Soldier".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
                    ..Default::default()
                },
            ),
        }],
        ..creature("Benalish Commander", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Soldier], 0, 0)
    }
}

/// Captain America, Unbowed — {3}{W} legendary 3/4 flash. ETB: Soldiers and
/// Heroes you control gain indestructible until end of turn.
pub fn captain_america_unbowed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(grant(
            yours(R::Creature.and(R::HasCreatureType(CreatureType::Soldier).or(R::HasCreatureType(CreatureType::Hero)))),
            Keyword::Indestructible,
        ))],
        ..legend(
            "Captain America, Unbowed",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            3,
            4,
        )
    }
}

// ── Doc Samson ───────────────────────────────────────────────────────────────

/// Doc Samson, Super Psychiatrist — {4}{G} legendary 3/6. Counters you'd put on
/// a permanent you control are that many plus one of each kind. {T}: add X mana
/// of any one color, X its power.
pub fn doc_samson_super_psychiatrist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would put one or more counters on a permanent you control, put that many plus one of each of those kinds instead.",
            effect: StaticEffect::ExtraCounterOnPermanentsYouPut,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::PowerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..legend(
            "Doc Samson, Super Psychiatrist",
            cost(&[generic(4), g()]),
            vec![CreatureType::Gamma, CreatureType::Doctor, CreatureType::Hero],
            3,
            6,
        )
    }
}

/// Restorative Technique — {2}{G} sorcery: target player gains 2 life and
/// fetches a basic land tapped; a +1/+1 counter on up to one target creature.
pub fn restorative_technique() -> CardDefinition {
    spell(
        "Restorative Technique",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        // Slot 0 (the player) is required; slot 1 ("up to one" creature) may be
        // declined.
        Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::GainLife { who: target_filtered(R::Player), amount: Value::Const(2) },
                Effect::Search {
                    who: PlayerRef::Target(0),
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::Target(0), tapped: true },
                },
                counters(Selector::TargetFiltered { slot: 1, filter: R::Creature }, Value::ONE),
            ])),
        },
    )
}

// ── Dyfed, the Guiding Hand ──────────────────────────────────────────────────

/// Dyfed, the Guiding Hand — {3}{U}{U} legendary planeswalker, loyalty 4; can
/// be your commander. +1: two tapped Powerstones. −X: untap X target
/// artifacts. −6: search for an artifact card onto the battlefield.
pub fn dyfed_the_guiding_hand() -> CardDefinition {
    CardDefinition {
        name: "Dyfed, the Guiding Hand",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Dyfed], ..Default::default() },
        base_loyalty: 4,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility { loyalty_cost: 1, effect: make(Value::Const(2), powerstone()), ..Default::default() },
            LoyaltyAbility { condition: None,
                loyalty_cost: 0,
                x_cost: true,
                effect: Effect::CapTargetsAtX {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: R::Artifact,
                        effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
                    }),
                },
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Artifact,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Geology Enthusiast — {3}{U}{U} 2/5 Human Artificer. Your end step: a tapped
/// Powerstone. {6}: draw a card and a +1/+1 counter on it.
pub fn geology_enthusiast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: make(Value::ONE, powerstone()),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            effect: Effect::Seq(vec![draw(Value::ONE), counters(Selector::This, Value::ONE)]),
            ..Default::default()
        }],
        ..creature("Geology Enthusiast", cost(&[generic(3), u(), u()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 5)
    }
}

// ── Gustha Ebbasdotter ───────────────────────────────────────────────────────

/// Gustha Ebbasdotter — {3}{U}{U} legendary 3/3, protection from red. Whenever
/// you cast your first spell during each opponent's turn, draw three, then put
/// two cards from your hand on top of your library.
pub fn gustha_ebbasdotter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                opponents_turn(),
                Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::ONE },
            ])),
            effect: Effect::Seq(vec![
                draw(Value::Const(3)),
                Effect::PutCardFromHandOnTopOfLibrary { who: Selector::You },
                Effect::PutCardFromHandOnTopOfLibrary { who: Selector::You },
            ]),
        }],
        ..legend("Gustha Ebbasdotter", cost(&[generic(3), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 3)
    }
}

/// Unwelcome Sprite — {1}{U} 2/1 flying Faerie Rogue. Whenever you cast a spell
/// during an opponent's turn, surveil 2.
pub fn unwelcome_sprite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(opponents_turn()),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
        }],
        ..creature("Unwelcome Sprite", cost(&[generic(1), u()]), vec![CreatureType::Faerie, CreatureType::Rogue], 2, 1)
    }
}

// ── Boss Uramon, Shadow's Reach ──────────────────────────────────────────────

/// Boss Uramon, Shadow's Reach — {3}{B} legendary 2/2 shadow. Attacking Ninjas
/// and Rogues you control have deathtouch. One or more Ninjas and/or Rogues you
/// control deal combat damage to a player: a 1/1 black Rat Rogue.
pub fn boss_uramon_shadows_reach() -> CardDefinition {
    let ninja_or_rogue = || R::HasCreatureType(CreatureType::Ninja).or(R::HasCreatureType(CreatureType::Rogue));
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        static_abilities: vec![StaticAbility {
            description: "Attacking Ninjas and Rogues you control have deathtouch.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::IsAttacking).and(ninja_or_rogue())),
                keyword: Keyword::Deathtouch,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(trigger_is(ninja_or_rogue()))
                .once_per_batch(),
            effect: make(
                Value::ONE,
                TokenDefinition {
                    name: "Rat Rogue".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Rat, CreatureType::Rogue], ..Default::default() },
                    ..Default::default()
                },
            ),
        }],
        ..legend("Boss Uramon, Shadow's Reach", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 2)
    }
}

/// Elektra, Daughter of the Hand — {2}{B}{B} legendary 3/3, sneak {1}{B}{B}.
/// ETB: destroy target creature an opponent controls with power 3 or less.
pub fn elektra_daughter_of_the_hand() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[generic(1), b(), b()]))],
        alternative_cost: Some(sneak(cost(&[generic(1), b(), b()]))),
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::PowerAtMost(3))),
        })],
        ..legend(
            "Elektra, Daughter of the Hand",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Ninja, CreatureType::Villain],
            3,
            3,
        )
    }
}

// ── Syr Faren, the Hengehammer ───────────────────────────────────────────────

/// Syr Faren, the Hengehammer — {G}{G} legendary 2/2. Attacks: another target
/// attacking creature gets +X/+X until end of turn, X Syr Faren's power.
pub fn syr_faren_the_hengehammer() -> CardDefinition {
    let x = || Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
            power: x(),
            toughness: x(),
            duration: Duration::EndOfTurn,
        })],
        ..legend("Syr Faren, the Hengehammer", cost(&[g(), g()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Hardy Outlander — {1}{G} legendary Background. Commander creatures you own
/// have "Whenever this creature attacks a player, if no opponent has more life
/// than that player, another target creature you control gets +X/+X until end
/// of turn, X this creature's power." Your own life total doesn't count.
pub fn hardy_outlander() -> CardDefinition {
    let x = || Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"Whenever this creature attacks a player, if no opponent has more life than that player, another target creature you control gets +X/+X until end of turn, where X is this creature's power.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: your_commanders(),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::All(vec![
                        trigger_is(R::IsAttackingOpponentPlayer),
                        Predicate::PlayerHasMostLifeAmongOpponents { who: PlayerRef::DefendingPlayer },
                    ])),
                    effect: Effect::PumpPT {
                        what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                        power: x(),
                        toughness: x(),
                        duration: Duration::EndOfTurn,
                    },
                }),
            },
        }],
        ..spell("Hardy Outlander", cost(&[generic(1), g()]), CardType::Enchantment, Effect::Noop)
    }
}

// ── Dragonlord Kolaghan ──────────────────────────────────────────────────────

/// Dragonlord Kolaghan — {4}{B}{R} legendary 6/5 flying, haste; other creatures
/// you control have haste. Whenever an opponent casts a creature or
/// planeswalker spell with the same name as a card in their graveyard, that
/// player loses 10 life.
pub fn dragonlord_kolaghan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have haste.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(R::Creature.and(R::OtherThanSource)), keyword: Keyword::Haste },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(trigger_is(R::Creature.or(R::Planeswalker))),
            // The name check reads the caster's graveyard, so it runs in the
            // caster's frame (`SharesNameWithControllerGraveyardCard` reads
            // "you").
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::Triggerer,
                body: Box::new(Effect::If {
                    cond: trigger_is(R::SharesNameWithControllerGraveyardCard),
                    then: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(10) }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        }],
        ..legend("Dragonlord Kolaghan", cost(&[generic(4), b(), r()]), vec![CreatureType::Elder, CreatureType::Dragon], 6, 5)
    }
}

// ── Wydwen, the Biting Gale ──────────────────────────────────────────────────

/// Runechanter's Pike — {2} Equipment: first strike and +X/+0, X the instant and
/// sorcery cards in your graveyard. Equip {2}.
pub fn runechanters_pike() -> CardDefinition {
    CardDefinition {
        name: "Runechanter's Pike",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::FirstStrike], ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Equipped creature gets +X/+0, where X is the number of instant and sorcery cards in your graveyard.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
                toughness: Value::ZERO,
            },
        }],
        ..Default::default()
    }
}

/// Glen Elendra Pranksters — {3}{U} 1/3 flying Faerie Wizard. Whenever you cast
/// a spell during an opponent's turn, you may return target creature you
/// control to its owner's hand.
pub fn glen_elendra_pranksters() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(opponents_turn()),
            effect: Effect::MayDo {
                description: "Return a creature you control to its owner's hand?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
        }],
        ..creature(
            "Glen Elendra Pranksters",
            cost(&[generic(3), u()]),
            vec![CreatureType::Faerie, CreatureType::Wizard],
            1,
            3,
        )
    }
}

// ── Edgar, Moonlit Sovereign ─────────────────────────────────────────────────

/// Edgar, Moonlit Sovereign — {3}{G}{G} legendary 4/4 flash. Your end step, if
/// you didn't cast a spell this turn: two +1/+1 counters on him. {4}{G}: a +1/+1
/// counter on each creature you control with a +1/+1 counter.
pub fn edgar_moonlit_sovereign() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::ZERO }),
            effect: counters(Selector::This, Value::Const(2)),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            effect: counters(yours(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))), Value::ONE),
            ..Default::default()
        }],
        ..legend(
            "Edgar, Moonlit Sovereign",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Werewolf, CreatureType::Noble],
            4,
            4,
        )
    }
}

/// Spirit of the Hunt — {1}{G}{G} 3/3 flash Wolf Spirit. ETB: each other Wolf or
/// Werewolf you control gets +0/+3 until end of turn.
pub fn spirit_of_the_hunt() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: yours(
                R::Creature
                    .and(R::OtherThanSource)
                    .and(R::HasCreatureType(CreatureType::Wolf).or(R::HasCreatureType(CreatureType::Werewolf))),
            ),
            power: Value::ZERO,
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Spirit of the Hunt", cost(&[generic(1), g(), g()]), vec![CreatureType::Wolf, CreatureType::Spirit], 3, 3)
    }
}

// ── Thor, Guardian of Midgard ────────────────────────────────────────────────

/// The Vision and Scarlet Witch — {2}{R}{R} legendary 3/3 flying artifact
/// creature. Whenever you cast a spell, add {R} and a +1/+1 counter on it.
pub fn the_vision_and_scarlet_witch() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) },
                counters(Selector::This, Value::ONE),
            ]),
        }],
        ..legend(
            "The Vision and Scarlet Witch",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Mutant, CreatureType::Hero],
            3,
            3,
        )
    }
}

// ── Super-Adaptoid ───────────────────────────────────────────────────────────

/// Viv Vision, Teen Synthezoid — {3} legendary 2/2 flying Robot Hero artifact
/// creature. Attacks: draw a card if her power is 4 or greater. Power-up {7}:
/// two +1/+1 counters.
pub fn viv_vision_teen_synthezoid() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::EntityMatches { what: Selector::This, filter: R::PowerAtLeast(4) },
            then: Box::new(draw(Value::ONE)),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7)]),
            power_up: true,
            effect: counters(Selector::This, Value::Const(2)),
            ..Default::default()
        }],
        ..legend("Viv Vision, Teen Synthezoid", cost(&[generic(3)]), vec![CreatureType::Robot, CreatureType::Hero], 2, 2)
    }
}

// ── Mister Fantastic, Reed Richards / Flitwing ───────────────────────────────

/// Daily Bugle Newspaper — {2} artifact. {2}, {T}: draw, then discard; create a
/// Treasure.
pub fn daily_bugle_newspaper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                draw(Value::ONE),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                make(Value::ONE, crabomination_base::tokens::treasure_token()),
            ]),
            ..Default::default()
        }],
        ..spell("Daily Bugle Newspaper", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// April O'Neil, Human Element — {3}{U} legendary 2/5. Whenever a player casts
/// an artifact, instant or sorcery spell, you create a Mutagen token.
pub fn april_oneil_human_element() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(trigger_is(
                R::HasCardType(CardType::Artifact).or(R::HasCardType(CardType::Instant)).or(R::HasCardType(CardType::Sorcery)),
            )),
            effect: make(Value::ONE, crabomination_base::tokens::mutagen_token()),
        }],
        ..legend(
            "April O'Neil, Human Element",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            5,
        )
    }
}
