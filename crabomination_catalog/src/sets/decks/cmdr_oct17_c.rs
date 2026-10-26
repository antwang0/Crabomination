//! Commander: the 2026-10-17 (`01Htv6cF`) partner-pair scan's one-card gaps,
//! second leg. Tests in `tests/recent_b/cmdr_oct17_b.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{boast, etb, etb_draw, investigate, landfall, mint_treasures, on_attack, on_dies, target_filtered};
use crate::card::MayPlayDuration;
use crate::effect::{DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Selector};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use crate::sets::{ascend, enters_tapped};
use crabomination_base::tokens::{food_token, mutagen_token};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    })
}

fn mint(count: i32, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(count), definition: def }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// A Food artifact with an ETB rider and "{2}, {T}, Sacrifice: gain 3 life".
fn pizza(name: &'static str, mana: ManaCost, on_enter: Effect, extra: Vec<ActivatedAbility>) -> CardDefinition {
    let food = food_token();
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        triggered_abilities: vec![etb(on_enter)],
        activated_abilities: extra.into_iter().chain(food.activated_abilities).collect(),
        ..Default::default()
    }
}

/// Chance for Glory — {1}{R}{W} Instant. Creatures you control gain
/// indestructible. Take an extra turn; lose at that turn's end step.
pub fn chance_for_glory() -> CardDefinition {
    spell(
        "Chance for Glory",
        cost(&[generic(1), r(), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Indestructible, duration: Duration::Permanent },
            Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextTurnsEndStep,
                body: Box::new(Effect::LoseGame { who: PlayerRef::You }),
            },
        ]),
    )
}

/// Sight of the Scalelords — {4}{G} Enchantment. Beginning of combat on your
/// turn: your creatures with toughness 4+ get +2/+2 and vigilance.
pub fn sight_of_the_scalelords() -> CardDefinition {
    let big = || yours(R::Creature.and(R::ToughnessAtLeast(4)));
    CardDefinition {
        name: "Sight of the Scalelords",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![pump(big(), 2, 2), grant(big(), Keyword::Vigilance)]),
        }],
        ..Default::default()
    }
}

/// Flamekin Gildweaver — {3}{R} 4/3 trample Elemental Sorcerer. ETB: a
/// Treasure.
pub fn flamekin_gildweaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(mint_treasures(1))],
        ..creature("Flamekin Gildweaver", cost(&[generic(3), r()]), vec![CreatureType::Elemental, CreatureType::Sorcerer], 4, 3)
    }
}

/// Manhole Missile — {1}{R} Instant. 3 damage to target creature; you may put
/// a card from your hand on the bottom of your library to draw a card.
pub fn manhole_missile() -> CardDefinition {
    spell(
        "Manhole Missile",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Hand, filter: R::Any }),
                then: Box::new(Effect::MayDo {
                    description: "Put a card from your hand on the bottom of your library to draw a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::PutCardsFromHandOnBottom { who: Selector::You, count: Value::ONE },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Guac & Marshmallow Pizza — {G} flash Food. ETB: target creature gets
/// +2/+2 and untaps.
pub fn guac_and_marshmallow_pizza() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..pizza(
            "Guac & Marshmallow Pizza",
            cost(&[g()]),
            Effect::Seq(vec![pump(target_filtered(R::Creature), 2, 2), Effect::Untap { what: Selector::Target(0), up_to: None }]),
            vec![],
        )
    }
}

/// Omni-Cheese Pizza — {2} Food. ETB: draw a card. {1}, {T}, sacrifice: one
/// mana of any color.
pub fn omni_cheese_pizza() -> CardDefinition {
    pizza(
        "Omni-Cheese Pizza",
        cost(&[generic(2)]),
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::ONE) },
            ..Default::default()
        }],
    )
}

/// Frostbite Pyromental — {U}{R}{R} 4/4 trample, haste. Combat damage to a
/// player: draw two. Sacrifice it at the beginning of the end step.
pub fn frostbite_pyromental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
                effect: Effect::SacrificeSource,
            },
        ],
        ..creature("Frostbite Pyromental", cost(&[u(), r(), r()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Timberland Ancient — {4}{G}{G} 6/5 reach, trample Treefolk.
/// Forestcycling {2}.
pub fn timberland_ancient() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample, Keyword::Landcycling(cost(&[generic(2)]), LandType::Forest)],
        ..creature("Timberland Ancient", cost(&[generic(4), g(), g()]), vec![CreatureType::Treefolk], 6, 5)
    }
}

