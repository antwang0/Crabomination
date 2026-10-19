//! Commander: 2026-10-19 (`01DJHkWk`) — five-gap decks of plain cards:
//! She-Hulk Jade Defender's green tricks, Agent Maria Hill's white removal,
//! Fangorn's Treefolk. Tests in `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, cost, g, generic, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn kindred_treefolk(def: CardDefinition) -> CardDefinition {
    let kind = def.card_types[0].clone();
    CardDefinition {
        card_types: vec![CardType::Kindred, kind],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        ..def
    }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT {
        what,
        power: Value::Const(p),
        toughness: Value::Const(t),
        duration: Duration::EndOfTurn,
    }
}

fn treefolk() -> R {
    R::HasCreatureType(CreatureType::Treefolk)
}

// ── She-Hulk, Jade Defender ─────────────────────────────────────────────────

/// Wary Okapi — {2}{G} 3/2 vigilance Antelope.
pub fn wary_okapi() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..creature(
            "Wary Okapi",
            cost(&[generic(2), g()]),
            vec![CreatureType::Antelope],
            3,
            2,
        )
    }
}

/// Bouncer's Beatdown — {2}{G} instant, {2} less aimed at a black permanent.
/// X damage to target creature or planeswalker, X the greatest power among
/// your creatures; exiled if it would die this turn.
pub fn bouncers_beatdown() -> CardDefinition {
    let it = || Selector::Target(0);
    CardDefinition {
        self_cost_reduction_if_target: Some((R::HasColor(Color::Black), 2)),
        ..spell(
            "Bouncer's Beatdown",
            cost(&[generic(2), g()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::ExileIfWouldDieThisTurn {
                    what: target_filtered(R::Creature.or(R::Planeswalker)),
                },
                Effect::DealDamage {
                    to: it(),
                    amount: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                },
            ]),
        )
    }
}

/// Dragonscale Boon — {3}{G} instant. Two +1/+1 counters on target creature;
/// untap it.
pub fn dragonscale_boon() -> CardDefinition {
    spell(
        "Dragonscale Boon",
        cost(&[generic(3), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
        ]),
    )
}

/// Thwart the Enemy — {2}{G} instant. Prevent all damage creatures your
/// opponents control would deal this turn.
pub fn thwart_the_enemy() -> CardDefinition {
    spell(
        "Thwart the Enemy",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::PreventAllDamageByMatchingThisTurn {
            filter: R::Creature.and(R::ControlledByOpponent),
        },
    )
}

/// Monstrous Growth — {1}{G} sorcery. Target creature gets +4/+4.
pub fn monstrous_growth() -> CardDefinition {
    spell(
        "Monstrous Growth",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        pump(target_filtered(R::Creature), 4, 4),
    )
}

// ── Agent Maria Hill ────────────────────────────────────────────────────────

/// Wooden Stake — {2} Equipment, +1/+0, equip {1}. Whenever the equipped
/// creature blocks or becomes blocked by a Vampire, destroy that creature
/// (no regeneration).
pub fn wooden_stake() -> CardDefinition {
    let stake = || Effect::DestroyNoRegen {
        what: Selector::EachPermanent(
            R::HasCreatureType(CreatureType::Vampire).and(R::InCombatWithSource),
        ),
    };
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            triggered_abilities: vec![
                TriggeredAbility {
                    event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                    effect: stake(),
                },
                TriggeredAbility {
                    event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                    effect: stake(),
                },
            ],
            ..Default::default()
        }),
        ..spell(
            "Wooden Stake",
            cost(&[generic(2)]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

/// Hieromancer's Cage — {3}{W} enchantment. Entering, exile target nonland
/// permanent an opponent controls until it leaves.
pub fn hieromancers_cage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        ..spell(
            "Hieromancer's Cage",
            cost(&[generic(3), w()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Helicarrier Strike — {W} instant, teamwork 2. 2 damage to target attacking
/// or blocking creature — 4 with teamwork.
pub fn helicarrier_strike() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower {
            filter: mine(),
            power: 2,
        }),
        ..spell(
            "Helicarrier Strike",
            cost(&[w()]),
            CardType::Instant,
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(4)),
                    else_: Box::new(Value::Const(2)),
                },
            },
        )
    }
}

