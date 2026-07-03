//! Commander 2021 / Strixhaven Commander (C21) — cards from the five
//! Strixhaven college precons that weren't already covered by their
//! original printings. Most ride existing engine primitives.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, LandType,
    Predicate, SelectionRequirement, Selector, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::{etb, etb_gain_life, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, w, Color};

use super::{
    dual_land_with, etb_tap_then_scry_one, tap_add, tap_add_colorless,
};

// ══════════════════════════════════════════════════════════════════════════
// Lands
// ══════════════════════════════════════════════════════════════════════════

// ── Theros scrylands (enters tapped, scry 1) ──────────────────────────────

pub fn temple_of_epiphany() -> CardDefinition {
    dual_land_with(
        "Temple of Epiphany", LandType::Island, LandType::Mountain,
        Color::Blue, Color::Red, vec![etb_tap_then_scry_one()],
    )
}
pub fn temple_of_malady() -> CardDefinition {
    dual_land_with(
        "Temple of Malady", LandType::Swamp, LandType::Forest,
        Color::Black, Color::Green, vec![etb_tap_then_scry_one()],
    )
}
pub fn temple_of_mystery() -> CardDefinition {
    dual_land_with(
        "Temple of Mystery", LandType::Forest, LandType::Island,
        Color::Green, Color::Blue, vec![etb_tap_then_scry_one()],
    )
}
pub fn temple_of_silence() -> CardDefinition {
    dual_land_with(
        "Temple of Silence", LandType::Plains, LandType::Swamp,
        Color::White, Color::Black, vec![etb_tap_then_scry_one()],
    )
}
pub fn temple_of_triumph() -> CardDefinition {
    dual_land_with(
        "Temple of Triumph", LandType::Mountain, LandType::Plains,
        Color::Red, Color::White, vec![etb_tap_then_scry_one()],
    )
}

/// Radiant Fountain — Land. When it enters, you gain 2 life. {T}: Add {C}.
pub fn radiant_fountain() -> CardDefinition {
    CardDefinition {
        name: "Radiant Fountain",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb_gain_life(2)],
        activated_abilities: vec![tap_add_colorless()],
        ..Default::default()
    }
}