/// Fearless Pup — {R} 1/1 first strike Wolf. Boast — {2}{R}: +2/+0.
pub fn fearless_pup() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![boast(cost(&[generic(2), r()]), pump(Selector::This, 2, 0))],
        ..creature("Fearless Pup", cost(&[r()]), vec![CreatureType::Wolf], 1, 1)
    }
}

/// Safehold Elite — {1}{G/W} 2/2 Elf Scout. Persist.
pub fn safehold_elite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Persist],
        ..creature(
            "Safehold Elite",
            cost(&[generic(1), hybrid(Color::Green, Color::White)]),
            vec![CreatureType::Elf, CreatureType::Scout],
            2,
            2,
        )
    }
}

/// Thundersteel Colossus — {7} 7/7 trample, haste Vehicle. Crew 2.
pub fn thundersteel_colossus() -> CardDefinition {
    CardDefinition {
        name: "Thundersteel Colossus",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Crew(2)],
        ..Default::default()
    }
}

/// Furious Bellow — {1}{R} Instant. +3/+0 and first strike; scry 1.
pub fn furious_bellow() -> CardDefinition {
    spell(
        "Furious Bellow",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump(target_filtered(R::Creature), 3, 0),
            grant(Selector::Target(0), Keyword::FirstStrike),
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
    )
}

/// Billiard Room / Kitchen — tapped duals with "{4}, {T}: Investigate."
fn investigate_land(name: &'static str, a: Color, b2: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![a, b2], Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility { mana_cost: cost(&[generic(4)]), tap_cost: true, effect: investigate(1), ..Default::default() },
        ],
        ..Default::default()
    }
}

/// Billiard Room — B/R tapped land; {4}, {T}: investigate.
pub fn billiard_room() -> CardDefinition {
    investigate_land("Billiard Room", Color::Black, Color::Red)
}

/// Kitchen — G/U tapped land; {4}, {T}: investigate.
pub fn kitchen() -> CardDefinition {
    investigate_land("Kitchen", Color::Green, Color::Blue)
}

/// Kitesail Cleric — {W} 1/1 flying Kor Cleric. Kicker {2}{W}: when it enters
/// kicked, tap up to two target creatures.
pub fn kitesail_cleric() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[generic(2), w()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::SpellWasKicked),
            effect: Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
            },
        }],
        ..creature("Kitesail Cleric", cost(&[w()]), vec![CreatureType::Kor, CreatureType::Cleric], 1, 1)
    }
}

/// Long Lake Nuisance — {3}{U} 3/1 flying Bird. ETB: recruit (draw, discard;
/// a nonland discard makes a 1/1 white Human Soldier).
pub fn long_lake_nuisance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::DiscardedThisResolution { filter: R::Nonland }),
                then: Box::new(mint(1, token("Human Soldier", 1, 1, vec![Color::White], vec![CreatureType::Human, CreatureType::Soldier], vec![]))),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Long Lake Nuisance", cost(&[generic(3), u()]), vec![CreatureType::Bird], 3, 1)
    }
}

/// Sizzling Changeling — {2}{R} 3/2 changeling. Dies: exile your top card,
/// playable until the end of your next turn.
pub fn sizzling_changeling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        triggered_abilities: vec![on_dies(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        ..creature("Sizzling Changeling", cost(&[generic(2), r()]), vec![CreatureType::Shapeshifter], 3, 2)
    }
}

/// Dusk Charger — {3}{B} 3/3 Horse. Ascend; +2/+2 with the city's blessing.
pub fn dusk_charger() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            ascend(),
            StaticAbility {
                description: "This creature gets +2/+2 as long as you have the city's blessing.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::HasCityBlessing { who: PlayerRef::You },
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                },
            },
        ],
        ..creature("Dusk Charger", cost(&[generic(3), b()]), vec![CreatureType::Horse], 3, 3)
    }
}

/// Cenn's Enlistment — {3}{W} Sorcery. Two 1/1 white Kithkin Soldiers.
/// Retrace.
pub fn cenns_enlistment() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Retrace],
        ..spell(
            "Cenn's Enlistment",
            cost(&[generic(3), w()]),
            CardType::Sorcery,
            mint(2, token("Kithkin Soldier", 1, 1, vec![Color::White], vec![CreatureType::Kithkin, CreatureType::Soldier], vec![])),
        )
    }
}

fn zombie() -> Arc<TokenDefinition> {
    token("Zombie", 2, 2, vec![Color::Black], vec![CreatureType::Zombie], vec![])
}

