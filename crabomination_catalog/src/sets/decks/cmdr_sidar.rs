//! Commander: the cards the **Cavalry Charge** precon (MOC, Sidar Jabari of
//! Zhalfir) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_sidar.rs`.
//!
//! Residuals (each also on its card):
//! - **Path of the Enigma** — with no planar deck the Will of the Planeswalkers
//!   vote isn't held (planeswalk and chaos do nothing without one).
//! - **Syr Elenora** — her hand-size power is a battlefield static, not a CDA
//!   read in every zone.
//! - **Aryel** — X is the target's power (the least that makes it legal), not
//!   a free choice.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Effect, LookPick, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, u, w};
use crabomination_base::tokens::blood_token;
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

fn human_knight(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    creature(name, mana, vec![CreatureType::Human, CreatureType::Knight], p, t)
}

fn knight() -> R {
    R::HasCreatureType(CreatureType::Knight)
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn token(name: &str, colors: Vec<Color>, p: i32, t: i32, kinds: Vec<CreatureType>, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: kinds, ..Default::default() },
        ..Default::default()
    }
}

/// A 2/2 Knight with vigilance — white (Aryel, the Sword) or white and blue.
fn knight_token(blue: bool) -> Arc<TokenDefinition> {
    let colors = if blue { vec![Color::White, Color::Blue] } else { vec![Color::White] };
    Arc::new(token("Knight", colors, 2, 2, vec![CreatureType::Knight], vec![Keyword::Vigilance]))
}

fn make(count: Value, definition: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition }
}

fn each_opponent_loses(n: i32) -> Effect {
    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n) }
}

fn attacking_knights_at_least(n: i32) -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: yours(R::Creature.and(knight()).and(R::IsAttacking)),
        n: Value::Const(n),
    }
}

/// Sidar Jabari of Zhalfir — eminence: whenever you attack with one or more
/// Knights, loot (from the command zone or the battlefield); flying, first
/// strike; combat damage to a player returns a Knight creature card from your
/// graveyard to the battlefield.
pub fn sidar_jabari_of_zhalfir() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                    .with_filter(attacking_knights_at_least(1))
                    .in_command_zone(),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(knight()).and(R::InYourGraveyard),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
        ],
        ..legendary(human_knight("Sidar Jabari of Zhalfir", cost(&[generic(1), w(), u(), b()]), 4, 3))
    }
}

/// Acclaimed Contender — enters, if you control another Knight: look at the
/// top five; a Knight, Aura, Equipment or legendary artifact may go to hand,
/// the rest to the bottom at random (CR 603.4 — checked again on resolution).
pub fn acclaimed_contender() -> CardDefinition {
    let another_knight =
        || Predicate::SelectorCountAtLeast { sel: yours(knight().and(R::OtherThanSource)), n: Value::ONE };
    let findable = knight()
        .or(R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Aura))
        .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment))
        .or(R::Artifact.and(R::HasSupertype(Supertype::Legendary)));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(another_knight()),
            effect: Effect::If {
                cond: another_knight(),
                then: Box::new(Effect::LookPickToHand(Box::new(LookPick {
                    count: Value::Const(5),
                    pick_filter: Some(findable),
                    optional: true,
                    rest_bottom_random: true,
                    ..Default::default()
                }))),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..human_knight("Acclaimed Contender", cost(&[generic(2), w()]), 3, 3)
    }
}

/// Aryel, Knight of Windgrace — vigilance; {2}{W}, {T}: a 2/2 Knight with
/// vigilance; {B}, {T}, tap X untapped Knights: destroy target creature with
/// power X or less. Residual: X is the target's power.
pub fn aryel_knight_of_windgrace() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                tap_cost: true,
                effect: make(Value::ONE, knight_token(false)),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                tap_n_filter: Some((R::Creature.and(knight()), 0)),
                tap_n_x: true,
                // The target can only be as big as the Knights you could tap;
                // the ability does nothing if it has grown past X by then.
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::XFromCost,
                        Value::PowerOf(Box::new(Selector::Target(0))),
                    ),
                    then: Box::new(Effect::Destroy {
                        what: target_filtered(
                            R::Creature.and(R::PowerAtMostYourCount(Box::new(
                                R::Creature.and(knight()).and(R::OtherThanSource),
                            ))),
                        ),
                    }),
                    else_: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
        ],
        ..legendary(human_knight("Aryel, Knight of Windgrace", cost(&[generic(2), w(), b()]), 4, 4))
    }
}

