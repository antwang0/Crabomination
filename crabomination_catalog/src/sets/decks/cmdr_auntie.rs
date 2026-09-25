//! Commander: the cards the **Blight Curse** precon (ECC, Auntie Ool,
//! Cursewretch) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_auntie.rs`.
//!
//! Residuals (each also on its card):
//! - **Eventide's Shadow** — each chosen permanent loses all its counters.
//! - **Puca's Covenant** — the returned card is chosen on resolution (not
//!   targeted), and the dying creature's own card is among the choices.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, mono_hybrid, r, Color, ManaCost};

use super::super::tap_add_colorless;

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

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn minus(what: Selector, n: i32) -> Effect {
    Effect::AddCounter { what, kind: CounterType::MinusOneMinusOne, amount: Value::Const(n) }
}

fn with_minus() -> R {
    R::WithCounter(CounterType::MinusOneMinusOne)
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

/// "Whenever a creature [scope] with a -1/-1 counter on it dies" — the
/// dying creature's last-known counters (CR 603.10).
fn dies_with_minus(scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::CreatureDied, scope).with_filter(trigger_is(with_minus()))
}

/// "Whenever one or more -1/-1 counters are put on a creature".
fn minus_counters_put() -> EventSpec {
    EventSpec::new(EventKind::CounterAdded(CounterType::MinusOneMinusOne), EventScope::AnyPlayer)
        .with_filter(trigger_is(R::Creature))
}

fn your_graveyard(filter: R) -> Selector {
    Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter }
}

fn to_battlefield(tapped: bool) -> ZoneDest {
    ZoneDest::Battlefield { controller: PlayerRef::You, tapped }
}

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(def: TokenDefinition, count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(def) }
}

fn may(description: &str, body: Effect) -> Effect {
    Effect::MayDo { description: description.into(), body: Box::new(body) }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// The Reaper, King No More — {2/B}{2/R}{2/G} Legendary Artifact Creature —
/// Scarecrow 3/3. When The Reaper enters, put a -1/-1 counter on each of up
/// to two target creatures. Whenever a creature an opponent controls with a
/// -1/-1 counter on it dies, you may put that card onto the battlefield under
/// your control. Do this only once each turn.
pub fn the_reaper_king_no_more() -> CardDefinition {
    let mut steal = TriggeredAbility {
        event: dies_with_minus(EventScope::OpponentControl),
        effect: may("Put that creature onto the battlefield under your control?", Effect::Move {
            what: Selector::TriggerSource,
            to: to_battlefield(false),
        }),
    };
    steal.event.once_per_turn = true;
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 2,
                filter: R::Creature,
                effect: Box::new(minus(Selector::Target(0), 1)),
            }),
            steal,
        ],
        ..legend(
            "The Reaper, King No More",
            cost(&[mono_hybrid(2, Color::Black), mono_hybrid(2, Color::Red), mono_hybrid(2, Color::Green)]),
            vec![CreatureType::Scarecrow],
            3,
            3,
        )
    }
}

/// Carnifex Demon — {4}{B}{B} Creature — Phyrexian Demon 6/6. Flying. Enters
/// with two -1/-1 counters. {B}, Remove a -1/-1 counter from this creature:
/// Put a -1/-1 counter on each other creature.
pub fn carnifex_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            remove_counter_cost: Some((CounterType::MinusOneMinusOne, 1)),
            effect: minus(Selector::EachPermanent(R::Creature.and(R::OtherThanSource)), 1),
            ..Default::default()
        }],
        ..creature(
            "Carnifex Demon",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Demon],
            6,
            6,
        )
    }
}

/// Channeler Initiate — {1}{G} Creature — Human Druid 3/4. When it enters, put
/// three -1/-1 counters on target creature you control. {T}, Remove a -1/-1
/// counter from this creature: Add one mana of any color.
pub fn channeler_initiate() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(minus(target_filtered(R::Creature.and(R::ControlledByYou)), 3))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::MinusOneMinusOne, 1)),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..creature(
            "Channeler Initiate",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            3,
            4,
        )
    }
}

