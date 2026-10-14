//! Commander: gaps of EDHREC's commanders ranked 1501-2000 (the 2026-10-13
//! twenty-page rescan), seventh file — three-card gaps. Tests in
//! `tests/recent_b/cmdr_rank2k_g.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, exalted, mentor, mint_treasures, on_attack, on_dies, target_any, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};
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

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn grant_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

/// "When this Equipment enters, create `def`, then attach this Equipment to
/// it."
fn germ_etb(def: TokenDefinition) -> TriggeredAbility {
    etb(Effect::Seq(vec![make(1, def), Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken }]))
}

/// "Whenever a creature you control enters, you may attach this Equipment to
/// it."
fn follow_the_newcomer() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
        effect: Effect::MayDo {
            description: "Attach this Equipment to it?".into(),
            body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
        },
    }
}

fn shadow() -> R {
    R::HasKeyword(Keyword::Shadow)
}

/// Aegis of the Legion — {R}{W} Equipment: +1/+1 and mentor; the creature it
/// mentors also gets a shield counter. Equip {3}.
pub fn aegis_of_the_legion() -> CardDefinition {
    let mut shielded = mentor();
    shielded.effect = Effect::Seq(vec![
        shielded.effect.clone(),
        Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Shield, amount: Value::ONE },
    ]);
    equipment(
        "Aegis of the Legion",
        cost(&[r(), w()]),
        cost(&[generic(3)]),
        EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![shielded],
            ..Default::default()
        },
    )
}

/// Augur il-Vec — {1}{W} 1/3 shadow Human Cleric; sacrifice it during your
/// upkeep: gain 4 life.
pub fn augur_il_vec() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::CurrentStepIs(TurnStep::Upkeep),
            ])),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ..Default::default()
        }],
        ..creature("Augur il-Vec", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 3)
    }
}

/// Battlegrace Angel — {3}{W}{W} 4/4 flying Angel, exalted; a creature of
/// yours attacking alone gains lifelink until end of turn.
pub fn battlegrace_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            exalted(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::AttackingAlone),
                effect: grant_eot(Selector::TriggerSource, Keyword::Lifelink),
            },
        ],
        ..creature("Battlegrace Angel", cost(&[generic(3), w(), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Blightning — {1}{B}{R} sorcery: 3 damage to a player, who discards two.
/// (Planeswalker targets are not offered.)
pub fn blightning() -> CardDefinition {
    spell(
        "Blightning",
        cost(&[generic(1), b(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(3) },
            Effect::Discard { who: Selector::Target(0), amount: Value::Const(2), random: false },
        ]),
    )
}

/// Blizzard Specter — {2}{U}{B} snow 2/3 flying Specter. Its combat damage to
/// a player makes them return a permanent they control to hand, or discard.
pub fn blizzard_specter() -> CardDefinition {
    let defender = || PlayerRef::DefendingPlayer;
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::OpponentChoosesPermanentThen {
                    filter: R::Permanent,
                    body: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
                    chooser: Some(defender()),
                    theirs: true,
                },
                Effect::Discard { who: Selector::Player(defender()), amount: Value::ONE, random: false },
            ]),
        }],
        ..creature("Blizzard Specter", cost(&[generic(2), u(), b()]), vec![CreatureType::Specter], 2, 3)
    }
}

/// Boros Challenger — {R}{W} 2/3 mentor Human Soldier; {2}{R}{W}: +1/+1.
pub fn boros_challenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![mentor()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Boros Challenger", cost(&[r(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 3)
    }
}

/// Enkira, Hostile Scavenger — {3}{B}{G} legendary 3/3 Human Warrior. Enters
/// with two 2/2 black Zombie Walkers; must be blocked while equipped; gains
/// indestructible when it attacks beside two Zombies.
pub fn enkira_hostile_scavenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(make(2, token("Walker", 2, 2, Color::Black, vec![CreatureType::Zombie], vec![]))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                    Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(yours(
                            R::Creature
                                .and(R::HasCreatureType(CreatureType::Zombie))
                                .and(R::IsAttacking)
                                .and(R::OtherThanSource),
                        )),
                        n: Value::Const(2),
                    },
                ),
                effect: grant_eot(Selector::This, Keyword::Indestructible),
            },
        ],
        static_abilities: vec![static_ab(
            "As long as Enkira is equipped, it must be blocked if able.",
            StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::MustBeBlocked,
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::IsEquipped },
            },
        )],
        ..legend(
            "Enkira, Hostile Scavenger",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Flitterwing Nuisance — {U} 2/2 flying Faerie Rogue; enters with a -1/-1
/// counter. {2}{U}, remove a counter: this turn, your creatures' combat damage
/// to a player draws you a card.
pub fn flitterwing_nuisance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            remove_counter_cost: Some((CounterType::MinusOneMinusOne, 1)),
            effect: Effect::GrantTriggeredAbilityThisTurnToMatching {
                filter: yours(R::Creature),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: draw(1),
                }),
            },
            ..Default::default()
        }],
        ..creature("Flitterwing Nuisance", cost(&[u()]), vec![CreatureType::Faerie, CreatureType::Rogue], 2, 2)
    }
}

