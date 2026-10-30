//! Commander: the two-card EDHREC gaps of commanders ranked 1001-1500 (the
//! 2026-10-12 top-1500 rescan) — each card is one of the two a deck lacked.
//! Tests in `tests/recent_b/cmdr_edhrec15.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost};
use crate::sets::{tap_add, tap_add_colorless};
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

fn permanent(name: &'static str, mana: ManaCost, kind: CardType) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], ..Default::default() }
}

fn land(name: &'static str, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition { activated_abilities: abilities, ..permanent(name, ManaCost::default(), CardType::Land) }
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

fn gain(n: i32) -> Effect {
    Effect::GainLife { who: Selector::You, amount: Value::Const(n) }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn may(description: &str, body: Effect) -> Effect {
    Effect::MayDo { description: description.into(), body: Box::new(body) }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..permanent(name, mana, CardType::Artifact)
    }
}

/// Kor Celebrant — {2}{W} 1/4 Kor Cleric. Whenever it or another creature you
/// control enters, you gain 1 life.
pub fn kor_celebrant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: gain(1),
        }],
        ..creature("Kor Celebrant", cost(&[generic(2), w()]), vec![CreatureType::Kor, CreatureType::Cleric], 1, 4)
    }
}

/// Gleaming Bastion — land. {T}: {C}. {T}: {W} or {U}, only if it entered
/// this turn or you control a basic land.
pub fn gleaming_bastion() -> CardDefinition {
    let gated = |color: Color| ActivatedAbility {
        condition: Some(Predicate::Any(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn },
            Predicate::SelectorExists(Selector::EachPermanent(yours(R::IsBasicLand))),
        ])),
        ..tap_add(color)
    };
    land("Gleaming Bastion", vec![tap_add_colorless(), gated(Color::White), gated(Color::Blue)])
}

/// Vector Asp — {1} 1/1 artifact creature Phyrexian Snake. {B}: it gains
/// infect until end of turn.
pub fn vector_asp() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Infect, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature("Vector Asp", cost(&[generic(1)]), vec![CreatureType::Phyrexian, CreatureType::Snake], 1, 1)
    }
}

/// Dwarven Ruins — land; enters tapped. {T}: {R}. {T}, sacrifice it: {R}{R}.
pub fn dwarven_ruins() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        ..land(
            "Dwarven Ruins",
            vec![
                tap_add(Color::Red),
                ActivatedAbility {
                    tap_cost: true,
                    sac_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red, Color::Red]) },
                    ..Default::default()
                },
            ],
        )
    }
}

/// Waves of Aggression — {3}{R/W}{R/W} sorcery with retrace. Untap all
/// creatures that attacked this turn; an additional combat and main phase.
pub fn waves_of_aggression() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    CardDefinition {
        keywords: vec![Keyword::Retrace],
        ..spell(
            "Waves of Aggression",
            cost(&[generic(3), rw(), rw()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Untap { what: Selector::EachPermanent(R::Creature.and(R::AttackedThisTurn)), up_to: None },
                Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE },
            ]),
        )
    }
}

/// Faerie Trickery — {1}{U}{U} Kindred instant — Faerie. Counter target
/// non-Faerie spell; exile it.
pub fn faerie_trickery() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        ..spell(
            "Faerie Trickery",
            cost(&[generic(1), u(), u()]),
            CardType::Instant,
            Effect::CounterSpellToZone {
                what: target_filtered(R::IsSpellOnStack.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Faerie))))),
                zone: crate::effect::CounteredSpellZone::Exile,
            },
        )
    }
}

/// Echo, Perceptive Prodigy — {2}{U} 1/4 legendary Human Hero with vigilance.
/// {1}, {T}: copy target activated or triggered ability you control from a
/// creature source.
pub fn echo_perceptive_prodigy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::CopyAbility {
                what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByYou).and(R::Creature)),
                times: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend("Echo, Perceptive Prodigy", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Hero], 1, 4)
    }
}

/// Return to Action — {1}{B} instant. Until end of turn, target creature gets
/// +1/+0, lifelink and "when this dies, return it to the battlefield tapped".
pub fn return_to_action() -> CardDefinition {
    spell(
        "Return to Action",
        cost(&[generic(1), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(crate::effect::shortcut::on_dies(Effect::ReturnSelfTapped)),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Kor Halberd — {W} Equipment. +1/+1 and vigilance. Equip {1}.
pub fn kor_halberd() -> CardDefinition {
    equipment(
        "Kor Halberd",
        cost(&[w()]),
        cost(&[generic(1)]),
        EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Vigilance], ..Default::default() },
    )
}

/// Prism Ring — {1} artifact. As it enters, choose a color; whenever you cast
/// a spell of that color, gain 1 life.
pub fn prism_ring() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellSharesChosenColorOfSource),
            effect: gain(1),
        }],
        ..permanent("Prism Ring", cost(&[generic(1)]), CardType::Artifact)
    }
}

/// Tatterkite — {3} 2/1 flying artifact creature Scarecrow; it can't have
/// counters put on it.
pub fn tatterkite() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab("This creature can't have counters put on it.", StaticEffect::CantHaveCountersPutOn { applies_to: Selector::This })],
        ..creature("Tatterkite", cost(&[generic(3)]), vec![CreatureType::Scarecrow], 2, 1)
    }
}

/// Gretchen Titchwillow — {G}{U} 0/4 legendary Halfling Druid. {2}{G}{U}:
/// draw a card; you may put a land card from your hand onto the battlefield.
pub fn gretchen_titchwillow() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), u()]),
            effect: Effect::Seq(vec![
                draw(1),
                may(
                    "put a land card from your hand onto the battlefield",
                    Effect::Move {
                        what: Selector::take(
                            Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Hand, filter: R::Land },
                            Value::ONE,
                        ),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ),
            ]),
            ..Default::default()
        }],
        ..legend("Gretchen Titchwillow", cost(&[g(), u()]), vec![CreatureType::Halfling, CreatureType::Druid], 0, 4)
    }
}

/// Lacerate Flesh — {4}{R} sorcery. 4 damage to target creature; a Blood
/// token for each point of excess damage.
pub fn lacerate_flesh() -> CardDefinition {
    spell(
        "Lacerate Flesh",
        cost(&[generic(4), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ExcessDamageDealtThisResolution,
                definition: Arc::new(crabomination_base::tokens::blood_token()),
            },
        ]),
    )
}

/// Mist-Syndicate Naga — {2}{U} 3/1 Snake Ninja, ninjutsu {2}{U}. Combat
/// damage to a player makes a token copy of it.
pub fn mist_syndicate_naga() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(2), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, Selector::This),
        }],
        ..creature("Mist-Syndicate Naga", cost(&[generic(2), u()]), vec![CreatureType::Snake, CreatureType::Ninja], 3, 1)
    }
}

/// Battletide Alchemist — {3}{W}{W} 3/4 Kithkin Cleric. If a source would deal
/// damage to a player, you may prevent X of it, X = the Clerics you control.
pub fn battletide_alchemist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "If a source would deal damage to a player, you may prevent X of that damage, where X is the number of Clerics you control.",
            StaticEffect::ReduceDamageToYourTeamByCount { filter: R::HasCreatureType(CreatureType::Cleric) },
        )],
        ..creature("Battletide Alchemist", cost(&[generic(3), w(), w()]), vec![CreatureType::Kithkin, CreatureType::Cleric], 3, 4)
    }
}

/// King T'Challa // Black Panther, Hope Enduring — {1}{W}{U} 3/2 flash.
/// Whenever a player draws their second card each turn, you draw a card.
/// {4}{W}{U}: transform (sorcery speed). Back: 3/3 flash, double strike,
/// prevent all damage dealt to it, combat damage to a player draws a card.
pub fn king_tchalla() -> CardDefinition {
    let panther = CardDefinition {
        name: "Black Panther, Hope Enduring",
        keywords: vec![Keyword::Flash, Keyword::DoubleStrike],
        static_abilities: vec![static_ab(
            "Prevent all damage that would be dealt to Black Panther.",
            StaticEffect::PreventAllDamageToThis,
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: draw(1),
        }],
        // A modal DFC whose front also transforms (CR 712.11b): the back is
        // cast from hand for {4}{W}{U}, or the front turns over.
        ..legend("Black Panther, Hope Enduring", cost(&[generic(4), w(), u()]), vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero], 3, 3)
    };
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::AnyPlayer),
            effect: draw(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w(), u()]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(panther)),
        ..legend("King T'Challa", cost(&[generic(1), w(), u()]), vec![CreatureType::Human, CreatureType::Noble, CreatureType::Hero], 3, 2)
    }
}

/// Retribution of the Ancients — {B} enchantment. {B}, remove X +1/+1
/// counters from among creatures you control: target creature gets -X/-X.
pub fn retribution_of_the_ancients() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            remove_counter_among_x: Some((Some(CounterType::PlusOnePlusOne), yours(R::Creature))),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..permanent("Retribution of the Ancients", cost(&[b()]), CardType::Enchantment)
    }
}

/// Smoke Shroud — {1}{U} Aura: +1/+1 and flying. When a Ninja you control
/// enters, you may return this from your graveyard attached to it.
pub fn smoke_shroud() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Flying], ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: yours(R::HasCreatureType(CreatureType::Ninja)),
                },
            ),
            effect: may("Return Smoke Shroud attached to that Ninja?", Effect::ReturnSelfAttachedToTrigger),
        }],
        ..permanent("Smoke Shroud", cost(&[generic(1), u()]), CardType::Enchantment)
    }
}

/// Dire Undercurrents — {3}{U/B}{U/B} enchantment. A blue creature of yours
/// entering: you may have target player draw a card. A black one: you may
/// have target player discard a card.
pub fn dire_undercurrents() -> CardDefinition {
    let enters = |color: Color, effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::HasColor(color)) },
        ),
        effect,
    };
    CardDefinition {
        triggered_abilities: vec![
            enters(
                Color::Blue,
                may(
                    "Have target player draw a card?",
                    Effect::Draw { who: target_filtered(R::Player), amount: Value::ONE },
                ),
            ),
            enters(
                Color::Black,
                may(
                    "Have target player discard a card?",
                    Effect::Discard { who: target_filtered(R::Player.and(R::ControlledByOpponent)), amount: Value::ONE, random: false },
                ),
            ),
        ],
        ..permanent(
            "Dire Undercurrents",
            cost(&[generic(3), hybrid(Color::Blue, Color::Black), hybrid(Color::Blue, Color::Black)]),
            CardType::Enchantment,
        )
    }
}

/// Bilbo's Ring — {3} legendary Equipment. On your turn the equipped creature
/// has hexproof and can't be blocked; when it attacks alone, you draw a card
/// and lose 1 life. Equip Halfling {1}; equip {4}.
pub fn bilbos_ring() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        equip_filtered_cost: Some((R::HasCreatureType(CreatureType::Halfling), cost(&[generic(1)]))),
        ..equipment(
            "Bilbo's Ring",
            cost(&[generic(3)]),
            cost(&[generic(4)]),
            EquipBonus {
                during_your_turn_keywords: vec![Keyword::Hexproof, Keyword::Unblockable],
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::AttackingAlone),
                    effect: Effect::Seq(vec![
                        draw(1),
                        Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                    ]),
                }],
                ..Default::default()
            },
        )
    }
}

/// Galvanic Alchemist — {2}{U} 1/4 Human Wizard, soulbond: each paired
/// creature has "{2}{U}: Untap this creature."
pub fn galvanic_alchemist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Soulbond],
        soulbond_bonus: Some(crate::card::SoulbondBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..creature("Galvanic Alchemist", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 4)
    }
}

/// Super-Skrull — {1}{B}{B}{B} 4/5 legendary flier with four colored
/// activations: a 0/4 Wall, +4/+4, 4 damage to a creature, a player draws four.
pub fn super_skrull() -> CardDefinition {
    let wall = TokenDefinition {
        keywords: vec![Keyword::Defender],
        ..token("Wall", 0, 4, vec![], vec![CreatureType::Wall])
    };
    let ab = |mana: ManaCost, effect: Effect| ActivatedAbility { mana_cost: mana, effect, ..Default::default() };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ab(cost(&[generic(2), w()]), make(Value::ONE, wall)),
            ab(
                cost(&[generic(3), g()]),
                Effect::PumpPT { what: Selector::This, power: Value::Const(4), toughness: Value::Const(4), duration: Duration::EndOfTurn },
            ),
            ab(cost(&[generic(4), r()]), Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) }),
            ab(cost(&[generic(5), u()]), Effect::Draw { who: target_filtered(R::Player), amount: Value::Const(4) }),
        ],
        ..legend(
            "Super-Skrull",
            cost(&[generic(1), b(), b(), b()]),
            vec![CreatureType::Skrull, CreatureType::Shapeshifter, CreatureType::Villain],
            4,
            5,
        )
    }
}

/// Vessel of Volatility — {1}{R} enchantment. {1}{R}, sacrifice it: add
/// {R}{R}{R}{R}.
pub fn vessel_of_volatility() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red; 4]) },
            ..Default::default()
        }],
        ..permanent("Vessel of Volatility", cost(&[generic(1), r()]), CardType::Enchantment)
    }
}

/// Olivia's Attendants — {4}{R}{R} 6/6 Vampire, menace. Whenever it deals
/// damage, create that many Blood tokens. {2}{R}: 1 damage to any target.
pub fn olivias_attendants() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: Arc::new(crabomination_base::tokens::blood_token()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Olivia's Attendants", cost(&[generic(4), r(), r()]), vec![CreatureType::Vampire], 6, 6)
    }
}

/// Mirror-Mad Phantasm — {3}{U}{U} 5/1 Spirit flier. {1}{U}: its owner
/// shuffles it into their library, then reveals until a card named
/// Mirror-Mad Phantasm, puts that onto the battlefield and the rest into
/// their graveyard.
pub fn mirror_mad_phantasm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::ShuffleSelfRevealUntilSameNameToBattlefield,
            ..Default::default()
        }],
        ..creature("Mirror-Mad Phantasm", cost(&[generic(3), u(), u()]), vec![CreatureType::Spirit], 5, 1)
    }
}

/// Black Cat, Cunning Thief — {3}{B}{B} 2/3 legendary. ETB: look at the top
/// nine of target opponent's library, exile two face down, the rest to the
/// bottom; you may play them while exiled, with mana of any type.
pub fn black_cat_cunning_thief() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookTopExileOneFaceDownMayPlay {
            who: PlayerRef::Target(0),
            count: Value::Const(9),
            rest_to_graveyard: false,
            any_color: false,
            take: 2,
        })],
        ..legend(
            "Black Cat, Cunning Thief",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Villain],
            2,
            3,
        )
    }
}

/// Iymrith, Desert Doom — {3}{U}{U} 5/5 legendary Dragon flier, ward {4} while
/// untapped. Combat damage to a player: draw a card, then draw up to three
/// cards in hand.
pub fn iymrith_desert_doom() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab(
            "Iymrith has ward {4} as long as it's untapped.",
            StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(4)]))),
                condition: R::Untapped,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::NonNeg(Box::new(Value::Diff(
                        Box::new(Value::Const(3)),
                        Box::new(Value::HandSizeOf(PlayerRef::You)),
                    ))),
                },
            ]),
        }],
        ..legend("Iymrith, Desert Doom", cost(&[generic(3), u(), u()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Invoke Justice — {1}{W}{W}{W}{W} sorcery. Return target permanent card
/// from your graveyard to the battlefield, then distribute four +1/+1
/// counters among creatures and Vehicles target player controls.
pub fn invoke_justice() -> CardDefinition {
    spell(
        "Invoke Justice",
        cost(&[generic(1), w(), w(), w(), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::DistributeCountersAmong {
                what: Selector::ControlledBy {
                    who: PlayerRef::Target(1),
                    filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                },
                total: Value::Const(4),
                counter: CounterType::PlusOnePlusOne,
            },
        ]),
    )
}

// ── Batch B: both cards of a two-card gap ───────────────────────────────────

fn on_combat_hit(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn eot_pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn grant_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn top_card() -> Selector {
    Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE }
}

/// Champion of the Flame — {1}{R} 1/1 Human Warrior, trample; +2/+2 for each
/// Aura and Equipment attached to it.
pub fn champion_of_the_flame() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(crate::card::DynamicPt::BasePlusPerAttached {
            base_p: 1,
            base_t: 1,
            per_p: 2,
            per_t: 2,
            auras: true,
            equipment: true,
        }),
        ..creature("Champion of the Flame", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Warrior], 1, 1)
    }
}

/// Koll, the Forgemaster — {R}{W} 2/2 legendary Dwarf Warrior. Another
/// nontoken creature of yours dying enchanted or equipped returns to its
/// owner's hand. Enchanted or equipped creature tokens you control get +1/+1.
pub fn koll_the_forgemaster() -> CardDefinition {
    let attachment = R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).or(R::HasArtifactSubtype(ArtifactSubtype::Equipment));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::All(vec![
                trigger_is(R::Not(Box::new(R::IsToken)).and(R::OtherThanSource)),
                Predicate::ValueAtLeast(Value::AttachmentsOnDyingSubject { filter: attachment }, Value::ONE),
            ])),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
            },
        }],
        static_abilities: vec![static_ab(
            "Creature tokens you control that are enchanted or equipped get +1/+1.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(yours(R::Creature.and(R::IsToken).and(R::IsEnchanted.or(R::IsEquipped)))),
                power: 1,
                toughness: 1,
            },
        )],
        ..legend("Koll, the Forgemaster", cost(&[r(), w()]), vec![CreatureType::Dwarf, CreatureType::Warrior], 2, 2)
    }
}

/// Sedraxis Specter — {U}{B}{R} 3/2 flier; combat damage to a player makes
/// that player discard a card. Unearth {1}{B}.
pub fn sedraxis_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_combat_hit(Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::ONE,
            random: false,
        })],
        activated_abilities: vec![crate::effect::shortcut::unearth(cost(&[generic(1), b()]))],
        ..creature("Sedraxis Specter", cost(&[u(), b(), r()]), vec![CreatureType::Specter], 3, 2)
    }
}

/// Slave of Bolas — {3}{U/R}{B} sorcery. Gain control of target creature,
/// untap it, it gains haste; sacrifice it at the beginning of the next end step.
pub fn slave_of_bolas() -> CardDefinition {
    spell(
        "Slave of Bolas",
        cost(&[generic(3), hybrid(Color::Blue, Color::Red), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            // No duration (CR 611.2a): the steal is permanent; the end-step sacrifice
            // is what ends it, so a creature that can't be sacrificed stays yours.
            Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::Permanent },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            grant_eot(Selector::Target(0), Keyword::Haste),
            Effect::SacrificeAtNextEndStep { what: Selector::Target(0) },
        ]),
    )
}

