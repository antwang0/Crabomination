//! Commander: the top 600's next EDHREC gaps — Magnus the Red, Yusri,
//! Nine-Fingers Keene, Xu-Ifit, Duskana, Hapatra, Grenzo and Sergeant John
//! Benton (`cmdr_edhrec16` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec17.rs`.

use crate::card::{
    AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    DynamicPt, EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, LandType,
    Predicate, SelectionRequirement as R, Selector, SoulbondBonus, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, w, x, Color, ManaCost};
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

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(def) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn pump(what: Selector, n: Value) -> Effect {
    Effect::PumpPT { what, power: n.clone(), toughness: n, duration: Duration::EndOfTurn }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

// ── Magnus the Red ───────────────────────────────────────────────────────────

/// Storm King's Thunder — {X}{R}{R}{R} instant. When you next cast an instant
/// or sorcery spell this turn, copy it X times (new targets allowed).
pub fn storm_kings_thunder() -> CardDefinition {
    spell(
        "Storm King's Thunder",
        cost(&[x(), r(), r(), r()]),
        CardType::Instant,
        Effect::CopyNextInstantSorceryThisTurn { count: Value::XFromCost },
    )
}

/// Ral's Reinforcements — {1}{R} sorcery. Two 1/1 blue and red Elementals.
pub fn rals_reinforcements() -> CardDefinition {
    spell(
        "Ral's Reinforcements",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        make(Value::Const(2), token("Elemental", 1, 1, vec![Color::Blue, Color::Red], vec![CreatureType::Elemental])),
    )
}

/// Release the Gremlins — {X}{X}{R} sorcery. Destroy X target artifacts;
/// create X 2/2 red Gremlins.
pub fn release_the_gremlins() -> CardDefinition {
    spell(
        "Release the Gremlins",
        cost(&[x(), x(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Artifact,
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
            make(Value::XFromCost, token("Gremlin", 2, 2, vec![Color::Red], vec![CreatureType::Gremlin])),
        ]),
    )
}

// ── Yusri, Fortune's Flame ───────────────────────────────────────────────────

/// Creepy Doll — {5} 1/1 Construct, indestructible. Whenever it deals combat
/// damage to a creature, flip a coin; if you win, destroy that creature.
pub fn creepy_doll() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                on_tails: Box::new(Effect::Noop),
            },
        }],
        ..creature("Creepy Doll", cost(&[generic(5)]), vec![CreatureType::Construct], 1, 1)
    }
}

/// Game of Chaos — {R}{R}{R} sorcery. Flip a coin: the winner gains the stake
/// and the loser loses it, and the winner decides whether to flip again at
/// twice the stake (1, 2, 4, …).
pub fn game_of_chaos() -> CardDefinition {
    spell(
        "Game of Chaos",
        cost(&[r(), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            // Names the target opponent (slot 0).
            Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::ZERO },
            Effect::GameOfChaosRound { opponent: PlayerRef::Target(0), stake: 1 },
        ]),
    )
}

/// Molten Birth — {1}{R}{R} sorcery. Two 1/1 red Elementals, then flip a coin;
/// if you win, return Molten Birth to its owner's hand.
pub fn molten_birth() -> CardDefinition {
    spell(
        "Molten Birth",
        cost(&[generic(1), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            make(Value::Const(2), token("Elemental", 1, 1, vec![Color::Red], vec![CreatureType::Elemental])),
            Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::ReturnResolvingSpellToHand),
                on_tails: Box::new(Effect::Noop),
            },
        ]),
    )
}

// ── Nine-Fingers Keene ───────────────────────────────────────────────────────