/// Gilded Pinions — {2} Equipment: entering makes a Treasure; flying. Equip
/// {2}.
pub fn gilded_pinions() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(mint_treasures(1))],
        ..equipment(
            "Gilded Pinions",
            cost(&[generic(2)]),
            cost(&[generic(2)]),
            EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() },
        )
    }
}

/// Goblin Champion — {R} 0/1 haste Goblin Warrior, exalted.
pub fn goblin_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![exalted()],
        ..creature("Goblin Champion", cost(&[r()]), vec![CreatureType::Goblin, CreatureType::Warrior], 0, 1)
    }
}

/// Graf Harvest — {B} enchantment: your Zombies have menace; {3}{B}, exile a
/// creature card from your graveyard: a 2/2 black Zombie.
pub fn graf_harvest() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Zombies you control have menace.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Creature.and(R::HasCreatureType(CreatureType::Zombie)))),
                keyword: Keyword::Menace,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
            effect: make(1, token("Zombie", 2, 2, Color::Black, vec![CreatureType::Zombie], vec![])),
            ..Default::default()
        }],
        ..spell("Graf Harvest", cost(&[b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Headsplitter — {1}{R} Equipment: entering makes a 1/1 black menace Assassin
/// and attaches to it; +1/+0. Equip {2}.
pub fn headsplitter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![germ_etb(token(
            "Assassin",
            1,
            1,
            Color::Black,
            vec![CreatureType::Assassin],
            vec![Keyword::Menace],
        ))],
        ..equipment("Headsplitter", cost(&[generic(1), r()]), cost(&[generic(2)]), EquipBonus { power: 1, ..Default::default() })
    }
}

/// Hunter's Axe — {G} Equipment: +2/+0, and attacking gives the creature your
/// choice of trample or deathtouch. Equip {2}.
pub fn hunters_axe() -> CardDefinition {
    equipment(
        "Hunter's Axe",
        cost(&[g()]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 2,
            triggered_abilities: vec![on_attack(Effect::ChooseMode(vec![
                grant_eot(Selector::This, Keyword::Trample),
                grant_eot(Selector::This, Keyword::Deathtouch),
            ]))],
            ..Default::default()
        },
    )
}

/// Light of the Legion — {4}{W}{W} 5/5 flying, mentor Angel; dying puts a
/// +1/+1 counter on each white creature you control.
pub fn light_of_the_legion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            mentor(),
            on_dies(Effect::AddCounter {
                what: Selector::EachPermanent(yours(R::Creature.and(R::HasColor(Color::White)))),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        ],
        ..creature("Light of the Legion", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 5, 5)
    }
}

/// Lyna, Veil of Vengeance — {W}{U}{B} legendary 2/3 shadow Soltari Cleric.
/// A shadow creature of yours dealing combat damage to a player draws a
/// card; {3}: a shadow counter on another creature (sorcery speed).
pub fn lyna_veil_of_vengeance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: shadow() }),
            effect: draw(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sorcery_speed: true,
            effect: Effect::AddKeywordCounter {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                keyword: Keyword::Shadow,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend(
            "Lyna, Veil of Vengeance",
            cost(&[w(), u(), b()]),
            vec![CreatureType::Soltari, CreatureType::Cleric],
            2,
            3,
        )
    }
}

/// Lyra, Tolarian Archangel — {1}{U}{U} legendary 3/3 flying Angel Wizard.
/// Each end step, if you drew three or more cards this turn, a 3/3 blue
/// flying Angel. {3}{U}{U}: this turn its combat damage to a player draws two.
pub fn lyra_tolarian_archangel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::You, n: 3 }),
            effect: make(1, token("Angel", 3, 3, Color::Blue, vec![CreatureType::Angel], vec![Keyword::Flying])),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u(), u()]),
            effect: Effect::GrantTriggeredAbility {
                what: Selector::This,
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: draw(2),
                }),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Lyra, Tolarian Archangel",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Angel, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Mask of Immolation — {1}{R} Equipment: entering makes a 1/1 red Elemental