/// Overflowing Insight — {4}{U}{U}{U} sorcery. Target player draws seven.
pub fn overflowing_insight() -> CardDefinition {
    spell(
        "Overflowing Insight",
        cost(&[generic(4), u(), u(), u()]),
        CardType::Sorcery,
        Effect::Draw { who: target_filtered(R::Player), amount: Value::Const(7) },
    )
}

/// The Magic Mirror — {6}{U}{U}{U} legendary artifact, {1} less per instant
/// and sorcery card in your graveyard. No maximum hand size. Upkeep: a
/// knowledge counter, then draw a card per knowledge counter.
pub fn the_magic_mirror() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            static_ab(
                "This spell costs {1} less to cast for each instant and sorcery card in your graveyard.",
                StaticEffect::SelfCostReducedByValue {
                    amount: Value::count(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    }),
                },
            ),
            static_ab("You have no maximum hand size.", StaticEffect::NoMaximumHandSize),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Knowledge, amount: Value::ONE },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Knowledge },
                },
            ]),
        }],
        ..permanent("The Magic Mirror", cost(&[generic(6), u(), u(), u()]), CardType::Artifact)
    }
}

/// Doctor Doom, Unrivaled — {2}{B}{B} 4/4 legendary, lifelink. {T}: draw a
/// card and lose 1 life; then if your library is empty, you win the game.
pub fn doctor_doom_unrivaled() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                draw(1),
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtMost(Value::LibrarySizeOf(PlayerRef::You), Value::ZERO),
                    then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Doctor Doom, Unrivaled",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Sorcerer, CreatureType::Villain],
            4,
            4,
        )
    }
}

/// Doombot Harbinger — {2}{B} 2/1 artifact creature Robot Villain, flying.
/// ETB: you may mill four. Dies: you may exile it to return target creature
/// card from your graveyard to your hand.
pub fn doombot_harbinger() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(may("Mill four cards?", Effect::Mill { who: Selector::You, amount: Value::Const(4) })),
            // CR 603.12 — "you may exile this card. When you do": the return
            // targets after the exile (Undead Butler's shape).
            crate::effect::shortcut::on_dies(Effect::MayExileSelfThen {
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::InYourGraveyard.and(R::Creature)),
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                }),
            }),
        ],
        ..creature("Doombot Harbinger", cost(&[generic(2), b()]), vec![CreatureType::Robot, CreatureType::Villain], 2, 1)
    }
}

/// Thrasher Brute — {3}{B} 4/3 Orc Warrior. It or another Warrior your team
/// controls entering: target opponent loses 1 life and you gain 1.
pub fn thrasher_brute() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(trigger_is(R::HasCreatureType(CreatureType::Warrior))),
            effect: Effect::Drain { from: target_filtered(R::OpponentPlayer), to: Selector::You, amount: Value::ONE },
        }],
        ..creature("Thrasher Brute", cost(&[generic(3), b()]), vec![CreatureType::Orc, CreatureType::Warrior], 4, 3)
    }
}

/// Raiders' Spoils — {3}{B} enchantment. Creatures you control get +1/+0. A
/// Warrior of yours dealing combat damage to a player: you may pay 1 life to
/// draw a card.
pub fn raiders_spoils() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Creatures you control get +1/+0.",
            StaticEffect::PumpPT { applies_to: Selector::EachPermanent(yours(R::Creature)), power: 1, toughness: 0 },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(trigger_is(R::HasCreatureType(CreatureType::Warrior))),
            effect: Effect::MayPayLife {
                description: "Pay 1 life to draw a card?".into(),
                amount: Value::ONE,
                body: Box::new(draw(1)),
                else_: None,
            },
        }],
        ..permanent("Raiders' Spoils", cost(&[generic(3), b()]), CardType::Enchantment)
    }
}

/// Gallia, the Merrymaker — {1}{R} 2/1 legendary Satyr, haste. Other creatures
/// you control with a +1/+1 counter have haste. {1}{R}, {T}: a +1/+1 counter
/// on target creature that entered this turn.
pub fn gallia_the_merrymaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![static_ab(
            "Each other creature you control with a +1/+1 counter on it has haste.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(
                    R::Creature.and(R::OtherThanSource).and(R::WithCounter(CounterType::PlusOnePlusOne)),
                )),
                keyword: Keyword::Haste,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::EnteredThisTurn)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend("Gallia, the Merrymaker", cost(&[generic(1), r()]), vec![CreatureType::Satyr], 2, 1)
    }
}

/// Ferocity of the Wilds — {2}{R} enchantment. Attacking non-Human creatures
/// you control get +1/+0 and have trample.
pub fn ferocity_of_the_wilds() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Attacking non-Human creatures you control get +1/+0 and have trample.",
            StaticEffect::AnthemForFilter {
                filter: yours(R::Creature.and(R::IsAttacking).and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human))))),
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Trample],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        )],
        ..permanent("Ferocity of the Wilds", cost(&[generic(2), r()]), CardType::Enchantment)
    }
}

/// Archon of Absolution — {3}{W} 3/2 Archon, flying, protection from white.
/// Creatures can't attack you or your planeswalkers unless their controller
/// pays {1} for each.
pub fn archon_of_absolution() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::White)],
        static_abilities: vec![static_ab(
            "Creatures can't attack you or planeswalkers you control unless their controller pays {1} for each of those creatures.",
            StaticEffect::AttackTaxToController { amount: Value::ONE, protect_planeswalkers: true, filter: None },
        )],
        ..creature("Archon of Absolution", cost(&[generic(3), w()]), vec![CreatureType::Archon], 3, 2)
    }
}

/// Pentarch Paladin — {2}{W}{W}{W} 3/3 Human Knight, flanking. As it enters,
/// choose a color. {W}{W}, {T}: destroy target permanent of that color.
pub fn pentarch_paladin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w()]),
            tap_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Permanent.and(R::HasChosenColorOfSource)) },
            ..Default::default()
        }],
        ..creature("Pentarch Paladin", cost(&[generic(2), w(), w(), w()]), vec![CreatureType::Human, CreatureType::Knight], 3, 3)
    }
}

/// Clarion Spirit — {1}{W} 2/2 Spirit. Your second spell each turn makes a
/// 1/1 white Spirit flier.
pub fn clarion_spirit() -> CardDefinition {
    let spirit = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Spirit", 1, 1, vec![Color::White], vec![CreatureType::Spirit])
    };
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::flurry(make(Value::ONE, spirit))],
        ..creature("Clarion Spirit", cost(&[generic(1), w()]), vec![CreatureType::Spirit], 2, 2)
    }
}

/// Otherworld Atlas — {4} artifact Book. {T}: a charge counter. {T}: each
/// player draws a card for each charge counter on it.
pub fn otherworld_atlas() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Book], ..Default::default() },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                },
                ..Default::default()
            },
        ],
        ..permanent("Otherworld Atlas", cost(&[generic(4)]), CardType::Artifact)
    }
}

/// Farsight Adept — {2}{W} 3/3 Kor Wizard. ETB: you and target opponent each
/// draw a card.
pub fn farsight_adept() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(1),
            Effect::Draw { who: target_filtered(R::OpponentPlayer), amount: Value::ONE },
        ]))],
        ..creature("Farsight Adept", cost(&[generic(2), w()]), vec![CreatureType::Kor, CreatureType::Wizard], 3, 3)
    }
}

/// Adept Watershaper — {2}{W} 3/4 Merfolk Cleric. Other tapped creatures you
/// control have indestructible.
pub fn adept_watershaper() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Other tapped creatures you control have indestructible.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Creature.and(R::Tapped).and(R::OtherThanSource))),
                keyword: Keyword::Indestructible,
            },
        )],
        ..creature("Adept Watershaper", cost(&[generic(2), w()]), vec![CreatureType::Merfolk, CreatureType::Cleric], 3, 4)
    }
}

/// Light the Way — {W} instant. Choose one: a +1/+1 counter on target
/// creature or Vehicle and untap it; or return target permanent you control
/// to its owner's hand.
pub fn light_the_way() -> CardDefinition {
    spell(
        "Light the Way",
        cost(&[w()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Untap { what: Selector::Target(0), up_to: None },
            ]),
            Effect::Move {
                what: target_filtered(yours(R::Permanent)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
    )
}

/// Iron Lad, Diverging Destiny — {2}{U} 2/2 legendary artifact creature Human
/// Hero, flying, vigilance. You may look at the top card of your library any
/// time. {T}: reveal your top card; if it's an artifact card, draw a card.
pub fn iron_lad_diverging_destiny() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "You may look at the top card of your library any time.",
            effect: StaticEffect::MayLookAtOwnLibraryTop,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: top_card(), filter: R::Artifact },
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..legend("Iron Lad, Diverging Destiny", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Hero], 2, 2)
    }
}

/// Tuya Bearclaw — {1}{R}{G} 2/2 legendary Human Warrior. Attacking: +X/+X,
/// X = the greatest power among other creatures you control.
pub fn tuya_bearclaw() -> CardDefinition {
    let x = || {
        Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(R::Creature.and(R::OtherThanSource))))
    };
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::PumpPT {
            what: Selector::This,
            power: x(),
            toughness: x(),
            duration: Duration::EndOfTurn,
        })],
        ..legend("Tuya Bearclaw", cost(&[generic(1), r(), g()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2)
    }
}

/// Orcish Spy — {R} 1/1 Orc Rogue. {T}: look at the top three cards of
/// target player's library.
pub fn orcish_spy() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LookAtTop { who: PlayerRef::Target(0), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..creature("Orcish Spy", cost(&[r()]), vec![CreatureType::Orc, CreatureType::Rogue], 1, 1)
    }
}

/// Rush of Vitality — {1}{B} instant. Target creature gets +1/+0 and gains
/// lifelink and indestructible until end of turn.
pub fn rush_of_vitality() -> CardDefinition {
    spell(
        "Rush of Vitality",
        cost(&[generic(1), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            eot_pump(target_filtered(R::Creature), 1, 0),
            grant_eot(Selector::Target(0), Keyword::Lifelink),
            grant_eot(Selector::Target(0), Keyword::Indestructible),
        ]),
    )
}

/// Dreamscape Artist — {1}{U} 1/1 Human Spellshaper. {2}{U}, {T}, discard a
/// card, sacrifice a land: search for up to two basic lands onto the
/// battlefield.
pub fn dreamscape_artist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                count: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature("Dreamscape Artist", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Spellshaper], 1, 1)
    }
}

/// Rolling Earthquake — {X}{R} sorcery. X damage to each creature without
/// horsemanship and each player.
pub fn rolling_earthquake() -> CardDefinition {
    spell(
        "Rolling Earthquake",
        cost(&[crate::mana::x(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Horsemanship).negate())),
                body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::XFromCost }),
            },
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::XFromCost }),
            },
        ]),
    )
}

/// Insult // Injury — {2}{R} sorcery: damage can't be prevented this turn and
/// your sources deal double damage this turn. Injury ({2}{R}, aftermath): 2
/// damage to target creature and 2 to target player or planeswalker.
pub fn insult_injury() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(2), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) },
                    Effect::DealDamage {
                        to: Selector::TargetFiltered { slot: 1, filter: R::Player.or(R::Planeswalker) },
                        amount: Value::Const(2),
                    },
                ]),
            },
            fuse: false,
            aftermath: true,
        })),
        ..spell(
            "Insult // Injury",
            cost(&[generic(2), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![Effect::DamageCantBePreventedThisTurn, Effect::DoubleYourSourcesDamageThisTurn]),
        )
    }
}

/// Barrin, Tolarian Archmage — {1}{U}{U} 2/2 legendary Human Wizard. ETB:
/// return up to one other target creature or planeswalker to its owner's
/// hand. Your end step: draw a card if a permanent was put into your hand
/// from the battlefield this turn.
pub fn barrin_tolarian_archmage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.or(R::Planeswalker).and(R::OtherThanSource)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::PermanentPutIntoHandThisTurn { who: PlayerRef::You }),
                effect: draw(1),
            },
        ],
        ..legend("Barrin, Tolarian Archmage", cost(&[generic(1), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Watcher for Tomorrow — {1}{U} 2/1 Human Wizard, hideaway 4, enters tapped.
/// Leaving the battlefield puts the hidden card into its owner's hand.
pub fn watcher_for_tomorrow() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![
            etb(Effect::Hideaway { count: Value::Const(4) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::CardExiledWithSource))),
                },
            },
        ],
        ..creature("Watcher for Tomorrow", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 1)
    }
}

/// Scuzzback Marauders — {4}{R/G} 5/2 Goblin Warrior, trample, persist.
pub fn scuzzback_marauders() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Persist],
        ..creature(
            "Scuzzback Marauders",
            cost(&[generic(4), hybrid(Color::Red, Color::Green)]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            5,
            2,
        )
    }
}

/// Rulik Mons, Warren Chief — {1}{R}{G}{G} 3/3 legendary Goblin, menace.
/// Attacking: look at your top card; a land may go onto the battlefield
/// tapped, otherwise create a 1/1 red Goblin.
pub fn rulik_mons_warren_chief() -> CardDefinition {
    let goblin = || make(Value::ONE, token("Goblin", 1, 1, vec![Color::Red], vec![CreatureType::Goblin]));
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::If {
            cond: Predicate::EntityMatches { what: top_card(), filter: R::Land },
            then: Box::new(Effect::MayDoElse {
                description: "Put the land on top of your library onto the battlefield tapped?".into(),
                body: Box::new(Effect::Move {
                    what: top_card(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
                else_: Box::new(goblin()),
            }),
            else_: Box::new(goblin()),
        })],
        ..legend("Rulik Mons, Warren Chief", cost(&[generic(1), r(), g(), g()]), vec![CreatureType::Goblin], 3, 3)
    }
}

/// Teferi's Tutelage — {2}{U} enchantment. ETB: draw, then discard. Whenever
/// you draw a card, target opponent mills two.
pub fn teferis_tutelage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                draw(1),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::Mill { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
            },
        ],
        ..permanent("Teferi's Tutelage", cost(&[generic(2), u()]), CardType::Enchantment)
    }
}

/// Thundering Djinn — {3}{U}{R} 3/4 Djinn flier. Attacking: damage to any
/// target equal to the cards you've drawn this turn.
pub fn thundering_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::DealDamage {
            to: target_any(),
            amount: Value::CardsDrawnThisTurn(PlayerRef::You),
        })],
        ..creature("Thundering Djinn", cost(&[generic(3), u(), r()]), vec![CreatureType::Djinn], 3, 4)
    }
}

/// Lightning Crafter — {3}{R} 3/3 Goblin Shaman. Champion a Goblin or Shaman.
/// {T}: 3 damage to any target.
pub fn lightning_crafter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Champion {
            filter: R::HasCreatureType(CreatureType::Goblin).or(R::HasCreatureType(CreatureType::Shaman)),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..creature("Lightning Crafter", cost(&[generic(3), r()]), vec![CreatureType::Goblin, CreatureType::Shaman], 3, 3)
    }
}

/// Moria Marauder — {R}{R} 1/1 Goblin Warrior, double strike. A Goblin or Orc
/// of yours dealing combat damage to a player exiles your top card; you may
/// play it this turn.
pub fn moria_marauder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                trigger_is(R::HasCreatureType(CreatureType::Goblin).or(R::HasCreatureType(CreatureType::Orc))),
            ),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..creature("Moria Marauder", cost(&[r(), r()]), vec![CreatureType::Goblin, CreatureType::Warrior], 1, 1)
    }
}

/// Ghalta the Unstoppable — {8}{G} 8/8 legendary Elder Dinosaur, trample; {X}
/// less, X = the greatest power among your creatures. Other creatures you
/// control have trample.
pub fn ghalta_the_unstoppable() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![
            static_ab(
                "This spell costs {X} less to cast, where X is the greatest power among creatures you control.",
                StaticEffect::SelfCostReducedByValue {
                    amount: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                },
            ),
            static_ab(
                "Other creatures you control have trample.",
                StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(yours(R::Creature.and(R::OtherThanSource))),
                    keyword: Keyword::Trample,
                },
            ),
        ],
        ..legend("Ghalta the Unstoppable", cost(&[generic(8), g()]), vec![CreatureType::Elder, CreatureType::Dinosaur], 8, 8)
    }
}

/// The Tarrasque — {6}{G}{G}{G} 10/10 legendary Dinosaur; haste and ward {10}
/// as long as it was cast. Attacking: it fights target creature defending
/// player controls.
pub fn the_tarrasque() -> CardDefinition {
    let was_cast = || R::EnteredByCast;
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "The Tarrasque has haste as long as it was cast.",
                StaticEffect::SelfHasKeywordWhile { keyword: Keyword::Haste, condition: was_cast() },
            ),
            static_ab(
                "The Tarrasque has ward {10} as long as it was cast.",
                StaticEffect::SelfHasKeywordWhile {
                    keyword: Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(10)]))),
                    condition: was_cast(),
                },
            ),
        ],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Fight {
            attacker: Selector::This,
            defender: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
        })],
        ..legend("The Tarrasque", cost(&[generic(6), g(), g(), g()]), vec![CreatureType::Dinosaur], 10, 10)
    }
}

/// Ancient Imperiosaur — {5}{G}{G} 6/6 Dinosaur, convoke, trample, ward {2};
/// enters with two +1/+1 counters for each creature that convoked it.
pub fn ancient_imperiosaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Convoke,
            Keyword::Trample,
            Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(2)]))),
        ],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::count(Selector::CreaturesThatConvokedSource)),
            ),
        })],
        ..creature("Ancient Imperiosaur", cost(&[generic(5), g(), g()]), vec![CreatureType::Dinosaur], 6, 6)
    }
}

/// Cult Conscript — {B} 2/1 Skeleton Warrior, enters tapped. {1}{B}: return it
/// from your graveyard to the battlefield, only if a non-Skeleton creature
/// died under your control this turn.
pub fn cult_conscript() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_graveyard: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::CreatureDeathsThisTurnMatching {
                    filter: R::Not(Box::new(R::HasCreatureType(CreatureType::Skeleton))).and(R::ControlledByYou),
                },
                Value::ONE,
            )),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Cult Conscript", cost(&[b()]), vec![CreatureType::Skeleton, CreatureType::Warrior], 2, 1)
    }
}