/// Heroic Teamwork — {2}{W} instant, teamwork 3. One or two target creatures
/// each get +2/+1; with teamwork, draw a card.
pub fn heroic_teamwork() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower {
            filter: mine(),
            power: 3,
        }),
        ..spell(
            "Heroic Teamwork",
            cost(&[generic(2), w()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 1,
                    filter: R::Creature,
                    effect: Box::new(pump(Selector::Target(0), 2, 1)),
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Swift Reckoning — {1}{W} sorcery; spell mastery — flash with two or more
/// instants and/or sorceries in your graveyard. Destroy target tapped
/// creature.
pub fn swift_reckoning() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spell mastery — If there are two or more instant and/or sorcery cards in your graveyard, you may cast this spell as though it had flash.",
            effect: StaticEffect::SelfFlashIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery)),
                    },
                    n: Value::Const(2),
                },
            },
        }],
        ..spell(
            "Swift Reckoning",
            cost(&[generic(1), w()]),
            CardType::Sorcery,
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::Tapped)),
            },
        )
    }
}

// ── Fangorn, Tree Shepherd ──────────────────────────────────────────────────

/// Battlewand Oak — {2}{G} 1/3 Treefolk Warrior. A Forest of yours entering,
/// or your Treefolk spell, gives it +2/+2 until end of turn.
pub fn battlewand_oak() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasLandType(LandType::Forest),
                    }),
                effect: pump(Selector::This, 2, 2),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(treefolk())),
                effect: pump(Selector::This, 2, 2),
            },
        ],
        ..creature(
            "Battlewand Oak",
            cost(&[generic(2), g()]),
            vec![CreatureType::Treefolk, CreatureType::Warrior],
            1,
            3,
        )
    }
}

/// Everbark Shaman — {4}{G} 3/5 Treefolk Shaman. {T}, exile a Treefolk card
/// from your graveyard: up to two Forest cards from your library onto the
/// battlefield tapped.
pub fn everbark_shaman() -> CardDefinition {
    let forest = || Effect::Search {
        who: PlayerRef::You,
        filter: R::HasLandType(LandType::Forest),
        to: ZoneDest::Battlefield {
            controller: PlayerRef::You,
            tapped: true,
        },
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_other_filter: Some((treefolk().and(R::InYourGraveyard), 1)),
            effect: Effect::Seq(vec![forest(), forest()]),
            ..Default::default()
        }],
        ..creature(
            "Everbark Shaman",
            cost(&[generic(4), g()]),
            vec![CreatureType::Treefolk, CreatureType::Shaman],
            3,
            5,
        )
    }
}

/// Rootless Yew — {3}{G}{G} 5/4 Treefolk. Dying, search for a creature card
/// with power or toughness 6 or greater to hand.
pub fn rootless_yew() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Search {
            who: PlayerRef::You,
            filter: R::Creature.and(R::PowerAtLeast(6).or(R::ToughnessAtLeast(6))),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Rootless Yew",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Treefolk],
            5,
            4,
        )
    }
}

/// Rootgrapple — {4}{G} Kindred instant — Treefolk. Destroy target noncreature
/// permanent; draw a card if you control a Treefolk.
pub fn rootgrapple() -> CardDefinition {
    kindred_treefolk(spell(
        "Rootgrapple",
        cost(&[generic(4), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Permanent.and(R::Creature.negate())),
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: treefolk(),
                }),
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    ))
}

/// Tend the Sprigs — {2}{G} sorcery. A basic land onto the battlefield
/// tapped; then with seven or more lands and/or Treefolk, a 3/4 reach
/// Treefolk.
pub fn tend_the_sprigs() -> CardDefinition {
    let sprig = TokenDefinition {
        name: "Treefolk".into(),
        power: 3,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        keywords: vec![Keyword::Reach],
        ..Default::default()
    };
    spell(
        "Tend the Sprigs",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::Land.or(treefolk()),
                    },
                    n: Value::Const(7),
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(sprig),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}