/// and attaches to it; the equipped creature has "Sacrifice this creature: 1
/// damage to any target." Equip {2}.
pub fn mask_of_immolation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![germ_etb(token("Elemental", 1, 1, Color::Red, vec![CreatureType::Elemental], vec![]))],
        ..equipment(
            "Mask of Immolation",
            cost(&[generic(1), r()]),
            cost(&[generic(2)]),
            EquipBonus {
                activated_abilities: vec![ActivatedAbility {
                    sac_cost: true,
                    effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
    }
}

/// Rashel, Fist of Torm — {2}{W}{W} legendary 2/4 double strike Human
/// Knight; Auras you control have exalted.
pub fn rashel_fist_of_torm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        static_abilities: vec![static_ab(
            "Auras you control have exalted.",
            StaticEffect::GrantTriggeredAbility {
                filter: yours(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
                ability: Box::new(exalted()),
            },
        )],
        ..legend("Rashel, Fist of Torm", cost(&[generic(2), w(), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 4)
    }
}

/// Sai of the Shinobi — {1} Equipment: +1/+1; a creature of yours entering
/// may take it. Equip {2}.
pub fn sai_of_the_shinobi() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![follow_the_newcomer()],
        ..equipment("Sai of the Shinobi", cost(&[generic(1)]), cost(&[generic(2)]), EquipBonus {
            power: 1,
            toughness: 1,
            ..Default::default()
        })
    }
}

/// Stormrider Rig — {2} Equipment: +1/+1; a creature of yours entering may
/// take it. Equip {2}.
pub fn stormrider_rig() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![follow_the_newcomer()],
        ..equipment("Stormrider Rig", cost(&[generic(2)]), cost(&[generic(2)]), EquipBonus {
            power: 1,
            toughness: 1,
            ..Default::default()
        })
    }
}

/// Stronghold Overseer — {3}{B}{B}{B} 5/5 flying, shadow Demon. {B}{B}:
/// creatures with shadow get +1/+0 and creatures without it get -1/-0 until
/// end of turn.
pub fn stronghold_overseer() -> CardDefinition {
    let pump = |filter: R, p: i32| Effect::PumpPT {
        what: Selector::EachPermanent(R::Creature.and(filter)),
        power: Value::Const(p),
        toughness: Value::ZERO,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Shadow],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            effect: Effect::Seq(vec![pump(shadow(), 1), pump(shadow().negate(), -1)]),
            ..Default::default()
        }],
        ..creature("Stronghold Overseer", cost(&[generic(3), b(), b(), b()]), vec![CreatureType::Demon], 5, 5)
    }
}

/// Sublime Archangel — {2}{W}{W} 4/3 flying Angel, exalted; your other
/// creatures have exalted.
pub fn sublime_archangel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![exalted()],
        static_abilities: vec![static_ab(
            "Other creatures you control have exalted.",
            StaticEffect::GrantTriggeredAbility {
                filter: yours(R::Creature).and(R::OtherThanSource),
                ability: Box::new(exalted()),
            },
        )],
        ..creature("Sublime Archangel", cost(&[generic(2), w(), w()]), vec![CreatureType::Angel], 4, 3)
    }
}

/// The Lonely Mountain — legendary land — Mountain; enters tapped unless you
/// control an Equipment. {4}{R}, {T}: a 2/2 red Dwarf, {1} less per
/// Equipment you control (sorcery speed).
pub fn the_lonely_mountain() -> CardDefinition {
    let equipment = || yours(R::HasArtifactSubtype(ArtifactSubtype::Equipment));
    CardDefinition {
        name: "The Lonely Mountain",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Mountain], ..Default::default() },
        static_abilities: vec![static_ab(
            "This land enters tapped unless you control an Equipment.",
            StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::EachPermanent(equipment())),
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r()]),
            tap_cost: true,
            sorcery_speed: true,
            cost_reduction_per: Some(equipment()),
            effect: make(1, token("Dwarf", 2, 2, Color::Red, vec![CreatureType::Dwarf], vec![])),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thirst for Identity — {2}{U} instant: draw three, then discard two unless
/// you discard a creature card.
pub fn thirst_for_identity() -> CardDefinition {
    spell(
        "Thirst for Identity",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            draw(3),
            Effect::DiscardUnlessKind { who: PlayerRef::You, count: Value::Const(2), instead: R::Creature },
        ]),
    )
}

/// Unexpected Assistance — {3}{U}{U} instant with convoke: draw three, then
/// discard a card.
pub fn unexpected_assistance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..spell(
            "Unexpected Assistance",
            cost(&[generic(3), u(), u()]),
            CardType::Instant,
            Effect::Seq(vec![draw(3), Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }]),
        )
    }
}

/// Urgoros, the Empty One — {4}{B}{B} legendary 4/3 flying Specter. Its combat
/// damage to a player discards a card at random; if they can't, you draw.
pub fn urgoros_the_empty_one() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE, random: true },
                Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::DiscardedThisEffect { who: PlayerRef::DefendingPlayer })),
                    then: Box::new(draw(1)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend("Urgoros, the Empty One", cost(&[generic(4), b(), b()]), vec![CreatureType::Specter], 4, 3)
    }
}