/// Ozox, the Clattering King — {2}{B} 3/2 legendary Skeleton Noble, can't
/// block. Dies: create Jumblebones, a legendary 2/1 Skeleton that can't block
/// and, leaving the battlefield, returns target card named Ozox from your
/// graveyard to your hand.
pub fn ozox_the_clattering_king() -> CardDefinition {
    let jumblebones = TokenDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(R::HasName("Ozox, the Clattering King".into()).and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..token("Jumblebones", 2, 1, vec![Color::Black], vec![CreatureType::Skeleton])
    };
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(make(Value::ONE, jumblebones))],
        ..legend("Ozox, the Clattering King", cost(&[generic(2), b()]), vec![CreatureType::Skeleton, CreatureType::Noble], 3, 2)
    }
}

/// Shadow-Rite Priest — {1}{B} 2/2 Human Cleric. Other Clerics you control get
/// +1/+1. {3}{B}{B}, {T}, sacrifice another Cleric: search for a black
/// creature card onto the battlefield.
pub fn shadow_rite_priest() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Other Clerics you control get +1/+1.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Cleric).and(R::OtherThanSource))),
                power: 1,
                toughness: 1,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b(), b()]),
            tap_cost: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Cleric).and(R::OtherThanSource), 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature.and(R::HasColor(Color::Black)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Shadow-Rite Priest", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 2)
    }
}

/// Relic Vial — {3} artifact. {2}, {T}, sacrifice a creature: draw a card.
/// While you control a Cleric, a creature of yours dying drains each opponent
/// for 1.
pub fn relic_vial() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: draw(1),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::SelectorExists(Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Cleric)))),
            ),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                gain(1),
            ]),
        }],
        ..permanent("Relic Vial", cost(&[generic(3)]), CardType::Artifact)
    }
}

/// Windrider Wizard — {2}{U} 2/2 Human Wizard flier. Casting an instant,
/// sorcery, or Wizard spell: you may draw, then discard.
pub fn windrider_wizard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(
                R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .or(R::HasCreatureType(CreatureType::Wizard)),
            )),
            effect: may(
                "Draw a card, then discard a card?",
                Effect::Seq(vec![draw(1), Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }]),
            ),
        }],
        ..creature("Windrider Wizard", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Dreampod Druid — {1}{G} 2/2 Human Druid. Each upkeep, if it's enchanted,
/// create a 1/1 green Saproling.
pub fn dreampod_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::IsEnchanted }),
            effect: make(Value::ONE, token("Saproling", 1, 1, vec![Color::Green], vec![CreatureType::Saproling])),
        }],
        ..creature("Dreampod Druid", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Druid], 2, 2)
    }
}

/// Krond the Dawn-Clad — {G}{G}{G}{W}{W}{W} 6/6 legendary Archon, flying,
/// vigilance. Attacking while enchanted: exile target permanent.
pub fn krond_the_dawn_clad() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::IsEnchanted }),
            effect: Effect::Exile { what: target_filtered(R::Permanent) },
        }],
        ..legend("Krond the Dawn-Clad", cost(&[g(), g(), g(), w(), w(), w()]), vec![CreatureType::Archon], 6, 6)
    }
}

/// Call to the Feast — {2}{W}{B} sorcery. Three 1/1 white Vampires with
/// lifelink.
pub fn call_to_the_feast() -> CardDefinition {
    let vampire = TokenDefinition {
        keywords: vec![Keyword::Lifelink],
        ..token("Vampire", 1, 1, vec![Color::White], vec![CreatureType::Vampire])
    };
    spell("Call to the Feast", cost(&[generic(2), w(), b()]), CardType::Sorcery, make(Value::Const(3), vampire))
}

/// Phial of Galadriel — {3} legendary artifact. Drawing with an empty hand
/// draws two instead; gaining life at 5 or less life gains twice that much.
/// {T}: add one mana of any color.
pub fn phial_of_galadriel() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            static_ab(
                "If you would draw a card while you have no cards in hand, draw two cards instead.",
                StaticEffect::EmptyHandDrawBonus { extra: 1, life_loss: 0 },
            ),
            static_ab(
                "If you would gain life while you have 5 or less life, you gain twice that much life instead.",
                StaticEffect::ControllerLifeGainDoubledAtOrBelow { life: 5 },
            ),
        ],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..permanent("Phial of Galadriel", cost(&[generic(3)]), CardType::Artifact)
    }
}

/// Henrika Domnathi // Henrika, Infernal Seer — {2}{B}{B} 1/3 legendary
/// Vampire flier. Beginning of combat on your turn, choose one not chosen
/// before: each player sacrifices a creature; you draw a card and lose 1
/// life; transform Henrika. Back: 3/4 flying, deathtouch, lifelink; {1}{B}{B}:
/// your creatures with flying, deathtouch and/or lifelink get +1/+0.
pub fn henrika_domnathi() -> CardDefinition {
    let seer = CardDefinition {
        name: "Henrika, Infernal Seer",
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), b()]),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(yours(
                    R::Creature.and(
                        R::HasKeyword(Keyword::Flying)
                            .or(R::HasKeyword(Keyword::Deathtouch))
                            .or(R::HasKeyword(Keyword::Lifelink)),
                    ),
                )),
                body: Box::new(eot_pump(Selector::TriggerSource, 1, 0)),
            },
            ..Default::default()
        }],
        ..legend("Henrika, Infernal Seer", ManaCost::default(), vec![CreatureType::Vampire], 3, 4)
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    Effect::Sacrifice { who: Selector::Player(PlayerRef::EachPlayer), count: Value::ONE, filter: R::Creature },
                    Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]),
                    Effect::Transform { what: Selector::This },
                ],
            },
        }],
        back_face: Some(Box::new(seer)),
        ..legend("Henrika Domnathi", cost(&[generic(2), b(), b()]), vec![CreatureType::Vampire], 1, 3)
    }
}

/// The Restoration of Eiganjo // Architect of Restoration — {2}{W} Saga. I:
/// search for a basic Plains to hand. II: you may discard a card to return a
/// permanent card with mana value 2 or less from your graveyard tapped. III:
/// return transformed — a 3/4 Fox Monk, vigilance, whose attacks and blocks
/// make 1/1 colorless Spirits.
pub fn the_restoration_of_eiganjo() -> CardDefinition {
    let spirit = || make(Value::ONE, token("Spirit", 1, 1, vec![], vec![CreatureType::Spirit]));
    let architect = CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: spirit() },
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: spirit() },
        ],
        ..creature("Architect of Restoration", ManaCost::default(), vec![CreatureType::Fox, CreatureType::Monk], 3, 4)
    };
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand.and(R::HasLandType(LandType::Plains)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
            (
                2,
                may(
                    "Discard a card to return a permanent card with mana value 2 or less from your graveyard?",
                    Effect::Seq(vec![
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                        Effect::Move {
                            what: target_filtered(R::PermanentCard.and(R::ManaValueAtMost(2)).and(R::InYourGraveyard)),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                        },
                    ]),
                ),
            ),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(architect)),
        ..permanent("The Restoration of Eiganjo", cost(&[generic(2), w()]), CardType::Enchantment)
    }
}

/// Draconic Visitor — {3}{R}{R} 5/5 Dragon flier. Artifact tokens you would
/// create are that many 5/5 red Dragon fliers instead.
pub fn draconic_visitor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab(
            "If one or more artifact tokens would be created under your control, that many 5/5 red Dragon creature tokens with flying are created instead.",
            StaticEffect::ArtifactTokensBecome {
                into: TokenDefinition {
                    keywords: vec![Keyword::Flying],
                    ..token("Dragon", 5, 5, vec![Color::Red], vec![CreatureType::Dragon])
                },
            },
        )],
        ..creature("Draconic Visitor", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

// ── Batch C ──────────────────────────────────────────────────────────────────

/// Street Urchin — {1}{R} legendary enchantment Background. Commander
/// creatures you own have "{1}, sacrifice another creature or an artifact: 1
/// damage to any target."
pub fn street_urchin() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![static_ab(
            "Commander creatures you own have \"{1}, Sacrifice another creature or an artifact: This creature deals 1 damage to any target.\"",
            StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsCommander).and(R::OwnedByYou)),
                ability: ActivatedAbility {
                    mana_cost: cost(&[generic(1)]),
                    sac_other_filter: Some((R::Creature.and(R::OtherThanSource).or(R::Artifact), 1)),
                    effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                    ..Default::default()
                },
                condition: None,
            },
        )],
        ..permanent("Street Urchin", cost(&[generic(1), r()]), CardType::Enchantment)
    }
}

/// Sarinth Greatwurm — {4}{R}{G} 7/6 Wurm, trample. Whenever a land enters,
/// create a tapped Powerstone token.
pub fn sarinth_greatwurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(trigger_is(R::Land)),
            effect: Effect::Seq(vec![
                make(Value::ONE, crabomination_base::tokens::powerstone_token()),
                Effect::Tap { what: Selector::LastCreatedToken },
            ]),
        }],
        ..creature("Sarinth Greatwurm", cost(&[generic(4), r(), g()]), vec![CreatureType::Wurm], 7, 6)
    }
}

/// Atrocious Experiment — {2}{B} sorcery. Target player mills two, draws two,
/// and loses 2 life.
pub fn atrocious_experiment() -> CardDefinition {
    spell(
        "Atrocious Experiment",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(2) },
            Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
        ]),
    )
}

/// Primal Amulet // Primal Wellspring — {4} artifact. Instants and sorceries
/// you cast cost {1} less; each one adds a charge counter, and at four you may
/// remove them and transform it. Back: land, {T}: any color; spent on an
/// instant or sorcery, copy that spell.
pub fn primal_amulet() -> CardDefinition {
    let is = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    let wellspring = CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                    crate::mana::SpendRestriction::InstantSorceryCopy,
                ),
            },
            ..Default::default()
        }],
        ..land("Primal Wellspring", vec![])
    };
    CardDefinition {
        static_abilities: vec![static_ab(
            "Instant and sorcery spells you cast cost {1} less to cast.",
            StaticEffect::CostReduction { filter: is(), amount: 1 },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(is())),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                        Value::Const(4),
                    ),
                    then: Box::new(may(
                        "Remove the charge counters and transform Primal Amulet?",
                        Effect::Seq(vec![
                            Effect::RemoveCounter {
                                what: Selector::This,
                                kind: CounterType::Charge,
                                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                            },
                            Effect::Transform { what: Selector::This },
                        ]),
                    )),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(wellspring)),
        ..permanent("Primal Amulet", cost(&[generic(4)]), CardType::Artifact)
    }
}

/// Locket of Yesterdays — {1} artifact. Spells you cast cost {1} less for each
/// card with the same name in your graveyard.
pub fn locket_of_yesterdays() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Spells you cast cost {1} less to cast for each card with the same name as that spell in your graveyard.",
            StaticEffect::SpellsCostLessPerSameNameInYourGraveyard,
        )],
        ..permanent("Locket of Yesterdays", cost(&[generic(1)]), CardType::Artifact)
    }
}

/// Gelatinous Cube — {2}{B}{B} 4/3 Ooze. Engulf: ETB exiles target non-Ooze
/// creature an opponent controls until it leaves. Dissolve — {X}{B}: put
/// target creature card with mana value X exiled with it into its owner's
/// graveyard.
pub fn gelatinous_cube() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                R::Creature.and(R::ControlledByOpponent).and(R::Not(Box::new(R::HasCreatureType(CreatureType::Ooze)))),
            ),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x(), b()]),
            effect: Effect::Move {
                what: target_filtered(R::ExiledWithSource.and(R::Creature).and(R::ManaValueExactlyXFromCost)),
                to: ZoneDest::Graveyard,
            },
            ..Default::default()
        }],
        ..creature("Gelatinous Cube", cost(&[generic(2), b(), b()]), vec![CreatureType::Ooze], 4, 3)
    }
}

/// Harbin, Vanguard Aviator — {W}{U} 3/2 legendary Human Soldier flier.
/// Attacking with five or more Soldiers gives your creatures +1/+1 and flying
/// until end of turn.
pub fn harbin_vanguard_aviator() -> CardDefinition {
    let yours_all = || Selector::EachPermanent(yours(R::Creature));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(Predicate::ValueAtLeast(
                Value::count(Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Soldier).and(R::IsAttacking)))),
                Value::Const(5),
            )),
            effect: Effect::Seq(vec![eot_pump(yours_all(), 1, 1), grant_eot(yours_all(), Keyword::Flying)]),
        }],
        ..legend("Harbin, Vanguard Aviator", cost(&[w(), u()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 2)
    }
}

/// Skystrike Officer — {2}{U} 2/3 Human Soldier flier. Attacking: a 1/1
/// colorless Soldier artifact creature token. Tap three untapped Soldiers you
/// control: draw a card.
pub fn skystrike_officer() -> CardDefinition {
    let soldier = TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..token("Soldier", 1, 1, vec![], vec![CreatureType::Soldier])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: make(Value::ONE, soldier),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::HasCreatureType(CreatureType::Soldier), 3)),
            effect: draw(1),
            ..Default::default()
        }],
        ..creature("Skystrike Officer", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 3)
    }
}

/// Lo and Li, Royal Advisors — {2}{B}{B} 3/3 legendary Human Advisor. An
/// opponent discarding or milling puts a +1/+1 counter on each Advisor you
/// control. {2}{U/B}: target player mills four.
pub fn lo_and_li_royal_advisors() -> CardDefinition {
    let grow = || Effect::AddCounter {
        what: Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Advisor))),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::DiscardedOneOrMore, EventScope::OpponentControl), effect: grow() },
            // "mills one or more cards" — once per mill, not per card.
            TriggeredAbility {
                event: EventSpec::new(EventKind::Milled, EventScope::OpponentControl).once_per_batch(),
                effect: grow(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), hybrid(Color::Blue, Color::Black)]),
            effect: Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(4) },
            ..Default::default()
        }],
        ..legend("Lo and Li, Royal Advisors", cost(&[generic(2), b(), b()]), vec![CreatureType::Human, CreatureType::Advisor], 3, 3)
    }
}

/// Master's Councillors — {1}{U} 1/3 Human Advisor, vigilance; +2/+0 for each
/// graveyard with seven or more cards. Your second draw each turn: target
/// player mills three.
pub fn masters_councillors() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![static_ab(
            "This creature gets +2/+0 for each graveyard with seven or more cards in it.",
            StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: Value::Times(Box::new(Value::Const(2)), Box::new(Value::GraveyardsWithAtLeast(7))),
                toughness: Value::ZERO,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(3) },
        }],
        ..creature("Master's Councillors", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Advisor], 1, 3)
    }
}

/// Krav, the Unredeemed — {4}{B} 3/3 legendary Demon, partner with Regna.
/// {B}, sacrifice X creatures: target player draws X and gains X life; X
/// +1/+1 counters on Krav.
pub fn krav_the_unredeemed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Regna, the Redeemer".into())],
        triggered_abilities: vec![crate::effect::shortcut::partner_with_search("Regna, the Redeemer")],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_x: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: target_filtered(R::Player), amount: Value::XFromCost },
                Effect::GainLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::XFromCost },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::XFromCost },
            ]),
            ..Default::default()
        }],
        ..legend("Krav, the Unredeemed", cost(&[generic(4), b()]), vec![CreatureType::Demon], 3, 3)
    }
}

/// Regna, the Redeemer — {5}{W} 4/4 legendary Angel flier, partner with Krav.
/// Each end step, if your team gained life this turn, two 1/1 white Warriors.
pub fn regna_the_redeemer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::PartnerWith("Krav, the Unredeemed".into())],
        triggered_abilities: vec![
            crate::effect::shortcut::partner_with_search("Krav, the Unredeemed"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                    .with_filter(Predicate::PlayerGainedLifeThisTurn { who: PlayerRef::You }),
                effect: make(Value::Const(2), token("Warrior", 1, 1, vec![Color::White], vec![CreatureType::Warrior])),
            },
        ],
        ..legend("Regna, the Redeemer", cost(&[generic(5), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Rigo, Streetwise Mentor — {G/W}{W}{W/U} 2/2 legendary Cat Citizen, enters
/// with a shield counter. Attacking with one or more creatures with power 1
/// or less draws a card.
pub fn rigo_streetwise_mentor() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Shield, Value::ONE)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(Predicate::SelectorExists(
                Selector::EachPermanent(yours(R::IsAttacking.and(R::PowerAtMost(1)))),
            )),
            effect: draw(1),
        }],
        ..legend(
            "Rigo, Streetwise Mentor",
            cost(&[hybrid(Color::Green, Color::White), w(), hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Cat, CreatureType::Citizen],
            2,
            2,
        )
    }
}

/// Protect the Negotiators — {1}{U} instant, kicker {W}. Kicked: a 1/1 white
/// Soldier. Counter target spell unless its controller pays {1} for each
/// creature you control.
pub fn protect_the_negotiators() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[w()]))],
        ..spell(
            "Protect the Negotiators",
            cost(&[generic(1), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(make(Value::ONE, token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier]))),
                    else_: Box::new(Effect::Noop),
                },
                Effect::CounterUnlessPaid {
                    what: target_filtered(R::IsSpellOnStack),
                    mana_cost: ManaCost::default(),
                    exile: false,
                    extra_generic: Some(Value::count(Selector::EachPermanent(yours(R::Creature)))),
                    if_paid: None,
                },
            ]),
        )
    }
}

/// Hawkeye, Young Avenger — {3}{R} 2/4 legendary Human Archer Hero, reach.
/// Your sources' noncombat damage to opponents and their permanents gets +X,
/// X = Hawkeye's power.
pub fn hawkeye_young_avenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        static_abilities: vec![static_ab(
            "If a source you control would deal noncombat damage to an opponent or a permanent an opponent controls, instead it deals that much damage plus X, where X is Hawkeye's power.",
            StaticEffect::NoncombatDamageToOpponentsPlusOwnPower,
        )],
        ..legend(
            "Hawkeye, Young Avenger",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Archer, CreatureType::Hero],
            2,
            4,
        )
    }
}

/// Lapis Orb of Dragonkind — {2}{U} artifact. {T}: {U}; spent on a Dragon
/// creature spell, scry 2.
pub fn lapis_orb_of_dragonkind() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::OfColor(Color::Blue, Value::ONE)),
                    crate::mana::SpendRestriction::DragonCreatureScryTwo,
                ),
            },
            ..Default::default()
        }],
        ..permanent("Lapis Orb of Dragonkind", cost(&[generic(2), u()]), CardType::Artifact)
    }
}

