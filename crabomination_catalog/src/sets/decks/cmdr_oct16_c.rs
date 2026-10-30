//! Commander: the 2026-10-16 rescan's one-card gaps (Michelangelo Mutant BFF,
//! Spider-Man India, Spectacular Spider-Man, Lydia Frye, Bortuk Bonerattle,
//! Lozhan). Tests in `tests/recent_b/cmdr_oct16_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EnchantmentSubtype, EntersAsCopy, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::card::{StaticAbility, StaticEffect};
use crate::effect::ActivationTrait;
use crate::effect::shortcut::{boast, etb, landfall, on_attack, on_dies, target_filtered};
use crate::game::TurnStep;
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, RevealMissDest, Selector, ZoneDest};
use crate::mana::SpendRestriction;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, phyrexian, r, u, w};
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
        // "Put all Elf cards from among them" — every one, no pick.
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(4) },
            Effect::Move {
                what: Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter: elf() },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]))],
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
/// an artifact or discard a card; if you do, draw a card. The "or" is a mode
/// pick ahead of the optional cost, so the two asks never share an answer.
pub fn kun_lun_warrior() -> CardDefinition {
    let draw = || Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE });
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::MaySacrifice {
                description: "Sacrifice an artifact to draw a card?".into(),
                filter: R::Artifact.and(R::ControlledByYou),
                count: Value::ONE,
                then: draw(),
                else_: None,
            },
            Effect::MayDiscard { description: "Discard a card to draw a card?".into(), count: Value::ONE, then: draw(), else_: None },
        ]))],
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

// ── Lady Evangela ───────────────────────────────────────────────────────────

/// Way of the Mind Sculptor — {4}{U} legendary enchantment. Enters: empower
/// Jace 5. A loyalty ability you activate by removing two or more loyalty
/// counters draws a card (the activation event's amount is the counters paid).
pub fn way_of_the_mind_sculptor() -> CardDefinition {
    CardDefinition {
        name: "Way of the Mind Sculptor",
        cost: cost(&[generic(4), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(5) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::AbilityActivatedWith(ActivationTrait::Loyalty), EventScope::YourControl)
                    .with_filter(Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(2))),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

// ── Ian Chesterton ──────────────────────────────────────────────────────────

fn saga_back(name: &'static str, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        color_indicator: vec![Color::White],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        ..creature(name, ManaCost::default(), types, p, t)
    }
}

/// Era of Enlightenment — {1}{W} Saga. I: scry 2. II: gain 2 life. III: exile
/// it and return it transformed (Hand of Enlightenment, a 2/2 first-strike
/// Human Monk enchantment creature).
pub fn era_of_enlightenment() -> CardDefinition {
    CardDefinition {
        back_face: Some(Box::new(CardDefinition {
            keywords: vec![Keyword::FirstStrike],
            ..saga_back("Hand of Enlightenment", vec![CreatureType::Human, CreatureType::Monk], 2, 2)
        })),
        ..saga(
            "Era of Enlightenment",
            cost(&[generic(1), w()]),
            vec![
                (1, Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                (2, Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
                (3, Effect::ExileSelfReturnTransformed),
            ],
        )
    }
}

/// The Fall of Lord Konda — {2}{W} Saga. I: exile target creature an opponent
/// controls with mana value 4 or greater. II: each player gains control of all
/// permanents they own. III: exile it and return it transformed (Fragment of
/// Konda, a 1/3 defender Human Noble that draws a card when it dies).
pub fn the_fall_of_lord_konda() -> CardDefinition {
    CardDefinition {
        back_face: Some(Box::new(CardDefinition {
            keywords: vec![Keyword::Defender],
            triggered_abilities: vec![on_dies(Effect::Draw { who: Selector::You, amount: Value::ONE })],
            ..saga_back("Fragment of Konda", vec![CreatureType::Human, CreatureType::Noble], 1, 3)
        })),
        ..saga(
            "The Fall of Lord Konda",
            cost(&[generic(2), w()]),
            vec![
                (
                    1,
                    Effect::Exile {
                        what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::ManaValueAtLeast(4))),
                    },
                ),
                (2, Effect::OwnersGainControlOf { filter: R::Permanent }),
                (3, Effect::ExileSelfReturnTransformed),
            ],
        )
    }
}

// ── Polukranos Reborn ───────────────────────────────────────────────────────

/// Polukranos Reborn — {G}{G}{G} legendary 4/5 reach Hydra. {6}{W/P}, sorcery
/// speed: transform into Polukranos, Engine of Ruin (6/6 reach, lifelink; it or
/// another nontoken Hydra of yours dying makes a 3/3 reach Phyrexian Hydra and
/// a 3/3 lifelink one).
pub fn polukranos_reborn() -> CardDefinition {
    let hydra = |kw: Keyword| TokenDefinition {
        name: "Phyrexian Hydra".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian, CreatureType::Hydra], ..Default::default() },
        keywords: vec![kw],
        ..Default::default()
    };
    let mint = |kw| Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(hydra(kw)) };
    let engine = CardDefinition {
        supertypes: vec![Supertype::Legendary],
        color_indicator: vec![Color::Green, Color::White],
        keywords: vec![Keyword::Reach, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCreatureType(CreatureType::Hydra).and(R::NotToken),
            }),
            effect: Effect::Seq(vec![mint(Keyword::Reach), mint(Keyword::Lifelink)]),
        }],
        ..creature("Polukranos, Engine of Ruin", ManaCost::default(), vec![CreatureType::Phyrexian, CreatureType::Hydra], 6, 6)
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), phyrexian(Color::White)]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(engine)),
        ..creature("Polukranos Reborn", cost(&[g(), g(), g()]), vec![CreatureType::Hydra], 4, 5)
    }
}

