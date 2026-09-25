//! Commander: the cards the **Desert Bloom** precon (OTC, Yuma, Proud
//! Protector) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_yuma.rs`.
//!
//! Residuals (each also on its card):
//! - **Cataclysmic Prospecting** — mana spent from Deserts isn't tracked; the
//!   Treasures count the tapped Deserts you control as it resolves.
//! - **Dune Chanter** — land cards off the battlefield aren't Deserts (lands
//!   you control are).

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered, unearth};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, w, x, Color, ManaCost};

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

fn desert(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Desert], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        ..Default::default()
    }
}

fn is_desert() -> R {
    R::HasLandType(LandType::Desert)
}

fn sand_warrior() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Sand Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::Green, Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sand, CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    })
}

fn plant_warrior() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Plant Warrior".into(),
        power: 4,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant, CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Reach],
        ..Default::default()
    })
}

fn add(pool: ManaPayload) -> Effect {
    Effect::AddMana { who: PlayerRef::You, pool }
}

/// Cactus Preserve — Desert; enters tapped; {T}: mana of a type a land of
/// yours could make; {3}: an X/X reach Plant until end of turn, X your
/// commanders' greatest mana value.
pub fn cactus_preserve() -> CardDefinition {
    let x = Value::GreatestCommanderManaValue(PlayerRef::You);
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: add(ManaPayload::AnyColorYouCouldProduce), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: x.clone(),
                    toughness: x,
                    creature_types: vec![CreatureType::Plant],
                    keywords: vec![Keyword::Reach],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..desert("Cactus Preserve")
    }
}

/// Cataclysmic Prospecting — X damage to each creature; a tapped Treasure per
/// Desert mana spent. Residual: the Treasures count your tapped Deserts.
pub fn cataclysmic_prospecting() -> CardDefinition {
    CardDefinition {
        name: "Cataclysmic Prospecting",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::XFromCost },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::EachPermanent(
                    is_desert().and(R::ControlledByYou).and(R::Tapped),
                ))),
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
            Effect::Tap { what: Selector::LastCreatedTokens },
        ]),
        ..Default::default()
    }
}

/// Descend upon the Sinful — exile all creatures; delirium makes a 4/4
/// flying Angel.
pub fn descend_upon_the_sinful() -> CardDefinition {
    let angel = TokenDefinition {
        name: "Angel".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Descend upon the Sinful",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Exile { what: Selector::EachPermanent(R::Creature) },
            Effect::If {
                cond: Predicate::DeliriumActive { who: PlayerRef::You },
                then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(angel) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Dune Chanter — reach; your lands are Deserts and tap for any color; {T}:
/// mill two, a life per land milled. Residual: land cards off the battlefield
/// aren't Deserts.
pub fn dune_chanter() -> CardDefinition {
    let your_lands = || Selector::EachPermanent(R::Land.and(R::ControlledByYou));
    CardDefinition {
        keywords: vec![Keyword::Reach],
        static_abilities: vec![
            StaticAbility {
                description: "Lands you control are Deserts in addition to their other types.",
                effect: StaticEffect::LandTypeChanger { applies_to: your_lands(), land_type: LandType::Desert, replace: false },
            },
            StaticAbility {
                description: "Lands you control have \"{T}: Add one mana of any color.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: your_lands(),
                    ability: crate::sets::tap_add_any_color(),
                    condition: None,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                Effect::GainLife { who: Selector::You, amount: Value::CardsMilledThisEffectMatching { filter: R::Land } },
            ]),
            ..Default::default()
        }],
        ..creature("Dune Chanter", cost(&[generic(2), g()]), vec![CreatureType::Plant, CreatureType::Druid], 2, 3)
    }
}

/// Dunes of the Dead — Desert; going to a graveyard from the battlefield
/// makes a 2/2 Zombie.
pub fn dunes_of_the_dead() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(zombie) },
        }],
        ..desert("Dunes of the Dead")
    }
}

/// Embrace the Unknown — exile the top two, playable until the end of your
/// next turn; retrace.
pub fn embrace_the_unknown() -> CardDefinition {
    CardDefinition {
        name: "Embrace the Unknown",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Retrace],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(2),
            duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: false,
            uncast_penalty: None,
        },
        ..Default::default()
    }
}

/// Hazezon, Shaper of Sand — desertwalk; play Deserts from your graveyard; a
/// Desert of yours entering makes two 1/1 Sand Warriors.
pub fn hazezon_shaper_of_sand() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Landwalk(LandType::Desert)],
        static_abilities: vec![StaticAbility {
            description: "You may play Desert lands from your graveyard.",
            effect: StaticEffect::MayPlayLandsFromGraveyardMatching(is_desert()),
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: is_desert() }),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: sand_warrior() },
        }],
        ..creature(
            "Hazezon, Shaper of Sand",
            cost(&[r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Kirri, Talented Sprout — other Plants and Treefolk get +2/+0; each of your
/// postcombat mains returns a Plant, Treefolk or land card to hand.
pub fn kirri_talented_sprout() -> CardDefinition {
    let plantish = || R::HasCreatureType(CreatureType::Plant).or(R::HasCreatureType(CreatureType::Treefolk));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Other Plants and Treefolk you control get +2/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(plantish().and(R::ControlledByYou).and(R::OtherThanSource)),
                power: 2,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::ActivePlayer),
            effect: Effect::Move {
                what: target_filtered(plantish().or(R::Land).from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..creature(
            "Kirri, Talented Sprout",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Plant, CreatureType::Druid],
            0,
            3,
        )
    }
}

/// Painted Bluffs — Desert; {T}: {C}; {1}, {T}: any color.
pub fn painted_bluffs() -> CardDefinition {
    let mut d = desert("Painted Bluffs");
    d.activated_abilities.push(ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        ..crate::sets::tap_add_any_color()
    });
    d
}

/// Perennial Behemoth — you may play lands from your graveyard; unearth
/// {G}{G}.
pub fn perennial_behemoth() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "You may play lands from your graveyard.",
            effect: StaticEffect::MayPlayLandsFromGraveyard,
        }],
        activated_abilities: vec![unearth(cost(&[g(), g()]))],
        ..creature("Perennial Behemoth", cost(&[generic(5)]), vec![CreatureType::Beast], 2, 7)
    }
}

