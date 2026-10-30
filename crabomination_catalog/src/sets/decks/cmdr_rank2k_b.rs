//! Commander: the one-card gaps of EDHREC's commanders ranked 1501-2000 (the
//! 2026-10-13 twenty-page rescan), second file. Tests in
//! `tests/recent_b/cmdr_rank2k_b.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, etb, mint_treasures, on_attack, on_you_attack, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, mono_hybrid, r, u, w, Color, ManaCost};
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

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn grant_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn level_up(mana: ManaCost, from_level: u8) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from_level)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    }
}

/// Animist's Might — {2}{G} sorcery, {2} less if it targets a legendary
/// creature you control: your creature deals twice its power to a creature or
/// planeswalker you don't control.
pub fn animists_might() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_target: Some((R::HasSupertype(Supertype::Legendary), 2)),
        ..spell(
            "Animist's Might",
            cost(&[generic(2), g()]),
            CardType::Sorcery,
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.or(R::Planeswalker).and(R::ControlledByYou.negate()),
                },
                amount: Value::Times(
                    Box::new(Value::PowerOf(Box::new(target_filtered(yours(R::Creature))))),
                    Box::new(Value::Const(2)),
                ),
            },
        )
    }
}

/// Blinding Fog — {2}{G} instant: prevent all damage to creatures this turn;
/// your creatures gain hexproof until end of turn.
pub fn blinding_fog() -> CardDefinition {
    spell(
        "Blinding Fog",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PreventAllDamageThisTurn { target: Selector::EachPermanent(R::Creature), redirect_to: None },
            grant_eot(Selector::EachPermanent(yours(R::Creature)), Keyword::Hexproof),
        ]),
    )
}

/// Forging the Tyrite Sword — {1}{R}{W} Saga. I, II: a Treasure. III: tutor
/// Halvar, God of Battle or an Equipment card to hand.
pub fn forging_the_tyrite_sword() -> CardDefinition {
    saga(
        "Forging the Tyrite Sword",
        cost(&[generic(1), r(), w()]),
        vec![
            (1, mint_treasures(1)),
            (2, mint_treasures(1)),
            (
                3,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasName("Halvar, God of Battle".into())
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
        ],
    )
}

/// Headliner Scarlett — {3}{R} legendary 3/3 haste Human Warlock. Entering,
/// creatures target player controls can't block this turn; each upkeep,
/// exile your top card face down — you may look at and play it this turn.
pub fn headliner_scarlett() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            etb(Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(grant_eot(
                    Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                    Keyword::CantBlock,
                )),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            },
        ],
        ..legend("Headliner Scarlett", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Warlock], 3, 3)
    }
}

/// Hostile Negotiations — {3}{B} instant: two piles of three from your
/// library; an opponent bins one, the other goes to your hand; lose 3 life.
/// (The engine's pile split fixes which pile is face up rather than letting
/// you turn one; the opponent still picks with one pile seen.)
pub fn hostile_negotiations() -> CardDefinition {
    spell(
        "Hostile Negotiations",
        cost(&[generic(3), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::FaceDownFaceUpPiles { count: 3 },
            Effect::Move {
                what: Selector::ExiledThisResolution { filter: R::Any },
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        ]),
    )
}

/// Irencrag Pyromancer — {2}{R} 0/4 Human Wizard. Your second draw each turn
/// deals 3 damage to any target.
pub fn irencrag_pyromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) },
        }],
        ..creature("Irencrag Pyromancer", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Wizard], 0, 4)
    }
}

/// Jedit Ojanen, Mercenary — {1}{W}{U} legendary 3/3 Cat Mercenary. It or
/// another legendary creature of yours entering may pay {G} for a 2/2 green
/// Cat Warrior with forestwalk.
pub fn jedit_ojanen_mercenary() -> CardDefinition {
    let cat = TokenDefinition {
        name: "Cat Warrior".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat, CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {G} for a 2/2 Cat Warrior?".into(),
                mana_cost: cost(&[g()]),
                body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(cat) }),
                else_: None,
            },
        }],
        ..legend(
            "Jedit Ojanen, Mercenary",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Cat, CreatureType::Mercenary],
            3,
            3,
        )
    }
}