/// Dread Tiller — {1}{B}{G} Artifact Creature — Scarecrow 2/4. When it
/// enters, put a -1/-1 counter on target creature. Whenever a creature with a
/// -1/-1 counter on it dies, you may put a land card from your hand or
/// graveyard onto the battlefield tapped.
pub fn dread_tiller() -> CardDefinition {
    let lands_in = |zone| Selector::CardsInZone { who: PlayerRef::You, zone, filter: R::Land };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![
            etb(minus(target_filtered(R::Creature), 1)),
            TriggeredAbility {
                event: dies_with_minus(EventScope::AnyPlayer),
                effect: Effect::MoveChosen {
                    from: Selector::Both(
                        Box::new(lands_in(crate::card::Zone::Hand)),
                        Box::new(lands_in(crate::card::Zone::Graveyard)),
                    ),
                    filter: None,
                    count: Value::ONE,
                    up_to: true,
                    to: to_battlefield(true),
                },
            },
        ],
        ..creature("Dread Tiller", cost(&[generic(1), b(), g()]), vec![CreatureType::Scarecrow], 2, 4)
    }
}

/// Dusk Urchins — {2}{B} Creature — Ouphe 4/3. Whenever it attacks or blocks,
/// put a -1/-1 counter on it. When it dies, draw a card for each -1/-1 counter
/// on it.
pub fn dusk_urchins() -> CardDefinition {
    let wilt = || minus(Selector::This, 1);
    CardDefinition {
        triggered_abilities: vec![
            on_attack(wilt()),
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: wilt() },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: draw(Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::MinusOneMinusOne,
                }),
            },
        ],
        ..creature("Dusk Urchins", cost(&[generic(2), b()]), vec![CreatureType::Ouphe], 4, 3)
    }
}

/// Ferrafor, Young Yew — {6}{G} Legendary Creature — Treefolk Druid 4/7. When
/// Ferrafor enters, create a 1/1 green Saproling for each counter among
/// creatures target player controls. {T}: Double the number of each kind of
/// counter on target creature.
pub fn ferrafor_young_yew() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(make(
                token("Saproling", vec![Color::Green], vec![CreatureType::Saproling], 1, 1),
                Value::TotalCountersOn {
                    what: Box::new(Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature }),
                },
            )),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DoubleAllCountersOn { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..legend(
            "Ferrafor, Young Yew",
            cost(&[generic(6), g()]),
            vec![CreatureType::Treefolk, CreatureType::Druid],
            4,
            7,
        )
    }
}

/// Grim Poppet — {7} Artifact Creature — Scarecrow 4/4. Enters with three
/// -1/-1 counters. Remove a -1/-1 counter from it: Put a -1/-1 counter on
/// another target creature.
pub fn grim_poppet() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::MinusOneMinusOne, 1)),
            effect: minus(target_filtered(R::Creature.and(R::OtherThanSource)), 1),
            ..Default::default()
        }],
        ..creature("Grim Poppet", cost(&[generic(7)]), vec![CreatureType::Scarecrow], 4, 4)
    }
}

/// Hapatra, Vizier of Poisons — {B}{G} Legendary Creature — Human Cleric 2/2.
/// Whenever Hapatra deals combat damage to a player, you may put a -1/-1
/// counter on target creature. Whenever you put one or more -1/-1 counters on
/// a creature, create a 1/1 green Snake creature token with deathtouch.
pub fn hapatra_vizier_of_poisons() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: may("Put a -1/-1 counter on target creature?", minus(target_filtered(R::Creature), 1)),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::MinusOneMinusOne), EventScope::YouPutCounters)
                    .with_filter(trigger_is(R::Creature)),
                effect: make(
                    TokenDefinition {
                        keywords: vec![Keyword::Deathtouch],
                        ..token("Snake", vec![Color::Green], vec![CreatureType::Snake], 1, 1)
                    },
                    Value::ONE,
                ),
            },
        ],
        ..legend(
            "Hapatra, Vizier of Poisons",
            cost(&[b(), g()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Kulrath Knight — {3}{B/R}{B/R} Creature — Elemental Knight 3/3. Flying,
/// wither. Creatures your opponents control with counters on them can't
/// attack or block.
pub fn kulrath_knight() -> CardDefinition {
    let marked = || Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent).and(R::WithAnyCounter));
    let br = || hybrid(Color::Black, Color::Red);
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Wither],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures your opponents control with counters on them can't attack.",
                effect: StaticEffect::GrantKeyword { applies_to: marked(), keyword: Keyword::CantAttack },
            },
            StaticAbility {
                description: "Creatures your opponents control with counters on them can't block.",
                effect: StaticEffect::GrantKeyword { applies_to: marked(), keyword: Keyword::CantBlock },
            },
        ],
        ..creature(
            "Kulrath Knight",
            cost(&[generic(3), br(), br()]),
            vec![CreatureType::Elemental, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Midnight Banshee — {3}{B}{B}{B} Creature — Spirit 5/5. Wither. At the
/// beginning of your upkeep, put a -1/-1 counter on each nonblack creature.
pub fn midnight_banshee() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Wither],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: minus(
                Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
                1,
            ),
        }],
        ..creature("Midnight Banshee", cost(&[generic(3), b(), b(), b()]), vec![CreatureType::Spirit], 5, 5)
    }
}