/// Glaive of the Guildpact — {2} Equipment. Equipped creature gets +1/+0 for
/// each Gate you control and has vigilance and menace. Equip {3}.
pub fn glaive_of_the_guildpact() -> CardDefinition {
    CardDefinition {
        name: "Glaive of the Guildpact",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Vigilance, Keyword::Menace],
            scale: Some(EquipScale {
                filter: R::HasLandType(LandType::Gate).and(R::ControlledByYou),
                per_power: 1,
                per_toughness: 0,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Thran Portal — land — Gate. Enters tapped unless you control two or fewer
/// other lands. As it enters, choose a basic land type; it is that type too.
/// Its mana abilities cost an additional 1 life to activate.
pub fn thran_portal() -> CardDefinition {
    CardDefinition {
        name: "Thran Portal",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Gate], ..Default::default() },
        as_enters_effect: Some(Effect::ChooseBasicLandTypeForSource),
        static_abilities: vec![
            crate::sets::fastland_enters_tapped(),
            static_ab(
                "This land is the chosen type in addition to its other types.",
                StaticEffect::SelfIsChosenLandType,
            ),
            static_ab(
                "Mana abilities of this land cost an additional 1 life to activate.",
                StaticEffect::SelfManaAbilitiesCostLife(1),
            ),
        ],
        ..Default::default()
    }
}

/// Explore the Underdark — {4}{G} sorcery. Search for up to two basic land
/// and/or Gate cards, put them onto the battlefield tapped; take the
/// initiative.
pub fn explore_the_underdark() -> CardDefinition {
    spell(
        "Explore the Underdark",
        cost(&[generic(4), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand.or(R::HasLandType(LandType::Gate)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(2),
            },
            Effect::TakeInitiative { who: PlayerRef::You },
        ]),
    )
}

// ── Xu-Ifit, Osteoharmonist ──────────────────────────────────────────────────

/// Altar of the Wretched // Wretched Bonemass — {2}{B} artifact. When it
/// enters, you may sacrifice a nontoken creature to draw and then mill X (its
/// power). Craft with a creature {2}{B}{B} (the printed "one or more" crafts
/// with exactly one; the Bonemass doesn't copy keywords — INCOMPLETE_CARDS).
/// {2}{B}: return it from your graveyard to your hand. Bonemass: P/T each the
/// total power of the cards used to craft it.
pub fn altar_of_the_wretched() -> CardDefinition {
    let bonemass = CardDefinition {
        name: "Wretched Bonemass",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Horror],
            ..Default::default()
        },
        dynamic_pt: Some(DynamicPt::ExiledWithSourceTotalPower),
        ..Default::default()
    };
    CardDefinition {
        name: "Altar of the Wretched",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Sacrifice a nontoken creature to draw and mill its power?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::Not(Box::new(R::IsToken))),
                },
                Effect::Draw { who: Selector::You, amount: Value::SacrificedPower },
                Effect::Mill { who: Selector::You, amount: Value::SacrificedPower },
            ])),
        })],
        activated_abilities: vec![
            crate::effect::shortcut::craft(cost(&[generic(2), b(), b()]), R::Creature, 1),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                from_graveyard: true,
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
                ..Default::default()
            },
        ],
        back_face: Some(Box::new(bonemass)),
        ..Default::default()
    }
}

/// Death-Priest of Myrkul — {2}{B}{B} 2/2 Tiefling Cleric. Skeletons,
/// Vampires and Zombies you control get +1/+1. At the beginning of your end
/// step, if a creature died this turn, you may pay {1} for a 1/1 Skeleton.
pub fn death_priest_of_myrkul() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Skeletons, Vampires, and Zombies you control get +1/+1.",
            StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Skeleton)
                    .or(R::HasCreatureType(CreatureType::Vampire))
                    .or(R::HasCreatureType(CreatureType::Zombie)),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
            effect: Effect::MayPay {
                description: "Pay {1} for a 1/1 Skeleton?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(make(
                    Value::ONE,
                    token("Skeleton", 1, 1, vec![Color::Black], vec![CreatureType::Skeleton]),
                )),
                else_: None,
            },
        }],
        ..creature(
            "Death-Priest of Myrkul",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Tiefling, CreatureType::Cleric],
            2,
            2,
        )
    }
}

// ── Duskana, the Rage Mother ─────────────────────────────────────────────────

/// Ashcoat Bear — {1}{G} 2/2 Bear, flash.
pub fn ashcoat_bear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..creature("Ashcoat Bear", cost(&[generic(1), g()]), vec![CreatureType::Bear], 2, 2)
    }
}

/// Druid's Familiar — {3}{G} 2/2 Bear, soulbond: while paired, each gets
/// +2/+2.
pub fn druids_familiar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Soulbond],
        soulbond_bonus: Some(SoulbondBonus { power: 2, toughness: 2, keywords: vec![], ..Default::default() }),
        ..creature("Druid's Familiar", cost(&[generic(3), g()]), vec![CreatureType::Bear], 2, 2)
    }
}

