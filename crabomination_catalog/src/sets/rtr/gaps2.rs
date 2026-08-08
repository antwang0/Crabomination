//! Return to Ravnica (RTR) gap wave 2: more commons/uncommons — vanilla and
//! french-vanilla creatures, Scavenge (`shortcut::scavenge`), simple activated
//! abilities, and Tavern Swindler's coin flip. Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword, LandType,
    SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::shortcut::{etb, on_dies, scavenge, target_filtered};
use crate::effect::{Duration, PlayerRef, Selector};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Rubbleback Rhino — {4}{G} 3/4 Rhino with hexproof.
pub fn rubbleback_rhino() -> CardDefinition {
    CardDefinition {
        name: "Rubbleback Rhino",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Hexproof],
        ..Default::default()
    }
}

/// Runewing — {3}{U} 2/2 Bird with flying. When it dies, draw a card.
pub fn runewing() -> CardDefinition {
    CardDefinition {
        name: "Runewing",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Sunspire Griffin — {1}{W}{W} 2/3 Griffin with flying.
pub fn sunspire_griffin() -> CardDefinition {
    CardDefinition {
        name: "Sunspire Griffin",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Tenement Crasher — {5}{R} 5/4 Beast with haste.
pub fn tenement_crasher() -> CardDefinition {
    CardDefinition {
        name: "Tenement Crasher",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Towering Indrik — {3}{G} 2/4 Beast with reach.
pub fn towering_indrik() -> CardDefinition {
    CardDefinition {
        name: "Towering Indrik",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Skyline Predator — {4}{U}{U} 3/4 Drake with flash and flying.
pub fn skyline_predator() -> CardDefinition {
    CardDefinition {
        name: "Skyline Predator",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        ..Default::default()
    }
}

/// Splatter Thug — {2}{R} 2/2 Human Warrior with first strike and unleash.
pub fn splatter_thug() -> CardDefinition {
    CardDefinition {
        name: "Splatter Thug",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Unleash],
        triggered_abilities: vec![crate::effect::shortcut::unleash()],
        ..Default::default()
    }
}

/// Thrill-Kill Assassin — {1}{B} 1/2 Human Assassin with deathtouch and unleash.
pub fn thrill_kill_assassin() -> CardDefinition {
    CardDefinition {
        name: "Thrill-Kill Assassin",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch, Keyword::Unleash],
        triggered_abilities: vec![crate::effect::shortcut::unleash()],
        ..Default::default()
    }
}

/// Perilous Shadow — {2}{B}{B} 0/4 Insect Shade. {1}{B}: +2/+2 until end of turn.
pub fn perilous_shadow() -> CardDefinition {
    CardDefinition {
        name: "Perilous Shadow",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Shade],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stonefare Crocodile — {2}{G} 3/2 Crocodile. {2}{B}: gains lifelink until end
/// of turn.
pub fn stonefare_crocodile() -> CardDefinition {
    CardDefinition {
        name: "Stonefare Crocodile",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Crocodile],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Selesnya Sentry — {2}{W} 3/2 Elephant Soldier. {5}{G}: regenerate this.
pub fn selesnya_sentry() -> CardDefinition {
    CardDefinition {
        name: "Selesnya Sentry",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stealer of Secrets — {2}{U} 2/2 Human Rogue. Whenever it deals combat damage
/// to a player, draw a card.
pub fn stealer_of_secrets() -> CardDefinition {
    CardDefinition {
        name: "Stealer of Secrets",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// 1/1 white Bird creature token with flying.
fn bird_token() -> TokenDefinition {
    TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Seller of Songbirds — {2}{W} 1/2 Human. When it enters, create a 1/1 white
/// Bird creature token with flying.
pub fn seller_of_songbirds() -> CardDefinition {
    CardDefinition {
        name: "Seller of Songbirds",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(bird_token()),
        })],
        ..Default::default()
    }
}

/// Keening Apparition — {1}{W} 2/2 Spirit. Sacrifice this: destroy target
/// enchantment.
pub fn keening_apparition() -> CardDefinition {
    CardDefinition {
        name: "Keening Apparition",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Enchantment),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Korozda Monitor — {2}{G}{G} 3/3 Lizard with trample. Scavenge {5}{G}{G}.
pub fn korozda_monitor() -> CardDefinition {
    CardDefinition {
        name: "Korozda Monitor",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![scavenge(cost(&[generic(5), g(), g()]))],
        ..Default::default()
    }
}

/// Sewer Shambler — {2}{B} 2/1 Zombie with swampwalk. Scavenge {2}{B}.
pub fn sewer_shambler() -> CardDefinition {
    CardDefinition {
        name: "Sewer Shambler",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        activated_abilities: vec![scavenge(cost(&[generic(2), b()]))],
        ..Default::default()
    }
}

/// Terrus Wurm — {6}{B} 5/5 Zombie Wurm. Scavenge {6}{B}.
pub fn terrus_wurm() -> CardDefinition {
    CardDefinition {
        name: "Terrus Wurm",
        cost: cost(&[generic(6), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        activated_abilities: vec![scavenge(cost(&[generic(6), b()]))],
        ..Default::default()
    }
}

/// Tavern Swindler — {1}{B} 2/2 Human Rogue. {T}, Pay 3 life: flip a coin;
/// if you win, gain 6 life.
pub fn tavern_swindler() -> CardDefinition {
    CardDefinition {
        name: "Tavern Swindler",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 3,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(6),
                }),
                on_tails: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
