//! Commander: the 2026-10-16 rescan's one-card gaps (Michelangelo Mutant BFF,
//! Spider-Man India, Spectacular Spider-Man, Lydia Frye, Bortuk Bonerattle,
//! Lozhan). Tests in `tests/recent_b/cmdr_oct16_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::card::{StaticAbility, StaticEffect};
use crate::effect::shortcut::{boast, etb, landfall, target_filtered};
use crate::game::TurnStep;
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::SpendRestriction;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use std::sync::Arc;

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

fn your_creature() -> Selector {
    target_filtered(R::Creature.and(R::ControlledByYou))
}

// ── Michelangelo, Mutant BFF ────────────────────────────────────────────────

/// Cowabunga! — {G} sorcery. Look at the top four; you may reveal a Mutant,
/// Ninja, Turtle or land card and put it into your hand; the rest go to the
/// bottom in a random order.
pub fn cowabunga() -> CardDefinition {
    let tmnt = R::HasCreatureType(CreatureType::Mutant)
        .or(R::HasCreatureType(CreatureType::Ninja))
        .or(R::HasCreatureType(CreatureType::Turtle))
        .or(R::Land);
    CardDefinition {
        name: "Cowabunga!",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(tmnt),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

// ── Spider-Man India ────────────────────────────────────────────────────────

/// Pictures of Spider-Man — {2}{G} artifact. Enters: look at the top five, up
/// to two creature cards into your hand, the rest on the bottom at random.
/// {1}, {T}, sacrifice it: create a Treasure.
pub fn pictures_of_spider_man() -> CardDefinition {
    CardDefinition {
        name: "Pictures of Spider-Man",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(5),
            pick_filter: Some(R::Creature),
            take: Some(Value::Const(2)),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Spectacular Spider-Man ──────────────────────────────────────────────────

/// Origin of Spider-Man — {1}{W} Saga. I: a 2/1 green Spider with reach.
/// II: a +1/+1 counter on target creature you control; it becomes a
/// legendary Spider Hero in addition to its other types. III: target
/// creature you control gains double strike until end of turn.
pub fn origin_of_spider_man() -> CardDefinition {
    let spider = TokenDefinition {
        name: "Spider".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        keywords: vec![Keyword::Reach],
        ..Default::default()
    };
    saga(
        "Origin of Spider-Man",
        cost(&[generic(1), w()]),
        vec![
            (1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(spider) }),
            (
                2,
                Effect::Seq(vec![
                    Effect::AddCounter { what: your_creature(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::BecomeLegendary { what: Selector::Target(0), duration: Duration::Permanent },
                    Effect::AddCreatureTypes {
                        what: Selector::Target(0),
                        creature_types: vec![CreatureType::Spider, CreatureType::Hero],
                        duration: Duration::Permanent,
                    },
                ]),
            ),
            (
                3,
                Effect::GrantKeyword { what: your_creature(), keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
            ),
        ],
    )
}

// ── Lydia Frye ──────────────────────────────────────────────────────────────

/// Auditore Ambush — {2}{U}{B} sorcery. Choose one or both: return target
/// creature to its owner's hand; target player searches their library and/or
/// graveyard for a card named Ezio, Blade of Vengeance and puts it into hand.
pub fn auditore_ambush() -> CardDefinition {
    CardDefinition {
        name: "Auditore Ambush",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                Effect::SearchZones {
                    who: PlayerRef::Target(0),
                    zones: vec![Zone::Library, Zone::Graveyard],
                    filter: R::HasName("Ezio, Blade of Vengeance".into()),
                    to: ZoneDest::Hand(PlayerRef::Target(0)),
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

// ── Bortuk Bonerattle ───────────────────────────────────────────────────────

/// Bortuk Bonerattle — {4}{B}{G} legendary 4/4 Troll Shaman. Domain — when it
/// enters, if you cast it, choose target creature card in your graveyard: onto
/// the battlefield if its mana value is at most your domain count, else to hand.
pub fn bortuk_bonerattle() -> CardDefinition {
    let back = |to: ZoneDest| Effect::Move { what: target_filtered(R::Creature.from_your_graveyard()), to };
    CardDefinition {
        name: "Bortuk Bonerattle",
        cost: cost(&[generic(4), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Troll, CreatureType::Shaman], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::SourceWasCast),
            effect: Effect::If {
                cond: Predicate::ValueAtMost(Value::ManaValueOf(Box::new(Selector::Target(0))), Value::DomainCount(PlayerRef::You)),
                then: Box::new(back(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false })),
                else_: Box::new(back(ZoneDest::Hand(PlayerRef::You))),
            },
        }],
        ..Default::default()
    }
}

// ── Lozhan, Dragons' Legacy ─────────────────────────────────────────────────

/// Sailors' Bane — {7}{U}{U} 7/7 Dragon Turtle, ward {4}. Costs {1} less for
/// each instant, sorcery or Adventure card you own in exile and in your
/// graveyard.
pub fn sailors_bane() -> CardDefinition {
    let spells = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).or(R::HasAdventure);
    let count = |zone: Zone| Value::CountOf(Box::new(Selector::CardsInZone { who: PlayerRef::You, zone, filter: spells() }));
    CardDefinition {
        name: "Sailors' Bane",
        cost: cost(&[generic(7), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon, CreatureType::Turtle], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Ward(WardCost::generic(4))],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each card you own in exile and in your graveyard that's an instant card, a sorcery card, or a card that has an Adventure.",
            effect: StaticEffect::SelfCostReducedByValue { amount: Value::Sum(vec![count(Zone::Exile), count(Zone::Graveyard)]) },
        }],
        ..Default::default()
    }
}

// ── Superior Spider-Man ─────────────────────────────────────────────────────

/// Symbiote Spider-Man — {2}{U/B} legendary 2/4. Combat damage to a player:
/// look at that many cards, one into your hand, the rest into your graveyard.
/// Find New Host — {2}{U/B}, exile it from your graveyard: a +1/+1 counter on
/// target creature you control, which gains that combat-damage ability.
pub fn symbiote_spider_man() -> CardDefinition {
    let dig = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::TriggerEventAmount,
            rest_to_graveyard: true,
            ..Default::default()
        })),
    };
    let ub = || hybrid(Color::Blue, Color::Black);
    CardDefinition {
        name: "Symbiote Spider-Man",
        cost: cost(&[generic(2), ub()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Symbiote, CreatureType::Spider, CreatureType::Hero],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![dig.clone()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), ub()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: your_creature(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::GrantTriggeredAbility { what: Selector::Target(0), trigger: Box::new(dig), duration: Duration::Permanent },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Jasmine Boreal / Jasmine Boreal of the Seven ────────────────────────────

/// Jasmine Boreal of the Seven — {1}{G}{W} legendary 2/4 Human Druid. {T}: add
/// {G}{W}, spendable only on creature spells with no abilities. Creatures you
/// control with no abilities can't be blocked by creatures with abilities.
/// "Abilities" reads the printed card on both sides (`R::HasNoAbilities`).
pub fn jasmine_boreal_of_the_seven() -> CardDefinition {
    let plain = || R::HasNoAbilities;
    let with_abilities = R::Creature.and(R::Not(Box::new(plain())));
    CardDefinition {
        name: "Jasmine Boreal of the Seven",
        cost: cost(&[generic(1), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Druid], ..Default::default() },
        power: 2,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Green, Color::White])),
                    SpendRestriction::CreatureSpellWithNoAbilities,
                ),
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with no abilities can't be blocked by creatures with abilities.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(plain())),
                keyword: Keyword::CantBeBlockedBy(Box::new(with_abilities)),
            },
        }],
        ..Default::default()
    }
}

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