/// Tuya Bearclaw — {1}{R}{G} 2/2 Human Warrior. Attacking, it gets +X/+X
/// where X is the greatest power among other creatures you control.
pub fn tuya_bearclaw() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(pump(
            Selector::This,
            Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(mine().and(R::OtherThanSource)))),
        ))],
        ..legend(
            "Tuya Bearclaw",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Ulvenwald Bear — {2}{G} 2/2 Bear. Morbid — when it enters, if a creature
/// died this turn, put two +1/+1 counters on target creature.
pub fn ulvenwald_bear() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..creature("Ulvenwald Bear", cost(&[generic(2), g()]), vec![CreatureType::Bear], 2, 2)
    }
}

/// Wild Pair — {4}{G}{G} enchantment. Whenever a creature enters, if you cast
/// it from your hand, you may search for a creature card with the same total
/// power and toughness and put it onto the battlefield.
pub fn wild_pair() -> CardDefinition {
    CardDefinition {
        name: "Wild Pair",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::WasCastFromHand) },
            ),
            effect: Effect::WithX {
                x: Value::Sum(vec![
                    Value::PowerOf(Box::new(Selector::TriggerSource)),
                    Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                ]),
                body: Box::new(Effect::MayDo {
                    description: "Search for a creature with the same total power and toughness?".into(),
                    body: Box::new(Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Creature.and(R::PowerPlusToughnessExactlyXFromCost),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Hapatra, Vizier of Poisons ───────────────────────────────────────────────

/// Ammit Eternal — {2}{B} 5/5 Zombie Crocodile Demon, afflict 3. Whenever an
/// opponent casts a spell, put a -1/-1 counter on it; whenever it deals
/// combat damage to a player, remove all -1/-1 counters from it.
pub fn ammit_eternal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::afflict(3),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::MinusOneMinusOne, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::MinusOneMinusOne },
                },
            },
        ],
        ..creature(
            "Ammit Eternal",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Crocodile, CreatureType::Demon],
            5,
            5,
        )
    }
}

/// Banewhip Punisher — {2}{B} 2/2 Human Warrior. When it enters, you may put a
/// -1/-1 counter on target creature. {B}, sacrifice it: destroy target
/// creature with a -1/-1 counter on it.
pub fn banewhip_punisher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::WithCounter(CounterType::MinusOneMinusOne))),
            },
            ..Default::default()
        }],
        ..creature("Banewhip Punisher", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2)
    }
}

/// Splendid Agony — {2}{B} instant. Distribute two -1/-1 counters among one or
/// two target creatures.
pub fn splendid_agony() -> CardDefinition {
    spell(
        "Splendid Agony",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::DistributeCounters {
            total: Value::Const(2),
            counter: CounterType::MinusOneMinusOne,
            filter: R::Creature,
            max_targets: 2,
        },
    )
}

/// Lethal Sting — {2}{B} sorcery. As an additional cost, put a -1/-1 counter
/// on a creature you control (blight 1). Destroy target creature.
pub fn lethal_sting() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::Blight { n: 1 }],
        ..spell(
            "Lethal Sting",
            cost(&[generic(2), b()]),
            CardType::Sorcery,
            Effect::Destroy { what: target_filtered(R::Creature) },
        )
    }
}