/// Chivalric Alliance — whenever you attack with two or more creatures, draw;
/// {2}, discard a card: a 2/2 white and blue Knight with vigilance.
pub fn chivalric_alliance() -> CardDefinition {
    CardDefinition {
        name: "Chivalric Alliance",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                Predicate::SelectorCountAtLeast {
                    sel: yours(R::Creature.and(R::IsAttacking)),
                    n: Value::Const(2),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            discard_cost: Some((R::Any, 1)),
            effect: make(Value::ONE, knight_token(true)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Conjurer's Mantle — +1/+1 and vigilance; whenever the equipped creature
/// attacks, look at the top six and take a card sharing a creature type with
/// it, the rest to the bottom at random. Equip {1}.
pub fn conjurers_mantle() -> CardDefinition {
    CardDefinition {
        name: "Conjurer's Mantle",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Vigilance],
            triggered_abilities: vec![on_attack(Effect::LookPickToHand(Box::new(LookPick {
                count: Value::Const(6),
                pick_filter: Some(R::SharesCreatureTypeWithAttachedHost),
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            })))],
            // The pick reads the Equipment's host, so the trigger is the
            // Equipment's own.
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Elenda and Azor — flying, ward {2}; attacking: you may pay {X}{W}{U}{B} to
/// draw X; at the beginning of each end step you may pay 4 life for a 1/1
/// lifelink Vampire Knight per card you drew this turn.
pub fn elenda_and_azor() -> CardDefinition {
    let vampire_knight = Arc::new(token(
        "Vampire Knight",
        vec![Color::Black],
        1,
        1,
        vec![CreatureType::Vampire, CreatureType::Knight],
        vec![Keyword::Lifelink],
    ));
    let drawn = || Value::CardsDrawnThisTurn(PlayerRef::You);
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![
            // {X}{W}{U}{B}: the colored part first, then any X (0 included).
            on_attack(Effect::MayPay {
                description: "Pay {W}{U}{B} (then {X}) to draw X cards?".into(),
                mana_cost: cost(&[w(), u(), b()]),
                body: Box::new(Effect::MayPayX {
                    description: "Pay {X} to draw X cards".into(),
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::XFromCost }),
                }),
                else_: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(drawn(), Value::ONE),
                    then: Box::new(Effect::MayPayLife {
                        description: "Pay 4 life for a Vampire Knight per card drawn this turn?".into(),
                        amount: Value::Const(4),
                        body: Box::new(make(drawn(), vampire_knight)),
                        else_: None,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..legendary(creature(
            "Elenda and Azor",
            cost(&[generic(3), w(), u(), b()]),
            vec![CreatureType::Vampire, CreatureType::Knight, CreatureType::Sphinx],
            6,
            6,
        ))
    }
}

/// Exsanguinator Cavalry — menace, lifelink; a Knight of yours dealing combat
/// damage to a player gets a +1/+1 counter and you make a Blood token.
pub fn exsanguinator_cavalry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: knight() }),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                make(Value::ONE, Arc::new(blood_token())),
            ]),
        }],
        ..creature(
            "Exsanguinator Cavalry",
            cost(&[generic(2), b()]),
            vec![CreatureType::Vampire, CreatureType::Knight],
            2,
            3,
        )
    }
}

/// Haakon, Stromgald Scourge — cast only from your graveyard; while it is on
/// the battlefield you may cast Knight spells from your graveyard; dies: you
/// lose 2 life.
pub fn haakon_stromgald_scourge() -> CardDefinition {
    CardDefinition {
        // The graveyard route (`Keyword::GraveyardCast`) skips the cast gate;
        // every other route — hand, a granted zone — fails it.
        keywords: vec![Keyword::GraveyardCast],
        cast_condition: Some(Predicate::Not(Box::new(Predicate::All(vec![])))),
        static_abilities: vec![StaticAbility {
            description: "As long as Haakon is on the battlefield, you may cast Knight spells from your graveyard.",
            effect: StaticEffect::CastFromGraveyardMatching { filter: knight() },
        }],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::LoseLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..legendary(creature(
            "Haakon, Stromgald Scourge",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Knight],
            3,
            3,
        ))
    }
}

/// Herald of Hoofbeats — horsemanship; other Knights you control have it.
pub fn herald_of_hoofbeats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship],
        static_abilities: vec![StaticAbility {
            description: "Other Knights you control have horsemanship.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(knight()).and(R::OtherThanSource)),
                keyword: Keyword::Horsemanship,
            },
        }],
        ..human_knight("Herald of Hoofbeats", cost(&[generic(3), u()]), 3, 3)
    }
}

/// Knights' Charge — a Knight of yours attacking drains each opponent for 1
/// (you gain 1); {6}{W}{B}, sacrifice it: every Knight creature card in your
/// graveyard returns to the battlefield.
pub fn knights_charge() -> CardDefinition {
    CardDefinition {
        name: "Knights' Charge",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: knight() }),
            effect: Effect::Seq(vec![
                each_opponent_loses(1),
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), w(), b()]),
            sac_cost: true,
            effect: Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Creature.and(knight()),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Locthwain Lancer — menace; a nontoken Knight of yours dying drains each
