//! Commander: gaps of EDHREC's commanders ranked 1501-2000 (the 2026-10-13
//! twenty-page rescan), third file — mostly the two-card gaps. Tests in
//! `tests/recent_b/cmdr_rank2k_c.rs`.

use crate::card::{
    AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost};
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

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn gain(n: i32) -> Effect {
    Effect::GainLife { who: Selector::You, amount: Value::Const(n) }
}

fn grant_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn pump_eot(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn team() -> Selector {
    Selector::EachPermanent(yours(R::Creature))
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..artifact(name, mana)
    }
}

fn lord(description: &'static str, filter: R, keyword: Option<Keyword>) -> Vec<StaticAbility> {
    let others = || Selector::EachPermanent(yours(R::Creature.and(filter.clone())).and(R::OtherThanSource));
    let mut out = vec![static_ab(description, StaticEffect::PumpPT { applies_to: others(), power: 1, toughness: 1 })];
    if let Some(keyword) = keyword {
        out.push(static_ab(description, StaticEffect::GrantKeyword { applies_to: others(), keyword }));
    }
    out
}

/// Abzan Banner — {3} artifact: {T}: {W}, {B} or {G}; {W}{B}{G}, {T},
/// sacrifice it: draw a card.
pub fn abzan_banner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::White, Color::Black, Color::Green], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w(), b(), g()]),
                tap_cost: true,
                sac_cost: true,
                effect: draw(1),
                ..Default::default()
            },
        ],
        ..artifact("Abzan Banner", cost(&[generic(3)]))
    }
}

/// Altar of Bone — {G}{W} sorcery, sacrifice a creature as an additional
/// cost: tutor a creature card to hand.
pub fn altar_of_bone() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 1 }],
        ..spell(
            "Altar of Bone",
            cost(&[g(), w()]),
            CardType::Sorcery,
            Effect::Search { who: PlayerRef::You, filter: R::Creature, to: ZoneDest::Hand(PlayerRef::You) },
        )
    }
}

/// Ant Queen — {3}{G}{G} 5/5 Insect. {1}{G}: a 1/1 green Insect.
pub fn ant_queen() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: make(1, token("Insect", 1, 1, Color::Green, vec![CreatureType::Insect])),
            ..Default::default()
        }],
        ..creature("Ant Queen", cost(&[generic(3), g(), g()]), vec![CreatureType::Insect], 5, 5)
    }
}

/// Bagel and Schmear — {1} Food. Share — {W}, {T}, sacrifice: a +1/+1 counter
/// on up to one creature, draw (sorcery speed). Nosh — {2}, {T}, sacrifice:
/// gain 3, draw.
pub fn bagel_and_schmear() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::AddCounter {
                            what: target_filtered(R::Creature),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                    },
                    draw(1),
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![gain(3), draw(1)]),
                ..Default::default()
            },
        ],
        ..artifact("Bagel and Schmear", cost(&[generic(1)]))
    }
}

/// Boartusk Liege — {1}{R/G}{R/G}{R/G} 3/4 trample Goblin Knight; your other
/// red creatures and your other green creatures each get +1/+1.
pub fn boartusk_liege() -> CardDefinition {
    let rg = || hybrid(Color::Red, Color::Green);
    let mut statics = lord("Other red creatures you control get +1/+1.", R::HasColor(Color::Red), None);
    statics.extend(lord("Other green creatures you control get +1/+1.", R::HasColor(Color::Green), None));
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: statics,
        ..creature(
            "Boartusk Liege",
            cost(&[generic(1), rg(), rg(), rg()]),
            vec![CreatureType::Goblin, CreatureType::Knight],
            3,
            4,
        )
    }
}

