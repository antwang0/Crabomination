//! Commander: the cards the **Blood Rites** precon (The Lost Caverns of Ixalan
//! Commander, Clavileño) needed beyond what the catalog had. Built on the
//! Vampire shapes in `cmdr_edgar`. Tests in `tests/recent_b/cmdr_clavileno.rs`.

use super::cmdr_edgar::{
    creature, draw, legend, mint_one, vampire, vampire_demon, white_lifelinker,
};
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, exalted, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, StaticAbility, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, generic, w};
use std::sync::Arc;

fn to_battlefield(tapped: bool) -> ZoneDest {
    ZoneDest::Battlefield { controller: PlayerRef::You, tapped }
}

/// Martyr of Dusk — {1}{W} Creature — Vampire Soldier 2/1. When this creature
/// dies, create a 1/1 white Vampire creature token with lifelink.
pub fn martyr_of_dusk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(mint_one(white_lifelinker()))],
        ..creature(
            "Martyr of Dusk",
            cost(&[generic(1), w()]),
            2,
            1,
            vec![CreatureType::Vampire, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Vona, Butcher of Magan — {3}{W}{B} Legendary Creature — Vampire Knight 4/4.
/// Vigilance, lifelink. {T}, Pay 7 life: Destroy target nonland permanent.
/// Activate only during your turn.
pub fn vona_butcher_of_magan() -> CardDefinition {
    legend(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 7,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Destroy { what: target_filtered(R::Nonland) },
            ..Default::default()
        }],
        ..creature(
            "Vona, Butcher of Magan",
            cost(&[generic(3), w(), b()]),
            4,
            4,
            vec![CreatureType::Vampire, CreatureType::Knight],
            vec![Keyword::Vigilance, Keyword::Lifelink],
        )
    })
}

/// Bloodline Necromancer — {4}{B} Creature — Vampire Wizard 3/2. Lifelink.
/// When this creature enters, you may return target Vampire or Wizard creature
/// card from your graveyard to the battlefield.
pub fn bloodline_necromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return a Vampire or Wizard creature card to the battlefield?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(vampire().or(R::HasCreatureType(CreatureType::Wizard)))
                        .and(R::InYourGraveyard),
                ),
                to: to_battlefield(false),
            }),
        })],
        ..creature(
            "Bloodline Necromancer",
            cost(&[generic(4), b()]),
            3,
            2,
            vec![CreatureType::Vampire, CreatureType::Wizard],
            vec![Keyword::Lifelink],
        )
    }
}

/// Bloodtracker — {3}{B} Creature — Vampire Wizard 2/2. Flying. {B}, Pay 2
/// life: Put a +1/+1 counter on this creature. When this creature leaves the
/// battlefield, draw a card for each +1/+1 counter on it.
pub fn bloodtracker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            life_cost: 2,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            // CR 603.10 — the counters are read off its last-known information.
            effect: draw(
                Selector::You,
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
            ),
        }],
        ..creature(
            "Bloodtracker",
            cost(&[generic(3), b()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Wizard],
            vec![Keyword::Flying],
        )
    }
}

/// Dusk Legion Sergeant — {1}{B} Creature — Vampire Soldier 2/2. Menace. {1}{B},
/// Sacrifice this creature: Each nontoken Vampire creature you control gains
/// persist until end of turn.
pub fn dusk_legion_sergeant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Creature.and(vampire()).and(R::NotToken),
                },
                keyword: Keyword::Persist,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Dusk Legion Sergeant",
            cost(&[generic(1), b()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Soldier],
            vec![Keyword::Menace],
        )
    }
}

