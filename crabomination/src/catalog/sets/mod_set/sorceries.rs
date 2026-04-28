//! Modern-staple sorceries — sweepers, ramp, removal, recursion.

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Effect, SelectionRequirement, Subtypes};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::treasure_token;
use crate::mana::{ManaCost, b, cost, g, generic, r, u};

/// Anger of the Gods — {1}{R}{R} Sorcery. Deals 3 damage to each creature.
/// If a creature would die this turn, exile it instead.
///
/// Approximation: the "exile if would die" replacement is omitted (no
/// generic SBA-replacement primitive yet). Damage to each creature is
/// wired via `ForEach + DealDamage`.
pub fn anger_of_the_gods() -> CardDefinition {
    CardDefinition {
        name: "Anger of the Gods",
        cost: cost(&[generic(1), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(3),
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Disentomb — {B} Sorcery. Return target creature card from your graveyard
/// to your hand.
///
/// Approximation: the engine's target filter has no zone constraint, so
/// "from your graveyard" is dropped — any creature card the picker can
/// reference (battlefield / graveyard / exile / stack via the same
/// fallback Reanimate uses) is valid. The auto-target heuristic prefers
/// the caster's own graveyard creatures first.
pub fn disentomb() -> CardDefinition {
    CardDefinition {
        name: "Disentomb",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Vandalblast — {R} Sorcery. Destroy target artifact you don't control.
///
/// Approximation: the Overload {4}{R} mode (destroy each artifact you
/// don't control) is omitted — Overload as an alternate-cost mode isn't
/// modeled yet. Single-target version is wired faithfully.
pub fn vandalblast() -> CardDefinition {
    CardDefinition {
        name: "Vandalblast",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Nature's Lore — {1}{G} Sorcery. Search your library for a Forest card,
/// put it onto the battlefield, then shuffle.
///
/// `LandType::Forest` is the predicate; destination is
/// `ZoneDest::Battlefield { tapped: false }`. The engine's `Search`
/// primitive shuffles the library implicitly after the move.
pub fn natures_lore() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Nature's Lore",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Land
                .and(SelectionRequirement::HasLandType(LandType::Forest)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Fell — {2}{B} Sorcery. Destroy target tapped creature, then surveil 2.
///
/// The "tapped" predicate uses `SelectionRequirement::Tapped`; the surveil
/// half rides on the existing primitive (interactive when `wants_ui`,
/// otherwise auto-decided to keep cards on top).
pub fn fell() -> CardDefinition {
    CardDefinition {
        name: "Fell",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::Tapped),
                ),
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Blasphemous Edict — {4}{B} Sorcery. Each player sacrifices a creature.
/// Reuses `Effect::Sacrifice` with `Selector::Player(EachPlayer)`; the
/// resolver iterates each alive seat and forces one creature sacrifice
/// per player matching the filter.
pub fn blasphemous_edict() -> CardDefinition {
    CardDefinition {
        name: "Blasphemous Edict",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Upheaval — {4}{U}{U} Sorcery. Return all permanents to their owners'
/// hands. Implemented as `ForEach` over every battlefield permanent
/// (resolved once at the start) with a `Move → Hand(OwnerOf(Self))` body.
pub fn upheaval() -> CardDefinition {
    CardDefinition {
        name: "Upheaval",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Any),
            body: Box::new(Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Rakshasa's Bargain — {4}{B} Sorcery. Real Oracle: as an additional
/// cost, exile a creature card from your graveyard or pay 4 life. Draw
/// four cards.
///
/// Approximation: collapses the modal additional cost into a flat 4 life
/// payment (the more common play). The "exile a creature" alternative
/// would need a multi-mode additional-cost primitive that isn't modeled.
pub fn rakshasas_bargain() -> CardDefinition {
    CardDefinition {
        name: "Rakshasa's Bargain",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(4) },
            Effect::Draw { who: Selector::You, amount: Value::Const(4) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Sundering Eruption — {1}{R} Sorcery. Sundering Eruption deals 3 damage
/// to target creature or planeswalker. Modal-double-faced; the back face
/// (Mount Tyrhus) is a Mountain that enters tapped and taps for {R}.
///
/// The front (Sorcery) is cast normally; the back is played via
/// `GameAction::PlayLandBack`. The `back_face` slot only swaps in the
/// back's `CardDefinition` after `play_land_with_face` swaps faces, so
/// the front retains its sorcery effect when cast from hand.
pub fn sundering_eruption() -> CardDefinition {
    use crate::card::LandType;
    use super::super::etb_tap;
    let back = CardDefinition {
        name: "Mount Tyrhus",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Mountain],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![super::super::tap_add(crate::mana::Color::Red)],
        triggered_abilities: vec![etb_tap()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    };
    CardDefinition {
        name: "Sundering Eruption",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: Some(Box::new(back)),
        opening_hand: None,
    }
}

/// Big Score — {3}{R} Sorcery. As an additional cost, discard a card.
/// Create two Treasure tokens. Draw two cards.
///
/// The discard "additional cost" is folded into the resolution sequence
/// (the same simplification Thud uses for its sac-as-cost). Treasure
/// tokens are wired via the built-in `treasure_token()` helper, but note
/// the engine's `TokenDefinition` carries no activated abilities yet — so
/// the resulting Treasure tokens enter as colorless artifacts on the
/// battlefield without their canonical "{T}, sac: add one mana of any
/// color" ability. They count for spells/effects that key off "you
/// control an artifact" / "Treasures you control" but can't actually be
/// spent for mana until a sac-as-cost activation primitive lands.
pub fn big_score() -> CardDefinition {
    CardDefinition {
        name: "Big Score",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: treasure_token(),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Blasphemous Act — {8}{R} Sorcery, "this spell costs {1} less to cast for
/// each creature on the battlefield." Deals 13 damage to each creature.
///
/// Cost-reduction by creature-count is approximated as a flat {4}{R} cost
/// (a typical board state has 4–5 creatures across both players). The
/// damage half is wired faithfully via `ForEach + DealDamage`.
pub fn blasphemous_act() -> CardDefinition {
    CardDefinition {
        name: "Blasphemous Act",
        cost: cost(&[generic(4), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(13),
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