/// Patch Up — {2}{W} sorcery. Return up to three creature cards with total
/// mana value 3 or less from your graveyard to the battlefield (cheapest
/// first, the `MoveWithinTotalManaValue` shape March from the Tomb uses).
pub fn patch_up() -> CardDefinition {
    CardDefinition {
        name: "Patch Up",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::MoveWithinTotalManaValue {
            from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature },
            filter: R::Creature,
            cap: Value::Const(3),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            max_count: Some(3),
            by_power: false,
        },
        ..Default::default()
    }
}

// ── Korlash, Heir to Blackblade ─────────────────────────────────────────────

/// Korlash, Heir to Blackblade — {2}{B}{B} legendary */* Zombie Warrior, P/T
/// the Swamps you control. {1}{B}: regenerate. Grandeur — discard another
/// Korlash: search for up to two Swamp cards, onto the battlefield tapped.
pub fn korlash_heir_to_blackblade() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        dynamic_pt: Some(DynamicPt::BasePlusLandsOfTypeControlled { land_type: LandType::Swamp, base_p: 0, base_t: 0 }),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
            ActivatedAbility {
                discard_cost: Some((R::HasName("Korlash, Heir to Blackblade".to_string()), 1)),
                effect: Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: R::HasLandType(LandType::Swamp),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    count: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..creature("Korlash, Heir to Blackblade", cost(&[generic(2), b(), b()]), vec![CreatureType::Zombie, CreatureType::Warrior], 0, 0)
    }
}

/// Activated Sleeper — {2}{B} 0/0 flash Phyrexian Shapeshifter. It may enter
/// as a copy of a creature card in a graveyard that was put there from the
/// battlefield this turn, a Phyrexian in addition to its other types.
pub fn activated_sleeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::PutIntoGraveyardFromBattlefieldThisTurn),
            from_graveyards: true,
            extra_creature_types: vec![CreatureType::Phyrexian],
            ..Default::default()
        }),
        ..creature("Activated Sleeper", cost(&[generic(2), b()]), vec![CreatureType::Phyrexian, CreatureType::Shapeshifter], 0, 0)
    }
}

// ── Doran, the Siege Tower ──────────────────────────────────────────────────

/// Unstoppable Ash — {3}{G} 5/5 trample Treefolk Warrior; champion a Treefolk
/// or Warrior. A creature you control becoming blocked gets +0/+5 until end
/// of turn.
pub fn unstoppable_ash() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::Champion {
                filter: R::HasCreatureType(CreatureType::Treefolk).or(R::HasCreatureType(CreatureType::Warrior)),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::ZERO,
                    toughness: Value::Const(5),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..creature("Unstoppable Ash", cost(&[generic(3), g()]), vec![CreatureType::Treefolk, CreatureType::Warrior], 5, 5)
    }
}