/// Break Through the Line — {1}{R} enchantment. {R}: a creature with power 2
/// or less gains haste and can't be blocked this turn.
pub fn break_through_the_line() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeywords {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keywords: vec![Keyword::Haste, Keyword::Unblockable],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..spell("Break Through the Line", cost(&[generic(1), r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Devil Dinosaur — {2}{G}{G} legendary 5/5 trample Dinosaur; your other
/// Dinosaurs get +1/+1 and have hexproof.
pub fn devil_dinosaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: lord(
            "Other Dinosaurs you control get +1/+1 and have hexproof.",
            R::HasCreatureType(CreatureType::Dinosaur),
            Some(Keyword::Hexproof),
        ),
        ..legend("Devil Dinosaur", cost(&[generic(2), g(), g()]), vec![CreatureType::Dinosaur], 5, 5)
    }
}

/// Eye of Vecna — {2} legendary artifact: entering, draw and lose 2; each
/// upkeep, you may pay {2} to do it again.
pub fn eye_of_vecna() -> CardDefinition {
    let tick = || Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::Const(2) }]);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(tick()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::MayPay {
                    description: "Pay {2} to draw a card and lose 2 life?".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(tick()),
                    else_: None,
                },
            },
        ],
        ..artifact("Eye of Vecna", cost(&[generic(2)]))
    }
}

/// Generous Revival — {2}{W} sorcery: reanimate a creature card with mana
/// value 3 or less with an extra +1/+1 counter. Flashback {4}{W}.
pub fn generous_revival() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), w()]))],
        ..spell(
            "Generous Revival",
            cost(&[generic(2), w()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::SpellEntersWithCounters { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3))),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ]),
        )
    }
}

/// Goblin Tunneler — {1}{R} 1/1 Goblin Rogue. {T}: a creature with power 2 or
/// less can't be blocked this turn.
pub fn goblin_tunneler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: grant_eot(target_filtered(R::Creature.and(R::PowerAtMost(2))), Keyword::Unblockable),
            ..Default::default()
        }],
        ..creature("Goblin Tunneler", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Rogue], 1, 1)
    }
}

/// Goggles of Night — {1}{U} Equipment: the equipped creature's combat damage
/// to a player scries 1, then draws. Equip {2}.
pub fn goggles_of_night() -> CardDefinition {
    equipment(
        "Goggles of Night",
        cost(&[generic(1), u()]),
        cost(&[generic(2)]),
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![Effect::Scry { who: PlayerRef::You, amount: Value::ONE }, draw(1)]),
            }],
            ..Default::default()
        },
    )
}

/// Impede Momentum — {1}{U} sorcery: tap a creature, three stun counters on
/// it; scry 1.
pub fn impede_momentum() -> CardDefinition {
    spell(
        "Impede Momentum",
        cost(&[generic(1), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::Const(3) },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
    )
}

/// Ivorytusk Fortress — {2}{W}{B}{G} 5/7 Elephant. Your creatures with
/// +1/+1 counters untap during each other player's untap step.
pub fn ivorytusk_fortress() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Untap each creature you control with a +1/+1 counter on it during each other player's untap step.",
            StaticEffect::UntapYoursEachUntapStepFiltered(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
        )],
        ..creature("Ivorytusk Fortress", cost(&[generic(2), w(), b(), g()]), vec![CreatureType::Elephant], 5, 7)
    }
}

/// Loch Mare — {1}{U} 4/5 Horse Serpent; enters with three -1/-1 counters.
/// {1}{U}, remove a counter: draw. {2}{U}, remove two: tap a creature and stun
/// it. Any kind of counter pays (CR 122.1).
pub fn loch_mare() -> CardDefinition {
    let m = CounterType::MinusOneMinusOne;
    let own = || R::Not(Box::new(R::OtherThanSource));
    CardDefinition {
        enters_with_counters: Some((m, Value::Const(3))),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                remove_counter_among_filter: Some((None, 1, own())),
                effect: draw(1),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                remove_counter_among_filter: Some((None, 2, own())),
                effect: Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(R::Creature) },
                    Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Loch Mare", cost(&[generic(1), u()]), vec![CreatureType::Horse, CreatureType::Serpent], 4, 5)
    }
}

