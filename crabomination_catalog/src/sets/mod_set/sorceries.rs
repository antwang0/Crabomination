//! Modern-staple sorceries — sweepers, ramp, removal, recursion.

use crate::card::{
    CardDefinition, CardType, Effect, Keyword, LandType, SelectionRequirement, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::treasure_token;
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w};

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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Sweltering Suns — {1}{R}{R} Sorcery. Deals 3 damage to each creature.
/// Cycling {3}. Damage to each creature via `ForEach + DealDamage`; the
/// Cycling keyword rides the engine's existing cycling path.
pub fn sweltering_suns() -> CardDefinition {
    CardDefinition {
        name: "Sweltering Suns",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(3)]))],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(3),
            }),
        },
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Vandalblast — {R} Sorcery. Destroy target artifact you don't control.
/// Overload {4}{R}: destroy each artifact you don't control.
pub fn vandalblast() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Vandalblast",
        cost: cost(&[r()]),
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
        triggered_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(4), r()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
            exile_from_graveyard_count: 0,
            return_to_hand: None,
            sacrifice_permanents: None,
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Artifact
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::Destroy {
                    what: Selector::TriggerSource,
                }),
            }),
        }),
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fell — {1}{B} Sorcery. Destroy target tapped creature, then surveil 2.
///
/// The "tapped" predicate uses `SelectionRequirement::Tapped`; the surveil
/// half rides on the existing primitive (interactive when `wants_ui`,
/// otherwise auto-decided to keep cards on top).
pub fn fell() -> CardDefinition {
    CardDefinition {
        name: "Fell",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Upheaval — {4}{U}{U} Sorcery. Return all permanents to their owners'
/// hands. Implemented as `ForEach` over every battlefield permanent
/// (resolved once at the start) with a `Move → Hand(OwnerOf(Self))` body.
pub fn upheaval() -> CardDefinition {
    CardDefinition {
        name: "Upheaval",
        cost: cost(&[generic(4), u(), u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(4) },
            Effect::Draw { who: Selector::You, amount: Value::Const(4) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Sundering Eruption — {2}{R} Sorcery. Sundering Eruption deals 3 damage
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
        ..Default::default()
    };
    CardDefinition {
        name: "Sundering Eruption",
        // Real Oracle: `{1}{R}` Sorcery (DSK).
        cost: cost(&[generic(1), r()]),
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
        triggered_abilities: vec![],
        back_face: Some(Box::new(back)),
        ..Default::default()
    }
}

/// Big Score — {3}{R} Sorcery. As an additional cost, discard a card.
/// Create two Treasure tokens. Draw two cards. Discard is a real cast-time
/// cost via `AdditionalCastCost::Discard`.
pub fn big_score() -> CardDefinition {
    CardDefinition {
        name: "Big Score",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::Discard { count: 1 }],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: treasure_token(),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Windfall — {2}{U} Sorcery. Each player discards their hand, then draws
/// cards equal to the greatest number of cards a player discarded.
///
/// Push (modern_decks batch 115): the "draws equal to the greatest
/// number of cards a player discarded this way" half is now wired
/// faithfully via the new `Value::MaxCardsDiscardedThisEffectByAnyPlayer`
/// primitive. The Discard step bumps a per-player counter
/// (`cards_discarded_per_player_this_resolution`); the follow-up Draw
/// step reads the max across all players. Identical pattern usable by
/// future Wheel of Fortune / Jace's Archivist-class effects that share
/// the "discard, then draw based on what was discarded" shape.
pub fn windfall() -> CardDefinition {
    CardDefinition {
        name: "Windfall",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(100),
                random: false,
            },
            Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::MaxCardsDiscardedThisEffectByAnyPlayer,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Reckless Charge — {R} Sorcery. Target creature gets +3/+0 and gains
/// haste until end of turn. Flashback {2}{R}.
///
/// `Seq([PumpPT(+3/+0 EOT), GrantKeyword(Haste, EOT)])` + `Keyword::
/// Flashback({2}{R})`. The engine's existing flashback path lets the
/// caster cast from graveyard for the flashback cost; the body is
/// identical on both casts.
pub fn reckless_charge() -> CardDefinition {
    let flashback_cost = ManaCost {
        symbols: vec![
            crate::mana::ManaSymbol::Generic(2),
            crate::mana::ManaSymbol::Colored(crate::mana::Color::Red),
        ],
    };
    CardDefinition {
        name: "Reckless Charge",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Boil — {2}{R} Sorcery. Destroy all Islands.
///
/// `ForEach(EachPermanent(HasLandType(Island))) → Destroy(TriggerSource)`.
/// Hits every Island on the battlefield, regardless of controller.
pub fn boil() -> CardDefinition {
    CardDefinition {
        name: "Boil",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::HasLandType(LandType::Island)),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Compulsive Research — {2}{U} Sorcery. Target player draws three cards.
/// Then that player discards two cards unless they discard a land card.
///
/// Approximation: caster draws three then discards two (auto-decider
/// picks the first two non-land cards in hand). The "land instead of two"
/// branch is omitted.
pub fn compulsive_research() -> CardDefinition {
    CardDefinition {
        name: "Compulsive Research",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Demolish — {3}{R} Sorcery. Destroy target artifact or land.
pub fn demolish() -> CardDefinition {
    CardDefinition {
        name: "Demolish",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Land),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Mind Sculpt — {2}{U} Sorcery. Target opponent mills seven cards.
pub fn mind_sculpt() -> CardDefinition {
    CardDefinition {
        name: "Mind Sculpt",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Mill {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(7),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Cabal Therapy — {B} Sorcery. Choose a card name, then target player
/// reveals their hand and discards all copies of that card. Flashback —
/// sacrifice a creature.
///
/// Approximation: the auto-decider picks the highest-CMC nonland card in
/// the opponent's hand (via `DiscardChosen`), so the "name a card" half
/// collapses into "pick the most threatening card in their hand".
/// Flashback's "sacrifice a creature" cost is the engine's stock
/// flashback {0} cost — we don't yet model alt-costs that aren't mana.
pub fn cabal_therapy() -> CardDefinition {
    let flashback_cost = ManaCost { symbols: vec![] };
    CardDefinition {
        name: "Cabal Therapy",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Nonland,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Wear Down — {1}{W} Sorcery. Destroy target artifact and target
/// enchantment. (Renamed from real "Wear // Tear"; condensed to a single
/// half — modal-double-targeted sorceries are still future work.)
///
/// Approximation: a single-target Naturalize-style "destroy artifact or
/// enchantment" — the multi-target half waits on a multi-target spell
/// primitive.
pub fn wear_down() -> CardDefinition {
    CardDefinition {
        name: "Wear Down",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Dread Return — {2}{B}{B} Sorcery. "Return target creature card from
/// your graveyard to the battlefield. Flashback — Sacrifice three
/// creatures."
///
/// `Move(target creature card → Battlefield)`. The free flashback cost
/// (`Keyword::Flashback({0})`) is gated on the name-keyed flashback
/// additional cost "sacrifice three creatures" (`cast_flashback` validates +
/// pays it; AutoDecider sacrifices the cheapest three).
pub fn dread_return() -> CardDefinition {
    CardDefinition {
        name: "Dread Return",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(crate::mana::ManaCost::default())],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        ..Default::default()
    }
}

/// Archdruid's Charm — {G}{G}{G} Instant.
///
/// Oracle: Choose one — Search your library for a creature card, reveal it,
/// put it into your hand / Put two +1/+1 counters on target creature you
/// control / Destroy target artifact or non-Forest land an opponent controls.
///
/// All three modes wired via ChooseMode.
pub fn archdruids_charm() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Archdruid's Charm",
        cost: cost(&[g(), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Land)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Awaken the Honored Dead — {5}{W}{B} Sorcery.
///
/// Oracle: "Return all creature cards from your graveyard to the battlefield."
///
/// Mass reanimation; approximation of the printed text's "all creature cards."
pub fn awaken_the_honored_dead() -> CardDefinition {
    CardDefinition {
        name: "Awaken the Honored Dead",
        cost: cost(&[generic(5), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::EachMatching {
                zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                filter: SelectionRequirement::Creature,
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        ..Default::default()
    }
}



// ── Explore ───────────────────────────────────────────────────────────────

/// Explore — {G} Sorcery. "You may play an additional land this turn.
/// Draw a card."
pub fn explore() -> CardDefinition {
    use crate::effect::{PlayerRef, Value};
    CardDefinition {
        name: "Explore",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GrantExtraLandPlay {
                who: PlayerRef::You,
                count: Value::Const(1),
            },
            Effect::Draw {
                who: crate::effect::Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}


/// Arc Trail — {1}{R} Sorcery. "Arc Trail deals 2 damage to any target and
/// 1 damage to another target." Two target slots: slot 0 takes 2, slot 1
/// takes 1.
pub fn arc_trail() -> CardDefinition {
    CardDefinition {
        name: "Arc Trail",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Any },
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Prey Upon — {G} Sorcery. "Target creature you control fights target
/// creature you don't control." Both slots fight via `Effect::Fight`.
pub fn prey_upon() -> CardDefinition {
    CardDefinition {
        name: "Prey Upon",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Fight {
            attacker: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            },
            defender: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            },
        },
        ..Default::default()
    }
}
