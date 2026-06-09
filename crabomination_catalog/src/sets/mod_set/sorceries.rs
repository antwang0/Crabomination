//! Modern-staple sorceries — sweepers, ramp, removal, recursion.

use crate::card::{
    CardDefinition, CardType, Effect, Keyword, LandType, SelectionRequirement, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::treasure_token;
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w, x};

/// Anger of the Gods — {1}{R}{R} Sorcery. Deals 3 damage to each creature.
/// If a creature would die this turn, exile it instead.
///
/// Installs the "exile if would die this turn" death replacement on every
/// creature first, then deals the 3 damage — so creatures it kills are
/// exiled rather than buried (`Effect::ExileIfWouldDieThisTurn`).
pub fn anger_of_the_gods() -> CardDefinition {
    CardDefinition {
        name: "Anger of the Gods",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: Selector::EachPermanent(SelectionRequirement::Creature),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(3),
                }),
            },
        ]),
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

/// Chandra's Pyrohelix — {1}{R} Sorcery. Deals 2 damage divided as you
/// choose among one or two targets.
pub fn chandras_pyrohelix() -> CardDefinition {
    CardDefinition {
        name: "Chandra's Pyrohelix",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamageDivided {
            total: Value::Const(2),
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::Planeswalker),
            max_targets: 2,
        },
        ..Default::default()
    }
}

/// Forked Lightning — {3}{R} Sorcery. Deals 4 damage divided as you choose
/// among one, two, or three target creatures.
pub fn forked_lightning() -> CardDefinition {
    CardDefinition {
        name: "Forked Lightning",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamageDivided {
            total: Value::Const(4),
            filter: SelectionRequirement::Creature,
            max_targets: 3,
        },
        ..Default::default()
    }
}