/// Overbeing of Myth — {G/U}x5 */* Spirit Avatar: P/T is your hand size; an
/// extra draw each draw step.
pub fn overbeing_of_myth() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ControllerHandSize),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::YourControl),
            effect: draw(1),
        }],
        ..creature(
            "Overbeing of Myth",
            cost(&[gu(), gu(), gu(), gu(), gu()]),
            vec![CreatureType::Spirit, CreatureType::Avatar],
            0,
            0,
        )
    }
}

/// Owlbear — {3}{G}{G} 4/4 trample Bird Bear; entering draws a card.
pub fn owlbear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(draw(1))],
        ..creature("Owlbear", cost(&[generic(3), g(), g()]), vec![CreatureType::Bird, CreatureType::Bear], 4, 4)
    }
}

/// Owlbear Shepherd — {2}{G} 1/4 Goblin Druid; your end step draws a card if
/// your creatures' total power is 8 or greater.
pub fn owlbear_shepherd() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(Value::TotalPowerControlled, Value::Const(8))),
            effect: draw(1),
        }],
        ..creature("Owlbear Shepherd", cost(&[generic(2), g()]), vec![CreatureType::Goblin, CreatureType::Druid], 1, 4)
    }
}

/// Paleoloth — {4}{G}{G} 5/5 Beast. Another creature of yours with power 5 or
/// greater entering may return a creature card from your graveyard to hand.
pub fn paleoloth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::PowerAtLeast(5)) },
            ),
            effect: Effect::MayDo {
                description: "Return a creature card from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature("Paleoloth", cost(&[generic(4), g(), g()]), vec![CreatureType::Beast], 5, 5)
    }
}

/// Potion of Healing — {1}{W} artifact: entering draws; {W}, {T}, sacrifice:
/// gain 3.
pub fn potion_of_healing() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            sac_cost: true,
            effect: gain(3),
            ..Default::default()
        }],
        ..artifact("Potion of Healing", cost(&[generic(1), w()]))
    }
}

/// Primalcrux — {G}{G}{G}{G}{G}{G} */* trample Elemental. Chroma: P/T is the
/// green mana symbols among your permanents' mana costs (devotion to green).
pub fn primalcrux() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![static_ab(
            "Chroma — toughness is the green mana symbols among your permanents' mana costs.",
            StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: Value::Const(0),
                toughness: Value::DevotionTo(vec![Color::Green]),
            },
        )],
        dynamic_pt: Some(DynamicPt::DevotionTo { color: Color::Green, base_t: 0 }),
        ..creature("Primalcrux", cost(&[g(), g(), g(), g(), g(), g()]), vec![CreatureType::Elemental], 0, 0)
    }
}

/// Ragged Short Spear — {1}{R} Equipment: +2/+0; entering, you may discard a
/// card to draw two. Equip {3}.
pub fn ragged_short_spear() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Discard a card to draw two?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                draw(2),
            ])),
        })],
        ..equipment(
            "Ragged Short Spear",
            cost(&[generic(1), r()]),
            cost(&[generic(3)]),
            EquipBonus { power: 2, ..Default::default() },
        )
    }
}

/// Return to the Light Realms — {7}{W}{W} sorcery: every nonland permanent
/// card in your graveyard returns to the battlefield.
pub fn return_to_the_light_realms() -> CardDefinition {
    spell(
        "Return to the Light Realms",
        cost(&[generic(7), w(), w()]),
        CardType::Sorcery,
        Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::PermanentCard.and(R::Land.negate()),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Sauron, the Lidless Eye — {3}{B}{R} legendary 4/4 Avatar Horror. Entering,
/// steal an opponent's creature until end of turn (untapped, hasty).
/// {1}{B}{R}: your creatures get +2/+0 and each opponent loses 2 life.
pub fn sauron_the_lidless_eye() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            grant_eot(Selector::Target(0), Keyword::Haste),
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), r()]),
            effect: Effect::Seq(vec![
                pump_eot(team(), 2, 0),
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Sauron, the Lidless Eye",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Avatar, CreatureType::Horror],
            4,
            4,
        )
    }
}