/// Reach of Branches — {4}{G} Kindred instant — Treefolk. Create a 2/5 green
/// Treefolk Shaman. A Forest entering under your control may return it from
/// your graveyard to your hand.
pub fn reach_of_branches() -> CardDefinition {
    let treefolk = TokenDefinition {
        name: "Treefolk Shaman".into(),
        power: 2,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Treefolk, CreatureType::Shaman], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Reach of Branches",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Treefolk], ..Default::default() },
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(treefolk) },
        triggered_abilities: vec![TriggeredAbility {
            // "Whenever a Forest you control ENTERS" — played or put there.
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasLandType(LandType::Forest),
            }),
            effect: Effect::MayDo {
                description: "Return Reach of Branches from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
            },
        }],
        ..Default::default()
    }
}

// ── Celeborn the Wise ───────────────────────────────────────────────────────

/// Galion, Elvenking's Butler — {2}{G}{G} legendary 4/4 Elf Advisor. Attacking:
/// up to one other target creature you control has base P/T equal to Galion's
/// until end of turn.
pub fn galion_elvenkings_butler() -> CardDefinition {
    let mine = || Box::new(Selector::This);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_attack(Effect::SetBasePT {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            power: Value::PowerOf(mine()),
            toughness: Value::ToughnessOf(mine()),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Galion, Elvenking's Butler", cost(&[generic(2), g(), g()]), vec![CreatureType::Elf, CreatureType::Advisor], 4, 4)
    }
}

/// Llanowar Empath — {3}{G} 2/2 Elf Shaman. Enters: scry 2, then reveal the
/// top card; a creature card goes to your hand.
pub fn llanowar_empath() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::If {
                cond: Predicate::EntityMatches { what: top(), filter: R::Creature },
                then: Box::new(Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Llanowar Empath", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Shaman], 2, 2)
    }
}

// ── Gríma Wormtongue ────────────────────────────────────────────────────────

/// Grond, the Gatebreaker — {3}{B} legendary 5/5 trample Vehicle, crew 3. An
/// artifact creature while it's your turn and you control an Army.
pub fn grond_the_gatebreaker() -> CardDefinition {
    CardDefinition {
        name: "Grond, the Gatebreaker",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample, Keyword::Crew(3)],
        static_abilities: vec![StaticAbility {
            description: "As long as it's your turn and you control an Army, Grond is an artifact creature.",
            effect: StaticEffect::SelfIsCreatureIf {
                condition: Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::SelectorExists(Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Army).and(R::ControlledByYou),
                    )),
                ]),
                creature_types: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Gorbag of Minas Morgul — {1}{B} legendary 2/2 Orc Soldier. A Goblin or Orc
/// of yours dealing combat damage to a player may be sacrificed; when it is,
/// draw a card or create a Treasure.
pub fn gorbag_of_minas_morgul() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Goblin).or(R::HasCreatureType(CreatureType::Orc)),
                },
            ),
            effect: Effect::MayDo {
                description: "Sacrifice it to draw a card or make a Treasure?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificePermanent { what: Selector::TriggerSource },
                    Effect::If {
                        cond: Predicate::SelectorExists(Selector::SacrificedThisResolution { filter: R::Any }),
                        then: Box::new(Effect::ChooseMode(vec![
                            Effect::Draw { who: Selector::You, amount: Value::ONE },
                            Effect::CreateToken {
                                who: PlayerRef::You,
                                count: Value::ONE,
                                definition: Arc::new(crabomination_base::tokens::treasure_token()),
                            },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        }],
        ..creature("Gorbag of Minas Morgul", cost(&[generic(1), b()]), vec![CreatureType::Orc, CreatureType::Soldier], 2, 2)
    }
}

// ── Wolverine, Fierce Fighter ───────────────────────────────────────────────

/// Training Compound — land. {T}: add {C}. {T}: add {R} or {G}, only if it
/// entered this turn or you control a basic land.
pub fn training_compound() -> CardDefinition {
    let gate = || {
        Some(Predicate::Any(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn },
            Predicate::SelectorExists(Selector::EachPermanent(R::IsBasicLand.and(R::ControlledByYou))),
        ]))
    };
    let tap = |c: Color| ActivatedAbility {
        tap_cost: true,
        condition: gate(),
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![c]) },
        ..Default::default()
    };
    CardDefinition {
        name: "Training Compound",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            tap(Color::Red),
            tap(Color::Green),
        ],
        ..Default::default()
    }
}