/// Necroskitter — {1}{B}{B} Creature — Elemental 1/4. Wither. Whenever a
/// creature an opponent controls with a -1/-1 counter on it dies, you may
/// return that card to the battlefield under your control.
pub fn necroskitter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Wither],
        triggered_abilities: vec![TriggeredAbility {
            event: dies_with_minus(EventScope::OpponentControl),
            effect: may("Return that creature under your control?", Effect::Move {
                what: Selector::TriggerSource,
                to: to_battlefield(false),
            }),
        }],
        ..creature("Necroskitter", cost(&[generic(1), b(), b()]), vec![CreatureType::Elemental], 1, 4)
    }
}

/// Oft-Nabbed Goat — {1}{B} Creature — Goat 0/5. {1}: Draw a card. Gain
/// control of this creature and put a -1/-1 counter on it. Only your opponents
/// may activate this ability and only as a sorcery. When it dies, if it had
/// one or more -1/-1 counters on it, its owner draws that many cards and each
/// other player loses that much life.
pub fn oft_nabbed_goat() -> CardDefinition {
    let owner = || PlayerRef::OwnerOf(Box::new(Selector::This));
    let counters = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::MinusOneMinusOne };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            opponents_only: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                draw(Value::ONE),
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::You),
                    duration: crate::effect::Duration::Permanent,
                },
                minus(Selector::This, 1),
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource)
                .with_filter(trigger_is(with_minus())),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(owner()), amount: counters() },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::OpponentOf(Box::new(owner()))),
                    amount: counters(),
                },
            ]),
        }],
        ..creature("Oft-Nabbed Goat", cost(&[generic(1), b()]), vec![CreatureType::Goat], 0, 5)
    }
}

/// Soul Snuffers — {2}{B}{B} Creature — Elemental Shaman 3/3. When it enters,
/// put a -1/-1 counter on each creature.
pub fn soul_snuffers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(minus(Selector::EachPermanent(R::Creature), 1))],
        ..creature(
            "Soul Snuffers",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Elemental, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// The Scorpion God — {3}{B}{R} Legendary Creature — God 6/5. Whenever a
/// creature with a -1/-1 counter on it dies, draw a card. {1}{B}{R}: Put a
/// -1/-1 counter on another target creature. When The Scorpion God dies,
/// return it to its owner's hand at the beginning of the next end step.
pub fn the_scorpion_god() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility { event: dies_with_minus(EventScope::AnyPlayer), effect: draw(Value::ONE) },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::AtNextEndStep {
                    body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), r()]),
            effect: minus(target_filtered(R::Creature.and(R::OtherThanSource)), 1),
            ..Default::default()
        }],
        ..legend("The Scorpion God", cost(&[generic(3), b(), r()]), vec![CreatureType::God], 6, 5)
    }
}

/// Tree of Perdition — {3}{B} Creature — Plant 0/13. Defender. {T}: Exchange
/// target opponent's life total with this creature's toughness.
pub fn tree_of_perdition() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::ExchangePlayerLifeWithSourceToughness { who: PlayerRef::Target(0) }),
            },
            ..Default::default()
        }],
        ..creature("Tree of Perdition", cost(&[generic(3), b()]), vec![CreatureType::Plant], 0, 13)
    }
}

/// Village Pillagers — {3}{R}{R} Creature — Goblin Warrior 5/5. Wither. When
/// it enters, it deals 1 damage to each creature your opponents control.
/// Whenever a creature an opponent controls with a counter on it dies, you
/// create a tapped Treasure token.
pub fn village_pillagers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Wither],
        triggered_abilities: vec![
            etb(Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::ONE,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl)
                    .with_filter(trigger_is(R::WithAnyCounter)),
                effect: make(TokenDefinition { tapped: true, ..crabomination_base::tokens::treasure_token() }, Value::ONE),
            },
        ],
        ..creature(
            "Village Pillagers",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            5,
            5,
        )
    }
}

