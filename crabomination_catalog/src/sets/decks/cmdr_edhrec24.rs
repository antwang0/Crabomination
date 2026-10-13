//! Commander: the 2026-10-13 rescan's two-card gaps on existing primitives —
//! The Serpent Society, The Master Multiplied, Mayael, Gix, Edric and friends
//! (`cmdr_edhrec23` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec24.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
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

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn on(kind: EventKind, scope: EventScope, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(kind, scope), effect }
}

fn source_is(kind: EventKind, scope: EventScope, filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(kind, scope)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

fn mint(definition: Arc<TokenDefinition>, count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition }
}

/// "Whenever a creature you control with deathtouch deals combat damage to a
/// player."
fn deathtouch_hit(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
            .dealt_by(R::HasKeyword(Keyword::Deathtouch)),
        effect,
    }
}

fn myriad() -> TriggeredAbility {
    on_attack(Effect::Myriad)
}

fn goblin() -> R {
    R::HasCreatureType(CreatureType::Goblin)
}

// ── The Serpent Society / Aveline de Grandpré ────────────────────────────────

/// Black Widow, Deadly Hunter — {2}{B} 3/3 legendary Human Assassin Hero,
/// deathtouch. A deathtouch creature of yours hitting a player draws you a
/// card for 1 life.
pub fn black_widow_deadly_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![deathtouch_hit(Effect::Seq(vec![
            draw(1),
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..legend(
            "Black Widow, Deadly Hunter",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Hero],
            3,
            3,
        )
    }
}

/// White Widow, Yelena Belova — {1}{B} 1/2 legendary Human Assassin Villain,
/// deathtouch. A deathtouch creature of yours hitting a player gets a +1/+1
/// counter.
pub fn white_widow_yelena_belova() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![deathtouch_hit(Effect::AddCounter {
            what: Selector::TriggerSource,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..legend(
            "White Widow, Yelena Belova",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Villain],
            1,
            2,
        )
    }
}

// ── The Master, Multiplied ───────────────────────────────────────────────────

/// Genasi Enforcers — {1}{R} 1/3 Elemental Shaman, myriad. {1}{R}: creatures
/// you control named Genasi Enforcers get +1/+0 until end of turn.
pub fn genasi_enforcers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![myriad()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::HasName("Genasi Enforcers".into()).and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Genasi Enforcers", cost(&[generic(1), r()]), vec![CreatureType::Elemental, CreatureType::Shaman], 1, 3)
    }
}

/// Gnoll War Band — {5}{R} 5/5 Gnoll, menace, myriad. Costs {1} less for each
/// opponent who was dealt damage this turn.
pub fn gnoll_war_band() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![myriad()],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each opponent who was dealt damage this turn.",
            effect: StaticEffect::SelfCostReducedByValue { amount: Value::OpponentsDealtDamageThisTurn },
        }],
        ..creature("Gnoll War Band", cost(&[generic(5), r()]), vec![CreatureType::Gnoll], 5, 5)
    }
}

/// Tiamat's Fanatics — {4}{R} 4/3 Dragon Warrior, haste, myriad.
pub fn tiamats_fanatics() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![myriad()],
        ..creature("Tiamat's Fanatics", cost(&[generic(4), r()]), vec![CreatureType::Dragon, CreatureType::Warrior], 4, 3)
    }
}

// ── Mayael the Anima ─────────────────────────────────────────────────────────

/// Godsire — {4}{R}{G}{G}{W} 8/8 Beast, vigilance. {T}: an 8/8 red, green and
/// white Beast.
pub fn godsire() -> CardDefinition {
    let beast = token("Beast", 8, 8, vec![Color::Red, Color::Green, Color::White], vec![CreatureType::Beast], vec![]);
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: mint(beast, Value::ONE),
            ..Default::default()
        }],
        ..creature("Godsire", cost(&[generic(4), r(), g(), g(), w()]), vec![CreatureType::Beast], 8, 8)
    }
}

/// Knotvine Mystic — {R}{G}{W} 2/2 Elf Druid. {1}, {T}: add {R}{G}{W}.
pub fn knotvine_mystic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Red, Color::Green, Color::White]),
            },
            ..Default::default()
        }],
        ..creature("Knotvine Mystic", cost(&[r(), g(), w()]), vec![CreatureType::Elf, CreatureType::Druid], 2, 2)
    }
}

// ── Arahbo / Ajani, Nacatl Pariah ────────────────────────────────────────────

/// Ajani's Sunstriker — {W}{W} 2/2 Cat Cleric, lifelink.
pub fn ajanis_sunstriker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        ..creature("Ajani's Sunstriker", cost(&[w(), w()]), vec![CreatureType::Cat, CreatureType::Cleric], 2, 2)
    }
}

/// Enlightened Ascetic — {1}{W} 1/1 Cat Monk. ETB: you may destroy target
/// enchantment.
pub fn enlightened_ascetic() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Destroy { what: target_filtered(R::Enchantment) }),
        })],
        ..creature("Enlightened Ascetic", cost(&[generic(1), w()]), vec![CreatureType::Cat, CreatureType::Monk], 1, 1)
    }
}