/// "Whenever one or more creature cards leave your graveyard" (CR 603.2c: one
/// fire per batch) — or each one, with `per_card`.
fn creature_cards_leave_graveyard(per_card: bool, effect: Effect) -> TriggeredAbility {
    let spec = EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature });
    TriggeredAbility { event: if per_card { spec } else { spec.once_per_batch() }, effect }
}

// ── Mary Jane Watson ────────────────────────────────────────────────────────

/// Web of Life and Destiny — {6}{G}{G} enchantment, convoke. At the beginning
/// of combat on your turn, look at the top five; you may put a creature card
/// from among them onto the battlefield; the rest go to the bottom at random.
pub fn web_of_life_and_destiny() -> CardDefinition {
    CardDefinition {
        name: "Web of Life and Destiny",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(R::Creature),
                to_battlefield: true,
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            })),
        }],
        ..Default::default()
    }
}

// ── Galadriel, Gift-Giver ───────────────────────────────────────────────────

/// Boughside Wanderers — {4}{G}{G} 4/4 Elf Scout. Enters: look at the top
/// four, you may take a permanent card, the rest to the bottom at random.
/// Landfall: +2/+2 until end of turn.
pub fn boughside_wanderers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(4),
                pick_filter: Some(R::PermanentCard),
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            }))),
            landfall(Effect::PumpPT { what: Selector::This, power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn }),
        ],
        ..creature("Boughside Wanderers", cost(&[generic(4), g(), g()]), vec![CreatureType::Elf, CreatureType::Scout], 4, 4)
    }
}

