//! A Wilds of Eldraine (WOE) wave: Adventure creatures, Royal Role tokens, and
//! Food/enchantment payoffs. Adds the `CantBeBlockedByPowerAtLeast` evasion
//! keyword (Squeak By). Tests in `crabomination/src/tests/recent132.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::game::effects::{food_token, treasure_token};
use crate::mana::{b, cost, g, generic, r, w};
use super::woe_roles::{royal_role};


// ── White ─────────────────────────────────────────────────────────────────────

/// Cheeky House-Mouse // Squeak By — {W} 2/1 Mouse; Adventure {W} Sorcery gives
/// target creature you control +1/+1 and "can't be blocked by power 3+" until
/// end of turn.
pub fn cheeky_house_mouse() -> CardDefinition {
    CardDefinition {
        name: "Cheeky House-Mouse",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mouse], ..Default::default() },
        power: 2,
        toughness: 1,
        adventure: Some(Box::new(Adventure {
            name: "Squeak By",
            cost: cost(&[w()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                filter: R::Creature.and(R::ControlledByYou),
                effect: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::CantBeBlockedByPowerAtLeast(3),
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        })),
        ..Default::default()
    }
}

/// Besotted Knight // Betroth the Beast — {3}{W} 3/3 Human Knight; Adventure {W}
/// Sorcery creates a Royal Role token attached to target creature you control.
pub fn besotted_knight() -> CardDefinition {
    CardDefinition {
        name: "Besotted Knight",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        adventure: Some(Box::new(Adventure {
            name: "Betroth the Beast",
            cost: cost(&[w()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                definition: royal_role(),
            },
        })),
        ..Default::default()
    }
}

/// Charmed Clothier — {4}{W} 3/3 Faerie Advisor with flying. ETB: create a Royal
/// Role token attached to another target creature you control.
pub fn charmed_clothier() -> CardDefinition {
    CardDefinition {
        name: "Charmed Clothier",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Advisor],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateTokenAttachedTo {
            target: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            definition: royal_role(),
        })],
        ..Default::default()
    }
}

// ── Black ─────────────────────────────────────────────────────────────────────

/// Ashiok's Reaper — {3}{B} 3/3 Nightmare. Whenever an enchantment you control
/// is put into a graveyard from the battlefield, draw a card.
pub fn ashioks_reaper() -> CardDefinition {
    CardDefinition {
        name: "Ashiok's Reaper",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

// ── Red ───────────────────────────────────────────────────────────────────────

/// Two-Headed Hunter // Twice the Rage — {4}{R} 5/4 Giant with menace; Adventure
/// {1}{R} Instant gives target creature double strike until end of turn.
pub fn two_headed_hunter() -> CardDefinition {
    CardDefinition {
        name: "Two-Headed Hunter",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        adventure: Some(Box::new(Adventure {
            name: "Twice the Rage",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Instant],
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        })),
        ..Default::default()
    }
}

/// Grabby Giant // That's Mine — {3}{R} 4/3 Giant with reach; {2}{R}, Sacrifice
/// an artifact or land: Draw a card. Adventure {1}{R} Instant creates a Treasure.
pub fn grabby_giant() -> CardDefinition {
    CardDefinition {
        name: "Grabby Giant",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_other_filter: Some((R::Artifact.or(R::Land), 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        adventure: Some(Box::new(Adventure {
            name: "That's Mine",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: treasure_token(),
            },
        })),
        ..Default::default()
    }
}

// ── Green ─────────────────────────────────────────────────────────────────────

/// Hollow Scavenger // Bakery Raid — {2}{G} 3/2 Wolf; {1}, Sacrifice a Food:
/// +2/+2 until end of turn (once each turn). Adventure {G} Sorcery creates a
/// Food.
pub fn hollow_scavenger() -> CardDefinition {
    CardDefinition {
        name: "Hollow Scavenger",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            once_per_turn: true,
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        adventure: Some(Box::new(Adventure {
            name: "Bakery Raid",
            cost: cost(&[g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: food_token() },
        })),
        ..Default::default()
    }
}

/// Redtooth Genealogist — {2}{G} 2/3 Elf Advisor. ETB: create a Royal Role token
/// attached to another target creature you control.
pub fn redtooth_genealogist() -> CardDefinition {
    CardDefinition {
        name: "Redtooth Genealogist",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateTokenAttachedTo {
            target: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            definition: royal_role(),
        })],
        ..Default::default()
    }
}

/// Skybeast Tracker — {3}{G} 2/4 Giant Archer with reach. Whenever you cast a
/// spell with mana value 5 or greater, create a Food token.
pub fn skybeast_tracker() -> CardDefinition {
    CardDefinition {
        name: "Skybeast Tracker",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::ManaValueAtLeast(5),
                }),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: food_token() },
        }],
        ..Default::default()
    }
}

/// Verdant Outrider — {2}{G} 4/2 Human Knight. {1}{G}: This creature can't be
/// blocked by creatures with power 2 or less this turn.
pub fn verdant_outrider() -> CardDefinition {
    CardDefinition {
        name: "Verdant Outrider",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::CantBeBlockedByPowerAtMost(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