/// Themberchaud — {4}{R}{R}{R} 5/5 legendary Dragon, trample. ETB: X damage
/// to each other creature without flying and each player, X = Mountains you
/// control. You may exert it as it attacks; when you do, it gains flying.
pub fn themberchaud() -> CardDefinition {
    let x = || Value::count(Selector::EachPermanent(yours(R::HasLandType(LandType::Mountain))));
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Exert],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::ForEach {
                    selector: Selector::EachPermanent(
                        R::Creature.and(R::OtherThanSource).and(R::HasKeyword(Keyword::Flying).negate()),
                    ),
                    body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: x() }),
                },
                Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachPlayer),
                    body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: x() }),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Exerted, EventScope::SelfSource),
                effect: grant_eot(Selector::This, Keyword::Flying),
            },
        ],
        ..legend("Themberchaud", cost(&[generic(4), r(), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Last Light of Durin's Day — {1}{R} enchantment. A Mountain of yours
/// entering adds a quest counter; at six or more, sacrifice it and put a
/// Dragon card from your hand or library onto the battlefield.
/// Mountaincycling {2}.
pub fn last_light_of_durins_day() -> CardDefinition {
    let to_bf = ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false };
    let dragon = || R::HasCreatureType(CreatureType::Dragon);
    CardDefinition {
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Mountain)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(trigger_is(R::HasLandType(LandType::Mountain))),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Quest },
                        Value::Const(6),
                    ),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSelected { what: Selector::This },
                        // A Dragon in hand comes first; otherwise the library.
                        Effect::If {
                            cond: Predicate::SelectorExists(Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: crate::card::Zone::Hand,
                                filter: dragon(),
                            }),
                            then: Box::new(Effect::Move {
                                what: Selector::TakeGreatestManaValue {
                                    inner: Box::new(Selector::CardsInZone {
                                        who: PlayerRef::You,
                                        zone: crate::card::Zone::Hand,
                                        filter: dragon(),
                                    }),
                                    count: Box::new(Value::ONE),
                                },
                                to: to_bf.clone(),
                            }),
                            else_: Box::new(Effect::Search { who: PlayerRef::You, filter: dragon(), to: to_bf }),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..permanent("Last Light of Durin's Day", cost(&[generic(1), r()]), CardType::Enchantment)
    }
}

// ── Batch D ──────────────────────────────────────────────────────────────────

/// Isengard Unleashed — {2}{R}{R}{R} sorcery. Damage can't be prevented this
/// turn; your sources deal triple damage to opponents and their permanents
/// this turn. Flashback {4}{R}{R}{R}.
pub fn isengard_unleashed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), r(), r(), r()]))],
        ..spell(
            "Isengard Unleashed",
            cost(&[generic(2), r(), r(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![Effect::DamageCantBePreventedThisTurn, Effect::TripleYourSourcesDamageToOpponentsThisTurn]),
        )
    }
}

/// Chromium, the Mutable — {4}{W}{U}{B} 7/7 legendary Elder Dragon, flash,
/// can't be countered, flying. Discard a card: until end of turn it becomes a
/// 1/1 Human that loses all abilities and gains hexproof; it can't be blocked
/// this turn.
pub fn chromium_the_mutable() -> CardDefinition {
    let eot = Duration::EndOfTurn;
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::CantBeCountered, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::BecomeCreatureType { what: Selector::This, creature_types: vec![CreatureType::Human], duration: eot },
                Effect::SetBasePT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: eot },
                Effect::LoseAllAbilities { what: Selector::This, duration: eot },
                grant_eot(Selector::This, Keyword::Hexproof),
                grant_eot(Selector::This, Keyword::Unblockable),
            ]),
            ..Default::default()
        }],
        ..legend(
            "Chromium, the Mutable",
            cost(&[generic(4), w(), u(), b()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            7,
            7,
        )
    }
}

/// Windshaper Planetar — {4}{W} 4/4 Angel, flash, flying. Entering during the
/// declare attackers step, you may reselect what each attacking creature is
/// attacking.
pub fn windshaper_planetar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::CurrentStepIs(TurnStep::DeclareAttackers)),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(R::IsAttacking),
                body: Box::new(Effect::ReselectAttackTarget { what: Selector::TriggerSource, players_only: false }),
            },
        }],
        ..creature("Windshaper Planetar", cost(&[generic(4), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Neerdiv, Devious Diver — {2}{U} 2/2 legendary Merfolk Rogue. Becoming
/// tapped: target player mills cards equal to its power. Casting a spell from
/// your graveyard or activating an ability of a card there: draw a card and a
/// +1/+1 counter on Neerdiv.
pub fn neerdiv_devious_diver() -> CardDefinition {
    let payoff = || Effect::Seq(vec![
        draw(1),
        Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
    ]);
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::Mill {
                    who: target_filtered(R::Player),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(trigger_is(R::WasCastFromGraveyard)),
                effect: payoff(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::AbilityActivated, EventScope::YourControl)
                    .with_filter(trigger_is(R::InYourGraveyard)),
                effect: payoff(),
            },
        ],
        ..legend("Neerdiv, Devious Diver", cost(&[generic(2), u()]), vec![CreatureType::Merfolk, CreatureType::Rogue], 2, 2)
    }
}

/// Rona, Sheoldred's Faithful — {1}{U}{B}{B} 3/4 legendary Human Wizard. Your
/// instant and sorcery casts drain each opponent of 1 life. You may cast it
/// from your graveyard by discarding two cards in addition to its other costs.
pub fn rona_sheoldreds_faithful() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::GraveyardCast],
        flashback_additional_cost: vec![crate::card::AdditionalCastCost::Discard { count: 2, filter: None }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            )),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        }],
        ..legend(
            "Rona, Sheoldred's Faithful",
            cost(&[generic(1), u(), b(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Owlbear Cub — {2}{G} 3/3 Bird Bear. Attacking a player who controls eight
/// or more lands: look at your top eight; you may put a creature card from
/// among them onto the battlefield tapped and attacking that player.
pub fn owlbear_cub() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            // "Attacks a PLAYER who controls eight or more lands" (CR 506.2).
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::This, filter: R::IsAttackingOpponentPlayer },
                Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByDefendingPlayer))),
                    Value::Const(8),
                ),
            ])),
            effect: Effect::LookTopMayDeployAttacking {
                count: Value::Const(8),
                filter: R::Creature,
                return_at_end_of_combat: false,
                plain: true,
            },
        }],
        ..creature("Owlbear Cub", cost(&[generic(2), g()]), vec![CreatureType::Bird, CreatureType::Bear], 3, 3)
    }
}

/// Ulvenwald Oddity // Ulvenwald Behemoth — {2}{G}{G} 4/4 Beast, trample,
/// haste; {5}{G}{G}: transform. Back: 8/8 trample, haste; other creatures you
/// control get +1/+1 and have trample and haste.
pub fn ulvenwald_oddity() -> CardDefinition {
    let behemoth = CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        static_abilities: vec![static_ab(
            "Other creatures you control get +1/+1 and have trample and haste.",
            StaticEffect::AnthemForFilter {
                filter: yours(R::Creature.and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Trample, Keyword::Haste],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        )],
        ..creature("Ulvenwald Behemoth", ManaCost::default(), vec![CreatureType::Beast, CreatureType::Horror], 8, 8)
    };
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g(), g()]),
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(behemoth)),
        ..creature("Ulvenwald Oddity", cost(&[generic(2), g(), g()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Shanna, Sisay's Legacy — {G}{W} 0/0 legendary Human Warrior. Can't be the
/// target of abilities your opponents control; +1/+1 for each creature you
/// control.
pub fn shanna_sisays_legacy() -> CardDefinition {
    let n = || Value::count(Selector::EachPermanent(yours(R::Creature)));
    CardDefinition {
        keywords: vec![Keyword::HexproofFromAbilities],
        static_abilities: vec![static_ab(
            "Shanna gets +1/+1 for each creature you control.",
            StaticEffect::PumpPTByValue { applies_to: Selector::This, power: n(), toughness: n() },
        )],
        ..legend("Shanna, Sisay's Legacy", cost(&[g(), w()]), vec![CreatureType::Human, CreatureType::Warrior], 0, 0)
    }
}

/// Brigid, Clachan's Heart // Brigid, Doun's Mind — {2}{W} 3/2 legendary
/// Kithkin. Entering or transforming into this face: a 1/1 green and white
/// Kithkin. First main phase: you may pay {G} to transform. Back: {T}: add X
/// {G} or X {W}, X = your other creatures; first main phase: you may pay {W}
/// to transform.
pub fn brigid_clachans_heart() -> CardDefinition {
    let main = |pay: ManaCost| TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
        effect: Effect::MayPay {
            description: "Pay to transform Brigid?".into(),
            mana_cost: pay,
            body: Box::new(Effect::Transform { what: Selector::This }),
            else_: None,
        },
    };
    let others = || Value::count(Selector::EachPermanent(yours(R::Creature.and(R::OtherThanSource))));
    let x_of = |c: Color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(c, others()) },
        ..Default::default()
    };
    let minds = CardDefinition {
        activated_abilities: vec![x_of(Color::Green), x_of(Color::White)],
        triggered_abilities: vec![main(cost(&[w()]))],
        ..legend("Brigid, Doun's Mind", ManaCost::default(), vec![CreatureType::Kithkin, CreatureType::Soldier], 3, 2)
    };
    let kithkin = || make(Value::ONE, token("Kithkin", 1, 1, vec![Color::Green, Color::White], vec![CreatureType::Kithkin]));
    CardDefinition {
        triggered_abilities: vec![
            etb(kithkin()),
            TriggeredAbility { event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource), effect: kithkin() },
            main(cost(&[g()])),
        ],
        back_face: Some(Box::new(minds)),
        ..legend("Brigid, Clachan's Heart", cost(&[generic(2), w()]), vec![CreatureType::Kithkin, CreatureType::Warrior], 3, 2)
    }
}

// ── Batch E: the last card of a nearly complete deck ─────────────────────────

/// The Grey Havens — legendary land. ETB: scry 1. {T}: {C}. {T}: one mana of
/// any color among legendary creature cards in your graveyard.
pub fn the_grey_havens() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
        ..land(
            "The Grey Havens",
            vec![
                tap_add_colorless(),
                ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyColorAmongLegendaryCreatureCardsInYourGraveyard,
                    },
                    ..Default::default()
                },
            ],
        )
    }
}

/// Mighty Servant of Leuk-o — {3} 6/6 artifact Vehicle, trample, ward—discard
/// a card, crew 4. Crewed for the first time each turn by exactly two
/// creatures: until end of turn it has "combat damage to a player draws two".
pub fn mighty_servant_of_leuk_o() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample, Keyword::Ward(crate::card::WardCost::Discard(1)), Keyword::Crew(4)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CrewsOrSaddles { main_phase_only: false }, EventScope::YourControl)
                // This crew is exactly two, and nothing crewed it earlier this
                // turn (its crewers this turn are just these two).
                .with_filter(Predicate::All(vec![
                    trigger_is(R::IsSource),
                    Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(2)),
                    Predicate::ValueAtMost(Value::TriggerEventAmount, Value::Const(2)),
                    Predicate::ValueAtMost(Value::SourceCrewerCount, Value::Const(2)),
                ]))
                .once_per_turn(),
            effect: Effect::GrantTriggeredAbility {
                what: Selector::This,
                trigger: Box::new(on_combat_hit(draw(2))),
                duration: Duration::EndOfTurn,
            },
        }],
        ..permanent("Mighty Servant of Leuk-o", cost(&[generic(3)]), CardType::Artifact)
    }
}

/// Stigma Lasher — {R}{R} 2/2 Elemental Shaman, wither. A player it damages
/// can't gain life for the rest of the game.
pub fn stigma_lasher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Wither],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LifeGainLockGame { who: Selector::Player(PlayerRef::TriggerEventPlayer) },
        }],
        ..creature("Stigma Lasher", cost(&[r(), r()]), vec![CreatureType::Elemental, CreatureType::Shaman], 2, 2)
    }
}

/// Phyrexian Purge — {2}{B}{R} sorcery: destroy any number of target
/// creatures, 3 life for each target.
pub fn phyrexian_purge() -> CardDefinition {
    spell(
        "Phyrexian Purge",
        cost(&[generic(2), b(), r()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 20,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::Destroy { what: Selector::Target(0) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
            ])),
        },
    )
}

/// Doors of Durin — {3}{R}{G} legendary artifact. Whenever you attack: scry 2,
/// then you may reveal your top card; a creature card enters tapped and attacking,
/// with trample if you control a Dwarf and hexproof if you control an Elf
/// until your next turn.
pub fn doors_of_durin() -> CardDefinition {
    let rider = |kind: CreatureType, keyword: Keyword| Effect::If {
        cond: Predicate::SelectorExists(Selector::EachPermanent(yours(R::HasCreatureType(kind)))),
        then: Box::new(Effect::GrantKeyword { what: Selector::LastMoved, keyword, duration: Duration::UntilNextTurn }),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                Effect::MayDo {
                    description: "Reveal the top card of your library?".into(),
                    body: Box::new(Effect::If {
                        cond: Predicate::EntityMatches { what: top_card(), filter: R::Creature },
                        then: Box::new(Effect::Seq(vec![
                            Effect::Move {
                                what: top_card(),
                                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                            },
                            Effect::JoinCombatAttacking { what: Selector::LastMoved },
                            rider(CreatureType::Dwarf, Keyword::Trample),
                            rider(CreatureType::Elf, Keyword::Hexproof),
                        ])),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
        }],
        ..permanent("Doors of Durin", cost(&[generic(3), r(), g()]), CardType::Artifact)
    }
}

// ── Batch F: one-card gaps of ranks 1001-1500 the rescan surfaced ────────────

/// Taunting Kobold — {R} 0/1 Kobold, haste. Attacking: goad target creature
/// an opponent controls.
pub fn taunting_kobold() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Goad { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
        }],
        ..creature("Taunting Kobold", cost(&[r()]), vec![CreatureType::Kobold], 0, 1)
    }
}

/// Battlefield Improvisation — {1}{W} instant. Target creature gets +2/+2;
/// if it's attacking, attach any number of your Equipment to it.
pub fn battlefield_improvisation() -> CardDefinition {
    spell(
        "Battlefield Improvisation",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            eot_pump(target_filtered(R::Creature), 2, 2),
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::IsAttacking },
                then: Box::new(Effect::ForEach {
                    selector: Selector::EachPermanent(yours(R::HasArtifactSubtype(ArtifactSubtype::Equipment))),
                    body: Box::new(Effect::Attach { what: Selector::TriggerSource, to: Selector::Target(0) }),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Clever Conjurer — {2}{U} 2/3 Gnome Wizard. Mage Hand — {T}: untap target
/// permanent not named Clever Conjurer (sorcery speed).
pub fn clever_conjurer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Untap {
                what: target_filtered(R::Permanent.and(R::Not(Box::new(R::HasName("Clever Conjurer".into()))))),
                up_to: None,
            },
            ..Default::default()
        }],
        ..creature("Clever Conjurer", cost(&[generic(2), u()]), vec![CreatureType::Gnome, CreatureType::Wizard], 2, 3)
    }
}

/// Brigid, Hero of Kinsbaile — {2}{W}{W} 2/3 legendary Kithkin Archer, first
/// strike. {T}: 2 damage to each attacking or blocking creature target player
/// controls.
pub fn brigid_hero_of_kinsbaile() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ForEach {
                selector: Selector::ControlledBy {
                    who: PlayerRef::Target(0),
                    filter: R::Creature.and(R::IsAttacking.or(R::IsBlocking)),
                },
                body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(2) }),
            },
            ..Default::default()
        }],
        ..legend("Brigid, Hero of Kinsbaile", cost(&[generic(2), w(), w()]), vec![CreatureType::Kithkin, CreatureType::Archer], 2, 3)
    }
}

/// Runadi, Behemoth Caller — {2}{G} 1/3 legendary Cat Shaman. Your creature
/// spells of mana value 5+ enter with (mana value − 4) more +1/+1 counters;
/// your creatures with three or more +1/+1 counters have haste. {T}: {G}.
pub fn runadi_behemoth_caller() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "Whenever you cast a creature spell with mana value 5 or greater, that creature enters with X additional +1/+1 counters on it, where X is its mana value minus 4.",
                StaticEffect::ExtraEtbCountersForCreatureCasts {
                    kind: CounterType::PlusOnePlusOne,
                    value: Value::NonNeg(Box::new(Value::Diff(
                        Box::new(Value::ManaValueOf(Box::new(Selector::This))),
                        Box::new(Value::Const(4)),
                    ))),
                },
            ),
            static_ab(
                "Creatures you control with three or more +1/+1 counters on them have haste.",
                StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(yours(
                        R::Creature.and(R::WithCounterAtLeast(CounterType::PlusOnePlusOne, 3)),
                    )),
                    keyword: Keyword::Haste,
                },
            ),
        ],
        activated_abilities: vec![tap_add(Color::Green)],
        ..legend("Runadi, Behemoth Caller", cost(&[generic(2), g()]), vec![CreatureType::Cat, CreatureType::Shaman], 1, 3)
    }
}

/// Demonlord Belzenlok — {4}{B}{B} 6/6 legendary Elder Demon, flying,
/// trample. ETB: dig to a nonland card and take it, again while it costs 4
/// or more; 1 damage to you per card taken.
pub fn demonlord_belzenlok() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::ExileUntilNonlandToHandRepeat { again_at_least: 4, damage_each: 1 })],
        ..legend("Demonlord Belzenlok", cost(&[generic(4), b(), b()]), vec![CreatureType::Elder, CreatureType::Demon], 6, 6)
    }
}

/// The Immortal Sun — {6} legendary artifact. No loyalty abilities; you draw
/// an additional card in your draw step; your spells cost {1} less; your
/// creatures get +1/+1.
pub fn the_immortal_sun() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            static_ab("Players can't activate planeswalkers' loyalty abilities.", StaticEffect::NoLoyaltyAbilities),
            static_ab("Spells you cast cost {1} less to cast.", StaticEffect::CostReduction { filter: R::Any, amount: 1 }),
            static_ab(
                "Creatures you control get +1/+1.",
                StaticEffect::PumpPT { applies_to: Selector::EachPermanent(yours(R::Creature)), power: 1, toughness: 1 },
            ),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::YourControl),
            effect: draw(1),
        }],
        ..permanent("The Immortal Sun", cost(&[generic(6)]), CardType::Artifact)
    }
}

/// Long Road Home — {1}{W} instant. Exile target creature; return it at the
/// next end step under its owner's control with a +1/+1 counter.
pub fn long_road_home() -> CardDefinition {
    spell(
        "Long Road Home",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::ExileReturnNextEndStep { what: target_filtered(R::Creature) },
    )
}