/// Karn, Gilded Guardian — {2/W}{2/U}{2/B}{2/R}{2/G} legendary 5/5 vigilance,
/// trample Golem; entering draws a card per color among your other artifacts.
pub fn karn_gilded_guardian() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![etb(draw(Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(
            yours(R::Artifact).and(R::OtherThanSource),
        )))))],
        ..legend(
            "Karn, Gilded Guardian",
            ManaCost::new(
                [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
                    .into_iter()
                    .map(|c| mono_hybrid(2, c))
                    .collect(),
            ),
            vec![CreatureType::Golem],
            5,
            5,
        )
    }
}

/// Lake of the Dead — land; entering costs a Swamp sacrifice; {T}: {B};
/// {T}, sacrifice a Swamp: {B}{B}{B}{B}.
pub fn lake_of_the_dead() -> CardDefinition {
    let swamp = || R::Land.and(R::HasLandType(LandType::Swamp));
    CardDefinition {
        name: "Lake of the Dead",
        card_types: vec![CardType::Land],
        static_abilities: vec![static_ab(
            "If this land would enter, sacrifice a Swamp instead. If you do, put this land onto the battlefield.",
            StaticEffect::EntersOnlyIfPaid {
                cost: WardCost::SacrificeMatchingN(Box::new(swamp().and(R::OtherThanSource)), 1),
                optional: false,
            },
        )],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Black),
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((swamp(), 1)),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Black; 4]) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Lucky Clover — {2} artifact: copy each Adventure instant or sorcery spell
/// you cast; new targets allowed.
pub fn lucky_clover() -> CardDefinition {
    CardDefinition {
        name: "Lucky Clover",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::All(vec![Predicate::CastSpellIsAdventure, cast_is_instant_or_sorcery()])),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Minas Tirith Garrison — {3}{U} */5 Human Soldier; power is your hand size.
/// Attacking, tap any number of untapped Humans to draw a card per Human.
pub fn minas_tirith_garrison() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(crate::card::DynamicPt::ControllerHandSizePower { base_t: 5 }),
        triggered_abilities: vec![on_attack(Effect::TapAnyNumberThenDraw {
            filter: R::Creature.and(R::HasCreatureType(CreatureType::Human)),
        })],
        ..creature(
            "Minas Tirith Garrison",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            0,
            5,
        )
    }
}

/// Misty Knight, Hero for Hire — {1}{R} legendary 3/1. {2}, {T}, discard a
/// card: draw a card per card you've discarded this turn.
pub fn misty_knight_hero_for_hire() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: draw(Value::CardsDiscardedThisTurn(PlayerRef::You)),
            ..Default::default()
        }],
        ..legend(
            "Misty Knight, Hero for Hire",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Detective, CreatureType::Hero],
            3,
            1,
        )
    }
}

/// Nazgûl Battle-Mace — {5} Equipment: menace, deathtouch, annihilator 1, and
/// an opponent's sacrificed nontoken permanent comes to you unless they pay 3
/// life. Equip {3}.
pub fn nazgul_battle_mace() -> CardDefinition {
    equipment(
        "Nazgûl Battle-Mace",
        cost(&[generic(5)]),
        cost(&[generic(3)]),
        EquipBonus {
            keywords: vec![Keyword::Menace, Keyword::Deathtouch, Keyword::Annihilator(1)],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::OpponentControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
                ),
                effect: Effect::UnlessPlayerPays {
                    who: PlayerRef::TriggerEventPlayer,
                    cost: WardCost::Life(3),
                    then: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    if_paid: None,
                },
            }],
            ..Default::default()
        },
    )
}

/// Paladin Class — {W} Class. 1: opponents' spells cost {1} more during your
/// turn. 2 ({2}{W}): creatures you control get +1/+1. 3 ({4}{W}): whenever you
/// attack, an attacking creature gets +1/+1 per other attacker and double
/// strike until end of turn.
pub fn paladin_class() -> CardDefinition {
    let others = || {
        Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::ControlledByYou))))
    };
    let attacker = || target_filtered(yours(R::Creature).and(R::IsAttacking));
    CardDefinition {
        name: "Paladin Class",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        static_abilities: vec![
            static_ab(
                "Spells your opponents cast during your turn cost {1} more to cast.",
                StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::OpponentSpellsCostMore { filter: R::Any, amount: 1 }),
                },
            ),
            static_ab(
                "Level 2 — Creatures you control get +1/+1.",
                StaticEffect::PumpTeamIf {
                    condition: Predicate::SourceClassLevelAtLeast(2),
                    applies_to: Selector::EachPermanent(yours(R::Creature)),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                },
            ),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: on_you_attack(Effect::Noop).event.with_filter(Predicate::SourceClassLevelAtLeast(3)),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: attacker(),
                    power: Value::Sum(vec![others(), Value::Const(-1)]),
                    toughness: Value::Sum(vec![others(), Value::Const(-1)]),
                    duration: Duration::EndOfTurn,
                },
                grant_eot(Selector::Target(0), Keyword::DoubleStrike),
            ]),
        }],
        activated_abilities: vec![level_up(cost(&[generic(2), w()]), 1), level_up(cost(&[generic(4), w()]), 2)],
        ..Default::default()
    }
}

