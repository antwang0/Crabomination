//! Modern-staple / cube artifacts.

use super::no_abilities;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, Effect, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{ManaCost, cost, generic};

/// Ornithopter — {0} Artifact Creature 0/2 with Flying. Pure vanilla; no
/// abilities beyond Flying.
pub fn ornithopter() -> CardDefinition {
    CardDefinition {
        name: "Ornithopter",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Ornithopter of Paradise — {1}{G} Artifact Creature 0/2 with Flying. {T}: Add
/// one mana of any color. Reuses `ManaPayload::AnyOneColor` so the engine
/// surfaces the color choice via the `ChooseColor` decision.
pub fn ornithopter_of_paradise() -> CardDefinition {
    CardDefinition {
        name: "Ornithopter of Paradise",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Millstone — {2} Artifact. {2}, {T}: Target player puts the top two cards
/// of their library into their graveyard.
pub fn millstone() -> CardDefinition {
    use crate::card::{Selector, Value};
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Millstone",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Mind Stone — {2} Artifact. {T}: Add {C}. {1}, {T}, Sacrifice this:
/// Draw a card.
///
/// Both abilities are wired: the first is a vanilla mana ability, the
/// second uses the new `sac_cost` field on `ActivatedAbility` so paying
/// the cost sacrifices Mind Stone before the Draw resolves.
pub fn mind_stone() -> CardDefinition {
    use crate::card::Selector;
    CardDefinition {
        name: "Mind Stone",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            },
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: crate::card::Value::Const(1),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Aether Spellbomb — {1} Artifact. {U}, Sacrifice this artifact: Return
/// target creature to its owner's hand. {1}, Sacrifice this artifact:
/// Draw a card. Both activated abilities use the new `sac_cost` field;
/// the first targets a creature on the battlefield (the bounce is wired
/// via the existing `Move(Target → Hand(OwnerOf(Target)))` pattern from
/// Vapor Snag).
pub fn aether_spellbomb() -> CardDefinition {
    use crate::card::{Selector, Value};
    use crate::effect::shortcut::target_filtered;
    use crate::effect::ZoneDest;
    use crate::card::SelectionRequirement;
    use crate::mana::u;
    CardDefinition {
        name: "Aether Spellbomb",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            // {U}, Sacrifice this: Return target creature to its owner's hand.
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: false,
                mana_cost: cost(&[u()]),
                effect: Effect::Move {
                    what: target_filtered(SelectionRequirement::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            },
            // {1}, Sacrifice this: Draw a card.
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: false,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Zuran Orb — {0} Artifact. Sacrifice a land: You gain 2 life.
///
/// The "Sacrifice a land" cost is now a proper pre-resolution activation
/// cost via `sac_other_filter: Some((Land, 1))` — the engine gates the
/// activation on the controller owning a land to sacrifice (rejecting
/// cleanly otherwise), rather than folding the sacrifice into resolution.
pub fn zuran_orb() -> CardDefinition {
    CardDefinition {
        name: "Zuran Orb",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None,
            // Sacrifice a land as an activation cost.
            sac_other_filter: Some((SelectionRequirement::Land, 1)),
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Soul Conduit — {6} Artifact. "{6}, {T}: Exchange life totals with target
/// player. Activate only as a sorcery." (CR 701.12c). Targets the opponent
/// in heads-up via `Selector::Player(EachOpponent)` rather than a player
/// prompt.
pub fn soul_conduit() -> CardDefinition {
    use crate::effect::Selector;
    CardDefinition {
        name: "Soul Conduit",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(6)]),
            sorcery_speed: true,
            effect: Effect::ExchangeLifeTotals {
                a: Selector::You,
                b: Selector::Player(PlayerRef::EachOpponent),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chromatic Star — {1} Artifact. {1}, {T}, Sacrifice this: Add one mana of
/// any color. When it's put into a graveyard from the battlefield, draw a
/// card. The cantrip is a self-source `PermanentLeavesBattlefield` trigger
/// (fires on the sac-for-mana path; over-fires only on the rare exile).
pub fn chromatic_star() -> CardDefinition {
    CardDefinition {
        name: "Chromatic Star",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Soul-Guide Lantern — {1} Artifact. {T}: target opponent exiles a card
/// from their graveyard. {2}, {T}, Sacrifice this: Each player exiles
/// each card from their graveyard. Draw a card.
///
/// The first ability is approximated as "exile every card from each
/// opponent's graveyard" (the engine has no "let opponent pick" exile
/// primitive yet) — strictly more powerful but the typical line is
/// against an opponent with one exile-target anyway, where this is
/// gameplay-equivalent. The second uses `sac_cost: true` for the
/// activation cost.
pub fn soul_guide_lantern() -> CardDefinition {
    CardDefinition {
        name: "Soul-Guide Lantern",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            // {T}: target opponent exiles a card from their graveyard.
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::EachOpponent,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    },
                    to: ZoneDest::Exile,
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            },
            // {2}, {T}, Sac: Each player exiles their graveyard, you draw.
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::CardsInZone {
                            who: PlayerRef::EachPlayer,
                            zone: Zone::Graveyard,
                            filter: SelectionRequirement::Any,
                        },
                        to: ZoneDest::Exile,
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ]),
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Cankerbloom — {1}{G}, 3/2 Fungus. {G}, Sacrifice this: Destroy target
/// artifact or enchantment. Then proliferate.
///
/// Reuses `sac_cost: true` (Haywire Mite shape) plus `Effect::Proliferate`
/// as the tail step. Plant subtype dropped because `CreatureType` doesn't
/// enumerate it; `Fungus` stands in.
pub fn cankerbloom() -> CardDefinition {
    use crate::card::CreatureType;
    use crate::mana::g;
    CardDefinition {
        name: "Cankerbloom",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[g()]),
            effect: Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Artifact
                            .or(SelectionRequirement::Enchantment),
                    ),
                },
                Effect::Proliferate,
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Fellwar Stone — {2} Artifact. {T}: Add one mana of any color a land an
/// opponent controls could produce.
///
/// Push (modern_decks batch 117): the "matches an opponent's land
/// colors" restriction is now wired faithfully via the new
/// `ManaPayload::AnyColorOpponentCouldProduce` primitive. Resolution
/// scans opponents' battlefield for basic-typed lands (Plains, Island,
/// Swamp, Mountain, Forest), builds the legal-color set from those
/// types, and the activator picks one color from that set. If no
/// opponent controls a basic-typed land, falls back to colorless
/// (matches the "never silently no-op" convention for mana abilities).
pub fn fellwar_stone() -> CardDefinition {
    CardDefinition {
        name: "Fellwar Stone",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColorOpponentCouldProduce,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Star Compass — {2} Artifact. Enters tapped. "{T}: Add one mana of any color
/// that a basic land you control could produce" — the controller-side mirror
/// of Fellwar Stone via `ManaPayload::AnyColorYouCouldProduce`.
pub fn star_compass() -> CardDefinition {
    CardDefinition {
        name: "Star Compass",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![enters_tapped_trigger()],
        activated_abilities: vec![tap_for(ManaPayload::AnyColorYouCouldProduce)],
        ..Default::default()
    }
}

// ── Monument to Endurance ──────────────────────────────────────────────────

/// Monument to Endurance — {3} Artifact. {2}, {T}: Target creature gets
/// +2/+2 until end of turn. Simple pump artifact.
pub fn monument_to_endurance() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Monument to Endurance",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    }
}

/// Contagion Clasp — {4} Artifact. ETB: put a -1/-1 counter on target
/// creature. `{4}, {T}: Proliferate.`
pub fn contagion_clasp() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Contagion Clasp",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::Const(1),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: Effect::Proliferate,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Throne of Geth — {1} Artifact. `{T}, Sacrifice this artifact: Proliferate.`
pub fn throne_of_geth() -> CardDefinition {
    CardDefinition {
        name: "Throne of Geth",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Proliferate,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Walking Ballista — {X}{X} Artifact Creature 0/0. Enters with X +1/+1
/// counters. "Remove a +1/+1 counter from this: it deals 1 damage to any
/// target." "{4}: Put a +1/+1 counter on this."
pub fn walking_ballista() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Predicate;
    use crate::mana::x;
    CardDefinition {
        name: "Walking Ballista",
        cost: cost(&[x(), x()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                condition: Some(Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                    Value::Const(1),
                )),
                effect: Effect::Seq(vec![
                    Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                mana_cost: cost(&[generic(4)]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Triskelion — {6} Artifact Creature 1/1 that enters with three +1/+1
/// counters. "Remove a +1/+1 counter from this: it deals 1 damage to any
/// target."
pub fn triskelion() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Predicate;
    CardDefinition {
        name: "Triskelion",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        power: 1,
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            condition: Some(Predicate::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
                Value::Const(1),
            )),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hangarback Walker — {X}{X} Artifact Creature 0/0. Enters with X +1/+1
/// counters. "{1}, {T}: Put a +1/+1 counter on this." "When this dies,
/// create a 1/1 colorless Thopter artifact creature token with flying for
/// each +1/+1 counter on it."
pub fn hangarback_walker() -> CardDefinition {
    use crate::card::{CounterType, CreatureType, TokenDefinition};
    use crate::mana::x;
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Hangarback Walker",
        cost: cost(&[x(), x()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
                definition: thopter,
            },
        }],
        ..Default::default()
    }
}

/// Arcbound Worker — {1} Artifact Creature — Construct, 0/0, Modular 1.
/// Enters with one +1/+1 counter; on death moves its counters to a target
/// artifact creature.
pub fn arcbound_worker() -> CardDefinition {
    use crate::card::{CounterType, CreatureType};
    CardDefinition {
        name: "Arcbound Worker",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        triggered_abilities: vec![crate::effect::shortcut::modular_dies()],
        ..Default::default()
    }
}

/// Arcbound Stinger — {2} Artifact Creature — Insect, 0/0, Flying, Modular 1.
pub fn arcbound_stinger() -> CardDefinition {
    use crate::card::{CounterType, CreatureType};
    CardDefinition {
        name: "Arcbound Stinger",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        triggered_abilities: vec![crate::effect::shortcut::modular_dies()],
        ..Default::default()
    }
}

/// Arcbound Ravager — {2} Artifact Creature — Beast, 0/0, Modular 1.
/// "Sacrifice an artifact: Put a +1/+1 counter on Arcbound Ravager."
pub fn arcbound_ravager() -> CardDefinition {
    use crate::card::{CounterType, CreatureType};
    CardDefinition {
        name: "Arcbound Ravager",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            sac_other_filter: Some((SelectionRequirement::Artifact, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::modular_dies()],
        ..Default::default()
    }
}

/// Arcbound Slith — {1} Artifact Creature — Slith, 1/1, Modular 1.
/// "Whenever this deals combat damage to a player, put a +1/+1 counter on it."
pub fn arcbound_slith() -> CardDefinition {
    use crate::card::{CounterType, CreatureType};
    CardDefinition {
        name: "Arcbound Slith",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Slith],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            crate::effect::shortcut::modular_dies(),
        ],
        ..Default::default()
    }
}

/// Arcbound Hybrid — {3} Artifact Creature — Beast, 0/0, Haste, Modular 2.
pub fn arcbound_hybrid() -> CardDefinition {
    use crate::card::{CounterType, CreatureType};
    CardDefinition {
        name: "Arcbound Hybrid",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        keywords: vec![Keyword::Haste],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![crate::effect::shortcut::modular_dies()],
        ..Default::default()
    }
}

/// Arcbound Bruiser — {4} Artifact Creature — Golem, 0/0, Modular 3.
pub fn arcbound_bruiser() -> CardDefinition {
    use crate::card::{CounterType, CreatureType};
    CardDefinition {
        name: "Arcbound Bruiser",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![crate::effect::shortcut::modular_dies()],
        ..Default::default()
    }
}

/// Hedron Archive — {4} Artifact. "{T}: Add {C}{C}." "{T}, Sacrifice this
/// artifact: Draw two cards."
pub fn hedron_archive() -> CardDefinition {
    CardDefinition {
        name: "Hedron Archive",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(2)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Mana rocks & utility artifacts ───────────────────────────────────────────

/// Triggered ability: this permanent enters the battlefield tapped.
fn enters_tapped_trigger() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Tap { what: Selector::This },
    }
}

/// Tap-for-mana ability shorthand (no mana cost).
fn tap_for(pool: ManaPayload) -> ActivatedAbility {
    ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool },
        ..Default::default()
    }
}

/// Worn Powerstone — {3} Artifact. Enters tapped. "{T}: Add {C}{C}." (mana
/// rock that ramps two colorless). (5ED)
pub fn worn_powerstone() -> CardDefinition {
    CardDefinition {
        name: "Worn Powerstone",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![enters_tapped_trigger()],
        activated_abilities: vec![tap_for(ManaPayload::Colorless(Value::Const(2)))],
        ..Default::default()
    }
}

/// Phyrexian Walker — {0} Artifact Creature — Construct 0/3 (vanilla). (HML)
pub fn phyrexian_walker() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Walker",
        cost: cost(&[generic(0)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![crate::card::CreatureType::Construct],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        ..Default::default()
    }
}

/// Prophetic Prism — {2} Artifact. "When this enters, draw a card. {1}, {T}:
/// Add one mana of any color." (CON)
pub fn prophetic_prism() -> CardDefinition {
    CardDefinition {
        name: "Prophetic Prism",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fountain of Renewal — {1} Artifact. "At the beginning of your upkeep, you
/// gain 1 life." (The {4},{T}, Sacrifice: draw a card mode is omitted.) (M19)
pub fn fountain_of_renewal() -> CardDefinition {
    CardDefinition {
        name: "Fountain of Renewal",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Cathodion — {3} Artifact Creature — Construct 3/3. When it dies, add {C}{C}{C}.
pub fn cathodion() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Cathodion",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(3)) },
        }],
        ..Default::default()
    }
}

/// Bottle Gnomes — {3} Artifact Creature — Gnome 1/3. "Sacrifice this: gain 3 life."
pub fn bottle_gnomes() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Bottle Gnomes",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Universal Automaton — {1} Artifact Creature — Shapeshifter 1/1 with Changeling.
pub fn universal_automaton() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Universal Automaton",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Changeling],
        ..Default::default()
    }
}

/// Affinity-for-artifacts vanilla body: costs {1} less per artifact you control.
fn affinity_body(name: &'static str, mv: u32, ct: crate::card::CreatureType, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(mv)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        power: p,
        toughness: t,
        affinity_filter: Some(SelectionRequirement::Artifact),
        ..Default::default()
    }
}

/// Frogmite — {4} Artifact Creature — Frog 2/2. Affinity for artifacts.
pub fn frogmite() -> CardDefinition {
    affinity_body("Frogmite", 4, crate::card::CreatureType::Frog, 2, 2)
}

/// Myr Enforcer — {7} Artifact Creature — Myr 4/4. Affinity for artifacts.
pub fn myr_enforcer() -> CardDefinition {
    affinity_body("Myr Enforcer", 7, crate::card::CreatureType::Myr, 4, 4)
}

/// Court Homunculus — {W} Artifact Creature — Homunculus 1/1. Gets +1/+1 as
/// long as you control another artifact.
pub fn court_homunculus() -> CardDefinition {
    use crate::card::{CreatureType, StaticAbility};
    use crate::effect::{Predicate, StaticEffect, Value};
    use crate::mana::w;
    CardDefinition {
        name: "Court Homunculus",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Homunculus], ..Default::default() },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "As long as you control another artifact, this creature gets +1/+1.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Artifact
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    n: Value::Const(1),
                },
                power: 1,
                toughness: 1,
                keyword: None,
            },
        }],
        ..Default::default()
    }
}

/// Chief of the Foundry — {3} Artifact Creature — Construct 2/3. Other artifact
/// creatures you control get +1/+1.
pub fn chief_of_the_foundry() -> CardDefinition {
    use crate::card::{CreatureType, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Chief of the Foundry",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Other artifact creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Artifact
                        .and(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}