/// Vraska, the Cutting Glare — {B}{B}{G} 4/4 legendary Gorgon Assassin,
/// deathtouch. ETB with six or more lands: destroy target permanent an
/// opponent controls; its controller creates a Treasure.
pub fn vraska_the_cutting_glare() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(
                Predicate::ValueAtLeast(Value::count(Selector::EachPermanent(yours(R::Land))), Value::Const(6)),
            ),
            effect: Effect::Seq(vec![
                // Their Treasure first, while the target still names its controller.
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(target_filtered(R::Permanent.and(R::ControlledByOpponent)))),
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
                Effect::Destroy { what: Selector::Target(0) },
            ]),
        }],
        ..legend("Vraska, the Cutting Glare", cost(&[b(), b(), g()]), vec![CreatureType::Gorgon, CreatureType::Assassin], 4, 4)
    }
}

// ── Batch G ──────────────────────────────────────────────────────────────────

/// Fabrication Module — {3} artifact. Whenever you get one or more {E}, a
/// +1/+1 counter on target creature you control. {4}, {T}: you get {E}.
pub fn fabrication_module() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EnergyGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(yours(R::Creature)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::AddEnergy(Value::ONE),
            ..Default::default()
        }],
        ..permanent("Fabrication Module", cost(&[generic(3)]), CardType::Artifact)
    }
}

/// Primal Prayers — {2}{G}{G} enchantment. ETB: {E}{E}. You may cast creature
/// spells with mana value 3 or less by paying {E} rather than their mana
/// costs (the "as though it had flash" rider isn't modeled).
pub fn primal_prayers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(2)))],
        static_abilities: vec![static_ab(
            "You may cast creature spells with mana value 3 or less by paying {E} rather than paying their mana costs.",
            StaticEffect::EnergyAlternativeCostForFilter { filter: R::Creature.and(R::ManaValueAtMost(3)), energy: 1 },
        )],
        ..permanent("Primal Prayers", cost(&[generic(2), g(), g()]), CardType::Enchantment)
    }
}

/// Sedris, the Traitor King — {3}{U}{B}{R} 5/5 legendary Zombie Warrior. Each
/// creature card in your graveyard has unearth {2}{B}.
pub fn sedris_the_traitor_king() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Each creature card in your graveyard has unearth {2}{B}.",
            StaticEffect::GraveyardCardsHaveUnearth { filter: R::Creature, cost: cost(&[generic(2), b()]) },
        )],
        ..legend(
            "Sedris, the Traitor King",
            cost(&[generic(3), u(), b(), r()]),
            vec![CreatureType::Zombie, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Obeka, Brute Chronologist — {1}{U}{B}{R} 3/4 legendary Ogre Wizard. {T}:
/// the player whose turn it is may end the turn.
pub fn obeka_brute_chronologist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::MayDoBy {
                who: PlayerRef::ActivePlayer,
                description: "End the turn?".into(),
                body: Box::new(Effect::EndTheTurn),
            },
            ..Default::default()
        }],
        ..legend(
            "Obeka, Brute Chronologist",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Ogre, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Mazzy, Truesword Paladin — {1}{R}{G}{W} 3/4 legendary Halfling Knight. An
/// enchanted creature of yours attacking gets +2/+0 and trample. An Aura of
/// yours put into your graveyard from the battlefield is exiled; you may cast
/// it until the end of your next turn.
pub fn mazzy_truesword_paladin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(trigger_is(R::IsEnchanted)),
                effect: Effect::Seq(vec![
                    eot_pump(Selector::TriggerSource, 2, 0),
                    grant_eot(Selector::TriggerSource, Keyword::Trample),
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                    .with_filter(trigger_is(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura))),
                effect: Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ]),
            },
        ],
        ..legend(
            "Mazzy, Truesword Paladin",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Halfling, CreatureType::Knight],
            3,
            4,
        )
    }
}

/// Unbridled Growth — {G} Aura, enchant land. Enchanted land has "{T}: add one
/// mana of any color." Sacrifice it: draw a card.
pub fn unbridled_growth() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(yours(R::Land)) },
        equipped_bonus: Some(EquipBonus { activated_abilities: vec![crate::sets::tap_add_any_color()], ..Default::default() }),
        activated_abilities: vec![ActivatedAbility { sac_cost: true, effect: draw(1), ..Default::default() }],
        ..permanent("Unbridled Growth", cost(&[g()]), CardType::Enchantment)
    }
}

/// Coastline Marauders — {2}{R} 0/3 Human Pirate, trample. Attacking: +1/+0
/// for each land defending player controls. Encore {4}{R}{R}.
pub fn coastline_marauders() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByDefendingPlayer))),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![crate::effect::shortcut::encore(cost(&[generic(4), r(), r()]))],
        ..creature("Coastline Marauders", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Pirate], 0, 3)
    }
}

/// Kellan, Planar Trailblazer — {R} 2/1 legendary Human Faerie Scout. {1}{R}:
/// a Scout becomes a Detective with "combat damage to a player exiles your
/// top card; you may play it this turn". {2}{R}: a Detective becomes a 3/2
/// Rogue with double strike.
pub fn kellan_planar_trailblazer() -> CardDefinition {
    let forever = Duration::Permanent;
    let is = |t: CreatureType| Predicate::EntityMatches { what: Selector::This, filter: R::HasCreatureType(t) };
    let types = |last: CreatureType| vec![CreatureType::Human, CreatureType::Faerie, last];
    let impulse = Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::ONE,
        duration: crate::card::MayPlayDuration::EndOfThisTurn,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                effect: Effect::If {
                    cond: is(CreatureType::Scout),
                    then: Box::new(Effect::Seq(vec![
                        Effect::BecomeCreatureType {
                            what: Selector::This,
                            creature_types: types(CreatureType::Detective),
                            duration: forever,
                        },
                        Effect::GrantTriggeredAbility {
                            what: Selector::This,
                            trigger: Box::new(on_combat_hit(impulse)),
                            duration: forever,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                effect: Effect::If {
                    cond: is(CreatureType::Detective),
                    then: Box::new(Effect::Seq(vec![
                        Effect::BecomeCreatureType {
                            what: Selector::This,
                            creature_types: types(CreatureType::Rogue),
                            duration: forever,
                        },
                        Effect::SetBasePT { what: Selector::This, power: Value::Const(3), toughness: Value::Const(2), duration: forever },
                        Effect::GrantKeyword { what: Selector::This, keyword: Keyword::DoubleStrike, duration: forever },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Kellan, Planar Trailblazer",
            cost(&[r()]),
            vec![CreatureType::Human, CreatureType::Faerie, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// View from Above — {1}{U} instant. Target creature gains flying until end
/// of turn; if you control a white permanent, return this to its owner's hand.
pub fn view_from_above() -> CardDefinition {
    spell(
        "View from Above",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            grant_eot(target_filtered(R::Creature), Keyword::Flying),
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(yours(R::Permanent.and(R::HasColor(Color::White))))),
                then: Box::new(Effect::ReturnResolvingSpellToHand),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Consuming Tide — {2}{U}{U} sorcery. Each player keeps one nonland
/// permanent; the rest return to their owners' hands. Then you draw a card
/// for each opponent with more cards in hand than you.
pub fn consuming_tide() -> CardDefinition {
    spell(
        "Consuming Tide",
        cost(&[generic(2), u(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::EachPlayerKeepsOneSacrificeRest {
                who: Selector::Player(PlayerRef::EachPlayer),
                filter: R::Permanent.and(R::Nonland),
                destroy: false,
                bounce: true,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::PlayersWithGreaterTally(crate::card::PlayerTally::CardsInHand),
            },
        ]),
    )
}

// ── Batch H ──────────────────────────────────────────────────────────────────

/// Wheel of Potential — {2}{R} sorcery. You get {E}{E}{E}, then may pay any
/// amount of {E}; each player may exile their hand and draw that many. At
/// seven or more, you may play your exiled cards until the end of your next
/// turn.
pub fn wheel_of_potential() -> CardDefinition {
    spell(
        "Wheel of Potential",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::AddEnergy(Value::Const(3)),
            Effect::PayAnyEnergy {
                then: Box::new(Effect::EachPlayerMayExileHandDraw {
                    amount: Value::EnergyPaidThisEffect,
                    play_if_at_least: 7,
                }),
            },
        ]),
    )
}

/// Gauntlet of Power — {5} artifact. As it enters, choose a color. Creatures
/// of that color get +1/+1; a basic land tapped for that color adds one more.
pub fn gauntlet_of_power() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![
            static_ab(
                "Creatures of the chosen color get +1/+1.",
                StaticEffect::AnthemForFilter {
                    filter: R::Creature.and(R::HasChosenColorOfSource),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    opponents: false,
                    all_players: true,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            ),
            static_ab(
                "Whenever a basic land is tapped for mana of the chosen color, its controller adds an additional one mana of that color.",
                StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: false,
                    filter: R::IsBasicLand,
                    extra: crate::effect::ExtraManaKind::ChosenColorIfProduced,
                    while_monarch: false,
                },
            ),
        ],
        ..permanent("Gauntlet of Power", cost(&[generic(5)]), CardType::Artifact)
    }
}

/// Storm Cauldron — {5} artifact. Each player may play an additional land
/// each turn. Whenever a land is tapped for mana, return it to its owner's
/// hand.
pub fn storm_cauldron() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Each player may play an additional land during each of their turns.",
            StaticEffect::ExtraLandPerTurnEachPlayer,
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::AnyPlayer).with_filter(trigger_is(R::Land)),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
            },
        }],
        ..permanent("Storm Cauldron", cost(&[generic(5)]), CardType::Artifact)
    }
}

/// Sheoldred // The True Scriptures — {3}{B}{B} 4/5 legendary Phyrexian
/// Praetor, menace. ETB: each opponent sacrifices a nontoken creature or
/// planeswalker. {4}{B}, sorcery speed, if an opponent has eight or more cards
/// in their graveyard: return it transformed. Saga: I — for each opponent,
/// destroy up to one of their creatures or planeswalkers; II — each opponent
/// discards three, then mills three; III — every creature card in every
/// graveyard onto the battlefield under your control, then flip back.
pub fn sheoldred() -> CardDefinition {
    let threat = || R::Creature.or(R::Planeswalker);
    let opps = || Selector::Player(PlayerRef::EachOpponent);
    let scriptures = CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::ForEachOpponentTarget {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: threat().and(R::ControlledByOpponent),
                        effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                    }),
                },
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::Discard { who: opps(), amount: Value::Const(3), random: false },
                    Effect::Mill { who: opps(), amount: Value::Const(3) },
                ]),
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::EachMatching {
                            zone: crate::effect::ZoneRef::Graveyard(PlayerRef::EachPlayer),
                            filter: R::Creature,
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::ExileSelfReturnFrontFace,
                ]),
            ),
        ],
        ..permanent("The True Scriptures", ManaCost::default(), CardType::Enchantment)
    };
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: opps(),
            count: Value::ONE,
            filter: threat().and(R::Not(Box::new(R::IsToken))),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            sorcery_speed: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::GreatestGraveyardSizeAmong(PlayerRef::EachOpponent),
                Value::Const(8),
            )),
            effect: Effect::ExileSelfReturnTransformed,
            ..Default::default()
        }],
        back_face: Some(Box::new(scriptures)),
        ..legend("Sheoldred", cost(&[generic(3), b(), b()]), vec![CreatureType::Phyrexian, CreatureType::Praetor], 4, 5)
    }
}

/// Unseal the Necropolis — {2}{B} instant. Each player mills three; you return
/// up to two creature cards from your graveyard to your hand.
pub fn unseal_the_necropolis() -> CardDefinition {
    spell(
        "Unseal the Necropolis",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(3) },
            Effect::MoveChosen {
                from: Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter: R::Creature },
                filter: None,
                count: Value::Const(2),
                up_to: true,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
    )
}

/// Boseiju Reaches Skyward // Branch of Boseiju — {3}{G} Saga. I: up to two
/// basic Forests to hand. II: up to one land card from your graveyard on top
/// of your library. III: return transformed — a 0/0 reach Plant with +1/+1
/// per land you control.
pub fn boseiju_reaches_skyward() -> CardDefinition {
    let lands = || Value::count(Selector::EachPermanent(yours(R::Land)));
    let branch = CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        keywords: vec![Keyword::Reach],
        static_abilities: vec![static_ab(
            "This creature gets +1/+1 for each land you control.",
            StaticEffect::PumpPTByValue { applies_to: Selector::This, power: lands(), toughness: lands() },
        )],
        ..creature("Branch of Boseiju", ManaCost::default(), vec![CreatureType::Plant], 0, 0)
    };
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand.and(R::HasLandType(LandType::Forest)),
                    to: ZoneDest::Hand(PlayerRef::You),
                    count: Value::Const(2),
                },
            ),
            (
                2,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Land.and(R::InYourGraveyard)),
                        to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
                    }),
                },
            ),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(branch)),
        ..permanent("Boseiju Reaches Skyward", cost(&[generic(3), g()]), CardType::Enchantment)
    }
}

/// Waking the Trolls — {4}{R}{G} Saga. I: destroy target land. II: put target
/// land card from a graveyard onto the battlefield under your control. III:
/// for target opponent with fewer lands than you, 4/4 green Troll Warriors
/// with trample equal to the difference.
pub fn waking_the_trolls() -> CardDefinition {
    let troll = TokenDefinition {
        keywords: vec![Keyword::Trample],
        ..token("Troll Warrior", 4, 4, vec![Color::Green], vec![CreatureType::Troll, CreatureType::Warrior])
    };
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::Destroy { what: target_filtered(R::Land) }),
            (
                2,
                Effect::Move {
                    what: target_filtered(R::Land.and(R::InGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
            (
                3,
                make(
                    Value::NonNeg(Box::new(Value::Diff(
                        Box::new(Value::count(Selector::EachPermanent(yours(R::Land)))),
                        Box::new(Value::count(Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Land })),
                    ))),
                    troll,
                ),
            ),
        ],
        ..permanent("Waking the Trolls", cost(&[generic(4), r(), g()]), CardType::Enchantment)
    }
}

/// Crovax, Ascendant Hero — {4}{W}{W} 4/4 legendary Human Noble. Other white
/// creatures get +1/+1; nonwhite creatures get -1/-1. Pay 2 life: return it
/// to its owner's hand.
pub fn crovax_ascendant_hero() -> CardDefinition {
    let anthem = |filter: R, d: i32| StaticEffect::AnthemForFilter {
        filter,
        power: d,
        toughness: d,
        keywords: vec![],
        opponents: false,
        all_players: true,
        only_your_turn: false,
        scale_by_counters_on_self: None,
    };
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "Other white creatures get +1/+1.",
                anthem(R::Creature.and(R::HasColor(Color::White)).and(R::OtherThanSource), 1),
            ),
            static_ab("Nonwhite creatures get -1/-1.", anthem(R::Creature.and(R::HasColor(Color::White).negate()), -1)),
        ],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))) },
            ..Default::default()
        }],
        ..legend("Crovax, Ascendant Hero", cost(&[generic(4), w(), w()]), vec![CreatureType::Human, CreatureType::Noble], 4, 4)
    }
}

/// Grafted Butcher — {1}{B} 2/2 Phyrexian Samurai. ETB: your Phyrexians gain
/// menace until end of turn. Other Phyrexians you control get +1/+1. {3}{B},
/// sacrifice an artifact or creature: return it from your graveyard to the
/// battlefield (sorcery speed).
pub fn grafted_butcher() -> CardDefinition {
    let phyrexians = || yours(R::HasCreatureType(CreatureType::Phyrexian));
    CardDefinition {
        triggered_abilities: vec![etb(grant_eot(Selector::EachPermanent(phyrexians()), Keyword::Menace))],
        static_abilities: vec![static_ab(
            "Other Phyrexians you control get +1/+1.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(phyrexians().and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            from_graveyard: true,
            sorcery_speed: true,
            sac_other_filter: Some((R::Artifact.or(R::Creature), 1)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Grafted Butcher", cost(&[generic(1), b()]), vec![CreatureType::Phyrexian, CreatureType::Samurai], 2, 2)
    }
}

// ── Batch I ──────────────────────────────────────────────────────────────────

/// Nevinyrral, Urborg Tyrant — {3}{W}{U}{B} 3/6 legendary Zombie Wizard,
/// hexproof from artifacts, creatures and enchantments. ETB: a tapped 2/2
/// black Zombie for each creature that died this turn. Dies: you may pay {1}
/// to destroy all artifacts, creatures and enchantments.
pub fn nevinyrral_urborg_tyrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::HexproofFromArtifactsCreaturesEnchantments],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                make(
                    Value::CreaturesDiedThisTurnTotal,
                    token("Zombie", 2, 2, vec![Color::Black], vec![CreatureType::Zombie]),
                ),
                Effect::Tap { what: Selector::LastCreatedTokens },
            ])),
            crate::effect::shortcut::on_dies(Effect::MayPay {
                description: "Pay {1} to destroy all artifacts, creatures, and enchantments?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Destroy {
                    what: Selector::EachPermanent(R::Artifact.or(R::Creature).or(R::Enchantment)),
                }),
                else_: None,
            }),
        ],
        ..legend(
            "Nevinyrral, Urborg Tyrant",
            cost(&[generic(3), w(), u(), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            3,
            6,
        )
    }
}

/// Rise of the Dread Marn — {2}{B} instant. X 2/2 black Zombie Berserkers, X =
/// nontoken creatures that died this turn. Foretell {B}.
pub fn rise_of_the_dread_marn() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[b()])),
        ..spell(
            "Rise of the Dread Marn",
            cost(&[generic(2), b()]),
            CardType::Instant,
            make(
                Value::CreatureDeathsThisTurnMatching { filter: R::Not(Box::new(R::IsToken)) },
                token("Zombie Berserker", 2, 2, vec![Color::Black], vec![CreatureType::Zombie, CreatureType::Berserker]),
            ),
        )
    }
}

/// Bane, Lord of Darkness — {1}{W}{U}{B} 5/2 legendary God. Indestructible
/// while your life is at most half your starting life. Another nontoken
/// creature of yours dying: target opponent may have you draw a card; if they
/// don't, you may put a creature card with equal or lesser toughness from
/// your hand onto the battlefield.
pub fn bane_lord_of_darkness() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "As long as your life total is less than or equal to half your starting life total, Bane has indestructible.",
            StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Indestructible,
                condition: Predicate::ValueAtMost(
                    Value::Times(Box::new(Value::Const(2)), Box::new(Value::LifeOf(PlayerRef::You))),
                    Value::StartingLifeTotal,
                ),
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(trigger_is(R::Not(Box::new(R::IsToken)).and(R::OtherThanSource))),
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::AnyPlayerMayAccept {
                    who: PlayerRef::Target(0),
                    prompt: "Have Bane's controller draw a card (else they may put a creature onto the battlefield)?"
                        .into(),
                    accepted: Box::new(draw(1)),
                    otherwise: Box::new(Effect::WithX {
                        x: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                        body: Box::new(Effect::MoveChosen {
                            from: Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Hand, filter: R::Creature },
                            filter: Some(R::ToughnessAtMostXFromCost),
                            count: Value::ONE,
                            up_to: true,
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                    }),
                }),
            },
        }],
        ..legend("Bane, Lord of Darkness", cost(&[generic(1), w(), u(), b()]), vec![CreatureType::God], 5, 2)
    }
}