/// Path of Bravery — {2}{W} enchantment. At or above your starting life
/// total, your creatures get +1/+1; attacking gains a life per attacker.
pub fn path_of_bravery() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "As long as your life total is greater than or equal to your starting life total, creatures you control get +1/+1.",
            StaticEffect::PumpTeamIf {
                condition: Predicate::PlayerLifeAtLeastAboveStarting { who: PlayerRef::You, delta: 0 },
                applies_to: Selector::EachPermanent(yours(R::Creature)),
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        )],
        triggered_abilities: vec![on_you_attack(Effect::GainLife {
            who: Selector::You,
            amount: Value::CountOf(Box::new(Selector::EachPermanent(yours(R::Creature).and(R::IsAttacking)))),
        })],
        ..spell("Path of Bravery", cost(&[generic(2), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Robe of the Archmagi — {2}{U} Equipment: the equipped creature's combat
/// damage to a player draws you that many cards. Equip {4}; equip Shaman,
/// Warlock, or Wizard {1}.
pub fn robe_of_the_archmagi() -> CardDefinition {
    CardDefinition {
        equip_filtered_cost: Some((
            R::HasCreatureType(CreatureType::Shaman)
                .or(R::HasCreatureType(CreatureType::Warlock))
                .or(R::HasCreatureType(CreatureType::Wizard)),
            cost(&[generic(1)]),
        )),
        ..equipment(
            "Robe of the Archmagi",
            cost(&[generic(2), u()]),
            cost(&[generic(4)]),
            EquipBonus {
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: draw(Value::TriggerEventAmount),
                }],
                ..Default::default()
            },
        )
    }
}

/// Sphinx's Approach — {1}{U}{U} instant: draw two; then you may exile it and
/// four namesakes from your graveyard to put a Sphinx onto the battlefield.
/// Any number in a deck.
pub fn sphinxs_approach() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::deck_may_have_copies(None)],
        ..spell(
            "Sphinx's Approach",
            cost(&[generic(1), u(), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                draw(Value::Const(2)),
                Effect::If {
                    cond: Predicate::SameNamedInZoneAtLeast {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        at_least: Value::Const(4),
                    },
                    then: Box::new(Effect::MayDo {
                        description: "Exile this and four Sphinx's Approach to search for a Sphinx?".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::Move {
                                what: Selector::Take {
                                    inner: Box::new(Selector::CardsInZone {
                                        who: PlayerRef::You,
                                        zone: Zone::Graveyard,
                                        filter: R::HasName("Sphinx's Approach".into()),
                                    }),
                                    count: Box::new(Value::Const(4)),
                                },
                                to: ZoneDest::Exile,
                            },
                            Effect::ExileResolvingSpell,
                            Effect::Search {
                                who: PlayerRef::You,
                                filter: R::Creature.and(R::HasCreatureType(CreatureType::Sphinx)),
                                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                            },
                        ])),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Tales of Master Seshiro // Seshiro's Living Legacy — {4}{G} Saga. I, II: a
/// +1/+1 counter and vigilance on your creature or Vehicle. III: it returns
/// transformed as a 5/5 vigilance, haste Snake Warrior enchantment creature.
pub fn tales_of_master_seshiro() -> CardDefinition {
    let grow = || {
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    yours(R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            grant_eot(Selector::Target(0), Keyword::Vigilance),
        ])
    };
    let legacy = CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        ..creature(
            "Seshiro's Living Legacy",
            ManaCost::default(),
            vec![CreatureType::Snake, CreatureType::Warrior],
            5,
            5,
        )
    };
    CardDefinition {
        back_face: Some(Box::new(legacy)),
        ..saga(
            "Tales of Master Seshiro",
            cost(&[generic(4), g()]),
            vec![(1, grow()), (2, grow()), (3, Effect::ExileSelfReturnTransformed)],
        )
    }
}
