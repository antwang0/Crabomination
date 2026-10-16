//! Commander: the 2026-10-16 rescan's one-card gaps (Michelangelo Mutant BFF,
//! Spider-Man India, Spectacular Spider-Man, Lydia Frye, Bortuk Bonerattle,
//! Lozhan). Tests in `tests/recent_b/cmdr_oct16_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::card::{StaticAbility, StaticEffect};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::SpendRestriction;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, u, w};
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
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::LookPickToHand(Box::new(LookPick {
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