/// Cantankerous Keepers — {5}{G} 4/3 Elf Soldier, affinity for Elves. Enters:
/// mill four, then put every Elf card milled this way into your hand.
pub fn cantankerous_keepers() -> CardDefinition {
    let elf = || R::HasCreatureType(CreatureType::Elf);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Affinity for Elves (This spell costs {1} less to cast for each Elf you control.)",
            effect: StaticEffect::SelfCostReducedPerPermanentMatching { filter: elf().and(R::ControlledByYou), per: 1 },
        }],
        triggered_abilities: vec![etb(Effect::MillThenToHandN {
            amount: Value::Const(4),
            filter: elf(),
            take: Value::Const(4),
            otherwise: None,
        })],
        ..creature("Cantankerous Keepers", cost(&[generic(5), g()]), vec![CreatureType::Elf, CreatureType::Soldier], 4, 3)
    }
}

// ── Glorfindel, Dauntless Rescuer ───────────────────────────────────────────

/// Sentinel Totem — {1} artifact. Enters: scry 1. {T}, exile it: exile all
/// graveyards.
pub fn sentinel_totem() -> CardDefinition {
    CardDefinition {
        name: "Sentinel Totem",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::ExilePlayerGraveyard { who: PlayerRef::EachPlayer, filter: None },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nissa's Revelation — {5}{G}{G} sorcery. Scry 5, then reveal the top card;
/// if it's a creature card, draw cards equal to its power and gain life equal
/// to its toughness (both read before the draw moves it).
pub fn nissas_revelation() -> CardDefinition {
    let top = || Box::new(Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE });
    CardDefinition {
        name: "Nissa's Revelation",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(5) },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE }, filter: R::Creature },
                then: Box::new(Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::ToughnessOf(top()) },
                    Effect::Draw { who: Selector::You, amount: Value::PowerOf(top()) },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Gollum the Abandoned ────────────────────────────────────────────────────

/// Undead Hand Ninja — {1}{B} 2/1 deathtouch Zombie Ninja Villain. One or more
/// creature cards leaving your graveyard: each opponent loses 1, you gain 1.
pub fn undead_hand_ninja() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![creature_cards_leave_graveyard(
            false,
            Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        )],
        ..creature("Undead Hand Ninja", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Ninja, CreatureType::Villain], 2, 1)
    }
}