/// Perpetual Timepiece — {T}: mill two; {2}, exile it: shuffle any number of
/// target cards from your graveyard into your library.
pub fn perpetual_timepiece() -> CardDefinition {
    CardDefinition {
        name: "Perpetual Timepiece",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                exile_self_cost: true,
                effect: Effect::ApplyToTargets {
                    max_targets: 20,
                    min_targets: 0,
                    filter: R::Any.from_your_graveyard(),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Shuffled },
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rumbleweed — {1} less per land card in your graveyard; vigilance, reach,
/// trample; entering gives your other creatures +3/+3 and trample.
pub fn rumbleweed() -> CardDefinition {
    let others = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Reach, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each land card in your graveyard.",
            effect: StaticEffect::SelfCostReducedPerGraveyardCardMatching { filter: R::Land, per: 1 },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT { what: others(), power: Value::Const(3), toughness: Value::Const(3), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: others(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Rumbleweed", cost(&[generic(10), g()]), vec![CreatureType::Plant, CreatureType::Elemental], 8, 8)
    }
}

/// Sand Scout — entering behind on lands fetches a Desert tapped; land cards
/// reaching your graveyard make a Sand Warrior, once a turn.
pub fn sand_scout() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::OpponentControlsMoreLandsThanYou),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: is_desert(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPutIntoGraveyard, EventScope::YourControl).once_per_turn(),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: sand_warrior() },
            },
        ],
        ..creature("Sand Scout", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Scout], 2, 2)
    }
}

/// Shefet Dunes — Desert; {T}, 1 life: {W}; {2}{W}{W}, {T}, sacrifice a
/// Desert: your creatures get +1/+1 (sorcery speed).
pub fn shefet_dunes() -> CardDefinition {
    let mut d = desert("Shefet Dunes");
    d.activated_abilities.push(ActivatedAbility { life_cost: 1, ..crate::sets::tap_add(Color::White) });
    d.activated_abilities.push(ActivatedAbility {
        mana_cost: cost(&[generic(2), w(), w()]),
        tap_cost: true,
        sorcery_speed: true,
        sac_other_filter: Some((is_desert(), 1)),
        sac_other_may_be_source: true,
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    });
    d
}

/// Vengeful Regrowth — up to three land cards from your graveyard, tapped,
/// and a 4/2 reach Plant Warrior for each; flashback {6}{G}{G}.
pub fn vengeful_regrowth() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Regrowth",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(6), g(), g()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Land.from_your_graveyard(),
            effect: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: plant_warrior() },
            ])),
        },
        ..Default::default()
    }
}

/// Winding Way — creature or land: of the top four, that type to hand, the
/// rest to the graveyard.
pub fn winding_way() -> CardDefinition {
    let take = |filter: R| Effect::RevealTopTakeMatchingRestToGraveyard { who: PlayerRef::You, count: Value::Const(4), filter };
    CardDefinition {
        name: "Winding Way",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![take(R::Creature), take(R::Land)]),
        ..Default::default()
    }
}

/// World Shaper — attacking may mill three; dying returns every land card in
/// your graveyard to the battlefield tapped.
pub fn world_shaper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::ForEach {
                    selector: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Land },
                    body: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    }),
                },
            },
        ],
        ..creature("World Shaper", cost(&[generic(3), g()]), vec![CreatureType::Merfolk, CreatureType::Shaman], 3, 3)
    }
}

/// Wreck and Rebuild — destroy an artifact or enchantment, or mill five and
/// put a land card from your graveyard onto the battlefield tapped;
/// flashback {3}{R}{G}.
pub fn wreck_and_rebuild() -> CardDefinition {
    CardDefinition {
        name: "Wreck and Rebuild",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r(), g()]))],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(5) },
                Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Land,
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            ]),
        ]),
        ..Default::default()
    }
}

/// Yuma, Proud Protector — {1} less per land card in your graveyard; entering
/// or attacking may sacrifice a land to draw; a Desert card reaching your
/// graveyard makes a 4/2 reach Plant Warrior.
pub fn yuma_proud_protector() -> CardDefinition {
    let sac_draw = || Effect::MaySacrifice {
        description: "Sacrifice a land to draw a card?".into(),
        filter: R::Land.and(R::ControlledByYou),
        count: Value::ONE,
        then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        else_: None,
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each land card in your graveyard.",
            effect: StaticEffect::SelfCostReducedPerGraveyardCardMatching { filter: R::Land, per: 1 },
        }],
        triggered_abilities: vec![
            etb(sac_draw()),
            on_attack(sac_draw()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: is_desert() }),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: plant_warrior() },
            },
        ],
        ..creature(
            "Yuma, Proud Protector",
            cost(&[generic(5), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Ranger],
            6,
            6,
        )
    }
}