/// Body Launderer — {2}{B}{B} 3/3 Ogre Rogue, deathtouch. Another nontoken
/// creature of yours dying: it connives. Dies: return another target
/// non-Rogue creature card with lesser or equal power from your graveyard to
/// the battlefield.
pub fn body_launderer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(trigger_is(R::Not(Box::new(R::IsToken)).and(R::OtherThanSource))),
                effect: Effect::Connive { what: Selector::This, amount: Value::ONE },
            },
            crate::effect::shortcut::on_dies(Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(R::InYourGraveyard)
                        .and(R::OtherThanSource)
                        .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Rogue))))
                        .and(R::PowerAtMostSourcePower),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        ],
        ..creature("Body Launderer", cost(&[generic(2), b(), b()]), vec![CreatureType::Ogre, CreatureType::Rogue], 3, 3)
    }
}

/// Argoth, Sanctum of Nature — land: tapped unless you control a legendary
/// green creature; {T}: {G}; {2}{G}{G},{T}: a 2/2 Bear, then mill three
/// (sorcery speed). Melds with Titania, Voice of Gaea.
pub fn argoth_sanctum_of_nature() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "This land enters tapped unless you control a legendary green creature.",
            StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::EachPermanent(yours(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::HasColor(Color::Green)),
                ))),
            },
        )],
        ..land(
            "Argoth, Sanctum of Nature",
            vec![
                tap_add(Color::Green),
                ActivatedAbility {
                    mana_cost: cost(&[generic(2), g(), g()]),
                    tap_cost: true,
                    sorcery_speed: true,
                    effect: Effect::Seq(vec![
                        make(Value::ONE, token("Bear", 2, 2, vec![Color::Green], vec![CreatureType::Bear])),
                        Effect::Mill { who: Selector::You, amount: Value::Const(3) },
                    ]),
                    ..Default::default()
                },
            ],
        )
    }
}

/// Titania, Voice of Gaea — {1}{G}{G} 3/4 reach. One or more land cards
/// into your graveyard: gain 2. Upkeep, with four or more land cards in
/// your graveyard and Argoth: meld into Titania, Gaea Incarnate (CR 701.42).
pub fn titania_voice_of_gaea() -> CardDefinition {
    let four_lands = Predicate::ValueAtLeast(
        Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Land },
        Value::Const(4),
    );
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPutIntoGraveyard, EventScope::YourControl).once_per_batch(),
                effect: gain(2),
            },
            TriggeredAbility {
                // CR 603.4 — intervening "if": checked on trigger and again
                // on resolution; the meld itself re-checks own-and-control.
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                    .with_filter(four_lands.clone()),
                effect: Effect::If {
                    cond: four_lands,
                    then: Box::new(Effect::Meld {
                        partner: "Argoth, Sanctum of Nature".into(),
                        into: "Titania, Gaea Incarnate".into(),
                        attacking: false,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..legend("Titania, Voice of Gaea", cost(&[generic(1), g(), g()]), vec![CreatureType::Elemental], 3, 4)
    }
}

/// Titania, Gaea Incarnate — the meld of Titania and Argoth: vigilance,
/// reach, trample, haste, */* = your lands. ETB: every land card in your
/// graveyard returns tapped. {3}{G}: four +1/+1 counters on target land of
/// yours; it becomes a 0/0 Elemental creature with haste, still a land.
pub fn titania_gaea_incarnate() -> CardDefinition {
    let lands = || Value::CountMatching { sel: Box::new(Selector::EachPermanent(R::ControlledByYou)), filter: R::Land };
    CardDefinition {
        // CR 202.1b — the melded face prints no mana cost.
        no_mana_cost: true,
        color_indicator: vec![Color::Green],
        keywords: vec![Keyword::Vigilance, Keyword::Reach, Keyword::Trample, Keyword::Haste],
        static_abilities: vec![static_ab(
            "Titania's power and toughness are each equal to the number of lands you control.",
            StaticEffect::SelfBasePtFromValue { power: lands(), toughness: lands() },
        )],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::EachMatching { zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You), filter: R::Land },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(4),
                },
                Effect::BecomeCreature {
                    what: Selector::Target(0),
                    power: Value::ZERO,
                    toughness: Value::ZERO,
                    creature_types: vec![CreatureType::Elemental],
                    keywords: vec![Keyword::Haste],
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Titania, Gaea Incarnate",
            ManaCost::default(),
            vec![CreatureType::Elemental, CreatureType::Avatar],
            0,
            0,
        )
    }
}

/// "Whenever you discard one or more artifact cards, …" — once each turn
/// (CR 603.2c batch, CR 603.3d-style cap): Mishra, Excavation Prodigy and
/// Urza, Powerstone Prodigy.
fn on_artifact_discard_once_a_turn(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec { once_per_turn: true, ..EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl) }
            .with_filter(trigger_is(R::Artifact)),
        effect,
    }
}

/// Mishra, Excavation Prodigy — {2}{R} 2/1 haste; {1},{T}, discard a card:
/// draw; discarding artifact cards adds {R}{R} once each turn.
pub fn mishra_excavation_prodigy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: draw(1),
            ..Default::default()
        }],
        triggered_abilities: vec![on_artifact_discard_once_a_turn(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(Color::Red, Value::Const(2)),
        })],
        ..legend(
            "Mishra, Excavation Prodigy",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            1,
        )
    }
}

/// Urza, Powerstone Prodigy — {2}{U} 1/3 vigilance; {1},{T}: loot;
/// discarding artifact cards makes a tapped Powerstone once each turn.
pub fn urza_powerstone_prodigy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                draw(1),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![on_artifact_discard_once_a_turn(make(
            Value::ONE,
            TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() },
        ))],
        ..legend(
            "Urza, Powerstone Prodigy",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            3,
        )
    }
}

/// Gruesome Menagerie — {3}{B}{B} sorcery. Choose a creature card with mana
/// value 1 in your graveyard, then 2 and 3; return those cards together.
pub fn gruesome_menagerie() -> CardDefinition {
    CardDefinition {
        name: "Gruesome Menagerie",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChoosePerManaValue {
            what: Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter: R::Creature },
            values: vec![1, 2, 3],
            up_to: false,
            chosen: Box::new(Effect::Move {
                what: Selector::SeparatedPile { chosen: true },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            other: Box::new(Effect::Noop),
        },
        ..Default::default()
    }
}

/// Queen Kayla bin-Kroog — {1}{R}{W} 2/3. {4},{T} (sorcery): discard your
/// hand, draw that many, then return up to one discarded artifact or
/// creature card of each mana value 1, 2 and 3, together.
pub fn queen_kayla_bin_kroog() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::DiscardHandDrawThatMany { who: Selector::You },
                Effect::ChoosePerManaValue {
                    what: Selector::DiscardedThisResolution { filter: R::Artifact.or(R::Creature) },
                    values: vec![1, 2, 3],
                    up_to: true,
                    chosen: Box::new(Effect::Move {
                        what: Selector::SeparatedPile { chosen: true },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    other: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend("Queen Kayla bin-Kroog", cost(&[generic(1), r(), w()]), vec![CreatureType::Human, CreatureType::Noble], 2, 3)
    }
}

/// Nexus of Becoming — {6} artifact. Your beginning of combat: draw, then
/// you may exile an artifact or creature card from your hand for a token
/// copy of it that's also a 3/3 Golem artifact creature.
pub fn nexus_of_becoming() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::ChooseSomeAmong {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Hand,
                        filter: R::Artifact.or(R::Creature),
                    },
                    chooser: PlayerRef::You,
                    count: Value::ONE,
                    up_to: true,
                    chosen: Box::new(Effect::Seq(vec![
                        Effect::Move { what: Selector::SeparatedPile { chosen: true }, to: ZoneDest::Exile },
                        Effect::CreateTokenCopyOf {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            source: Selector::ExiledThisResolution { filter: R::Any },
                            extra_creature_types: vec![CreatureType::Golem],
                            extra_card_types: vec![CardType::Artifact, CardType::Creature],
                            override_pt: Some((3, 3)),
                            override_colors: None,
                            enters_tapped: false,
                            non_legendary: false,
                            legendary: false,
                            extra_keywords: vec![],
                            no_mana_cost: false,
                            enters_with_counters: None,
                            remove_keywords: vec![],
                        },
                    ])),
                    other: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..permanent("Nexus of Becoming", cost(&[generic(6)]), CardType::Artifact)
    }
}

/// Drillworks Mole — {1} 1/1 artifact Mole. {2},{T}: a +1/+1 counter here
/// and on up to one target commander creature you control.
pub fn drillworks_mole() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature.and(R::IsCommander).and(R::ControlledByYou),
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Drillworks Mole", cost(&[generic(1)]), vec![CreatureType::Mole], 1, 1)
    }
}

/// Farmstead Gleaner — {3} 2/2 artifact Scarecrow; doesn't untap in your
/// untap step; {2},{Q}: a +1/+1 counter on it.
pub fn farmstead_gleaner() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![static_ab(
            "This creature doesn't untap during your untap step.",
            StaticEffect::PreventUntap { applies_to: Selector::This },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            untap_self_cost: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Farmstead Gleaner", cost(&[generic(3)]), vec![CreatureType::Scarecrow], 2, 2)
    }
}

/// Jackal, Genius Geneticist — {G}{U} 1/1 trample. Casting a creature
/// spell with mana value equal to Jackal's power: copy it, not legendary
/// (CR 707.9b), then a +1/+1 counter on Jackal.
pub fn jackal_genius_geneticist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                trigger_is(R::Creature),
                Predicate::ValueEquals(
                    Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    Value::PowerOf(Box::new(Selector::This)),
                ),
            ])),
            effect: Effect::Seq(vec![
                Effect::CopySpellNotLegendary { what: Selector::TriggerSource },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ]),
        }],
        ..legend(
            "Jackal, Genius Geneticist",
            cost(&[g(), u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Villain],
            1,
            1,
        )
    }
}

/// Errant, Street Artist — {U} 0/3 flash, defender, haste. {1}{U},{T}: copy
/// target spell you control that wasn't cast (a copy — CR 707.10), new
/// targets allowed.
pub fn errant_street_artist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Defender, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::CopySpellMayChooseTargets {
                what: target_filtered(R::IsSpellOnStack.and(R::IsToken).and(R::ControlledByYou)),
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend("Errant, Street Artist", cost(&[u()]), vec![CreatureType::Human, CreatureType::Rogue], 0, 3)
    }
}

/// Lluwen, Imperfect Naturalist — {B/G}{B/G} 1/3. ETB: mill four, then may
/// put a milled creature or land card on top. {2}{B/G}{B/G}{B/G},{T},
/// discard a land card: a 1/1 Worm per land card in your graveyard.
pub fn lluwen_imperfect_naturalist() -> CardDefinition {
    let bg = || hybrid(Color::Black, Color::Green);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(4) },
            Effect::ChooseSomeAmong {
                what: Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter: R::Creature.or(R::Land) },
                chooser: PlayerRef::You,
                count: Value::ONE,
                up_to: true,
                chosen: Box::new(Effect::Move {
                    what: Selector::SeparatedPile { chosen: true },
                    to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
                }),
                other: Box::new(Effect::Noop),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), bg(), bg(), bg()]),
            tap_cost: true,
            discard_cost: Some((R::Land, 1)),
            effect: make(
                Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Land },
                token("Worm", 1, 1, vec![Color::Black, Color::Green], vec![CreatureType::Worm]),
            ),
            ..Default::default()
        }],
        ..legend("Lluwen, Imperfect Naturalist", cost(&[bg(), bg()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 3)
    }
}

/// Tectonic Split — {4}{G}{G} enchantment, hexproof. Additional cost:
/// sacrifice half your lands, rounded up. Your lands have "{T}: add three
/// mana of any one color."
pub fn tectonic_split() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificeHalfRoundedUp { filter: R::Land }],
        static_abilities: vec![static_ab(
            "Lands you control have \"{T}: Add three mana of any one color.\"",
            StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(yours(R::Land)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(3)) },
                    ..Default::default()
                },
                condition: None,
            },
        )],
        ..permanent("Tectonic Split", cost(&[generic(4), g(), g()]), CardType::Enchantment)
    }
}

/// Inniaz, the Gale Force — {3}{U}{U} 4/4 flier. {2}{W/U}: attacking
/// fliers get +1/+1 until end of turn. Three or more of your fliers
/// attacking: each player gains control of a nonland permanent of your
/// choice from the player to their right.
pub fn inniaz_the_gale_force() -> CardDefinition {
    let attacking_fliers = || R::Creature.and(R::HasKeyword(Keyword::Flying)).and(R::IsAttacking);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), hybrid(Color::White, Color::Blue)]),
            effect: eot_pump(Selector::EachPermanent(attacking_fliers()), 1, 1),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(Predicate::ValueAtLeast(
                Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
                    filter: attacking_fliers(),
                },
                Value::Const(3),
            )),
            effect: Effect::EachPlayerGainsChosenFromRightNeighbor { filter: R::Nonland },
        }],
        ..legend("Inniaz, the Gale Force", cost(&[generic(3), u(), u()]), vec![CreatureType::Djinn], 4, 4)
    }
}

/// Faerie Slumber Party — {4}{U}{U} sorcery. Return all creatures to their
/// owners' hands; for each opponent who controlled one, two 1/1 blue
/// flying Faeries that can block only fliers. (Yours bounce first, then
/// each opponent's in turn order — one batch per seat, not one in all.)
pub fn faerie_slumber_party() -> CardDefinition {
    let faerie = TokenDefinition {
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..token("Faerie", 1, 1, vec![Color::Blue], vec![CreatureType::Faerie])
    };
    // One simultaneous bounce; the opponents who controlled a creature are
    // counted off the board it leaves.
    let all = || Selector::EachPermanent(R::Creature);
    spell(
        "Faerie Slumber Party",
        cost(&[generic(4), u(), u()]),
        CardType::Sorcery,
        Effect::WithX {
            x: Value::OpponentsControllingAnyOf(Box::new(all())),
            body: Box::new(Effect::Seq(vec![
                Effect::Move { what: all(), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                make(Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)), faerie),
            ])),
        },
    )
}

/// Tezzeret, Artifice Master — {3}{U}{U} planeswalker, loyalty 5. +1: a 1/1
/// Thopter artifact creature with flying. 0: draw a card, two with three or
/// more artifacts. −9: an emblem that puts a permanent card from your
/// library onto the battlefield at your end step.
pub fn tezzeret_artifice_master() -> CardDefinition {
    let thopter = TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        ..token("Thopter", 1, 1, vec![], vec![CreatureType::Thopter])
    };
    CardDefinition {
        name: "Tezzeret, Artifice Master",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Tezzeret],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            crate::effect::LoyaltyAbility { loyalty_cost: 1, effect: make(Value::ONE, thopter), ..Default::default() },
            crate::effect::LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::IfPred {
                        pred: Box::new(Predicate::ValueAtLeast(
                            Value::count(Selector::EachPermanent(yours(R::Artifact))),
                            Value::Const(3),
                        )),
                        then: Box::new(Value::Const(2)),
                        else_: Box::new(Value::ONE),
                    },
                },
                ..Default::default()
            },
            crate::effect::LoyaltyAbility {
                loyalty_cost: -9,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Tezzeret, Artifice Master".into(),
                    statics: vec![],
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                        effect: Effect::Search {
                            who: PlayerRef::You,
                            filter: R::Permanent,
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Karfell Harbinger — {1}{U} 1/3 Zombie Wizard. {T}: {U}, spent only to
/// foretell a card from your hand or cast an instant or sorcery spell.
pub fn karfell_harbinger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::OfColor(Color::Blue, Value::ONE)),
                    crate::mana::SpendRestriction::ForetellActionOrInstantSorcery,
                ),
            },
            ..Default::default()
        }],
        ..creature("Karfell Harbinger", cost(&[generic(1), u()]), vec![CreatureType::Zombie, CreatureType::Wizard], 1, 3)
    }
}

/// Poppet Stitcher // Poppet Factory — {2}{U} 2/3. Casting an instant or
/// sorcery makes a 2/2 black decayed Zombie; your upkeep with three or more
/// creature tokens: may transform. Back: your creature tokens lose all
/// abilities and have base power and toughness 3/3; your upkeep: may
/// transform back.
pub fn poppet_stitcher() -> CardDefinition {
    let upkeep_flip = |cond: Option<Predicate>| {
        let event = EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl);
        TriggeredAbility {
            event: match cond {
                Some(c) => event.with_filter(c),
                None => event,
            },
            effect: may("Transform?", Effect::Transform { what: Selector::This }),
        }
    };
    let tokens = || Selector::EachPermanent(yours(R::Creature.and(R::IsToken)));
    let factory = CardDefinition {
        static_abilities: vec![
            static_ab(
                "Creature tokens you control lose all abilities …",
                StaticEffect::MatchingLoseAllAbilities { applies_to: tokens() },
            ),
            static_ab(
                "… and have base power and toughness 3/3.",
                StaticEffect::SetBasePtForFilter { applies_to: tokens(), power: 3, toughness: 3 },
            ),
        ],
        triggered_abilities: vec![upkeep_flip(None)],
        ..permanent("Poppet Factory", ManaCost::default(), CardType::Artifact)
    };
    let zombie = TokenDefinition {
        keywords: vec![Keyword::Decayed],
        ..token("Zombie", 2, 2, vec![Color::Black], vec![CreatureType::Zombie])
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                )),
                effect: make(Value::ONE, zombie),
            },
            upkeep_flip(Some(Predicate::ValueAtLeast(
                Value::count(Selector::EachPermanent(yours(R::Creature.and(R::IsToken)))),
                Value::Const(3),
            ))),
        ],
        back_face: Some(Box::new(factory)),
        ..creature("Poppet Stitcher", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

/// The Sentry, Golden Guardian — {3}{W} 5/5 flying, vigilance,
/// indestructible. ETB: target opponent creates The Void, a legendary 5/5
/// black flying, indestructible Horror Villain that attacks each combat.
pub fn the_sentry_golden_guardian() -> CardDefinition {
    let void = TokenDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Indestructible, Keyword::MustAttack],
        ..token("The Void", 5, 5, vec![Color::Black], vec![CreatureType::Horror, CreatureType::Villain])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Indestructible],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 1,
            filter: R::OpponentPlayer,
            effect: Box::new(Effect::CreateToken { who: PlayerRef::Target(0), count: Value::ONE, definition: Arc::new(void) }),
        })],
        ..legend("The Sentry, Golden Guardian", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Hero], 5, 5)
    }
}