/// Along the Crooked Way — {2}{B} enchantment. Enters: return target creature
/// card from your graveyard to your hand. Each creature card leaving your
/// graveyard: amass Goblins 1. {1}{B}: Goblins and Orcs you control gain
/// menace until end of turn.
pub fn along_the_crooked_way() -> CardDefinition {
    let orcs = R::HasCreatureType(CreatureType::Goblin).or(R::HasCreatureType(CreatureType::Orc)).and(R::ControlledByYou);
    CardDefinition {
        name: "Along the Crooked Way",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Move { what: target_filtered(R::Creature.from_your_graveyard()), to: ZoneDest::Hand(PlayerRef::You) }),
            creature_cards_leave_graveyard(
                true,
                Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Goblin) },
            ),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::GrantKeyword { what: Selector::EachPermanent(orcs), keyword: Keyword::Menace, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Balin, Loremaster ───────────────────────────────────────────────────────

/// Balin, Loremaster — {3}{R}{R} legendary 4/4 Dwarf Bard, storied. A Dwarf you
/// control entering: you may discard your hand and draw that many; with an
/// enduring story, Balin also deals that much damage to each opponent.
pub fn balin_loremaster() -> CardDefinition {
    let discarded = || Value::CountOf(Box::new(Selector::DiscardedThisResolution { filter: R::Any }));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Storied (If you control three or more artifacts, legendaries, and/or Sagas, you have an enduring story for the rest of the game.)",
            effect: StaticEffect::Storied,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCreatureType(CreatureType::Dwarf),
            }),
            effect: Effect::MayDo {
                description: "Discard your hand and draw that many cards?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::DiscardHandDrawThatMany { who: Selector::You },
                    Effect::If {
                        cond: Predicate::HasEnduringStory { who: PlayerRef::You },
                        then: Box::new(Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: discarded() }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        }],
        ..creature("Balin, Loremaster", cost(&[generic(3), r(), r()]), vec![CreatureType::Dwarf, CreatureType::Bard], 4, 4)
    }
}

/// Fearless Liberator — {1}{R} 2/1 Dwarf Berserker. Boast — {2}{R}: a 2/1 red
/// Dwarf Berserker token.
pub fn fearless_liberator() -> CardDefinition {
    let token = TokenDefinition {
        name: "Dwarf Berserker".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Berserker], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![boast(
            cost(&[generic(2), r()]),
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(token) },
        )],
        ..creature("Fearless Liberator", cost(&[generic(1), r()]), vec![CreatureType::Dwarf, CreatureType::Berserker], 2, 1)
    }
}

// ── Human Torch, Johnny Storm ───────────────────────────────────────────────

/// K'un-Lun Warrior — {1}{R} 2/2 Human Warrior Hero. Enters: you may sacrifice
/// an artifact or discard a card; if you do, draw a card.
pub fn kun_lun_warrior() -> CardDefinition {
    let draw = || Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE });
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice an artifact to draw a card?".into(),
            filter: R::Artifact.and(R::ControlledByYou),
            count: Value::ONE,
            then: draw(),
            else_: Some(Box::new(Effect::MayDiscard {
                description: "Discard a card to draw a card?".into(),
                count: Value::ONE,
                then: draw(),
                else_: None,
            })),
        })],
        ..creature("K'un-Lun Warrior", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero], 2, 2)
    }
}

/// Wonder Man, Hollywood Hero — {3}{R}{R} legendary 4/4 flying Human Performer
/// Hero. Your permanents' power-up abilities can each be activated one more
/// time. Power-up — {5}{R}{R}: two +1/+1 counters on it.
pub fn wonder_man_hollywood_hero() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Each power-up ability of permanents you control can be activated an additional time.",
            effect: StaticEffect::PowerUpOnceMore,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), r(), r()]),
            power_up: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature(
            "Wonder Man, Hollywood Hero",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Performer, CreatureType::Hero],
            4,
            4,
        )
    }
}

// ── Shaun & Rebecca, Agents ─────────────────────────────────────────────────

/// Shaun & Rebecca, Agents — {1}{G}{W}{U} legendary 4/4 vigilance. Enters:
/// search your graveyard, hand and library for The Animus and put it onto the
/// battlefield. {T}: add {C}; when you do, mill two.
pub fn shaun_rebecca_agents() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::SearchZones {
            who: PlayerRef::You,
            zones: vec![Zone::Graveyard, Zone::Hand, Zone::Library],
            filter: R::HasName("The Animus".into()),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                Effect::ReflexiveTrigger { body: Box::new(Effect::Mill { who: Selector::You, amount: Value::Const(2) }) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Shaun & Rebecca, Agents",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Scientist],
            4,
            4,
        )
    }
}

/// Pull from Eternity — {W} instant. Put target face-up exiled card into its
/// owner's graveyard.
pub fn pull_from_eternity() -> CardDefinition {
    CardDefinition {
        name: "Pull from Eternity",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move { what: target_filtered(R::InExile.and(R::Not(Box::new(R::FaceDown)))), to: ZoneDest::Graveyard },
        ..Default::default()
    }
}