/// Ajani, Adversary of Tyrants — {2}{W}{W}, loyalty 4. +1: a +1/+1 counter
/// on each of up to two target creatures. −2: return a creature card with
/// mana value 2 or less from your graveyard to the battlefield. −7: an emblem
/// making three 1/1 lifelink Cats at your end step.
pub fn ajani_adversary_of_tyrants() -> CardDefinition {
    let cat = token("Cat", 1, 1, vec![Color::White], vec![CreatureType::Cat], vec![Keyword::Lifelink]);
    CardDefinition {
        name: "Ajani, Adversary of Tyrants",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Ajani], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Move {
                    what: target_filtered(R::Creature.and(R::ManaValueAtMost(2)).from_your_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Ajani, Adversary of Tyrants".into(),
                    triggered: vec![on(
                        EventKind::StepBegins(TurnStep::End),
                        EventScope::YourControl,
                        mint(cat, Value::Const(3)),
                    )],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Gix, Yawgmoth Praetor ────────────────────────────────────────────────────

/// Pilfering Imp — {B} 1/1 Imp, flying. {1}{B}, {T}, sacrifice it: target
/// opponent reveals their hand and discards a nonland card of your choice
/// (sorcery speed).
pub fn pilfering_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::DiscardChosen {
                from: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Nonland,
            },
            ..Default::default()
        }],
        ..creature("Pilfering Imp", cost(&[b()]), vec![CreatureType::Imp], 1, 1)
    }
}

/// Prickly Boggart — {B} 1/1 Goblin Rogue, fear.
pub fn prickly_boggart() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        ..creature("Prickly Boggart", cost(&[b()]), vec![CreatureType::Goblin, CreatureType::Rogue], 1, 1)
    }
}

// ── Edric, Spymaster of Trest ────────────────────────────────────────────────

/// Silver Raven — {U} 1/1 artifact creature Bird, flying. ETB: scry 1.
pub fn silver_raven() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
        ..creature("Silver Raven", cost(&[u()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Unified Will — {1}{U} instant. Counter target spell if you control more
/// creatures than that spell's controller.
pub fn unified_will() -> CardDefinition {
    let creatures_of = |who: PlayerRef| Value::count(Selector::ControlledBy { who, filter: R::Creature });
    spell(
        "Unified Will",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::ValueAtLeast(
                creatures_of(PlayerRef::You),
                Value::Sum(vec![
                    creatures_of(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    Value::ONE,
                ]),
            ),
            then: Box::new(Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) }),
            else_: Box::new(Effect::Noop),
        },
    )
}

// ── Six ──────────────────────────────────────────────────────────────────────

/// Circle of the Land Druid — {1}{G} 1/1 Gnome Druid. ETB: you may mill four.
/// Dies: return target land card from your graveyard to your hand.
pub fn circle_of_the_land_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Mill four cards?".into(),
                body: Box::new(Effect::Mill { who: Selector::You, amount: Value::Const(4) }),
            }),
            on(
                EventKind::CreatureDied,
                EventScope::SelfSource,
                Effect::Move {
                    what: target_filtered(R::Land.from_your_graveyard()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
        ],
        ..creature("Circle of the Land Druid", cost(&[generic(1), g()]), vec![CreatureType::Gnome, CreatureType::Druid], 1, 1)
    }
}

// ── Thrun, Breaker of Silence ────────────────────────────────────────────────

/// Tajuru Preserver — {1}{G} 2/1 Elf Shaman. Opponents' spells and abilities
/// can't make you sacrifice permanents.
pub fn tajuru_preserver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells and abilities your opponents control can't cause you to sacrifice permanents.",
            effect: StaticEffect::OpponentsCantMakeYouSacrifice,
        }],
        ..creature("Tajuru Preserver", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Shaman], 2, 1)
    }
}

/// Boar Umbra — {2}{G} Aura. +3/+3; umbra armor.
pub fn boar_umbra() -> CardDefinition {
    CardDefinition {
        name: "Boar Umbra",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::UmbraArmor],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 3, ..Default::default() }),
        ..Default::default()
    }
}

// ── Obyra / Ganax ────────────────────────────────────────────────────────────

