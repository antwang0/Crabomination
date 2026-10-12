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
        static_abilities: vec![static_ab("This creature can't have counters put on it.", StaticEffect::CountersCantBePlaced)],
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
        // The printed {4}{W}{U} back can't be cast directly here: the card is
        // modeled as transforming only (CR 712.2), see INCOMPLETE_CARDS.md.
        ..legend("Black Panther, Hope Enduring", ManaCost::default(), vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero], 3, 3)
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
                what: Selector::EachPermanent(yours(
                    R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                )),
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
            Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
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
            crate::effect::shortcut::on_dies(may(
                "Exile Doombot Harbinger to return a creature card from your graveyard?",
                Effect::Seq(vec![
                    Effect::Exile { what: Selector::This },
                    Effect::Move {
                        what: target_filtered(R::InYourGraveyard.and(R::Creature).and(R::OtherThanSource)),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                ]),
            )),
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
/// Hero, flying, vigilance. {T}: reveal your top card; if it's an artifact
/// card, draw a card. (Looking at the top card any time isn't modeled.)
pub fn iron_lad_diverging_destiny() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
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
/// Attacking: look at your top card; a land goes onto the battlefield tapped,
/// otherwise create a 1/1 red Goblin.
pub fn rulik_mons_warren_chief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::If {
            cond: Predicate::EntityMatches { what: top_card(), filter: R::Land },
            then: Box::new(Effect::Move {
                what: top_card(),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
            else_: Box::new(make(Value::ONE, token("Goblin", 1, 1, vec![Color::Red], vec![CreatureType::Goblin]))),
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
    let was_cast = || R::WasCastFromHand.or(R::WasCastFromLibrary).or(R::WasCastFromGraveyard);
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
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::ValueAtLeast(
                Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByDefendingPlayer))),
                Value::Const(8),
            )),
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