/// Stir the Sands — {4}{B}{B} Sorcery. Three 2/2 Zombies. Cycling {3}{B};
/// when you cycle it, one 2/2 Zombie.
pub fn stir_the_sands() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(3), b()]))],
        triggered_abilities: vec![TriggeredAbility { event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource), effect: mint(1, zombie()) }],
        ..spell("Stir the Sands", cost(&[generic(4), b(), b()]), CardType::Sorcery, mint(3, zombie()))
    }
}

fn thopter() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..(*token("Thopter", 1, 1, vec![], vec![CreatureType::Thopter], vec![Keyword::Flying])).clone()
    })
}

/// Aspiring Aeronaut — {3}{U} 1/2 flying Human Artificer. ETB: a 1/1 flying
/// Thopter.
pub fn aspiring_aeronaut() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(mint(1, thopter()))],
        ..creature("Aspiring Aeronaut", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Artificer], 1, 2)
    }
}

/// Ral, Caller of Storms — {4}{U}{R}, loyalty 4. +1: draw. −2: 3 damage
/// divided among one to three targets. −7: draw seven; 7 damage to each
/// creature your opponents control.
pub fn ral_caller_of_storms() -> CardDefinition {
    CardDefinition {
        name: "Ral, Caller of Storms",
        cost: cost(&[generic(4), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Ral], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility { loyalty_cost: 1, effect: Effect::Draw { who: Selector::You, amount: Value::ONE }, ..Default::default() },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::DealDamageDivided { retaliate_to_source: false, total: Value::Const(3), filter: R::any_target(), max_targets: 3 },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(7) },
                    Effect::DealDamage {
                        to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                        amount: Value::Const(7),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Press for Answers — {1}{U} Sorcery. Tap target creature; it doesn't untap
/// during its controller's next untap step. Investigate.
pub fn press_for_answers() -> CardDefinition {
    spell(
        "Press for Answers",
        cost(&[generic(1), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::SkipNextUntap { what: Selector::Target(0) },
            investigate(1),
        ]),
    )
}

/// Flora Colossus — {5}{G}{G} */* hexproof Treefolk; P/T = lands you control.
pub fn flora_colossus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        dynamic_pt: Some(crate::card::DynamicPt::PermanentsControlledMatching { base_p: 0, base_t: 0, filter: Box::new(R::Land) }),
        ..creature("Flora Colossus", cost(&[generic(5), g(), g()]), vec![CreatureType::Treefolk], 0, 0)
    }
}

/// Operations Officer — {3}{W} 2/2 lifelink Human Officer. ETB: draw.
/// Attacks: a +1/+1 counter on it.
pub fn operations_officer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![
            etb_draw(1),
            on_attack(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
        ],
        ..creature("Operations Officer", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Officer], 2, 2)
    }
}

/// Noosegraf Mob — {4}{B}{B} 0/0 Zombie, enters with five +1/+1 counters.
/// Whenever a player casts a spell, remove one; if you do, a 2/2 Zombie.
pub fn noosegraf_mob() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::PlusOnePlusOne, n: 1 },
                then: Box::new(Effect::Seq(vec![
                    Effect::RemoveCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    mint(1, zombie()),
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Noosegraf Mob", cost(&[generic(4), b(), b()]), vec![CreatureType::Zombie], 0, 0)
    }
}

/// Alchemax Slayer-Bots — {2}{U} 2/2 artifact Robot Villain. ETB: tap target
/// creature an opponent controls and put a stun counter on it.
pub fn alchemax_slayer_bots() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
        ]))],
        ..creature("Alchemax Slayer-Bots", cost(&[generic(2), u()]), vec![CreatureType::Robot, CreatureType::Villain], 2, 2)
    }
}

/// Rapid Rescue — {G} Instant. Mill two; you may put a permanent card from
/// among them into your hand. Gain 2 life.
pub fn rapid_rescue() -> CardDefinition {
    spell(
        "Rapid Rescue",
        cost(&[g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::MillThenToHand { amount: Value::Const(2), filter: R::PermanentCard, otherwise: None },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
    )
}

/// Crude Bent Blade — {2}{B} Equipment, equip {2}. ETB: target opponent
/// sacrifices a creature. Equipped creature gets +2/+1.
pub fn crude_bent_blade() -> CardDefinition {
    CardDefinition {
        name: "Crude Bent Blade",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Sacrifice { who: target_filtered(R::OpponentPlayer), count: Value::ONE, filter: R::Creature })],
        ..Default::default()
    }
}

/// Goblin-town Flunkies — {1}{R} 1/1 haste Goblin Soldier. ETB: amass
/// Goblins 1.
pub fn goblin_town_flunkies() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Goblin) })],
        ..creature("Goblin-town Flunkies", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Soldier], 1, 1)
    }
}