/// Rhino's Rampage — {R/G} sorcery. Target creature you control gets +1/+0
/// and fights target creature an opponent controls; when excess damage is
/// dealt to the latter this way, destroy up to one target noncreature
/// artifact with mana value 3 or less.
pub fn rhinos_rampage() -> CardDefinition {
    let theirs = || Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) };
    CardDefinition {
        name: "Rhino's Rampage",
        cost: cost(&[hybrid(Color::Red, Color::Green)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: your_creature(), power: Value::ONE, toughness: Value::ZERO, duration: Duration::EndOfTurn },
            Effect::Fight { attacker: Selector::Target(0), defender: theirs() },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::ExcessDamageDealtThisTurnTo(Box::new(Selector::Target(1))), Value::ONE),
                then: Box::new(Effect::ReflexiveTrigger {
                    // "Up to one": optional, so it never destroys its controller's own.
                    body: Box::new(Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::Destroy {
                            what: target_filtered(R::Artifact.and(R::Not(Box::new(R::Creature))).and(R::ManaValueAtMost(3))),
                        }),
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Sméagol, Helpful Guide ──────────────────────────────────────────────────

/// Sméagol, Helpful Guide — {1}{B}{G} legendary 4/2 Halfling Horror. Your end
/// step, if a creature of yours died this turn: the Ring tempts you. Whenever
/// the Ring tempts you, an opponent (the most hostile, not targeted) reveals
/// until a land; you get the land tapped, the rest go to their graveyard.
pub fn smeagol_helpful_guide() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer).with_filter(
                    Predicate::CreaturesDiedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
                ),
                effect: Effect::RingTempts { who: PlayerRef::You },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::RingTempted, EventScope::YourControl),
                effect: Effect::RevealUntilFind {
                    who: PlayerRef::HostileOpponent,
                    find: R::Land,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    cap: Value::Const(1000),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::Graveyard,
                },
            },
        ],
        ..creature("Sméagol, Helpful Guide", cost(&[generic(1), b(), g()]), vec![CreatureType::Halfling, CreatureType::Horror], 4, 2)
    }
}

/// Mirrormere Guardian — {2}{G} 4/2 Dwarf Soldier. Dies: the Ring tempts you.
pub fn mirrormere_guardian() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::RingTempts { who: PlayerRef::You })],
        ..creature("Mirrormere Guardian", cost(&[generic(2), g()]), vec![CreatureType::Dwarf, CreatureType::Soldier], 4, 2)
    }
}

// ── The Destined Thief ──────────────────────────────────────────────────────

fn full_party() -> Predicate {
    Predicate::ValueAtLeast(Value::PartyCount, Value::Const(4))
}

/// The Destined Thief — {2}{U} legendary 2/2 unblockable Human Rogue. {U},
/// {T}: another target creature you control can't be blocked this turn.
/// One or more of your creatures dealing combat damage to one or more
/// players: loot, or draw three with a full party.
pub fn the_destined_thief() -> CardDefinition {
    let draw = |n| Effect::Draw { who: Selector::You, amount: Value::Const(n) };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Unblockable],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch_across_players(),
            effect: Effect::If {
                cond: full_party(),
                then: Box::new(draw(3)),
                else_: Box::new(Effect::Seq(vec![
                    draw(1),
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ])),
            },
        }],
        ..creature("The Destined Thief", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 2)
    }
}

/// Nimble Trapfinder — {1}{U} 2/1 Human Rogue. Unblockable if another Cleric,
/// Rogue, Warrior or Wizard entered under your control this turn. At the
/// beginning of combat on your turn, with a full party, your creatures gain
/// "combat damage to a player: draw a card" until end of turn.
pub fn nimble_trapfinder() -> CardDefinition {
    let roles = [CreatureType::Cleric, CreatureType::Rogue, CreatureType::Warrior, CreatureType::Wizard];
    let entered = Value::Sum(roles.iter().map(|&t| Value::OtherCreaturesOfTypeEnteredThisTurn(t)).collect());
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature can't be blocked if you had another Cleric, Rogue, Warrior, or Wizard enter the battlefield under your control this turn.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Unblockable,
                condition: Predicate::ValueAtLeast(entered, Value::ONE),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer).with_filter(full_party()),
            effect: Effect::GrantTriggeredAbility {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                }),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Nimble Trapfinder", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 1)
    }
}