// ── Noncreature permanents ──────────────────────────────────────────────────

/// Liliana, Death Wielder — {5}{B}{B} Legendary Planeswalker — Liliana,
/// loyalty 5. +2: Put a -1/-1 counter on up to one target creature. −3:
/// Destroy target creature with a -1/-1 counter on it. −10: Return all
/// creature cards from your graveyard to the battlefield.
pub fn liliana_death_wielder() -> CardDefinition {
    CardDefinition {
        name: "Liliana, Death Wielder",
        cost: cost(&[generic(5), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Liliana], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 1,
                    filter: R::Creature,
                    effect: Box::new(minus(Selector::Target(0), 1)),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Destroy { what: target_filtered(R::Creature.and(with_minus())) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::Move { what: your_graveyard(R::Creature), to: to_battlefield(false) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Blowfly Infestation — {2}{B} Enchantment. Whenever a creature dies, if it
/// had a -1/-1 counter on it, put a -1/-1 counter on target creature.
pub fn blowfly_infestation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: dies_with_minus(EventScope::AnyPlayer),
            effect: minus(target_filtered(R::Creature), 1),
        }],
        ..enchantment("Blowfly Infestation", cost(&[generic(2), b()]))
    }
}

/// Flourishing Defenses — {4}{G} Enchantment. Whenever a -1/-1 counter is put
/// on a creature, you may create a 1/1 green Elf Warrior creature token.
pub fn flourishing_defenses() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: minus_counters_put(),
            // One token per counter: the event carries how many were put.
            effect: may(
                "Create an Elf Warrior for each -1/-1 counter?",
                make(
                    token("Elf Warrior", vec![Color::Green], vec![CreatureType::Elf, CreatureType::Warrior], 1, 1),
                    Value::TriggerEventAmount,
                ),
            ),
        }],
        ..enchantment("Flourishing Defenses", cost(&[generic(4), g()]))
    }
}

/// Grave Venerations — {3}{B} Enchantment. When it enters, you become the
/// monarch. At the beginning of your end step, if you're the monarch, return
/// up to one target creature card from your graveyard to your hand. Whenever
/// a creature you control dies, each opponent loses 1 life and you gain 1
/// life.
pub fn grave_venerations() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::IsMonarch { who: PlayerRef::You }),
                effect: Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 1,
                    filter: R::Creature.and(R::InYourGraveyard),
                    effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ]),
            },
        ],
        ..enchantment("Grave Venerations", cost(&[generic(3), b()]))
    }
}

/// Lasting Tarfire — {1}{R} Enchantment. At the beginning of each end step, if
/// you put a counter on a creature this turn, it deals 2 damage to each
/// opponent.
pub fn lasting_tarfire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::CounterPutOnCreatureThisTurn),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
        }],
        ..enchantment("Lasting Tarfire", cost(&[generic(1), r()]))
    }
}

/// Puca's Covenant — {2}{G} Enchantment. Whenever a creature you control with
/// a counter on it dies, you may return another target permanent card with
/// mana value less than or equal to the number of counters on that creature
/// from your graveyard to your hand. Do this only once each turn.
///
/// ⚠ Residual: the card is chosen on resolution, not targeted, and the dying
/// creature's own card is among the choices.
pub fn pucas_covenant() -> CardDefinition {
    let mut trigger = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
            .with_filter(trigger_is(R::WithAnyCounter)),
        effect: Effect::WithX {
            x: Value::TotalCountersOn { what: Box::new(Selector::TriggerSource) },
            body: Box::new(Effect::MoveChosen {
                from: your_graveyard(R::PermanentCard.and(R::ManaValueAtMostXFromCost)),
                filter: None,
                count: Value::ONE,
                up_to: true,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
    };
    trigger.event.once_per_turn = true;
    CardDefinition { triggered_abilities: vec![trigger], ..enchantment("Puca's Covenant", cost(&[generic(2), g()])) }
}

/// Wickersmith's Tools — {3} Artifact. Whenever one or more -1/-1 counters
/// are put on a creature, put a charge counter on this artifact. {T}: Add one
/// mana of any color. {5}, {T}, Sacrifice this artifact: Create X tapped 2/2
/// colorless Scarecrow artifact creature tokens, where X is the number of
/// charge counters on this artifact.
pub fn wickersmiths_tools() -> CardDefinition {
    CardDefinition {
        name: "Wickersmith's Tools",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: minus_counters_put(),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                sac_cost: true,
                effect: make(
                    TokenDefinition {
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        tapped: true,
                        ..token("Scarecrow", vec![], vec![CreatureType::Scarecrow], 2, 2)
                    },
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                ),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ifnir Deadlands — Land — Desert. {T}: Add {C}. {T}, Pay 1 life: Add {B}.
/// {2}{B}{B}, {T}, Sacrifice a Desert: Put two -1/-1 counters on target
/// creature an opponent controls. Activate only as a sorcery.
pub fn ifnir_deadlands() -> CardDefinition {
    CardDefinition {
        name: "Ifnir Deadlands",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Desert], ..Default::default() },
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Black]) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b(), b()]),
                tap_cost: true,
                sorcery_speed: true,
                sac_other_filter: Some((R::HasLandType(LandType::Desert), 1)),
                sac_other_may_be_source: true,
                effect: minus(target_filtered(R::Creature.and(R::ControlledByOpponent)), 2),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Instants and sorceries ──────────────────────────────────────────────────