/// Feywild Visitor — {2}{U} legendary enchantment — Background. Commander
/// creatures you own have "Whenever one or more nontoken creatures you control
/// deal combat damage to a player, create a 1/1 blue Faerie Dragon with
/// flying."
pub fn feywild_visitor() -> CardDefinition {
    let dragon = token(
        "Faerie Dragon",
        1,
        1,
        vec![Color::Blue],
        vec![CreatureType::Faerie, CreatureType::Dragon],
        vec![Keyword::Flying],
    );
    CardDefinition {
        name: "Feywild Visitor",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"Whenever one or more nontoken creatures you control deal combat damage to a player, you create a 1/1 blue Faerie Dragon creature token with flying.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                        .once_per_batch()
                        .dealt_by(R::NotToken),
                    effect: mint(dragon, Value::ONE),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Draconic Muralists — {3}{G} 4/3 Dragon Bard. Dies: you may search for a
/// Dragon card and put it into your hand.
pub fn draconic_muralists() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on(
            EventKind::CreatureDied,
            EventScope::SelfSource,
            Effect::MayDo {
                description: "Search for a Dragon card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Dragon),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        )],
        ..creature("Draconic Muralists", cost(&[generic(3), g()]), vec![CreatureType::Dragon, CreatureType::Bard], 4, 3)
    }
}

// ── Shattergang Brothers ─────────────────────────────────────────────────────

/// Boggart Cursecrafter — {B}{R} 2/3 Goblin Warlock, deathtouch. Another
/// Goblin of yours dying deals 1 damage to each opponent.
pub fn boggart_cursecrafter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![source_is(
            EventKind::CreatureDied,
            EventScope::AnotherOfYours,
            goblin(),
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        )],
        ..creature("Boggart Cursecrafter", cost(&[b(), r()]), vec![CreatureType::Goblin, CreatureType::Warlock], 2, 3)
    }
}

/// Wort, Boggart Auntie — {2}{B}{R} 3/3 legendary Goblin Shaman, fear. Your
/// upkeep: you may return target Goblin card from your graveyard to your hand.
pub fn wort_boggart_auntie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        triggered_abilities: vec![on(
            EventKind::StepBegins(TurnStep::Upkeep),
            EventScope::YourControl,
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Move {
                    what: target_filtered(goblin().from_your_graveyard()),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        )],
        ..legend("Wort, Boggart Auntie", cost(&[generic(2), b(), r()]), vec![CreatureType::Goblin, CreatureType::Shaman], 3, 3)
    }
}

// ── Tinybones, the Pickpocket ────────────────────────────────────────────────

/// Eye Collector — {B} 1/1 Faerie, flying. Its combat damage to a player
/// makes each player mill a card.
pub fn eye_collector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on(
            EventKind::DealsCombatDamageToPlayer,
            EventScope::SelfSource,
            Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
        )],
        ..creature("Eye Collector", cost(&[b()]), vec![CreatureType::Faerie], 1, 1)
    }
}

/// Thieves' Guild Enforcer — {B} 1/1 Human Rogue, flash. It or another Rogue
/// of yours entering mills each opponent two. With an opponent at eight or
/// more graveyard cards, +2/+1 and deathtouch.
pub fn thieves_guild_enforcer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![source_is(
            EventKind::EntersBattlefield,
            EventScope::YourControl,
            R::HasCreatureType(CreatureType::Rogue),
            Effect::Mill { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
        )],
        static_abilities: vec![StaticAbility {
            description: "As long as an opponent has eight or more cards in their graveyard, this creature gets +2/+1 and has deathtouch.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::GreatestGraveyardSizeAmong(PlayerRef::EachOpponent),
                    Value::Const(8),
                ),
                power: 2,
                toughness: 1,
                keywords: vec![Keyword::Deathtouch],
            },
        }],
        ..creature("Thieves' Guild Enforcer", cost(&[b()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 1)
    }
}

// ── Toshiro Umezawa ──────────────────────────────────────────────────────────

/// Grim Return — {2}{B} instant. Put target creature card that went to a
/// graveyard from the battlefield this turn onto the battlefield under your
/// control.
pub fn grim_return() -> CardDefinition {
    spell(
        "Grim Return",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::Move {
            what: target_filtered(R::Creature.and(R::InGraveyard).and(R::PutIntoGraveyardFromBattlefieldThisTurn)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Succumb to Temptation — {1}{B}{B} instant. Draw two, lose 2 life.
pub fn succumb_to_temptation() -> CardDefinition {
    spell(
        "Succumb to Temptation",
        cost(&[generic(1), b(), b()]),
        CardType::Instant,
        Effect::Seq(vec![draw(2), Effect::LoseLife { who: Selector::You, amount: Value::Const(2) }]),
    )
}

// ── Katilda and Lier / Inferno of the Star Mounts / Sun Quan ─────────────────

/// Mass Appeal — {2}{U} sorcery. Draw a card for each Human you control.
pub fn mass_appeal() -> CardDefinition {
    spell(
        "Mass Appeal",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::Draw {
            who: Selector::You,
            amount: Value::count(Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Human).and(R::ControlledByYou),
            )),
        },
    )
}

/// Rush of Blood — {2}{R} instant. Target creature gets +X/+0, X its power.
pub fn rush_of_blood() -> CardDefinition {
    spell(
        "Rush of Blood",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::PowerOf(Box::new(Selector::Target(0))),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Sturmgeist — {3}{U}{U} */* Spirit, flying. P/T = cards in your hand; its
/// combat damage to a player draws a card.
pub fn sturmgeist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Sturmgeist's power and toughness are each equal to the number of cards in your hand.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::HandSizeOf(PlayerRef::You),
                toughness: Value::HandSizeOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![on(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource, draw(1))],
        ..creature("Sturmgeist", cost(&[generic(3), u(), u()]), vec![CreatureType::Spirit], 0, 0)
    }
}