/// Lucy MacLean, Positively Armed — {4}{W} 3/5. Golden Rule: whenever a
/// token enters, you may have target player other than its controller
/// create a token copy of it, then draw if an opponent did — once each turn.
pub fn lucy_maclean_positively_armed() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::may_once_each_turn(
            EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(trigger_is(R::IsToken)),
            "Have another player copy that token?",
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 1,
                filter: R::Player.and(R::Not(Box::new(R::TriggerPlayer))),
                effect: Box::new(Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::Target(0),
                        count: Value::ONE,
                        source: Selector::TriggerSource,
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
                    Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::OpponentPlayer },
                        then: Box::new(draw(1)),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        )],
        ..legend(
            "Lucy MacLean, Positively Armed",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Survivor],
            3,
            5,
        )
    }
}

/// Superior Spider-Man — {2}{U}{B} 4/4. Mind Swap: may enter as a copy of
/// any creature card in a graveyard, except named Superior Spider-Man and a
/// 4/4 Spider Human Hero in addition; when you do, exile that card.
pub fn superior_spider_man() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(crate::card::EntersAsCopy {
            filter: R::Creature,
            from_graveyards: true,
            keep_name: true,
            extra_creature_types: vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            override_pt: Some((4, 4)),
            exile_copied: true,
            ..Default::default()
        }),
        ..legend(
            "Superior Spider-Man",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Taskmaster, Mercenary Mimic — {2}{U}{B} 3/5. At the beginning of your
/// first main phase, until your next turn he becomes a copy of up to one
/// target creature or creature card in a graveyard, except named Taskmaster
/// and a legendary Human Mercenary Villain creature.
pub fn taskmaster_mercenary_mimic() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::OnBattlefield.or(R::InGraveyard)).and(R::OtherThanSource),
                effect: Box::new(Effect::Seq(vec![
                    Effect::BecomeCopyOfFor {
                        what: Selector::This,
                        source: Selector::Target(0),
                        duration: Duration::UntilNextTurn,
                        non_legendary: false,
                    },
                    Effect::AmendCopy {
                        what: Selector::This,
                        keep_activated: vec![],
                        keep_triggered: vec![],
                        pt: None,
                        keep_name: true,
                        legendary: true,
                        keywords: vec![],
                        card_types: vec![],
                        creature_types: vec![],
                    },
                    Effect::SetCopyTypeLine {
                        what: Selector::This,
                        creature_types: vec![CreatureType::Human, CreatureType::Mercenary, CreatureType::Villain],
                    },
                ])),
            },
        }],
        ..legend(
            "Taskmaster, Mercenary Mimic",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary, CreatureType::Villain],
            3,
            5,
        )
    }
}

/// Huatli, Poet of Unity // Roar of the Fifth People — {2}{G} 2/3. ETB: a
/// basic land card to hand. {3}{R/W}{R/W} (sorcery): exile her, return
/// transformed. Saga: I — two 3/3 Dinosaurs; II — the Saga gains "your
/// creatures have '{T}: add {R}, {G} or {W}'"; III — a Dinosaur card to
/// hand; IV — your Dinosaurs gain double strike and trample this turn.
pub fn huatli_poet_of_unity() -> CardDefinition {
    let dinos = || Selector::EachPermanent(yours(R::Creature.and(R::HasCreatureType(CreatureType::Dinosaur))));
    let roar = CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        static_abilities: vec![static_ab(
            "Chapter II: creatures you control have \"{T}: Add {R}, {G}, or {W}.\"",
            StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(yours(R::Creature)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::OfColors(vec![Color::Red, Color::Green, Color::White], Value::ONE),
                    },
                    ..Default::default()
                },
                condition: Some(Predicate::SourceHasCountersAtLeast { counter: CounterType::Lore, n: 2 }),
            },
        )],
        saga_chapters: vec![
            (1, make(Value::Const(2), token("Dinosaur", 3, 3, vec![Color::Green], vec![CreatureType::Dinosaur]))),
            (2, Effect::Noop),
            (
                3,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Dinosaur),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
            (
                4,
                Effect::Seq(vec![grant_eot(dinos(), Keyword::DoubleStrike), grant_eot(dinos(), Keyword::Trample)]),
            ),
        ],
        ..permanent("Roar of the Fifth People", ManaCost::default(), CardType::Enchantment)
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White)]),
            sorcery_speed: true,
            effect: Effect::ExileSelfReturnTransformed,
            ..Default::default()
        }],
        back_face: Some(Box::new(roar)),
        ..legend(
            "Huatli, Poet of Unity",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Bard],
            2,
            3,
        )
    }
}

/// Intrepid Paleontologist — {1}{G} 2/2. {T}: any color. {2}: exile target
/// card from a graveyard. You may cast Dinosaur creature spells from among
/// cards you own exiled with it; each enters with a finality counter.
pub fn intrepid_paleontologist() -> CardDefinition {
    let mine = || R::Creature.and(R::HasCreatureType(CreatureType::Dinosaur)).and(R::OwnedByYou);
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Seq(vec![
                    Effect::Move { what: target_filtered(R::InGraveyard), to: ZoneDest::Exile },
                    Effect::GrantMayPlay {
                        what: Selector::ExiledThisResolution { filter: mine() },
                        duration: crate::card::MayPlayDuration::WhileSourceOnBattlefield {
                            source: crate::card::CardId(0),
                            one_spell_a_turn: false,
                            follows_control: false,
                        },
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                    Effect::SpellEntersWithCounters {
                        what: Selector::ExiledThisResolution { filter: mine() },
                        kind: CounterType::Finality,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Intrepid Paleontologist", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Druid], 2, 2)
    }
}

/// Nashi, Moon's Legacy — {B}{G}{U} 3/4 menace, ward {1}. Attacking: exile
/// up to one target legendary or Rat card from your graveyard and copy it;
/// you may cast the copy, paying its costs (CR 707.12).
pub fn nashi_moons_legacy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::HasSupertype(Supertype::Legendary)
                    .or(R::HasCreatureType(CreatureType::Rat))
                    .and(R::InYourGraveyard)
                    .and(R::Not(Box::new(R::Land))),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                    Effect::CastWithoutPayingImmediate {
                        what: Selector::LastMoved,
                        source_zone: crate::card::Zone::Exile,
                        exile_after: false,
                        copy: true,
                        reduce_generic: 0,
                        pay_own_cost: true,
                    },
                ])),
            },
        }],
        ..legend("Nashi, Moon's Legacy", cost(&[b(), g(), u()]), vec![CreatureType::Rat, CreatureType::Shaman], 3, 4)
    }
}

/// Elesh Norn // The Argent Etchings — {2}{W}{W} 3/5 vigilance. Whenever a
/// source an opponent controls deals damage to you or a permanent you
/// control, that source's controller loses 2 life unless they pay {1}.
/// {2}{W}, sacrifice three other creatures (sorcery): exile her, return
/// transformed. Saga: I — incubate 2 five times, then transform your
/// Incubators; II — your creatures +1/+1 and double strike this turn;
/// III — destroy all other permanents but artifacts, lands and Phyrexians,
/// then exile the Saga and return it front face up.
pub fn elesh_norn() -> CardDefinition {
    let toll = |who: PlayerRef| Effect::UnlessPlayerPays {
        who: who.clone(),
        cost: crate::card::WardCost::Mana(cost(&[generic(1)])),
        then: Box::new(Effect::LoseLife { who: Selector::Player(who), amount: Value::Const(2) }),
        if_paid: None,
    };
    let yours_all = || Selector::EachPermanent(yours(R::Creature));
    let etchings = CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(
                    (0..5)
                        .map(|_| Effect::Incubate { who: PlayerRef::You, amount: Value::Const(2) })
                        .chain(std::iter::once(Effect::Transform {
                            what: Selector::EachPermanent(yours(
                                R::IsToken.and(R::HasName("Incubator".into())).and(R::Not(Box::new(R::Creature))),
                            )),
                        }))
                        .collect(),
                ),
            ),
            (2, Effect::Seq(vec![eot_pump(yours_all(), 1, 1), grant_eot(yours_all(), Keyword::DoubleStrike)])),
            (
                3,
                Effect::Seq(vec![
                    Effect::Destroy {
                        what: Selector::EachPermanent(
                            R::Not(Box::new(R::Artifact))
                                .and(R::Not(Box::new(R::Land)))
                                .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Phyrexian))))
                                .and(R::OtherThanSource),
                        ),
                    },
                    Effect::ExileSelfReturnFrontFace,
                ]),
            ),
        ],
        ..permanent("The Argent Etchings", ManaCost::default(), CardType::Enchantment)
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerDamaged, EventScope::OpponentSourceDamagedYou),
                effect: toll(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
            },
            TriggeredAbility {
                // Recipient-keyed: the dealer's controller is the trigger
                // player (`PlayerRef::TriggerEventPlayer`).
                event: EventSpec::new(EventKind::DealtDamage, EventScope::OpponentSourceDamagedYourPermanent),
                effect: toll(PlayerRef::TriggerEventPlayer),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            sorcery_speed: true,
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource).and(R::ControlledByYou), 3)),
            effect: Effect::ExileSelfReturnTransformed,
            ..Default::default()
        }],
        back_face: Some(Box::new(etchings)),
        ..legend("Elesh Norn", cost(&[generic(2), w(), w()]), vec![CreatureType::Phyrexian, CreatureType::Praetor], 3, 5)
    }
}

/// Sivriss, Nightmare Speaker — {3}{B} 3/3; choose a Background. {T},
/// sacrifice another creature or an artifact: for each opponent, mill a
/// card, then return it to hand unless that player pays 3 life.
pub fn sivriss_nightmare_speaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource).or(R::Artifact).and(R::ControlledByYou), 1)),
            effect: Effect::ForEachOpponent {
                body: Box::new(Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::Mill { who: Selector::You, amount: Value::ONE },
                    Effect::UnlessPlayerPays {
                        who: PlayerRef::Triggerer,
                        cost: crate::card::WardCost::Life(3),
                        then: Box::new(Effect::Move { what: Selector::LastMoved, to: ZoneDest::Hand(PlayerRef::You) }),
                        if_paid: None,
                    },
                ])),
            },
            ..Default::default()
        }],
        ..legend(
            "Sivriss, Nightmare Speaker",
            cost(&[generic(3), b()]),
            vec![CreatureType::Snake, CreatureType::Cleric, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Tameshi, Reality Architect — {2}{U} 2/3. One or more noncreature
/// permanents returned to hand: draw, once each turn. {X}{W}, return a land
/// you control to hand (sorcery): an artifact or enchantment card with mana
/// value X or less from your graveyard to the battlefield.
pub fn tameshi_reality_architect() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec { once_per_turn: true, ..EventSpec::new(EventKind::PermanentReturnedToHand, EventScope::AnyPlayer) }
                .with_filter(trigger_is(R::Not(Box::new(R::Creature)))),
            effect: draw(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x(), w()]),
            sorcery_speed: true,
            return_permanent_cost: Some((R::Land.and(R::ControlledByYou), 1)),
            effect: Effect::Move {
                what: target_filtered(
                    R::Artifact.or(R::HasCardType(CardType::Enchantment)).and(R::InYourGraveyard).and(R::ManaValueAtMostXFromCost),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..legend("Tameshi, Reality Architect", cost(&[generic(2), u()]), vec![CreatureType::Moonfolk, CreatureType::Wizard], 2, 3)
    }
}

/// Starving Revenant — {2}{B}{B} 4/4. ETB: surveil 2, then draw a card and
/// lose 3 life for each card put on top. Descend 8 — drawing a card with
/// eight or more permanent cards in your graveyard drains a target
/// opponent for 1.
pub fn starving_revenant() -> CardDefinition {
    let kept = || Value::SurveilKeptOnTopThisEffect;
    let descend = Predicate::ValueAtLeast(
        Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Permanent },
        Value::Const(8),
    );
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
                Effect::Draw { who: Selector::You, amount: kept() },
                Effect::LoseLife { who: Selector::You, amount: Value::Times(Box::new(kept()), Box::new(Value::Const(3))) },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl).with_filter(descend),
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 1,
                    filter: R::OpponentPlayer,
                    effect: Box::new(Effect::Seq(vec![
                        Effect::LoseLife { who: Selector::Target(0), amount: Value::ONE },
                        gain(1),
                    ])),
                },
            },
        ],
        ..creature("Starving Revenant", cost(&[generic(2), b(), b()]), vec![CreatureType::Spirit, CreatureType::Horror], 4, 4)
    }
}

/// Sokrates, Athenian Teacher — {1}{W}{U} 0/4 defender; hexproof while
/// untapped. {T}: until end of turn, target creature's combat damage to a
/// player is prevented and both its controller and that player draw half
/// that many cards, rounded down.
pub fn sokrates_athenian_teacher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        static_abilities: vec![static_ab(
            "Sokrates has hexproof as long as it's untapped.",
            StaticEffect::SelfHasKeywordWhile { keyword: Keyword::Hexproof, condition: R::Untapped },
        )],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CombatDamageToPlayersBecomesDrawsThisTurn { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..legend(
            "Sokrates, Athenian Teacher",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            0,
            4,
        )
    }
}

/// Clone Shell — {5} 2/2 artifact Shapeshifter. Imprint: look at the top
/// four, exile one face down, the rest on the bottom. Dies: turn it face
/// up; a creature card enters under your control.
pub fn clone_shell() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![
            etb(Effect::Hideaway { count: Value::Const(4) }),
            crate::effect::shortcut::on_dies(Effect::RevealImprintDeployCreature),
        ],
        ..creature("Clone Shell", cost(&[generic(5)]), vec![CreatureType::Shapeshifter], 2, 2)
    }
}

/// Rowan's Talent — {2}{R}{R} Aura, enchant planeswalker. It has "[+1]: up
/// to one target creature gets +2/+0, first strike and trample this turn";
/// each loyalty ability you activate of it is copied.
pub fn rowans_talent() -> CardDefinition {
    let rally = Effect::ApplyToTargets {
        max_targets: 1,
        min_targets: 0,
        filter: R::Creature,
        effect: Box::new(Effect::Seq(vec![
            eot_pump(Selector::Target(0), 2, 0),
            grant_eot(Selector::Target(0), Keyword::FirstStrike),
            grant_eot(Selector::Target(0), Keyword::Trample),
        ])),
    };
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Planeswalker) },
        static_abilities: vec![
            static_ab(
                "Enchanted planeswalker has \"[+1]: Up to one target creature gets +2/+0 and gains first strike and trample until end of turn.\"",
                StaticEffect::AttachedPlaneswalkerHasLoyaltyAbilities {
                    abilities: vec![crate::effect::LoyaltyAbility { loyalty_cost: 1, effect: rally, ..Default::default() }],
                },
            ),
            static_ab(
                "Whenever you activate a loyalty ability of enchanted planeswalker, copy that ability.",
                StaticEffect::CopyAttachedPlaneswalkerLoyaltyAbilities,
            ),
        ],
        ..permanent("Rowan's Talent", cost(&[generic(2), r(), r()]), CardType::Enchantment)
    }
}

/// Vorinclex // The Grand Evolution — {3}{G}{G} 6/6 trample, reach. ETB: up
/// to two Forest cards to hand. {6}{G}{G} (sorcery): exile it, return
/// transformed. Saga: I — mill ten, up to two milled creature cards enter;
/// II — distribute seven +1/+1 counters among your creatures; III — your
/// creatures gain "{1}: fight target creature you don't control" this turn,
/// then the Saga returns front face up.
pub fn vorinclex() -> CardDefinition {
    let fight = ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        effect: Effect::Fight {
            attacker: Selector::This,
            defender: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
        },
        ..Default::default()
    };
    let evolution = CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::Mill { who: Selector::You, amount: Value::Const(10) },
                    Effect::ChooseSomeAmong {
                        what: Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter: R::Creature },
                        chooser: PlayerRef::You,
                        count: Value::Const(2),
                        up_to: true,
                        chosen: Box::new(Effect::Move {
                            what: Selector::SeparatedPile { chosen: true },
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                        other: Box::new(Effect::Noop),
                    },
                ]),
            ),
            (
                2,
                Effect::DistributeCounters {
                    total: Value::Const(7),
                    counter: CounterType::PlusOnePlusOne,
                    filter: yours(R::Creature),
                    max_targets: 7,
                },
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::GrantActivatedAbilityToMatching {
                        filter: yours(R::Creature),
                        ability: Box::new(fight),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::ExileSelfReturnFrontFace,
                ]),
            ),
        ],
        ..permanent("The Grand Evolution", ManaCost::default(), CardType::Enchantment)
    };
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Reach],
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::HasLandType(LandType::Forest),
            to: ZoneDest::Hand(PlayerRef::You),
            count: Value::Const(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), g(), g()]),
            sorcery_speed: true,
            effect: Effect::ExileSelfReturnTransformed,
            ..Default::default()
        }],
        back_face: Some(Box::new(evolution)),
        ..legend("Vorinclex", cost(&[generic(3), g(), g()]), vec![CreatureType::Phyrexian, CreatureType::Praetor], 6, 6)
    }
}

/// Jacob Hauken, Inspector // Hauken's Insight — {1}{U} 0/2. {T}: draw,
/// exile a card from hand face down (yours to look at); you may pay
/// {4}{U}{U} to transform it. Back: upkeep, exile your top card face down;
/// once each turn play a card exiled with it without paying its mana cost.
pub fn jacob_hauken_inspector() -> CardDefinition {
    let linked = || Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Exile, filter: R::ExiledWithSource };
    let insight = CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Hideaway { count: Value::ONE },
                // Renewed each upkeep and gone at end of turn: "once during
                // each of YOUR turns" (no free play on an opponent's turn).
                Effect::GrantMayPlay {
                    what: linked(),
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: false,
                    any_color: false,
                },
                Effect::OneCastPerTurnAmongGranted { what: linked() },
            ]),
        }],
        ..permanent("Hauken's Insight", ManaCost::default(), CardType::Enchantment)
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                draw(1),
                Effect::ExileChosenFromHand {
                    from: Selector::You,
                    count: Value::ONE,
                    filter: R::Any,
                    link_to_source: true,
                    face_down: true,
                },
                Effect::MayPay {
                    description: "Pay {4}{U}{U} to transform Jacob Hauken?".into(),
                    mana_cost: cost(&[generic(4), u(), u()]),
                    body: Box::new(Effect::Transform { what: Selector::This }),
                    else_: None,
                },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(insight)),
        ..legend("Jacob Hauken, Inspector", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Advisor], 0, 2)
    }
}