/// Order of Sacred Dusk — {6}{W}{B} Creature — Vampire Knight 5/5. Convoke.
/// Flying, lifelink, haste. Exalted. Other Vampires you control have exalted.
pub fn order_of_sacred_dusk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![exalted()],
        static_abilities: vec![StaticAbility {
            description: "Other Vampires you control have exalted.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(vampire()).and(R::ControlledByYou).and(R::OtherThanSource),
                ability: Box::new(exalted()),
            },
        }],
        ..creature(
            "Order of Sacred Dusk",
            cost(&[generic(6), w(), b()]),
            5,
            5,
            vec![CreatureType::Vampire, CreatureType::Knight],
            vec![Keyword::Convoke, Keyword::Flying, Keyword::Lifelink, Keyword::Haste],
        )
    }
}

/// Redemption Choir — {2}{W}{W} Creature — Vampire Cleric 3/3. Lifelink. Coven —
/// Whenever this creature enters or attacks, if you control three or more
/// creatures with different powers, return target permanent card with mana
/// value 3 or less from your graveyard to the battlefield.
pub fn redemption_choir() -> CardDefinition {
    let coven = || Predicate::CovenActive { who: PlayerRef::You };
    let body = || Effect::Move {
        what: target_filtered(R::PermanentCard.and(R::ManaValueAtMost(3)).and(R::InYourGraveyard)),
        to: to_battlefield(false),
    };
    let enters = etb(body());
    let attacks = on_attack(body());
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility { event: enters.event.with_filter(coven()), ..enters },
            TriggeredAbility { event: attacks.event.with_filter(coven()), ..attacks },
        ],
        ..creature(
            "Redemption Choir",
            cost(&[generic(2), w(), w()]),
            3,
            3,
            vec![CreatureType::Vampire, CreatureType::Cleric],
            vec![Keyword::Lifelink],
        )
    }
}

/// Timothar, Baron of Bats — {4}{B}{B} Legendary Creature — Vampire Noble 4/4.
/// Ward—Discard a card. Whenever another nontoken Vampire you control dies, you
/// may pay {1} and exile it. If you do, create a 1/1 black Bat creature token
/// with flying. It gains "When this token deals combat damage to a player,
/// sacrifice it and return the exiled card to the battlefield tapped."
///
/// The dead Vampire is exiled linked to the Bat (`ExileLinkedTo`), so the Bat's
/// own trigger finds it; it returns under your control.
pub fn timothar_baron_of_bats() -> CardDefinition {
    let bat = TokenDefinition {
        name: "Bat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::CardExiledWithSource, to: to_battlefield(true) },
                Effect::SacrificeSource,
            ]),
        }],
        ..Default::default()
    };
    legend(CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Discard(1))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: vampire().and(R::NotToken).and(R::OtherThanSource),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {1} to exile it and make a Bat?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(bat),
                    },
                    Effect::ExileLinkedTo {
                        what: Selector::TriggerSource,
                        link: Selector::LastCreatedToken,
                    },
                ])),
                else_: None,
            },
        }],
        ..creature(
            "Timothar, Baron of Bats",
            cost(&[generic(4), b(), b()]),
            4,
            4,
            vec![CreatureType::Vampire, CreatureType::Noble],
            vec![],
        )
    })
}

/// Promise of Aclazotz — {1}{B} Enchantment. At the beginning of your end step,
/// you may sacrifice a non-Demon creature. If you do, populate. // Foul Rebirth
/// — {2}{B} Sorcery — Adventure. Sacrifice a non-Demon creature. If you do,
/// create a 4/3 white and black Vampire Demon creature token with flying.
pub fn promise_of_aclazotz() -> CardDefinition {
    let non_demon = || R::Creature.and(R::HasCreatureType(CreatureType::Demon).negate());
    CardDefinition {
        name: "Promise of Aclazotz",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a non-Demon creature to populate?".into(),
                filter: non_demon(),
                count: Value::ONE,
                then: Box::new(Effect::Populate { who: PlayerRef::You }),
                else_: None,
            },
        }],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Foul Rebirth",
            cost: cost(&[generic(2), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: non_demon() },
                Effect::If {
                    cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                    then: Box::new(mint_one(vampire_demon(false))),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        })),
        ..Default::default()
    }
}