/// Rogue's Passage — Land. {T}: Add {C}. {4},{T}: Target creature can't be
/// blocked this turn.
pub fn rogues_passage() -> CardDefinition {
    CardDefinition {
        name: "Rogue's Passage",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mikokoro, Center of the Sea — Legendary Land. {T}: Add {C}. {2},{T}: Each
/// player draws a card.
pub fn mikokoro_center_of_the_sea() -> CardDefinition {
    CardDefinition {
        name: "Mikokoro, Center of the Sea",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// High Market — Land. {T}: Add {C}. {T}, Sacrifice a creature: You gain 1 life.
pub fn high_market() -> CardDefinition {
    CardDefinition {
        name: "High Market",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((SelectionRequirement::Creature, 1)),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Temple of the False God — Land. {T}: Add {C}{C}. Activate only if you
/// control five or more lands.
pub fn temple_of_the_false_god() -> CardDefinition {
    CardDefinition {
        name: "Temple of the False God",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            },
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(5),
            }),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blighted Woodland — Land. {T}: Add {C}. {3}{G},{T},Sacrifice this land:
/// Search your library for up to two basic land cards, put them onto the
/// battlefield tapped, then shuffle.
pub fn blighted_woodland() -> CardDefinition {
    CardDefinition {
        name: "Blighted Woodland",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3), g()]),
                tap_cost: true,
                sac_cost: true,
                effect: fetch_two_basics(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Myriad Landscape — Land. {T}: Add {C}. {2},{T},Sacrifice this land: Search
/// your library for up to two basic land cards that share a land type, put
/// them onto the battlefield tapped, then shuffle. (The "share a land type"
/// rider is approximated as any two basics.)
pub fn myriad_landscape() -> CardDefinition {
    CardDefinition {
        name: "Myriad Landscape",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: fetch_two_basics(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn fetch_two_basics() -> Effect {
    let search = || Effect::Search {
        who: PlayerRef::You,
        filter: SelectionRequirement::IsBasicLand,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
    };
    Effect::Seq(vec![search(), search()])
}

// ── Onslaught cycling lands (mono, enter tapped, Cycling {C}) ──────────────

fn cycling_land(name: &'static str, color: Color, land_type: LandType) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![land_type], ..Default::default() },
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![super::etb_tap()],
        activated_abilities: vec![tap_add(color)],
        ..Default::default()
    }
}

pub fn barren_moor() -> CardDefinition {
    cycling_land("Barren Moor", Color::Black, LandType::Swamp)
}
pub fn forgotten_cave() -> CardDefinition {
    cycling_land("Forgotten Cave", Color::Red, LandType::Mountain)
}
pub fn lonely_sandbar() -> CardDefinition {
    cycling_land("Lonely Sandbar", Color::Blue, LandType::Island)
}
pub fn secluded_steppe() -> CardDefinition {
    cycling_land("Secluded Steppe", Color::White, LandType::Plains)
}
pub fn tranquil_thicket() -> CardDefinition {
    cycling_land("Tranquil Thicket", Color::Green, LandType::Forest)
}

// ══════════════════════════════════════════════════════════════════════════
// Creatures
// ══════════════════════════════════════════════════════════════════════════
/// Zetalpa, Primal Dawn — {6}{W}{W} Legendary Creature — Elder Dinosaur 4/8
/// with flying, double strike, vigilance, trample, and indestructible.
pub fn zetalpa_primal_dawn() -> CardDefinition {
    CardDefinition {
        name: "Zetalpa, Primal Dawn",
        cost: cost(&[generic(6), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 4,
        toughness: 8,
        keywords: vec![
            Keyword::Flying, Keyword::DoubleStrike, Keyword::Vigilance,
            Keyword::Trample, Keyword::Indestructible,
        ],
        ..Default::default()
    }
}
/// Verdant Sun's Avatar — {5}{G}{G} Creature — Dinosaur Avatar 5/5. Whenever
/// this or another creature you control enters, you gain life equal to that
/// creature's toughness.
pub fn verdant_suns_avatar() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Verdant Sun's Avatar",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur, CreatureType::Avatar],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Sanctum Gargoyle — {3}{W} Artifact Creature — Gargoyle 2/3. Flying. When it
/// enters, you may return target artifact card from your graveyard to hand.
pub fn sanctum_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Sanctum Gargoyle",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "return target artifact card from your graveyard to your hand".into(),
            body: Box::new(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Artifact
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..Default::default()
    }
}
// ══════════════════════════════════════════════════════════════════════════
// Artifacts
// ══════════════════════════════════════════════════════════════════════════
/// Boros Locket — {3} Artifact. {T}: Add {R} or {W}. {R/W}{R/W}{R/W}{R/W},{T},
/// Sacrifice this artifact: Draw two cards.
pub fn boros_locket() -> CardDefinition {
    use crate::mana::hybrid;
    CardDefinition {
        name: "Boros Locket",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            tap_add(Color::Red),
            tap_add(Color::White),
            ActivatedAbility {
                mana_cost: cost(&[
                    hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White),
                    hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White),
                ]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ══════════════════════════════════════════════════════════════════════════
// Spells
// ══════════════════════════════════════════════════════════════════════════

/// Chain Reaction — {2}{R}{R} Sorcery. Deals X damage to each creature, where
/// X is the number of creatures on the battlefield.
pub fn chain_reaction() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Chain Reaction",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature,
                ))),
            }),
        },
        ..Default::default()
    }
}

/// Gaze of Granite — {X}{B}{B}{G} Sorcery. Destroy each nonland permanent with
/// mana value X or less.
pub fn gaze_of_granite() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Gaze of Granite",
        cost: cost(&[x(), b(), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(
                SelectionRequirement::Nonland
                    .and(SelectionRequirement::ManaValueAtMostXFromCost),
            ),
        },
        ..Default::default()
    }
}

/// Biomass Mutation — {X}{G/U}{G/U} Instant. Creatures you control have base
/// power and toughness X/X until end of turn.
pub fn biomass_mutation() -> CardDefinition {
    use crate::mana::{hybrid, x};
    CardDefinition {
        name: "Biomass Mutation",
        cost: cost(&[x(), hybrid(Color::Green, Color::Blue), hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Instant],
        effect: Effect::SetBasePT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::XFromCost,
            toughness: Value::XFromCost,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Perplexing Test — {3}{U}{U} Instant. Choose one — return all creature
/// tokens to their owners' hands; or return all nontoken creatures to their
/// owners' hands.
pub fn perplexing_test() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Perplexing Test",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::IsToken),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::Move {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::NotToken),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
        ..Default::default()
    }
}
/// Taste of Death — {4}{B}{B} Sorcery. Each player sacrifices three creatures
/// of their choice. You create three Food tokens.
pub fn taste_of_death() -> CardDefinition {
    CardDefinition {
        name: "Taste of Death",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::Const(3),
                filter: SelectionRequirement::Creature,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: crabomination_base::tokens::food_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Sculpting Steel — {3} Artifact. You may have it enter as a copy of any
/// artifact on the battlefield.
pub fn sculpting_steel() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Sculpting Steel",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        enters_as_copy: Some(EntersAsCopy {
            filter: SelectionRequirement::Artifact,
            extra_creature_types: vec![],
            extra_triggered: vec![],
            extra_keywords: vec![],
            keep_name: false,
            extra_card_types: vec![],
            non_legendary: false,
        }),
        ..Default::default()
    }
}

/// Phyrexia's Core — Land. {T}: Add {C}. {1},{T},Sacrifice an artifact: You
/// gain 1 life.
pub fn phyrexias_core() -> CardDefinition {
    CardDefinition {
        name: "Phyrexia's Core",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_other_filter: Some((SelectionRequirement::Artifact, 1)),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Brass's Bounty — {6}{R} Sorcery. For each land you control, create a
/// Treasure token.
pub fn brasss_bounty() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Brass's Bounty",
        cost: cost(&[generic(6), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                filter: SelectionRequirement::Land,
            },
            definition: crabomination_base::tokens::treasure_token(),
        },
        ..Default::default()
    }
}

/// Oblation — {2}{W} Instant. The owner of target nonland permanent shuffles
/// it into their library, then draws two cards.
pub fn oblation() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Oblation",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        // Draw for the owner while the target is still on the battlefield (so
        // OwnerOf resolves), then shuffle it away — same net result as the
        // printed "shuffle, then draw" order.
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Shuffled },
            },
        ]),
        ..Default::default()
    }
}