/// Exhibition Magician — {2}{R} 2/1 Human Wizard. ETB: choose one — a 1/1
/// green and white Citizen, or a Treasure.
pub fn exhibition_magician() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            mint(1, token("Citizen", 1, 1, vec![Color::Green, Color::White], vec![CreatureType::Citizen], vec![])),
            mint_treasures(1),
        ]))],
        ..creature("Exhibition Magician", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 1)
    }
}

/// Skalla Wolf — {3}{G}{G} 3/3 Wolf Spirit. ETB: look at the top five; you
/// may take a green card; the rest on the bottom in a random order.
pub fn skalla_wolf() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(crate::effect::LookPick {
            rest_bottom_random: true,
            who: PlayerRef::You,
            count: Value::Const(5),
            pick_filter: Some(R::HasColor(Color::Green)),
            optional: true,
            ..Default::default()
        })))],
        ..creature("Skalla Wolf", cost(&[generic(3), g(), g()]), vec![CreatureType::Wolf, CreatureType::Spirit], 3, 3)
    }
}

/// Vinelasher Adept — {4}{G}{G} 2/4 reach Rhino Soldier. ETB: three +1/+1
/// counters on target creature. Basic landcycling {2}.
pub fn vinelasher_adept() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))],
        triggered_abilities: vec![etb(Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::Const(3) })],
        ..creature("Vinelasher Adept", cost(&[generic(4), g(), g()]), vec![CreatureType::Rhino, CreatureType::Soldier], 2, 4)
    }
}

/// Gleam of Resistance — {4}{W} Instant. Your creatures get +1/+2 and untap.
/// Basic landcycling {1}{W}.
pub fn gleam_of_resistance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Typecycling(Box::new((cost(&[generic(1), w()]), R::IsBasicLand)))],
        ..spell(
            "Gleam of Resistance",
            cost(&[generic(4), w()]),
            CardType::Instant,
            Effect::Seq(vec![pump(yours(R::Creature), 1, 2), Effect::Untap { what: yours(R::Creature), up_to: None }]),
        )
    }
}

/// Killmonger, Scourge of Wakanda — {2}{B}{G} legendary 3/3. ETB: you may
/// sacrifice another creature; when you do, destroy target nonland permanent
/// an opponent controls. +2/+1 while two or more creature cards are in your
/// graveyard.
pub fn killmonger_scourge_of_wakanda() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice another creature to destroy a nonland permanent?".into(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            // CR 603.12 — "When you do": the target is chosen after the sacrifice.
            then: Box::new(Effect::ReflexiveTrigger {
                body: Box::new(Effect::Destroy { what: target_filtered(R::Nonland.and(R::Permanent).and(R::ControlledByOpponent)) }),
            }),
            else_: None,
        })],
        static_abilities: vec![StaticAbility {
            description: "As long as there are two or more creature cards in your graveyard, Killmonger gets +2/+1.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::count(Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter: R::HasCardType(CardType::Creature) }),
                    Value::Const(2),
                ),
                power: 2,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..creature(
            "Killmonger, Scourge of Wakanda",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Human, CreatureType::Mercenary, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Genghis Frog — {G}{U} legendary 1/3 trample Frog Mutant Rogue. Whenever it
/// or another Mutant you control enters, a Mutagen token.
pub fn genghis_frog() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::Any(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource },
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::HasCreatureType(CreatureType::Mutant)) },
            ])),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(mutagen_token()) },
        }],
        ..creature("Genghis Frog", cost(&[g(), u()]), vec![CreatureType::Frog, CreatureType::Mutant, CreatureType::Rogue], 1, 3)
    }
}

/// Saurian Explorer — {3}{G} 4/3 Dinosaur Scout. Landfall: +2/+2. {6}: a
/// tapped Planet land token "New Planet" ({T}: any color), sorcery speed.
pub fn saurian_explorer() -> CardDefinition {
    let planet = TokenDefinition {
        name: "New Planet".into(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        tapped: true,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![landfall(pump(Selector::This, 2, 2))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            sorcery_speed: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(planet) },
            ..Default::default()
        }],
        ..creature("Saurian Explorer", cost(&[generic(3), g()]), vec![CreatureType::Dinosaur, CreatureType::Scout], 4, 3)
    }
}

/// Stingcaster Mage — {1}{R} 2/1 haste Human Wizard. ETB: target instant or
/// sorcery card in your graveyard gains flashback (its mana cost) this turn.
pub fn stingcaster_mage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::GrantFlashbackThisTurn {
            what: target_filtered(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard)),
        })],
        ..creature("Stingcaster Mage", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 1)
    }
}