/// Soot Imp — {1}{B}{B} 1/2 flying Imp. Whenever a player casts a nonblack
/// spell, that player loses 1 life.
pub fn soot_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::Black).negate())),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..creature("Soot Imp", cost(&[generic(1), b(), b()]), vec![CreatureType::Imp], 1, 2)
    }
}

/// Spring-Leaf Avenger — {3}{G}{G} 6/5 Insect Ninja, ninjutsu {3}{G}; combat
/// damage to a player returns a permanent card from your graveyard to hand.
pub fn spring_leaf_avenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(3), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..creature(
            "Spring-Leaf Avenger",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Insect, CreatureType::Ninja],
            6,
            5,
        )
    }
}

/// Steward of Valeron — {G}{W} 2/2 vigilance Human Druid Knight; {T}: {G}.
pub fn steward_of_valeron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![crate::sets::tap_add(Color::Green)],
        ..creature(
            "Steward of Valeron",
            cost(&[g(), w()]),
            vec![CreatureType::Human, CreatureType::Druid, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Thran Temporal Gateway — {4} legendary artifact. {4}, {T}: you may put a
/// historic permanent card from your hand onto the battlefield.
pub fn thran_temporal_gateway() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::PermanentCard.and(R::historic()),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
            ..Default::default()
        }],
        ..artifact("Thran Temporal Gateway", cost(&[generic(4)]))
    }
}

/// Throatseeker — {2}{B} 3/2 Vampire Ninja; your unblocked attacking Ninjas
/// have lifelink.
pub fn throatseeker() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Unblocked attacking Ninjas you control have lifelink.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(
                    R::Creature.and(R::HasCreatureType(CreatureType::Ninja)).and(R::IsAttacking).and(R::IsUnblocked),
                )),
                keyword: Keyword::Lifelink,
            },
        )],
        ..creature("Throatseeker", cost(&[generic(2), b()]), vec![CreatureType::Vampire, CreatureType::Ninja], 3, 2)
    }
}

/// Thundermare — {5}{R} 5/5 haste Elemental Horse; entering taps every other
/// creature.
pub fn thundermare() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::Tap { what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)) })],
        ..creature("Thundermare", cost(&[generic(5), r()]), vec![CreatureType::Elemental, CreatureType::Horse], 5, 5)
    }
}

/// Titanic Ultimatum — {R}{R}{G}{G}{G}{W}{W} sorcery: your creatures get
/// +5/+5 and first strike, trample and lifelink until end of turn.
pub fn titanic_ultimatum() -> CardDefinition {
    spell(
        "Titanic Ultimatum",
        cost(&[r(), r(), g(), g(), g(), w(), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            pump_eot(team(), 5, 5),
            Effect::GrantKeywords {
                what: team(),
                keywords: vec![Keyword::FirstStrike, Keyword::Trample, Keyword::Lifelink],
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Traitorous Greed — {3}{R} sorcery: steal a creature until end of turn
/// (untapped, hasty), and add two mana of any one color.
pub fn traitorous_greed() -> CardDefinition {
    spell(
        "Traitorous Greed",
        cost(&[generic(3), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            grant_eot(Selector::Target(0), Keyword::Haste),
            Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(2)) },
        ]),
    )
}

/// Umara Mystic — {1}{U}{R} 1/3 flying Merfolk Wizard; an instant, sorcery or
/// Wizard spell you cast gives it +2/+0 until end of turn.
pub fn umara_mystic() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellMatches(
                R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .or(R::HasCreatureType(CreatureType::Wizard)),
            )),
            effect: pump_eot(Selector::This, 2, 0),
        }],
        ..creature("Umara Mystic", cost(&[generic(1), u(), r()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 1, 3)
    }
}

/// You See a Pair of Goblins — {2}{R} instant, choose one: your creatures get
/// +2/+0; or two 1/1 red Goblins.
pub fn you_see_a_pair_of_goblins() -> CardDefinition {
    spell(
        "You See a Pair of Goblins",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            pump_eot(team(), 2, 0),
            make(2, token("Goblin", 1, 1, Color::Red, vec![CreatureType::Goblin])),
        ]),
    )
}