/// Aberrant Return — {4}{B}{B} Sorcery. Put one, two, or three target creature
/// cards from graveyards onto the battlefield under your control. Each of them
/// enters with an additional -1/-1 counter on it.
pub fn aberrant_return() -> CardDefinition {
    spell(
        "Aberrant Return",
        cost(&[generic(4), b(), b()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            min_targets: 1,
            max_targets: 3,
            filter: R::Creature.and(R::InGraveyard),
            effect: Box::new(Effect::Seq(vec![
                Effect::Move { what: Selector::Target(0), to: to_battlefield(false) },
                minus(Selector::Target(0), 1),
            ])),
        },
    )
}

/// Burning Curiosity — {2}{R} Sorcery. As an additional cost, you may blight 1.
/// Exile the top two cards of your library (three if the blight was paid).
/// Until the end of your next turn, you may play those cards.
pub fn burning_curiosity() -> CardDefinition {
    let impulse = |n| Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::Const(n),
        duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    };
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::Blight { n: 1 }),
        ..spell(
            "Burning Curiosity",
            cost(&[generic(2), r()]),
            CardType::Sorcery,
            Effect::If { cond: Predicate::SpellWasKicked, then: Box::new(impulse(3)), else_: Box::new(impulse(2)) },
        )
    }
}

/// Eventide's Shadow — {4}{B} Sorcery. Remove any number of counters from among
/// permanents on the battlefield. You draw cards and lose life equal to the
/// number of counters removed this way.
///
/// ⚠ Residual: each chosen permanent loses all its counters.
pub fn eventides_shadow() -> CardDefinition {
    spell(
        "Eventide's Shadow",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::RemoveCountersFromAmongDrawAndLoseLife,
    )
}

/// Fire Covenant — {1}{B}{R} Instant. As an additional cost, pay X life. Fire
/// Covenant deals X damage divided as you choose among any number of target
/// creatures.
pub fn fire_covenant() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::PayLifeX],
        ..spell(
            "Fire Covenant",
            cost(&[generic(1), b(), r()]),
            CardType::Instant,
            Effect::DealDamageDivided {
                total: Value::XFromCost,
                filter: R::Creature,
                max_targets: 8,
                retaliate_to_source: false,
            },
        )
    }
}

/// Hoarder's Greed — {3}{B} Sorcery. You lose 2 life and draw two cards, then
/// clash with an opponent. If you win, repeat this process.
pub fn hoarders_greed() -> CardDefinition {
    spell(
        "Hoarder's Greed",
        cost(&[generic(3), b()]),
        CardType::Sorcery,
        Effect::RepeatWhileClashWon {
            body: Box::new(Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                draw(Value::Const(2)),
            ])),
        },
    )
}

/// Incremental Blight — {3}{B}{B} Sorcery. Put a -1/-1 counter on target
/// creature, two -1/-1 counters on another target creature, and three -1/-1
/// counters on a third target creature.
pub fn incremental_blight() -> CardDefinition {
    let slot = |n: u8, filter: R| Selector::TargetFiltered { slot: n, filter };
    spell(
        "Incremental Blight",
        cost(&[generic(3), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            minus(target_filtered(R::Creature), 1),
            minus(slot(1, R::Creature.and(R::OtherThanTargetSlot(0))), 2),
            minus(slot(2, R::Creature.and(R::OtherThanTargetSlot(0)).and(R::OtherThanTargetSlot(1))), 3),
        ]),
    )
}