/// opponent for 1 and draws you a card.
pub fn locthwain_lancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: knight().and(R::NotToken) },
            ),
            effect: Effect::Seq(vec![
                each_opponent_loses(1),
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..human_knight("Locthwain Lancer", cost(&[generic(4), b()]), 5, 5)
    }
}

/// Path of the Enigma — target player draws four. Residual: the Will of the
/// Planeswalkers vote isn't held (nothing to planeswalk to).
pub fn path_of_the_enigma() -> CardDefinition {
    CardDefinition {
        name: "Path of the Enigma",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw { who: target_filtered(R::Player), amount: Value::Const(4) },
        ..Default::default()
    }
}

/// Sigiled Sword of Valeron — +2/+0, vigilance, and a Knight; whenever the
/// equipped creature attacks, a 2/2 Knight with vigilance enters attacking.
/// Equip {3}.
pub fn sigiled_sword_of_valeron() -> CardDefinition {
    CardDefinition {
        name: "Sigiled Sword of Valeron",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            keywords: vec![Keyword::Vigilance],
            add_creature_types: vec![CreatureType::Knight],
            triggered_abilities: vec![on_attack(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: knight_token(false),
                cleanup: Default::default(),
                defender: None,
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Smitten Swordmaster // Curry Favor — a 2/1 lifelink Knight; its Adventure
/// drains each opponent for the number of Knights you control.
pub fn smitten_swordmaster() -> CardDefinition {
    let knights = || Value::CountOf(Box::new(yours(knight())));
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        adventure: Some(Box::new(Adventure {
            name: "Curry Favor",
            cost: cost(&[b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: knights() },
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: knights() },
            ]),
        })),
        ..human_knight("Smitten Swordmaster", cost(&[generic(1), b()]), 2, 1)
    }
}

/// Syr Elenora, the Discerning — power equal to your hand size; enters: draw;
/// opponents' spells targeting her cost {2} more. Residual: the power is a
/// battlefield static.
pub fn syr_elenora_the_discerning() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        static_abilities: vec![
            StaticAbility {
                description: "Syr Elenora's power is equal to the number of cards in your hand.",
                effect: StaticEffect::SelfBasePtFromValue {
                    power: Value::HandSizeOf(PlayerRef::You),
                    toughness: Value::Const(4),
                },
            },
            StaticAbility {
                description: "Spells your opponents cast that target Syr Elenora cost {2} more to cast.",
                effect: StaticEffect::TaxOpponentSpellsTargetingThis { amount: 2 },
            },
        ],
        ..legendary(human_knight("Syr Elenora, the Discerning", cost(&[generic(3), u(), u()]), 0, 4))
    }
}

/// Vodalian Wave-Knight — whenever you draw a card, each other Merfolk and/or
/// Knight you control gets a +1/+1 counter.
pub fn vodalian_wave_knight() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: yours(
                    R::Creature
                        .and(knight().or(R::HasCreatureType(CreatureType::Merfolk)))
                        .and(R::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Vodalian Wave-Knight",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Wintermoor Commander — deathtouch; toughness equal to the Knights you
/// control; attacking: another target Knight you control is indestructible
/// until end of turn.
pub fn wintermoor_commander() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        dynamic_pt: Some(crate::card::DynamicPt::PermanentsControlledMatchingToughness {
            base_p: 2,
            base_t: 0,
            filter: Box::new(knight()),
        }),
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(knight()).and(R::ControlledByYou).and(R::OtherThanSource)),
            keyword: Keyword::Indestructible,
            duration: crate::effect::Duration::EndOfTurn,
        })],
        ..human_knight("Wintermoor Commander", cost(&[w(), b()]), 2, 0)
    }
}

/// Worthy Knight — whenever you cast a Knight spell, a 1/1 white Human.
pub fn worthy_knight() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: knight() }),
            effect: make(
                Value::ONE,
                Arc::new(token("Human", vec![Color::White], 1, 1, vec![CreatureType::Human], vec![])),
            ),
        }],
        ..human_knight("Worthy Knight", cost(&[generic(1), w()]), 2, 2)
    }
}

/// Xerex Strobe-Knight — flying, vigilance; {T}: a 2/2 white and blue Knight
/// with vigilance, only if you've cast two or more spells this turn.
pub fn xerex_strobe_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::SpellsCastThisTurn(PlayerRef::You),
                Value::Const(2),
            )),
            effect: make(Value::ONE, knight_token(true)),
            ..Default::default()
        }],
        ..human_knight("Xerex Strobe-Knight", cost(&[generic(2), u()]), 2, 2)
    }
}
