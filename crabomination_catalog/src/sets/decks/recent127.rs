//! An OTJ commons/uncommons wave: crime payoffs, Mounts (Saddle), Desert
//! painlands, reanimation and modal removal. Introduces two engine primitives —
//! `DynamicPt::CardsDrawnThisTurnPower` (Duelist of the Mind) and
//! `Predicate::SacrificedWasOutlaw` (Boneyard Desecrator). Tests in
//! `crabomination/src/tests/recent127.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{drain, etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::effects::treasure_token;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// A Desert dual land: enters tapped, ETB pings an opponent for 1, taps for
/// either of its two colors (the OTJ "painland Desert" cycle).
fn desert_painland(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Desert],
            ..Default::default()
        },
        activated_abilities: vec![crate::sets::tap_add(a), crate::sets::tap_add(b)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::This,
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

pub fn bristling_backwoods() -> CardDefinition {
    desert_painland("Bristling Backwoods", Color::Red, Color::Green)
}

pub fn abraded_bluffs() -> CardDefinition {
    desert_painland("Abraded Bluffs", Color::Red, Color::White)
}

pub fn creosote_heath() -> CardDefinition {
    desert_painland("Creosote Heath", Color::Green, Color::White)
}

pub fn eroded_canyon() -> CardDefinition {
    desert_painland("Eroded Canyon", Color::Blue, Color::Red)
}

pub fn festering_gulch() -> CardDefinition {
    desert_painland("Festering Gulch", Color::Black, Color::Green)
}

pub fn forlorn_flats() -> CardDefinition {
    desert_painland("Forlorn Flats", Color::White, Color::Black)
}

pub fn jagged_barrens() -> CardDefinition {
    desert_painland("Jagged Barrens", Color::Black, Color::Red)
}

pub fn lonely_arroyo() -> CardDefinition {
    desert_painland("Lonely Arroyo", Color::White, Color::Blue)
}

pub fn lush_oasis() -> CardDefinition {
    desert_painland("Lush Oasis", Color::Green, Color::Blue)
}

pub fn soured_springs() -> CardDefinition {
    desert_painland("Soured Springs", Color::Blue, Color::Black)
}

/// Daring Thunder-Thief — {3}{U} 4/4 Turtle Rogue. Flash; enters tapped.
pub fn daring_thunder_thief() -> CardDefinition {
    CardDefinition {
        name: "Daring Thunder-Thief",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Turtle, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        ..Default::default()
    }
}

/// Deepmuck Desperado — {2}{U} 2/4 Homarid Mercenary. Whenever you commit a
/// crime, each opponent mills three (once each turn).
pub fn deepmuck_desperado() -> CardDefinition {
    CardDefinition {
        name: "Deepmuck Desperado",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Homarid, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}

/// Blood Hustler — {1}{B} 1/1 Vampire Rogue. Whenever you commit a crime, put a
/// +1/+1 counter on it (once each turn). {3}{B}: drain 1.
pub fn blood_hustler() -> CardDefinition {
    CardDefinition {
        name: "Blood Hustler",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            effect: drain(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blacksnag Buzzard — {2}{B} 2/1 Bird with flying, Plot {1}{B}. Enters with a
/// +1/+1 counter if a creature died this turn.
pub fn blacksnag_buzzard() -> CardDefinition {
    CardDefinition {
        name: "Blacksnag Buzzard",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        plot_cost: Some(cost(&[generic(1), b()])),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::ONE,
            },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Cactusfolk Sureshot — {2}{R}{G} 4/4 Plant Mercenary. Reach, Ward {2}. At
/// combat on your turn, other creatures you control with power 4+ gain trample
/// and haste until end of turn.
pub fn cactusfolk_sureshot() -> CardDefinition {
    use crate::game::types::TurnStep;
    let others_pw4 = Selector::EachPermanent(
        R::Creature
            .and(R::ControlledByYou)
            .and(R::OtherThanSource)
            .and(R::PowerAtLeast(4)),
    );
    CardDefinition {
        name: "Cactusfolk Sureshot",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Reach,
            Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: others_pw4.clone(),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: others_pw4,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Congregation Gryff — {1}{G}{W} 1/4 Hippogriff Mount. Flying, lifelink,
/// Saddle 3. Whenever it attacks while saddled, +X/+X where X = Mounts you
/// control.
pub fn congregation_gryff() -> CardDefinition {
    let mounts = Value::CountOf(Box::new(Selector::ControlledBy {
        who: PlayerRef::You,
        filter: R::HasCreatureType(CreatureType::Mount),
    }));
    CardDefinition {
        name: "Congregation Gryff",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hippogriff, CreatureType::Mount],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink, Keyword::Saddle(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::SourceSaddled),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: mounts.clone(),
                toughness: mounts,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Duelist of the Mind — {1}{U} */3 Human Advisor. Flying, vigilance; power =
/// cards you've drawn this turn. Crime → loot 1 (once each turn).
pub fn duelist_of_the_mind() -> CardDefinition {
    CardDefinition {
        name: "Duelist of the Mind",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        toughness: 3,
        dynamic_pt: Some(DynamicPt::CardsDrawnThisTurnPower { base_t: 3 }),
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Boneyard Desecrator — {3}{B} 3/4 Zombie Mercenary. Menace. {1}{B}, Sacrifice
/// another creature: +1/+1 counter; if an outlaw was sacrificed, make a Treasure.
pub fn boneyard_desecrator() -> CardDefinition {
    CardDefinition {
        name: "Boneyard Desecrator",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::SacrificedWasOutlaw,
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: treasure_token(),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skulduggery — {B} Instant. Target creature you control gets +1/+1 and target
/// creature an opponent controls gets -1/-1 until end of turn.
pub fn skulduggery() -> CardDefinition {
    CardDefinition {
        name: "Skulduggery",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Frontier Seeker — {1}{W} 2/1 Human Scout. ETB: look at the top five cards;
/// you may put a Mount or Plains card from among them into your hand.
pub fn frontier_seeker() -> CardDefinition {
    CardDefinition {
        name: "Frontier Seeker",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: Some(
                R::HasCreatureType(CreatureType::Mount).or(R::HasLandType(LandType::Plains)),
            ),
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: true,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
            rest_to_exile: false,
        })],
        ..Default::default()
    }
}

/// Colossal Rattlewurm — {2}{G}{G} 6/5 Wurm with trample and flash while you
/// control a Desert. {1}{G}, Exile this card from your graveyard: search your
/// library for a Desert card, put it onto the battlefield tapped, then shuffle.
pub fn colossal_rattlewurm() -> CardDefinition {
    CardDefinition {
        name: "Colossal Rattlewurm",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Colossal Rattlewurm has flash as long as you control a Desert.",
            effect: StaticEffect::SelfFlashIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::HasLandType(LandType::Desert),
                    },
                    n: Value::ONE,
                },
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Desert),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Badlands Revival — {3}{B}{G} Sorcery. Return up to one target creature card
/// from your graveyard to the battlefield, and up to one target permanent card
/// from your graveyard to your hand.
pub fn badlands_revival() -> CardDefinition {
    CardDefinition {
        name: "Badlands Revival",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Permanent.and(R::InYourGraveyard),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Betrayal at the Vault — {4}{G}{G} Instant. Target creature you control deals
/// damage equal to its power to each of two other target creatures.
pub fn betrayal_at_the_vault() -> CardDefinition {
    let power_of_0 = Value::PowerOf(Box::new(Selector::Target(0)));
    CardDefinition {
        name: "Betrayal at the Vault",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByYou)),
                amount: Value::ZERO,
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature,
                },
                amount: power_of_0.clone(),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 2,
                    filter: R::Creature,
                },
                amount: power_of_0,
            },
        ]),
        ..Default::default()
    }
}

/// Dust Animus — {1}{W} 2/3 Spirit with flying, Plot {1}{W}. If you control five
/// or more untapped lands, it enters with two +1/+1 counters. (The lifelink
/// counter is approximated as an extra +1/+1 — no keyword-counter primitive.)
pub fn dust_animus() -> CardDefinition {
    CardDefinition {
        name: "Dust Animus",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        plot_cost: Some(cost(&[generic(1), w()])),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::Land.and(R::ControlledByYou).and(R::Untapped)),
                n: Value::Const(5),
            },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Bandit's Haul — {3} Artifact. Whenever you commit a crime, put a loot counter
/// on it (once each turn). {T}: add any color. {2}, {T}, remove two loot
/// counters: draw a card. (Loot counters use the generic charge-counter store.)
pub fn bandits_haul() -> CardDefinition {
    CardDefinition {
        name: "Bandit's Haul",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 2)),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Claim Jumper — {2}{W} 3/3 Rabbit Mercenary with vigilance. ETB: if an
/// opponent controls more lands than you, search your library for a Plains and
/// put it onto the battlefield tapped. (The "repeat once" clause is dropped.)
pub fn claim_jumper() -> CardDefinition {
    CardDefinition {
        name: "Claim Jumper",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::OpponentControlsMoreLandsThanYou,
            then: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Plains),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Binding Negotiation — {1}{B} Sorcery. Target opponent reveals their hand; you
/// choose a nonland card from it and they discard it. (The face-up-exile
/// fallback is dropped.)
pub fn binding_negotiation() -> CardDefinition {
    CardDefinition {
        name: "Binding Negotiation",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::ONE,
            filter: R::Nonland,
        },
        ..Default::default()
    }
}