/// Liliana's Influence — {4}{B}{B} sorcery. A -1/-1 counter on each creature
/// you don't control; you may search your library and/or graveyard for
/// Liliana, Death Wielder and put it into your hand.
pub fn lilianas_influence() -> CardDefinition {
    spell(
        "Liliana's Influence",
        cost(&[generic(4), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
            Effect::SearchZones {
                who: PlayerRef::You,
                zones: vec![Zone::Library, Zone::Graveyard],
                filter: R::HasName("Liliana, Death Wielder".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
    )
}

// ── Grenzo, Dungeon Warden ───────────────────────────────────────────────────

/// The Cauldron of Eternity — {10}{B}{B} legendary artifact. Costs {2} less
/// per creature card in your graveyard. Whenever a creature you control dies,
/// put it on the bottom of its owner's library. {2}{B}, {T}, pay 2 life:
/// return target creature card from your graveyard to the battlefield (sorcery
/// speed).
pub fn the_cauldron_of_eternity() -> CardDefinition {
    CardDefinition {
        name: "The Cauldron of Eternity",
        cost: cost(&[generic(10), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        self_cost_reduction_per: Some((
            Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
            2,
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            life_cost: 2,
            sorcery_speed: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bronze Walrus — {3} 2/2 artifact creature Walrus. When it enters, scry 2.
/// {T}: add one mana of any color.
pub fn bronze_walrus() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..creature("Bronze Walrus", cost(&[generic(3)]), vec![CreatureType::Walrus], 2, 2)
    }
}

/// Epitaph Golem — {5} 3/5 artifact creature Golem. {2}: put target card from
/// your graveyard on the bottom of your library.
pub fn epitaph_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Move {
                what: target_filtered(R::InYourGraveyard),
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Bottom },
            },
            ..Default::default()
        }],
        ..creature("Epitaph Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 3, 5)
    }
}

/// Braid of Fire — {1}{R} enchantment. Cumulative upkeep — add {R}.
pub fn braid_of_fire() -> CardDefinition {
    CardDefinition {
        name: "Braid of Fire",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::AddMana(Color::Red))],
        ..Default::default()
    }
}

/// Into the Fire — {2}{R} sorcery. Choose one: 2 damage to each creature,
/// planeswalker and battle; or put any number of cards from your hand on the
/// bottom of your library, then draw that many plus one.
pub fn into_the_fire() -> CardDefinition {
    spell(
        "Into the Fire",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.or(R::Planeswalker).or(R::HasCardType(CardType::Battle))),
                amount: Value::Const(2),
            },
            Effect::Seq(vec![
                Effect::ShuffleAnyNumberFromHandThenDraw { who: PlayerRef::You, bottom: true },
                draw(1),
            ]),
        ]),
    )
}

// ── Sergeant John Benton ─────────────────────────────────────────────────────

/// Hand of Vecna — {3} legendary Equipment. At the beginning of combat on your
/// turn, equipped creature gets +X/+X until end of turn (X = cards in your
/// hand; "or a creature named Vecna" isn't modelled). Equip — pay 1 life per
/// card in your hand. Equip {2}.
pub fn hand_of_vecna() -> CardDefinition {
    CardDefinition {
        name: "Hand of Vecna",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: pump(Selector::AttachedTo(Box::new(Selector::This)), Value::HandSizeOf(PlayerRef::You)),
        }],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            life_cost_value: Some(Value::HandSizeOf(PlayerRef::You)),
            effect: Effect::Attach { what: Selector::This, to: target_filtered(mine()) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Flaming Fist — {2}{W} legendary enchantment — Background. Commander
/// creatures you own have "Whenever this creature attacks, it gains double
/// strike until end of turn."
pub fn flaming_fist() -> CardDefinition {
    CardDefinition {
        name: "Flaming Fist",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![static_ab(
            "Commander creatures you own have \"Whenever this creature attacks, it gains double strike until end of turn.\"",
            StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
                ability: Box::new(on_attack(Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                })),
            },
        )],
        ..Default::default()
    }
}

/// Wild Defiance — {2}{G} enchantment. Whenever a creature you control becomes
/// the target of an instant or sorcery spell, it gets +3/+3 until end of turn.
pub fn wild_defiance() -> CardDefinition {
    CardDefinition {
        name: "Wild Defiance",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourCreatureTargeted)
                .caused_by(R::IsSpellOnStack.and(instant_or_sorcery())),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Embiggen — {G} instant. Target non-Brushwagg creature gets +1/+1 until end
/// of turn for each supertype, card type and subtype it has.
pub fn embiggen() -> CardDefinition {
    spell(
        "Embiggen",
        cost(&[g()]),
        CardType::Instant,
        pump(
            target_filtered(R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Brushwagg))))),
            Value::TypeLineCountOf(Box::new(Selector::Target(0))),
        ),
    )
}

/// Kaya's Onslaught — {2}{W} instant, foretell {W}. Target creature gets
/// +1/+1 and gains double strike until end of turn.
pub fn kayas_onslaught() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[w()])),
        ..spell(
            "Kaya's Onslaught",
            cost(&[generic(2), w()]),
            CardType::Instant,
            Effect::Seq(vec![
                pump(target_filtered(R::Creature), Value::ONE),
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
            ]),
        )
    }
}
