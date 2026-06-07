use crate::card::{CardDefinition, CardType, Keyword, SelectionRequirement};
use crate::effect::shortcut::{deal, pump_target, return_target_to_hand, target, target_filtered};
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{ManaCost, b, colorless, cost, g, generic, r, u};
use crabomination_base::tokens::eldrazi_scion_token;

/// Murderous Compulsion — {1}{B} Sorcery. Destroy target tapped creature.
/// Madness {1}{B} (CR 702.35).
pub fn murderous_compulsion() -> CardDefinition {
    CardDefinition {
        name: "Murderous Compulsion",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Madness(ManaCost::new(vec![generic(1), b()]))],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
            ),
        },
        ..Default::default()
    }
}

/// Sweep Away — {2}{U} Instant. Return target creature to its owner's hand.
/// (The attacking-only "put it on top of its library instead" rider is
/// dropped — modal post-bounce choice not modeled.)
pub fn sweep_away() -> CardDefinition {
    CardDefinition {
        name: "Sweep Away",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: return_target_to_hand(),
        ..Default::default()
    }
}

/// Warping Wail — {1}{C} Devoid Instant. Choose one — exile target creature
/// with power or toughness 1 or less; counter target sorcery; or create a
/// 1/1 Eldrazi Scion.
pub fn warping_wail() -> CardDefinition {
    CardDefinition {
        name: "Warping Wail",
        cost: cost(&[generic(1), colorless(1)]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::ChooseMode(vec![
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature.and(
                    SelectionRequirement::PowerAtMost(1).or(SelectionRequirement::ToughnessAtMost(1)),
                )),
            },
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: eldrazi_scion_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Tar Snare — {2}{B} Devoid Instant. Target creature gets -3/-2 EOT.
pub fn tar_snare() -> CardDefinition {
    CardDefinition {
        name: "Tar Snare",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: pump_target(-3, -2),
        ..Default::default()
    }
}

/// Witness the End — {3}{B} Devoid Sorcery. Target opponent loses 2 life
/// and discards two cards. (The printed "exiles two cards from hand" is
/// approximated as a discard — no exile-from-hand-by-the-owner primitive.)
pub fn witness_the_end() -> CardDefinition {
    use crate::effect::Selector;
    CardDefinition {
        name: "Witness the End",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
                random: false,
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Scour from Existence — {7} Devoid Instant. Exile target permanent.
pub fn scour_from_existence() -> CardDefinition {
    CardDefinition {
        name: "Scour from Existence",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Permanent) },
        ..Default::default()
    }
}

/// Oblivion Strike — {3}{B} Devoid Sorcery. Exile target creature.
pub fn oblivion_strike() -> CardDefinition {
    CardDefinition {
        name: "Oblivion Strike",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Complete Disregard — {2}{B} Devoid Instant. Exile target creature with
/// power 3 or less.
pub fn complete_disregard() -> CardDefinition {
    CardDefinition {
        name: "Complete Disregard",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(3)),
            ),
        },
        ..Default::default()
    }
}

/// Spatial Contortion — {1}{C} Instant. Target creature gets +3/-3 EOT.
pub fn spatial_contortion() -> CardDefinition {
    CardDefinition {
        name: "Spatial Contortion",
        cost: cost(&[generic(1), colorless(1)]),
        card_types: vec![CardType::Instant],
        effect: pump_target(3, -3),
        ..Default::default()
    }
}

/// Unnatural Endurance — {B} Devoid Instant. Target creature gets +2/+0
/// until end of turn and is regenerated.
pub fn unnatural_endurance() -> CardDefinition {
    CardDefinition {
        name: "Unnatural Endurance",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            pump_target(2, 0),
            Effect::Regenerate { what: target() },
        ]),
        ..Default::default()
    }
}

/// Call the Scions — {2}{G} Devoid Sorcery. Create two 1/1 Eldrazi Scions.
pub fn call_the_scions() -> CardDefinition {
    CardDefinition {
        name: "Call the Scions",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: eldrazi_scion_token(),
        },
        ..Default::default()
    }
}

/// Reality Hemorrhage — {1}{R} Devoid Instant. Deals 2 damage to any target.
pub fn reality_hemorrhage() -> CardDefinition {
    CardDefinition {
        name: "Reality Hemorrhage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: deal(2, target()),
        ..Default::default()
    }
}

/// Touch of the Void — {2}{R} Devoid Sorcery. Deals 3 damage to any target.
/// (The "if a creature dies this turn, exile it" rider is dropped.)
pub fn touch_of_the_void() -> CardDefinition {
    CardDefinition {
        name: "Touch of the Void",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: deal(3, target()),
        ..Default::default()
    }
}