/// Havengul Laboratory // Havengul Mystery — legendary land. {T}: {C}.
/// {4},{T}: investigate. Your end step, three or more Clues sacrificed this
/// turn: transform. Back: transforming into it returns a creature card from
/// your graveyard; when that creature leaves, transform back. {T}, pay 1
/// life: {B}.
pub fn havengul_laboratory() -> CardDefinition {
    let flip = || Effect::Transform { what: Selector::This };
    let mystery = CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::Move {
                        what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::RememberPermanentOnSource { what: Selector::LastMoved },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::AnyPlayer)
                    .with_filter(trigger_is(R::ChosenPermanentOfSource)),
                effect: flip(),
            },
        ],
        ..land(
            "Havengul Mystery",
            vec![ActivatedAbility { life_cost: 1, ..tap_add(Color::Black) }],
        )
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(Value::CluesSacrificedThisTurn(PlayerRef::You), Value::Const(3)),
            ),
            effect: flip(),
        }],
        back_face: Some(Box::new(mystery)),
        ..land(
            "Havengul Laboratory",
            vec![
                tap_add_colorless(),
                ActivatedAbility {
                    mana_cost: cost(&[generic(4)]),
                    tap_cost: true,
                    effect: crate::effect::shortcut::investigate(1),
                    ..Default::default()
                },
            ],
        )
    }
}

fn garruk(name: &'static str, mana: ManaCost, loyalty: Vec<crate::effect::LoyaltyAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Garruk], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: loyalty,
        ..Default::default()
    }
}

fn walker_emblem(name: &str, event: EventSpec) -> Effect {
    Effect::CreateEmblem {
        who: PlayerRef::You,
        name: name.into(),
        statics: vec![],
        triggered: vec![TriggeredAbility {
            event,
            effect: may(
                "Search your library for a creature card?",
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
        }],
    }
}

/// Garruk, Caller of Beasts — {4}{G}{G}, loyalty 4. +1: reveal five,
/// creatures to hand, the rest on the bottom. −3: a green creature card from
/// hand onto the battlefield. −7: emblem — casting a creature spell may
/// fetch a creature onto the battlefield.
pub fn garruk_caller_of_beasts() -> CardDefinition {
    use crate::effect::LoyaltyAbility;
    garruk(
        "Garruk, Caller of Beasts",
        cost(&[generic(4), g(), g()]),
        vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::RevealTopTakeMatchingToHand {
                    who: PlayerRef::You,
                    count: Value::Const(5),
                    filter: R::Creature,
                    distinct_powers: false,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::HasColor(Color::Green)),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: walker_emblem(
                    "Garruk, Caller of Beasts",
                    EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(R::Creature)),
                ),
                ..Default::default()
            },
        ],
    )
}

/// Garruk, Unleashed — {2}{G}{G}, loyalty 4. +1: up to one target creature
/// gets +3/+3 and trample. −2: a 3/3 Beast, then a loyalty counter if an
/// opponent controls more creatures. −7: emblem — your end step may fetch a
/// creature onto the battlefield.
pub fn garruk_unleashed() -> CardDefinition {
    use crate::effect::LoyaltyAbility;
    garruk(
        "Garruk, Unleashed",
        cost(&[generic(2), g(), g()]),
        vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Seq(vec![
                        eot_pump(Selector::Target(0), 3, 3),
                        grant_eot(Selector::Target(0), Keyword::Trample),
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    make(Value::ONE, token("Beast", 3, 3, vec![Color::Green], vec![CreatureType::Beast])),
                    Effect::If {
                        cond: Predicate::AnOpponentControlsMoreCreatures,
                        then: Box::new(Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::Loyalty,
                            amount: Value::ONE,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: walker_emblem(
                    "Garruk, Unleashed",
                    EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                ),
                ..Default::default()
            },
        ],
    )
}

/// Copy Artifact — {1}{U} enchantment that may enter as a copy of any
/// artifact, still an enchantment.
pub fn copy_artifact() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(crate::card::EntersAsCopy {
            filter: R::Artifact,
            extra_card_types: vec![CardType::Enchantment],
            ..Default::default()
        }),
        ..permanent("Copy Artifact", cost(&[generic(1), u()]), CardType::Enchantment)
    }
}

/// Lim-Dûl's Hex — {1}{B} enchantment. Your upkeep: 1 damage to each player
/// unless they pay {B} or {3} (offered {B} first, then {3}).
pub fn lim_duls_hex() -> CardDefinition {
    let pay = |c: ManaCost, then: Effect| Effect::UnlessPlayerPays {
        who: PlayerRef::Triggerer,
        cost: crate::card::WardCost::Mana(c),
        then: Box::new(then),
        if_paid: None,
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(pay(
                    cost(&[b()]),
                    pay(
                        cost(&[generic(3)]),
                        Effect::DealDamage { to: Selector::Player(PlayerRef::Triggerer), amount: Value::ONE },
                    ),
                )),
            },
        }],
        ..permanent("Lim-Dûl's Hex", cost(&[generic(1), b()]), CardType::Enchantment)
    }
}

/// Wail of the Forgotten — {U}{B} sorcery. Choose one (descend 8: one or
/// more) — bounce target nonland permanent; target opponent discards; look
/// at the top three, one to hand and the rest to the graveyard.
pub fn wail_of_the_forgotten() -> CardDefinition {
    CardDefinition {
        modes_widen: Some(crate::card::ModesWiden::If(Predicate::ValueAtLeast(
            Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Permanent },
            Value::Const(8),
        ))),
        ..spell(
            "Wail of the Forgotten",
            cost(&[u(), b()]),
            CardType::Sorcery,
            Effect::ChooseModesCast {
                modes: vec![
                    Effect::Move {
                        what: target_filtered(R::Permanent.and(R::Nonland)),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                    Effect::Discard { who: target_filtered(R::OpponentPlayer), amount: Value::ONE, random: false },
                    Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                        who: PlayerRef::You,
                        count: Value::Const(3),
                        rest_to_graveyard: true,
                        take: Some(Value::ONE),
                        ..Default::default()
                    })),
                ],
                min: 1,
                max: 1,
                allow_repeats: false,
            },
        )
    }
}

/// Leonardo da Vinci — {2}{U} 3/3. {3}{U}{U}: your Thopters have base
/// power and toughness X/X this turn, X your hand size. {2}{U},{T}: loot; a
/// discarded artifact card is exiled for a token copy that's a 0/2 flying
/// Thopter artifact creature too.
pub fn leonardo_da_vinci() -> CardDefinition {
    let thopters = || Selector::EachPermanent(yours(R::Creature.and(R::HasCreatureType(CreatureType::Thopter))));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u(), u()]),
                effect: Effect::SetBasePT {
                    what: thopters(),
                    power: Value::HandSizeOf(PlayerRef::You),
                    toughness: Value::HandSizeOf(PlayerRef::You),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    draw(1),
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    Effect::Move {
                        what: Selector::DiscardedThisResolution { filter: R::Artifact },
                        to: ZoneDest::Exile,
                    },
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::ExiledThisResolution { filter: R::Artifact },
                        extra_creature_types: vec![CreatureType::Thopter],
                        extra_card_types: vec![CardType::Artifact, CardType::Creature],
                        override_pt: Some((0, 2)),
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![Keyword::Flying],
                        no_mana_cost: false,
                        enters_with_counters: None,
                        remove_keywords: vec![],
                    },
                ]),
                ..Default::default()
            },
        ],
        ..legend("Leonardo da Vinci", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Artificer], 3, 3)
    }
}

/// Tarrian's Journal // The Tomb of Aclazotz — {1}{B} legendary artifact.
/// {T}, sacrifice another artifact or creature (sorcery): draw. {2},{T},
/// discard your hand: transform. Back, a legendary land: {T}: {B}; {T}: you
/// may cast a creature spell from your graveyard this turn — it enters with
/// a finality counter and is a Vampire too.
pub fn tarrians_journal() -> CardDefinition {
    let tomb = CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { land_types: vec![LandType::Cave], ..Default::default() },
        ..land(
            "The Tomb of Aclazotz",
            vec![
                tap_add(Color::Black),
                ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::MayCastCreatureFromGraveyardThisTurnAs { creature_type: CreatureType::Vampire },
                    ..Default::default()
                },
            ],
        )
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                sac_other_filter: Some((R::Artifact.or(R::Creature).and(R::OtherThanSource).and(R::ControlledByYou), 1)),
                effect: draw(1),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                discard_hand_cost: true,
                effect: Effect::Transform { what: Selector::This },
                ..Default::default()
            },
        ],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Book], ..Default::default() },
        back_face: Some(Box::new(tomb)),
        ..permanent("Tarrian's Journal", cost(&[generic(1), b()]), CardType::Artifact)
    }
}

/// Mirrormind Crown — {4} Equipment, equip {2}. While attached to a
/// creature, the first time you'd create tokens each turn you instead
/// create that many copies of the equipped creature.
pub fn mirrormind_crown() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "The first time you would create one or more tokens each turn, you may instead create that many tokens that are copies of equipped creature.",
            StaticEffect::FirstTokensEachTurnCopyEquipped,
        )],
        ..equipment("Mirrormind Crown", cost(&[generic(4)]), cost(&[generic(2)]), EquipBonus::default())
    }
}

/// Defiler of Dreams — {3}{U}{U} 4/3 flying Phyrexian Sphinx. Blue
/// permanent spells may pay 2 life for {U}; casting one draws a card.
pub fn defiler_of_dreams() -> CardDefinition {
    let blue_permanent = || R::HasColor(Color::Blue).and(R::PermanentCard);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab(
            "Blue permanent spells may pay 2 life for {U}.",
            StaticEffect::PhyrexianPipForSpells { filter: blue_permanent(), color: Color::Blue },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(blue_permanent())),
            effect: draw(1),
        }],
        ..creature("Defiler of Dreams", cost(&[generic(3), u(), u()]), vec![CreatureType::Phyrexian, CreatureType::Sphinx], 4, 3)
    }
}

/// Thromok the Insatiable — {3}{R}{G} 0/0 Hellion. Devour X, X the number
/// devoured: that many squared +1/+1 counters (CR 702.82).
pub fn thromok_the_insatiable() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devour(0)],
        ..legend("Thromok the Insatiable", cost(&[generic(3), r(), g()]), vec![CreatureType::Hellion], 0, 0)
    }
}

/// Dragon Broodmother — {2}{R}{R}{R}{G} 4/4 flying Dragon. At the beginning
/// of each upkeep, a 1/1 red and green flying Dragon with devour 2.
pub fn dragon_broodmother() -> CardDefinition {
    let whelp = TokenDefinition {
        keywords: vec![Keyword::Flying, Keyword::Devour(2)],
        ..token("Dragon", 1, 1, vec![Color::Red, Color::Green], vec![CreatureType::Dragon])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: make(Value::ONE, whelp),
        }],
        ..creature("Dragon Broodmother", cost(&[generic(2), r(), r(), r(), g()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Ellie and Alan, Paleontologists — {2}{G}{W}{U} 2/5. {T}, exile a
/// creature card from your graveyard (sorcery): discover X, X its mana
/// value.
pub fn ellie_and_alan_paleontologists() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            exile_other_filter: Some((R::Creature, 1)),
            effect: Effect::Discover {
                n: Value::ManaValueOf(Box::new(Selector::CostExiledCards)),
                filter: None,
            },
            ..Default::default()
        }],
        ..legend(
            "Ellie and Alan, Paleontologists",
            cost(&[generic(2), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            2,
            5,
        )
    }
}

/// Savage Land Dinosaur — {4}{G}{G} 7/6 trample; basic landcycling {2}.
pub fn savage_land_dinosaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))],
        ..creature("Savage Land Dinosaur", cost(&[generic(4), g(), g()]), vec![CreatureType::Dinosaur], 7, 6)
    }
}

/// Baru, Wurmspeaker — {2}{G}{G} 3/3. Your Wurms get +2/+2 and trample.
/// {7}{G},{T}: a 4/4 Wurm, costing {X} less, X your greatest Wurm power.
pub fn baru_wurmspeaker() -> CardDefinition {
    let wurms = || Selector::EachPermanent(yours(R::Creature.and(R::HasCreatureType(CreatureType::Wurm))));
    CardDefinition {
        static_abilities: vec![
            static_ab("Wurms you control get +2/+2 …", StaticEffect::PumpPT { applies_to: wurms(), power: 2, toughness: 2 }),
            static_ab("… and have trample.", StaticEffect::GrantKeyword { applies_to: wurms(), keyword: Keyword::Trample }),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), g()]),
            tap_cost: true,
            cost_reduction_value: Some(Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(
                R::Creature.and(R::HasCreatureType(CreatureType::Wurm)),
            )))),
            effect: make(Value::ONE, token("Wurm", 4, 4, vec![Color::Green], vec![CreatureType::Wurm])),
            ..Default::default()
        }],
        ..legend("Baru, Wurmspeaker", cost(&[generic(2), g(), g()]), vec![CreatureType::Human, CreatureType::Druid], 3, 3)
    }
}

/// Mossbridge Troll — {5}{G}{G} 5/5. Destruction regenerates it instead.
/// Tap untapped other creatures you control with total power 10 or more:
/// +20/+20 until end of turn.
pub fn mossbridge_troll() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "If this creature would be destroyed, regenerate it.",
            StaticEffect::RegeneratesInsteadOfDestroyed,
        )],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature, 0)),
            tap_total_power: Some(10),
            effect: eot_pump(Selector::This, 20, 20),
            ..Default::default()
        }],
        ..creature("Mossbridge Troll", cost(&[generic(5), g(), g()]), vec![CreatureType::Troll], 5, 5)
    }
}

/// Havi, the All-Father — {3}{R}{G}{W} 6/6. Indestructible with four or
/// more historic cards in your graveyard. Sage Project: Havi or another
/// legendary creature of yours dying returns a legendary creature card with
/// lesser mana value from your graveyard, tapped.
pub fn havi_the_all_father() -> CardDefinition {
    let historic = R::Artifact
        .or(R::HasSupertype(Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga));
    CardDefinition {
        static_abilities: vec![static_ab(
            "Havi has indestructible as long as there are four or more historic cards in your graveyard.",
            StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Indestructible,
                condition: Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: historic },
                    Value::Const(4),
                ),
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(trigger_is(R::HasSupertype(Supertype::Legendary))),
            effect: Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(R::HasSupertype(Supertype::Legendary))
                        .and(R::InYourGraveyard)
                        .and(R::ManaValueLessThanEventAmount),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        }],
        ..legend("Havi, the All-Father", cost(&[generic(3), r(), g(), w()]), vec![CreatureType::God, CreatureType::Warrior], 6, 6)
    }
}

/// Bard Class — {R}{G} Class. Level 1: your legendary creatures enter with
/// an extra +1/+1 counter. Level 2 ({R}{G}): legendary spells cost {R}{G}
/// less (colored mana only). Level 3 ({3}{R}{G}): casting a legendary spell
/// exiles your top two; you may play them this turn.
pub fn bard_class() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    let legendary = || R::HasSupertype(Supertype::Legendary);
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        static_abilities: vec![
            static_ab(
                "Legendary creatures you control enter with an additional +1/+1 counter on them.",
                StaticEffect::MatchingEntersWithExtraCounters {
                    filter: R::Creature.and(legendary()).and(R::ControlledByYou),
                    kind: CounterType::PlusOnePlusOne,
                    amount: 1,
                },
            ),
            static_ab(
                "Level 2 — Legendary spells you cast cost {R}{G} less to cast.",
                StaticEffect::WhileClassLevelAtLeast {
                    n: 2,
                    inner: Box::new(StaticEffect::ColoredOnlyCostReduction { filter: legendary(), less: cost(&[r(), g()]) }),
                },
            ),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::SourceClassLevelAtLeast(3),
                trigger_is(legendary()),
            ])),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        activated_abilities: vec![level_up(cost(&[r(), g()]), 1), level_up(cost(&[generic(3), r(), g()]), 2)],
        ..permanent("Bard Class", cost(&[r(), g()]), CardType::Enchantment)
    }
}

/// Korvold, Gleeful Glutton — {5}{B}{R}{G} Legendary 4/4 Dragon Noble. Costs
/// {1} less for each card type among permanents you've sacrificed this turn.
/// Flying, trample, haste. Combat damage to a player: put X +1/+1 counters on
/// it and draw X cards, X = permanent types among cards in your graveyard.
pub fn korvold_gleeful_glutton() -> CardDefinition {
    let x = || Value::PermanentTypesInGraveyard(PlayerRef::You);
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Haste],
        self_cost_reduction_per: Some((Value::CardTypesSacrificedThisTurn(PlayerRef::You), 1)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: x() },
                Effect::Draw { who: Selector::You, amount: x() },
            ]),
        }],
        ..legend(
            "Korvold, Gleeful Glutton",
            cost(&[generic(5), b(), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::Noble],
            4,
            4,
        )
    }
}

/// Invasion of Ergamon // Truga Cliffcharger — {R}{G} Battle — Siege, defense
/// 5. ETB: create a Treasure, then you may discard a card; if you do, draw a
/// card. Back: a 3/4 trample Rhino; ETB: you may discard a card; if you do,
/// search your library for a land or battle card to your hand.
pub fn invasion_of_ergamon() -> CardDefinition {
    use crate::card::BattleSubtype;
    let discard_then = |rider: Effect| {
        may(
            "discard a card",
            Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::If {
                    cond: Predicate::DiscardedThisEffect { who: PlayerRef::You },
                    then: Box::new(rider),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    };
    let cliffcharger = CardDefinition {
        color_indicator: vec![Color::Red, Color::Green],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(discard_then(Effect::Search {
            who: PlayerRef::You,
            filter: R::Land.or(R::HasCardType(CardType::Battle)),
            to: ZoneDest::Hand(PlayerRef::You),
        }))],
        ..creature("Truga Cliffcharger", ManaCost::default(), vec![CreatureType::Rhino], 3, 4)
    };
    CardDefinition {
        name: "Invasion of Ergamon",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 5,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
            discard_then(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        ]))],
        back_face: Some(Box::new(cliffcharger)),
        ..Default::default()
    }
}