// ── Tresserhorn's Lord, Returned ────────────────────────────────────────────

/// Tresserhorn's Lord, Returned — {2}{U}{B}{R} legendary 10/4 trample,
/// indestructible Zombie Noble. Enters: sacrifice it unless you sacrifice
/// three other creatures, pay 3 life and have target opponent draw three.
/// Whenever you sacrifice a creature, draw a card.
pub fn tresserhorns_lord_returned() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample, Keyword::Indestructible],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::SacrificeSourceUnlessSacrifice { filter: R::Creature.and(R::OtherThanSource), count: 3 },
                Effect::If {
                    cond: Predicate::SourceOnBattlefield,
                    then: Box::new(Effect::Seq(vec![
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
                        Effect::Draw { who: target_filtered(R::OpponentPlayer), amount: Value::Const(3) },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..creature(
            "Tresserhorn's Lord, Returned",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Zombie, CreatureType::Noble],
            10,
            4,
        )
    }
}

// ── Aboshan, Cephalid Emperor ───────────────────────────────────────────────

/// Psychic Pickpocket — {4}{U} 3/2 Octopus Rogue. Enters: it connives; when it
/// does, return up to one target nonland permanent to its owner's hand.
pub fn psychic_pickpocket() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Connive { what: Selector::This, amount: Value::ONE },
            Effect::ReflexiveTrigger {
                // "Up to one": optional, so it never bounces its controller's own.
                body: Box::new(Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Permanent.and(R::Not(Box::new(R::Land)))),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                }),
            },
        ]))],
        ..creature("Psychic Pickpocket", cost(&[generic(4), u()]), vec![CreatureType::Octopus, CreatureType::Rogue], 3, 2)
    }
}

/// Dismiss into Dream — {6}{U} enchantment. Creatures your opponents control
/// are Illusions in addition to their other types and have "When this
/// creature becomes the target of a spell or ability, sacrifice it."
pub fn dismiss_into_dream() -> CardDefinition {
    let theirs = || R::Creature.and(R::ControlledByOpponent);
    CardDefinition {
        name: "Dismiss into Dream",
        cost: cost(&[generic(6), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Each creature your opponents control is an Illusion in addition to its other types.",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: Selector::EachPermanent(theirs()),
                    creature_type: CreatureType::Illusion,
                },
            },
            StaticAbility {
                description: "Each creature your opponents control has \"When this creature becomes the target of a spell or ability, sacrifice it.\"",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: theirs(),
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
                        effect: Effect::SacrificeSource,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

// ── King of the Oathbreakers ────────────────────────────────────────────────

/// King of the Oathbreakers — {2}{W}{B} legendary 3/3 flying Spirit Noble. A
/// Spirit you control becoming the target of a spell phases out; a Spirit you
/// control phasing in makes a tapped 1/1 white flying Spirit.
pub fn king_of_the_oathbreakers() -> CardDefinition {
    let spirit = || Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Spirit) };
    let token = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::YourCreatureTargeted)
                    .caused_by(R::IsSpellOnStack)
                    .with_filter(spirit()),
                effect: Effect::PhaseOut { what: Selector::TriggerSource, until_source_leaves: false },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PhasesIn, EventScope::YourControl).with_filter(spirit()),
                effect: Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(token) },
                    Effect::Tap { what: Selector::LastCreatedToken },
                ]),
            },
        ],
        ..creature("King of the Oathbreakers", cost(&[generic(2), w(), b()]), vec![CreatureType::Spirit, CreatureType::Noble], 3, 3)
    }
}

/// Cauldron Haze — {1}{W/B} instant. Any number of target creatures gain
/// persist until end of turn.
pub fn cauldron_haze() -> CardDefinition {
    CardDefinition {
        name: "Cauldron Haze",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Persist, duration: Duration::EndOfTurn }),
        },
        ..Default::default()
    }
}
