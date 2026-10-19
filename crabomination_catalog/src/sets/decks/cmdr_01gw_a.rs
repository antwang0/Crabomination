//! Commander: 2026-10-19 (`01GwrDKP`) closers. Tests in
//! `tests/recent_b/cmdr_01gw_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    ActivationTrait, Duration, Effect, KeywordAct, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::effect::{StaticAbility, StaticEffect};
use crate::mana::{b, cost, g, generic, hybrid, r, x, Color, ManaCost};
use crate::game::types::TurnStep;
use std::sync::Arc;

/// The Ruinous Wrecking Crew — {X}{B}{R} 2/2 legendary Human Villain. Enters
/// with X +1/+1 counters; ETB, choose up to X: rummage; target opponent loses
/// 2 life; destroy target token; each player sacrifices a creature.
pub fn the_ruinous_wrecking_crew() -> CardDefinition {
    CardDefinition {
        name: "The Ruinous Wrecking Crew",
        cost: cost(&[x(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Villain],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        // CR 700.2 — up to X modes, X read as the trigger resolves; the default
        // takes the drain and the token first, the edict last.
        triggered_abilities: vec![etb(Effect::ChooseN {
            picks: vec![1, 2, 0, 3],
            modes: vec![
                Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
                Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
                Effect::Destroy { what: target_filtered(R::IsToken) },
                Effect::Sacrifice { who: Selector::Player(PlayerRef::EachPlayer), count: Value::ONE, filter: R::Creature },
            ],
            up_to: Some(Box::new(Value::XFromCost)),
        })],
        ..Default::default()
    }
}

/// Wee Champion — {R} 0/1 Minotaur Child Guest. Whenever you place a sticker,
/// it gets +1/+1 until end of turn (an art sticker's counter never comes up:
/// only name stickers exist, CR 123.6).
pub fn wee_champion() -> CardDefinition {
    CardDefinition {
        name: "Wee Champion",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Child, CreatureType::Guest],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Performed(KeywordAct::StickerPlaced), EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Vazi, Keen Negotiator — {2}{B}{R}{G} 3/3 legendary Human Advisor, haste.
/// {T}: target opponent creates X Treasures (X = Treasures you created this
/// turn). Whenever an opponent casts a spell or activates an ability with
/// Treasure mana, +1/+1 counter on target creature, then draw a card.
pub fn vazi_keen_negotiator() -> CardDefinition {
    let reward = || Effect::Seq(vec![
        Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
        Effect::Draw { who: Selector::You, amount: Value::ONE },
    ]);
    CardDefinition {
        name: "Vazi, Keen Negotiator",
        cost: cost(&[generic(2), b(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::Target(0),
                    count: Value::TreasuresCreatedThisTurn(PlayerRef::You),
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                }),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                    .with_filter(Predicate::CastWithTreasureMana { what: Selector::TriggerSource }),
                effect: reward(),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::AbilityActivatedWith(ActivationTrait::TreasureMana),
                    EventScope::OpponentControl,
                ),
                effect: reward(),
            },
        ],
        ..Default::default()
    }
}

/// Draugr Necromancer — {3}{B} 4/4 snow Zombie Cleric. An opponent's nontoken
/// creature that would die is exiled with an ice counter instead; you may cast
/// spells from among those cards (CR 614). Snow mana spent as any color for
/// them is not modelled — they cost their printed colors.
pub fn draugr_necromancer() -> CardDefinition {
    CardDefinition {
        name: "Draugr Necromancer",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "If a nontoken creature an opponent controls would die, exile that card with an ice counter on it instead. You may cast spells from among cards in exile your opponents own with ice counters on them.",
            effect: StaticEffect::ExileDyingOpponentCreaturesWithCounterCastable { counter: CounterType::Ice },
        }],
        ..Default::default()
    }
}

/// Sanar, Innovative First-Year — {2}{U/R}{U/R} 2/4 legendary Goblin Sorcerer.
/// Vivid: at the beginning of your first main phase, reveal until X nonland
/// cards (X = colors among your permanents), exile one revealed card of each
/// of those colors, shuffle; you may cast the exiled cards this turn.
pub fn sanar_innovative_first_year() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        name: "Sanar, Innovative First-Year",
        cost: cost(&[generic(2), ur(), ur()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::RevealUntilNonlandsExileOnePerColor {
                count: Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::ControlledByYou))),
            },
        }],
        ..Default::default()
    }
}

/// "Return up to one target Goblin card from your graveyard to your hand."
fn goblin_back() -> Effect {
    Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::Move {
            what: target_filtered(R::HasCreatureType(CreatureType::Goblin).and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        }),
    }
}

/// Grub, Storied Matriarch // Grub, Notorious Auntie — {2}{B} 2/1 legendary
/// Goblin Warlock, menace. Entering or transforming into it returns up to one
/// Goblin card from your graveyard to your hand; at your first main phase you
/// may pay {R} to transform. Notorious Auntie (red 2/1 Goblin Warrior,
/// menace): whenever it attacks, you may blight 1 — if you do, a tapped and
/// attacking token copy of the blighted creature, sacrificed at the next end
/// step; at your first main phase you may pay {B} to transform back.
pub fn grub_storied_matriarch() -> CardDefinition {
    let main_phase = || EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl);
    let flip = |pip: ManaCost| Effect::MayPay {
        description: "Transform Grub?".into(),
        mana_cost: pip,
        body: Box::new(Effect::Transform { what: Selector::This }),
        else_: None,
    };
    let auntie = CardDefinition {
        name: "Grub, Notorious Auntie",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Warrior], ..Default::default() },
        color_indicator: vec![Color::Red],
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            on_attack(Effect::MayDo {
                description: "Blight 1 for an attacking copy?".into(),
                body: Box::new(Effect::BlightThen {
                    n: Value::ONE,
                    then: Box::new(Effect::Seq(vec![
                        Effect::TokenCopyTappedAttacking { source: Selector::LastMoved },
                        Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedToken },
                    ])),
                }),
            }),
            TriggeredAbility { event: main_phase(), effect: flip(cost(&[b()])) },
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Grub, Storied Matriarch",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Warlock], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(goblin_back()),
            TriggeredAbility { event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource), effect: goblin_back() },
            TriggeredAbility { event: main_phase(), effect: flip(cost(&[r()])) },
        ],
        back_face: Some(Box::new(auntie)),
        ..Default::default()
    }
}

/// Grub's Command — {3}{B}{R} Kindred sorcery — Goblin. Choose two: a token
/// copy of target Goblin you control; creatures target player controls get
/// +1/+1 and haste; destroy target artifact or creature; target player mills
/// five and puts the Goblin cards milled this way into their hand.
pub fn grubs_command() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature };
    CardDefinition {
        name: "Grub's Command",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        effect: Effect::ChooseModesCast {
            min: 2,
            max: 2,
            allow_repeats: false,
            modes: vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(R::HasCreatureType(CreatureType::Goblin).and(R::ControlledByYou)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                },
                Effect::Seq(vec![
                    Effect::PumpPT { what: theirs(), power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: theirs(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Creature)) },
                Effect::Seq(vec![
                    Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(5) },
                    Effect::Move {
                        what: Selector::MatchingAmong {
                            inner: Box::new(Selector::LastMoved),
                            filter: R::HasCreatureType(CreatureType::Goblin).and(R::InGraveyard),
                        },
                        to: ZoneDest::Hand(PlayerRef::Target(0)),
                    },
                ]),
            ],
        },
        ..Default::default()
    }
}
