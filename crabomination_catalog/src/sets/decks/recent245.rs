//! MKM (Murders at Karlov Manor) gap batch — the Clue Equipment cycle, the
//! "whenever you collect evidence" payoffs, and Detective value creatures.
//! Tests in `tests/recent_b/recent245.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, investigate, on_attack, on_attack_loot, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{b, cost, g, generic, r, u, w, ManaCost};

/// The shared "{2}, Sacrifice this Equipment: Draw a card." line on the four
/// MKM Clue Equipment.
fn sac_draw() -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        sac_cost: true,
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        ..Default::default()
    }
}

/// A Clue Equipment shell: the Clue+Equipment type line, the sac-to-draw
/// ability, and a printed Equip cost. Callers fill in the `equipped_bonus`.
fn clue_equipment(name: &'static str, mv: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mv,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Clue, ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        activated_abilities: vec![sac_draw()],
        ..Default::default()
    }
}

/// Wrench — {W} Clue Equipment. Equipped creature gets +1/+1, vigilance, and
/// "{3}, {T}: Tap target creature." Sac to draw. Equip {2}.
pub fn wrench() -> CardDefinition {
    clue_equipment(
        "Wrench",
        cost(&[w()]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Vigilance],
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::Tap { what: target_filtered(crate::card::SelectionRequirement::Creature) },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Rope — {G} Clue Equipment. Equipped creature gets +1/+2, reach, and can't be
/// blocked by more than one creature. Sac to draw. Equip {3}.
pub fn rope() -> CardDefinition {
    clue_equipment(
        "Rope",
        cost(&[g()]),
        cost(&[generic(3)]),
        EquipBonus {
            power: 1,
            toughness: 2,
            keywords: vec![Keyword::Reach, Keyword::CantBeBlockedByMoreThanOne],
            ..Default::default()
        },
    )
}

/// Knife — {R} Clue Equipment. During your turn, equipped creature gets +1/+0
/// and has first strike. Sac to draw. Equip {2}.
pub fn knife() -> CardDefinition {
    clue_equipment(
        "Knife",
        cost(&[r()]),
        cost(&[generic(2)]),
        EquipBonus {
            during_your_turn_pt: (1, 0),
            during_your_turn_keywords: vec![Keyword::FirstStrike],
            ..Default::default()
        },
    )
}

/// Candlestick — {U} Clue Equipment. Equipped creature gets +1/+1 and has
/// "Whenever this creature attacks, surveil 2." Sac to draw. Equip {2}.
pub fn candlestick() -> CardDefinition {
    clue_equipment(
        "Candlestick",
        cost(&[u()]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_attack(Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            })],
            ..Default::default()
        },
    )
}

/// Thinking Cap — {1} Equipment. Equipped creature gets +1/+2. Equip {3}
/// (the "Equip Detective {1}" discount is approximated by the flat Equip {3}).
pub fn thinking_cap() -> CardDefinition {
    CardDefinition {
        name: "Thinking Cap",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 2, ..Default::default() }),
        ..Default::default()
    }
}

/// A 1/1 colorless Thopter artifact creature with flying.
fn thopter_token() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Surveillance Monitor — {3}{U} Creature — Vedalken Detective 3/3. ETB you may
/// collect evidence 4. Whenever you collect evidence, create a 1/1 Thopter.
pub fn surveillance_monitor() -> CardDefinition {
    CardDefinition {
        name: "Surveillance Monitor",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Detective],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::CollectEvidence { amount: Value::Const(4), then: Box::new(Effect::Noop) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EvidenceCollected, EventScope::YourControl),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    definition: thopter_token(),
                    count: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Evidence Examiner — {G}{U} Creature — Merfolk Detective 2/2. Beginning of
/// combat on your turn, you may collect evidence 4. Whenever you collect
/// evidence, investigate.
pub fn evidence_examiner() -> CardDefinition {
    CardDefinition {
        name: "Evidence Examiner",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::CollectEvidence {
                    amount: Value::Const(4),
                    then: Box::new(Effect::Noop),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EvidenceCollected, EventScope::YourControl),
                effect: investigate(1),
            },
        ],
        ..Default::default()
    }
}

/// Unscrupulous Agent — {1}{B} Creature — Elf Detective 1/1. ETB target opponent
/// exiles a card from their hand.
pub fn unscrupulous_agent() -> CardDefinition {
    CardDefinition {
        name: "Unscrupulous Agent",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ExileFromHand {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Furtive Courier — {2}{U} Creature — Merfolk Advisor 3/2. Can't be blocked as
/// long as you've sacrificed an artifact this turn. Whenever it attacks, draw a
/// card, then discard a card.
pub fn furtive_courier() -> CardDefinition {
    use crate::card::{Keyword, SelectionRequirement as R, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Furtive Courier",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Advisor],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Can't be blocked while you've sacrificed an artifact this turn",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Unblockable,
                condition: R::ControllerSacrificedArtifactThisTurn,
            },
        }],
        triggered_abilities: vec![on_attack_loot()],
        ..Default::default()
    }
}

/// Undercity Eliminator — {3}{B}{B} Creature — Gorgon Assassin 3/3. ETB you may
/// sacrifice an artifact or creature; when you do, exile target creature an
/// opponent controls.
pub fn undercity_eliminator() -> CardDefinition {
    use crate::card::SelectionRequirement as R;
    CardDefinition {
        name: "Undercity Eliminator",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gorgon, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice an artifact or creature?".into(),
            filter: R::Artifact.or(R::Creature),
            count: Value::ONE,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::Exile {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                }),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}