/// Arc Lightning — {2}{R} Sorcery. Deals 3 damage divided as you choose
/// among one, two, or three targets (creatures and/or players).
pub fn arc_lightning() -> CardDefinition {
    CardDefinition {
        name: "Arc Lightning",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamageDivided {
            total: Value::Const(3),
            filter: SelectionRequirement::Creature.or(SelectionRequirement::Player),
            max_targets: 3,
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
            dash: false,
            blitz: false,
            flash: false,
            marks_kicked: false,
            emerge: None,
            impending: 0,
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
        cost: cost(&[generic(3), b(), b()]),
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

/// Rakshasa's Bargain — {2/B}{2/G}{2/U} Instant. Look at the top four cards
/// of your library. Put two of them into your hand and the rest into your
/// graveyard.
pub fn rakshasas_bargain() -> CardDefinition {
    use crate::mana::{mono_hybrid, Color};
    CardDefinition {
        name: "Rakshasa's Bargain",
        cost: cost(&[
            mono_hybrid(2, Color::Black),
            mono_hybrid(2, Color::Green),
            mono_hybrid(2, Color::Blue),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: true,
            pick_filter: None,
            take: Some(Value::Const(2)),
        },
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
/// The cost reduction rides the card-intrinsic Affinity hook
/// (`affinity_filter: Creature`) so it really shrinks by {1} per creature
/// on the battlefield; the damage half is `ForEach + DealDamage`.
pub fn blasphemous_act() -> CardDefinition {
    CardDefinition {
        name: "Blasphemous Act",
        cost: cost(&[generic(8), r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        affinity_filter: Some(SelectionRequirement::Creature),
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

/// Cone of Flame — {3}{R}{R} Sorcery. 1/2/3 damage to three distinct any
/// targets (slots 0/1/2). Extends the Arc Trail per-slot-fixed-damage shape
/// to three slots.
pub fn cone_of_flame() -> CardDefinition {
    CardDefinition {
        name: "Cone of Flame",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Any },
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 2, filter: SelectionRequirement::Any },
                amount: Value::Const(3),
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

/// Inspired Charge — {2}{W}{W} Sorcery. "Creatures you control get +2/+1
/// until end of turn." (M14)
pub fn inspired_charge() -> CardDefinition {
    CardDefinition {
        name: "Inspired Charge",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(2),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Servo Exhibition — {1}{W} Sorcery. "Create two 1/1 colorless Servo
/// artifact creature tokens." (KLD)
pub fn servo_exhibition() -> CardDefinition {
    CardDefinition {
        name: "Servo Exhibition",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: crate::card::TokenDefinition {
                name: "Servo".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Artifact, CardType::Creature],
                colors: vec![],
                subtypes: Subtypes {
                    creature_types: vec![crate::card::CreatureType::Servo],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Fire Ambush — {1}{R} Sorcery. "Fire Ambush deals 3 damage to any target."
/// (M19)
pub fn fire_ambush() -> CardDefinition {
    CardDefinition {
        name: "Fire Ambush",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Repay in Kind — {5}{B}{B} Sorcery. "Each player's life total becomes the
/// lowest life total among all players." (CR 119.7 — `SetLifeTotal` over
/// every player, evaluated once via `Value::LowestLifeTotal`.)
pub fn repay_in_kind() -> CardDefinition {
    CardDefinition {
        name: "Repay in Kind",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SetLifeTotal {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::LowestLifeTotal,
        },
        ..Default::default()
    }
}

/// Sizzle — {2}{R} Sorcery. "Sizzle deals 3 damage to each opponent."
pub fn sizzle() -> CardDefinition {
    CardDefinition {
        name: "Sizzle",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Sunlance — {W} Sorcery. "Sunlance deals 3 damage to target nonwhite creature."
pub fn sunlance() -> CardDefinition {
    CardDefinition {
        name: "Sunlance",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::Not(Box::new(
                    SelectionRequirement::HasColor(crate::mana::Color::White),
                ))),
            ),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Epic Confrontation — {1}{G} Sorcery. "Target creature you control gets
/// +1/+2 until end of turn. It fights target creature you don't control."
pub fn epic_confrontation() -> CardDefinition {
    CardDefinition {
        name: "Epic Confrontation",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(1),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Renegade Tactics — {R} Sorcery. "Target creature can't block this turn.
/// Draw a card."
pub fn renegade_tactics() -> CardDefinition {
    CardDefinition {
        name: "Renegade Tactics",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Ancient Craving — {3}{B} Sorcery. You draw three cards and you lose 3 life.
pub fn ancient_craving() -> CardDefinition {
    CardDefinition {
        name: "Ancient Craving",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Zombify — {3}{B} Sorcery. Return target creature card from your graveyard
/// to the battlefield.
pub fn zombify() -> CardDefinition {
    CardDefinition {
        name: "Zombify",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Lead the Stampede — {2}{G} Sorcery. Look at the top five cards of your
/// library; put all creature cards among them into your hand and the rest on
/// the bottom in a random order.
pub fn lead_the_stampede() -> CardDefinition {
    CardDefinition {
        name: "Lead the Stampede",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealTopTakeMatchingToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            filter: SelectionRequirement::Creature,
        },
        ..Default::default()
    }
}

/// Commune with Nature — {G} Sorcery. Look at the top five cards of your
/// library. You may put a creature card from among them into your hand. Put
/// the rest on the bottom in a random order.
pub fn commune_with_nature() -> CardDefinition {
    CardDefinition {
        name: "Commune with Nature",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: Some(SelectionRequirement::Creature),
            take: None,
        },
        ..Default::default()
    }
}

/// Fireball — {X}{R} Sorcery. Deals X damage to any target. (The "divide
/// among additional targets for {1} more each" rider is not modeled; the
/// common single-target line is faithful.)
pub fn fireball() -> CardDefinition {
    CardDefinition {
        name: "Fireball",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Disintegrate — {X}{R} Sorcery. Deals X damage to any target. If a creature
/// dealt damage this way would die this turn, exile it instead.
pub fn disintegrate() -> CardDefinition {
    CardDefinition {
        name: "Disintegrate",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: target_filtered(SelectionRequirement::Any) },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::XFromCost },
        ]),
        ..Default::default()
    }
}

/// Flame Sweep — {2}{R} Sorcery. Deals 2 damage to each creature without
/// flying.
pub fn flame_sweep() -> CardDefinition {
    CardDefinition {
        name: "Flame Sweep",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::Not(Box::new(
                    SelectionRequirement::HasKeyword(crate::card::Keyword::Flying),
                ))),
            ),
            body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(2) }),
        },
        ..Default::default()
    }
}

/// Tidings — {4}{U} Sorcery. Draw four cards.
pub fn tidings() -> CardDefinition {
    CardDefinition {
        name: "Tidings",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(4) },
        ..Default::default()
    }
}

/// Mind Spring — {X}{U}{U} Sorcery. Draw X cards.
pub fn mind_spring() -> CardDefinition {
    CardDefinition {
        name: "Mind Spring",
        cost: cost(&[x(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw { who: Selector::You, amount: Value::XFromCost },
        ..Default::default()
    }
}

/// Foresee — {2}{U} Sorcery. Scry 4, then draw two cards.
pub fn foresee() -> CardDefinition {
    CardDefinition {
        name: "Foresee",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(4) },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Final Judgment — {4}{W}{W} Sorcery. Exile all creatures.
pub fn final_judgment() -> CardDefinition {
    CardDefinition {
        name: "Final Judgment",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile { what: Selector::EachPermanent(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Planar Cleansing — {3}{W}{W} Sorcery. Destroy all nonland permanents.
pub fn planar_cleansing() -> CardDefinition {
    CardDefinition {
        name: "Planar Cleansing",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DestroyNoRegen {
            what: Selector::EachPermanent(SelectionRequirement::Not(Box::new(
                SelectionRequirement::Land,
            ))),
        },
        ..Default::default()
    }
}

/// Akroma's Vengeance — {4}{W}{W} Sorcery. Destroy all artifacts, creatures,
/// and enchantments. Cycling {3}.
pub fn akromas_vengeance() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Akroma's Vengeance",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(3)]))],
        effect: Effect::DestroyNoRegen {
            what: Selector::EachPermanent(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .or(SelectionRequirement::Enchantment),
            ),
        },
        ..Default::default()
    }
}

/// Fumigate — {3}{W}{W} Sorcery. Destroy all creatures. You gain 1 life for
/// each creature destroyed this way.
pub fn fumigate() -> CardDefinition {
    CardDefinition {
        name: "Fumigate",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Gain first so the count reflects the creatures about to die.
            Effect::GainLife {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(SelectionRequirement::Creature)),
            },
            Effect::DestroyNoRegen { what: Selector::EachPermanent(SelectionRequirement::Creature) },
        ]),
        ..Default::default()
    }
}

/// Terminus — {4}{W} Sorcery. Put all creatures on the bottom of their
/// owners' libraries.
pub fn terminus() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Terminus",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::EachPermanent(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: LibraryPosition::Bottom,
            },
        },
        ..Default::default()
    }
}

/// Gerrard's Wisdom — {3}{W} Sorcery. You gain 2 life for each card in your
/// hand.
pub fn gerrards_wisdom() -> CardDefinition {
    CardDefinition {
        name: "Gerrard's Wisdom",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(
                Box::new(Value::HandSizeOf(PlayerRef::You)),
                Box::new(Value::Const(2)),
            ),
        },
        ..Default::default()
    }
}

/// Grapple with the Past — {1}{G} Sorcery. Mill three cards, then return a
/// creature or land card from your graveyard to your hand.
pub fn grapple_with_the_past() -> CardDefinition {
    CardDefinition {
        name: "Grapple with the Past",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Land),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Selector for "each creature you control" — the team-pump target.
fn each_your_creature() -> Selector {
    Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    )
}

/// Overwhelming Stampede — {3}{G}{G} Sorcery. "Until end of turn, creatures you
/// control gain trample and get +X/+X, where X is the greatest power among
/// creatures you control." X is `Value::PowerOf(GreatestPowerYouControl)` — the
/// power of the single greatest-power creature you control.
pub fn overwhelming_stampede() -> CardDefinition {
    let x = || Value::PowerOf(Box::new(Selector::GreatestPowerYouControl));
    CardDefinition {
        name: "Overwhelming Stampede",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: each_your_creature(),
                power: x(),
                toughness: x(),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: each_your_creature(),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Triumph of the Hordes — {2}{G}{G} Sorcery. "Until end of turn, creatures you
/// control get +1/+1 and gain trample and infect." The infect rider turns the
/// alpha-strike into a poison kill (Infect is wired in combat damage).
pub fn triumph_of_the_hordes() -> CardDefinition {
    let grant = |kw: Keyword| Effect::GrantKeyword {
        what: each_your_creature(),
        keyword: kw,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Triumph of the Hordes",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: each_your_creature(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            grant(Keyword::Trample),
            grant(Keyword::Infect),
        ]),
        ..Default::default()
    }
}

/// Austere Command — {4}{W}{W} Sorcery. Choose two — destroy all artifacts;
/// destroy all enchantments; destroy all creatures with mana value 4+; destroy
/// all creatures with mana value 3 or less. Faithful "choose two" via
/// `Effect::ChooseN`; `picks: [2, 3]` is the AutoDecider default (destroy all
/// creatures — both creature halves), overridable by a UI/scripted decider.
pub fn austere_command() -> CardDefinition {
    let destroy_all = |req: SelectionRequirement| Effect::Destroy {
        what: Selector::EachPermanent(req),
    };
    CardDefinition {
        name: "Austere Command",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![2, 3],
            modes: vec![
                destroy_all(SelectionRequirement::Artifact),
                destroy_all(SelectionRequirement::Enchantment),
                destroy_all(
                    SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtLeast(4)),
                ),
                destroy_all(
                    SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
            ],
        },
        ..Default::default()
    }
}

/// Jeska's Will — {3}{R} Sorcery. Choose one — Add {R}{R}{R}; or exile the top
/// three cards of your library, you may play them this turn. (Printed: the
/// ritual scales with an opponent's hand size and you may choose both with a
/// commander; both are simplified — fixed {R}{R}{R}, single mode.)
pub fn jeskas_will() -> CardDefinition {
    use crate::card::MayPlayDuration;
    use crate::effect::ManaPayload;
    use crate::mana::Color;
    CardDefinition {
        name: "Jeska's Will",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::Const(3)),
            },
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(3),
                duration: MayPlayDuration::EndOfThisTurn,
            },
        ]),
        ..Default::default()
    }
}
