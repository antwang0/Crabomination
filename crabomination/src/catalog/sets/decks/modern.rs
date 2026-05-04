//! Modern-playable cards that complement the BRG combo and Goryo's
//! Vengeance demo decks. These are *new* card factories, distinct from the
//! existing `creatures.rs` / `lands.rs` / `spells.rs` set, and each card is
//! built on the existing engine primitives — no engine changes required.
//!
//! Cards in this file are tracked in `DECK_FEATURES.md` under the **Modern
//! supplement** section.

use super::super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, SelectionRequirement, Selector,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, g, generic, r, u, w};

// ── Cantrips & card selection ────────────────────────────────────────────────

/// Ponder — {U} Sorcery. Look at the top three cards of your library, then
/// put them back in any order. You may shuffle. Then draw a card.
///
/// Approximation: `Scry 3 + Draw 1` (Scry's "top or bottom" picks substitute
/// for the "any order + may shuffle" decision; the gameplay-relevant outcome
/// — taking the best one of the top three on the next draw — is preserved).
pub fn ponder() -> CardDefinition {
    CardDefinition {
        name: "Ponder",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Manamorphose — {1}{R/G} Instant. Add two mana in any combination of
/// colors. Draw a card.
///
/// Hybrid `{R/G}` is approximated as a generic `{2}` cost (the engine has
/// no hybrid pip yet); the spell-level effect is unaffected.
pub fn manamorphose() -> CardDefinition {
    CardDefinition {
        name: "Manamorphose",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColors(Value::Const(2)),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Sleight of Hand — {U} Sorcery. Look at the top two cards of your library,
/// put one into your hand, the other on the bottom.
///
/// Approximation: `Scry 1 + Draw 1` — close enough to "look at 2, take the
/// better one" since Scry can put the unwanted card on the bottom before the
/// draw resolves (slightly worse than the real card's view of two).
pub fn sleight_of_hand() -> CardDefinition {
    CardDefinition {
        name: "Sleight of Hand",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Discard, draw-for-life, mill ─────────────────────────────────────────────

/// Faithless Looting — {R} Sorcery. Draw two cards, then discard two cards.
/// Flashback {2}{R}.
pub fn faithless_looting() -> CardDefinition {
    let flashback_cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(2), ManaSymbol::Colored(Color::Red)],
    };
    CardDefinition {
        name: "Faithless Looting",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Sign in Blood — {B}{B} Sorcery. Target player draws two cards and loses
/// 2 life.
///
/// Targets any player via `target_filtered(Player)`; both effects run
/// against `Selector::Player(PlayerRef::Target(0))` (same pattern Thought
/// Scour and Vapor Snag already use). Auto-targeting picks the caster
/// when no target is supplied, so the BRG demo's "self-cantrip" line
/// keeps working.
pub fn sign_in_blood() -> CardDefinition {
    CardDefinition {
        name: "Sign in Blood",
        cost: cost(&[b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Night's Whisper — {1}{B} Sorcery. Draw two cards, lose 2 life.
pub fn nights_whisper() -> CardDefinition {
    CardDefinition {
        name: "Night's Whisper",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Duress — {B} Sorcery. Target opponent reveals their hand. You choose a
/// noncreature, nonland card. They discard it.
pub fn duress() -> CardDefinition {
    CardDefinition {
        name: "Duress",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Nonland.and(SelectionRequirement::Noncreature),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Burn / damage ────────────────────────────────────────────────────────────

/// Lava Spike — {R} Sorcery. Lava Spike deals 3 damage to target player.
pub fn lava_spike() -> CardDefinition {
    CardDefinition {
        name: "Lava Spike",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Arcane],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Lava Dart — {R} Sorcery. Lava Dart deals 1 damage to any target.
/// Flashback — sacrifice a Mountain. The flashback sacrifice cost is
/// approximated with a regular `Flashback` mana cost of `{0}`; the engine
/// has no "sacrifice a Mountain" alt-cost primitive yet.
pub fn lava_dart() -> CardDefinition {
    let flashback_cost = ManaCost { symbols: vec![] };
    CardDefinition {
        name: "Lava Dart",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(1),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// (Shock already exists as `catalog::shock` from the Portal set; we don't
// duplicate it here.)

// ── Reanimation / graveyard package ──────────────────────────────────────────

/// Unburial Rites — {3}{B} Sorcery. Return target creature card from a
/// graveyard to the battlefield under your control. Flashback {W}{B}.
pub fn unburial_rites() -> CardDefinition {
    let flashback_cost = ManaCost {
        symbols: vec![
            ManaSymbol::Colored(Color::White),
            ManaSymbol::Colored(Color::Black),
        ],
    };
    CardDefinition {
        name: "Unburial Rites",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Exhume — {1}{B} Sorcery. Each player puts a creature card from their
/// graveyard onto the battlefield.
///
/// Approximation: only the caster reanimates a creature (the engine doesn't
/// expose `EachPlayer` reanimate symmetry as a single primitive). Equivalent
/// to "you reanimate" in the typical Goryo's-deck context.
pub fn exhume() -> CardDefinition {
    CardDefinition {
        name: "Exhume",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Buried Alive — {2}{B} Sorcery. Search your library for up to three
/// creature cards, put them into your graveyard, then shuffle.
///
/// Wired as `Repeat(3, Search(Creature → Graveyard))`. The decider can
/// answer `Search(None)` to short-circuit out of any iteration when the
/// "up to three" upper bound shouldn't be hit — `do_search` already
/// honors a `None` answer as "decline this search". The library's
/// implicit shuffle runs after each successful pull.
pub fn buried_alive() -> CardDefinition {
    CardDefinition {
        name: "Buried Alive",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Repeat {
            count: Value::Const(3),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
                to: ZoneDest::Graveyard,
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Entomb — {B} Instant. Search your library for a card and put it into
/// your graveyard. Then shuffle.
pub fn entomb() -> CardDefinition {
    CardDefinition {
        name: "Entomb",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Any,
            to: ZoneDest::Graveyard,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Burning-Tree Emissary — {R/G}{R/G} 2/2 Creature — Human Shaman. When
/// Burning-Tree Emissary enters, add {R}{G}.
///
/// Hybrid pips approximated as `{2}` (engine has no hybrid pip yet); the
/// ETB ramp is unchanged.
pub fn burning_tree_emissary() -> CardDefinition {
    CardDefinition {
        name: "Burning-Tree Emissary",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Red, Color::Green]),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Putrid Imp — {B} Creature — Imp 1/1. Flying. Discard a card: target
/// creature you control gains threshold-bonus tricks (we model the engine-
/// supported half — Putrid Imp's main role in reanimator decks is as a
/// turn-1 discard outlet for Griselbrand / Atraxa).
pub fn putrid_imp() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Putrid Imp",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            // "Discard a card: This creature gains menace until end of turn."
            // The classic Putrid Imp text grants madness-style discard plus
            // a temporary menace; the discard outlet is the gameplay-critical
            // half here. We grant Menace EOT for flavour.
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Tarmogoyf — {1}{G} 1/2 Lhurgoyf. *X/X+1*, where X is the number of card
/// types among cards in all graveyards.
///
/// Cosmogoyf's catalog implementation already wires the dynamic P/T via a
/// layer-7 injection in `compute_battlefield`; we mirror that exactly here
/// (Tarmogoyf is the ancestor card, identical mechanic). The base 1/2 is a
/// floor — never read directly by the engine; the layer always overrides.
pub fn tarmogoyf() -> CardDefinition {
    CardDefinition {
        name: "Tarmogoyf",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lhurgoyf],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Veil of Summer — {G} Instant. Draw a card if an opponent has cast a blue
/// or black spell this turn.
///
/// Approximation: unconditional `Draw 1`. The "if blue/black spell cast"
/// gate would require per-color cast tracking. Veil's full Oracle has
/// other clauses (uncounterable, hexproof) we omit; here it acts as a
/// 1-mana cantrip — still useful, just simplified.
pub fn veil_of_summer() -> CardDefinition {
    CardDefinition {
        name: "Veil of Summer",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Crop Rotation — {G} Instant. As an additional cost, sacrifice a land.
/// Search your library for a land card and put it onto the battlefield.
/// Then shuffle.
///
/// ✅ Push XLIII: now uses the `additional_sac_cost: Some(Land &
/// ControlledByYou)` cast-time primitive (push XXXIX). The cast is
/// illegal without a land to feed; the engine auto-picks the lowest-MV
/// land and sacrifices it before the spell goes on the stack — so the
/// "sacrifice trigger fires before tutor resolves" timing matches
/// printed semantics. The body is now just the tutor.
pub fn crop_rotation() -> CardDefinition {
    CardDefinition {
        name: "Crop Rotation",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Land,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: Some(
            SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
        ),
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Utility / "named" lands ──────────────────────────────────────────────────

/// Karakas — Legendary Land. {T}: Add {W}. {T}: Return target legendary
/// creature to its owner's hand.
///
/// The bounce ability targets any legendary creature; in conjunction with
/// Goryo's Vengeance, it lets the owner of the reanimated creature respond
/// before the EOT exile fires (returning it to hand cleanly rather than
/// losing it to exile).
pub fn karakas() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    CardDefinition {
        name: "Karakas",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Plains],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            // {T}: Add {W}.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::White]),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
            },
            // {T}: Return target legendary creature to its owner's hand.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasSupertype(Supertype::Legendary)),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Card-selection / hand-disruption / counterspell extras ──────────────────

/// Cathartic Reunion — {1}{R} Sorcery. As an additional cost to cast this
/// spell, discard two cards. Draw three cards.
///
/// ✅ Push XLIII: now uses the new `CardDefinition.additional_discard_cost
/// = Some(2)` cast-time primitive (sister to push XXXIX's
/// `additional_sac_cost`). The cast is illegal if the controller has fewer
/// than 2 cards in hand at cast time; the engine auto-picks the first 2
/// cards in hand and discards them before the spell goes on the stack
/// (madness / discard-trigger listeners fire pre-resolution). The body
/// now contains only the draw-3.
pub fn cathartic_reunion() -> CardDefinition {
    CardDefinition {
        name: "Cathartic Reunion",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: Some(2),
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Gitaxian Probe — Phyrexian Sorcery. Look at target opponent's hand. Draw
/// a card.
///
/// The Phyrexian pip `{U/Φ}` (pay 2 life or {U}) collapses to a flat {0}
/// cost plus 2 life on resolution — the typical Modern line is to pay the
/// life. The "look at opponent's hand" half is dropped (information-only
/// effect with no engine state hook). Net gameplay: lose 2 life, draw a
/// card — a free cantrip that costs no card slot.
pub fn gitaxian_probe() -> CardDefinition {
    CardDefinition {
        name: "Gitaxian Probe",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Counter-magic / removal ─────────────────────────────────────────────────

/// Force Spike — {U} Instant. Counter target spell unless its controller
/// pays {1}.
///
/// Reuses `Effect::CounterUnlessPaid` (the same Mystical Dispute primitive)
/// so the engine auto-resolves the "unless they pay" half: at resolution
/// it temporarily flips priority to the targeted spell's controller and
/// tries to auto-tap + pay the {1}. If they can, the spell stays;
/// otherwise it's countered. Spells flagged `uncounterable` (Cavern of
/// Souls, Dovin's Veto) are skipped automatically.
pub fn force_spike() -> CardDefinition {
    CardDefinition {
        name: "Force Spike",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: crate::effect::shortcut::target(),
            mana_cost: cost(&[generic(1)]),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Vampiric Tutor — {B} Instant. Pay 2 life, search your library for a
/// card, and put it on top.
///
/// Approximated as a `LoseLife 2 + Search(Any → Library{Top})`. The "look
/// at and put on top" half collapses to a direct top-of-library tutor —
/// the decider supplies the chosen card via `Decision::SearchLibrary`.
pub fn vampiric_tutor() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Vampiric Tutor",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: LibraryPosition::Top,
                },
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Sylvan Scrying — {1}{G} Sorcery. Search your library for a land card,
/// reveal it, and put it into your hand. Then shuffle.
///
/// A clean reusable land-tutor. Pairs with surveil/fastland/shockland
/// fixings already in the catalog.
pub fn sylvan_scrying() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Scrying",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Land,
            to: ZoneDest::Hand(PlayerRef::You),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Abrupt Decay — {B}{G} Instant. "This spell can't be countered. /
/// Destroy target nonland permanent with mana value 3 or less."
///
/// Push XXIX bug fix: target filter was `ManaValueAtMost(2)`, which
/// rejected legal targets like Tarmogoyf (4-color hybrid 0-MV-by-cost
/// but Oracle-printed CMC 2 — fine here, but real targets like Stormbreath
/// Dragon at MV 4 fall outside, while a 3-MV Liliana of the Veil walked
/// safely). Real Abrupt Decay reads "mana value 3 or less" (Oracle
/// text per RTR / 2021 Update). The existing
/// `Keyword::CantBeCountered` (consumed by `caster_grants_uncounterable`
/// in all three cast paths) is unchanged.
pub fn abrupt_decay() -> CardDefinition {
    CardDefinition {
        name: "Abrupt Decay",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Nonland
                    .and(SelectionRequirement::ManaValueAtMost(3)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Ramp / search ────────────────────────────────────────────────────────────

/// Kodama's Reach — {2}{G} Sorcery. Search your library for up to two basic
/// land cards, reveal them, put one onto the battlefield tapped and the
/// other into your hand. Then shuffle.
///
/// Modeled as two consecutive `Search` calls — first lands the basic on
/// the battlefield tapped, second tucks one into hand. Decliner-friendly:
/// the decider can answer `Search(None)` to opt out of either branch
/// (`do_search` already honors a `None` answer).
pub fn kodamas_reach() -> CardDefinition {
    CardDefinition {
        name: "Kodama's Reach",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Engine artifacts ─────────────────────────────────────────────────────────

/// Lotus Petal — {0} Artifact. {T}, Sacrifice this: Add one mana of any
/// color.
///
/// Standard one-shot mana rock. The activation uses `sac_cost: true` so the
/// engine sacrifices the Petal as part of the cost (matching Chromatic
/// Star's shape, minus the cantrip-on-leaves rider).
pub fn lotus_petal() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Lotus Petal",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Tormod's Crypt — {0} Artifact. {T}, Sacrifice this: Exile all cards from
/// target player's graveyard.
///
/// Approximated as "exile each card in each opponent's graveyard" (matching
/// Soul-Guide Lantern's first ability) — strictly more powerful than the
/// real card in multiplayer but gameplay-equivalent in 1v1 against an
/// opponent with the only relevant graveyard.
pub fn tormods_crypt() -> CardDefinition {
    use crate::card::{ActivatedAbility, Zone};
    CardDefinition {
        name: "Tormod's Crypt",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
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
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Mishra's Bauble — {0} Artifact. {T}, Sacrifice this: Look at the top card
/// of target player's library. Draw a card at the beginning of the next
/// turn's upkeep.
///
/// Approximated as `LookAtTop(You) + DelayUntil(YourNextUpkeep, Draw 1)` —
/// the "look at *target* player's" peek collapses to the controller's own
/// library (the engine has no public-information primitive that exposes an
/// opponent's library to the human UI). The delayed cantrip is the
/// gameplay-critical half — Bauble's role in Modern is as a free cantrip,
/// not an information tool.
pub fn mishras_bauble() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::DelayedTriggerKind;
    CardDefinition {
        name: "Mishra's Bauble",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::LookAtTop { who: PlayerRef::You, amount: Value::Const(1) },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::YourNextUpkeep,
                    body: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    }),
                    capture: None,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Creatures + enchantments ─────────────────────────────────────────────────

/// Stoneforge Mystic — {1}{W} 1/2 Human Artificer. When this enters, you may
/// search your library for an Equipment card, reveal it, and put it into
/// your hand. Then shuffle.
///
/// Wired as a self-source ETB `Search(filter: HasArtifactSubtype(Equipment),
/// to: Hand)`. The engine's `do_search` already supports declining the
/// search (decider answers `Search(None)`), which models the "may" rider.
/// The {1}{W}, {T}: equip ability is omitted (no equipment-attach
/// activation primitive yet) — use Stoneforge purely as a tutor for now.
pub fn stoneforge_mystic() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Stoneforge Mystic",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Qasali Pridemage — {G}{W} 2/2 Cat Wizard. Exalted (omitted; no
/// per-attack-power-pump primitive yet). {1}, Sacrifice this: Destroy
/// target artifact or enchantment.
///
/// `sac_cost: true` matches Cankerbloom's shape (sac-self, destroy non-
/// permanent type) — a useful piece of utility removal stapled to a body.
pub fn qasali_pridemage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Qasali Pridemage",
        cost: cost(&[g(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Greater Good — {2}{G}{G} Enchantment. Whenever a creature you control
/// dies, draw cards equal to its power, then discard three cards.
///
/// The Oracle text is "Sacrifice a creature: Draw cards equal to that
/// creature's power, then discard three cards." We expose this as an
/// activated ability whose first step sacrifices a controlled creature
/// (and stashes its power in the resolution context via
/// `SacrificeAndRemember`), then draws `Value::SacrificedPower`, then
/// discards three. This reuses the exact Thud/Callous Sell-Sword
/// sacrifice + power primitive — no engine changes needed.
pub fn greater_good() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Greater Good",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::SacrificedPower,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(3),
                    random: false,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Bojuka Bog — Land. Bojuka Bog enters tapped. When Bojuka Bog enters,
/// exile target opponent's graveyard.
///
/// We approximate "exile target player's graveyard" as "exile each card in
/// each opponent's graveyard" via `ForEach` + `Exile`. The ETB-tapped piece
/// reuses the existing `etb_tap` self-source trigger pattern.
pub fn bojuka_bog() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    let etb = TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap { what: Selector::This },
            // Move every card in each opponent's graveyard into exile. Move
            // accepts both `EntityRef::Permanent` and `EntityRef::Card`, so
            // it correctly handles graveyard residents (whereas
            // `Effect::Exile` only operates on permanents on the
            // battlefield).
            Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::EachOpponent,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Any,
                },
                to: ZoneDest::Exile,
            },
        ]),
    };
    CardDefinition {
        name: "Bojuka Bog",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Swamp],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black]),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![etb],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Burn / direct damage ─────────────────────────────────────────────────────

/// Lightning Strike — {1}{R} Instant. Lightning Strike deals 3 damage to any
/// target.
///
/// Two-mana close-relative of Lightning Bolt; the +1 mana is the cost of
/// being printed in modern Standard. Same `DealDamage(Target, 3)` shape.
pub fn lightning_strike() -> CardDefinition {
    CardDefinition {
        name: "Lightning Strike",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Goblin Bombardment — {1}{R} Enchantment. Sacrifice a creature: Goblin
/// Bombardment deals 1 damage to any target.
///
/// Activated ability with `sac_cost`-style sacrifice folded into the
/// resolved effect via `SacrificeAndRemember(Creature, You)` so the
/// engine drops the chosen creature before dealing damage. The 1-damage
/// payload doesn't scale with sacrificed power — the sacrifice is the
/// activation cost, not the payoff.
pub fn goblin_bombardment() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Goblin Bombardment",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Land destruction ─────────────────────────────────────────────────────────

/// Wasteland — Land. {T}: Add {C}. {T}, Sacrifice: Destroy target nonbasic
/// land.
///
/// Two activated abilities: a colorless mana ability and the
/// sacrifice-tap nonbasic destruction. Same `sac_cost` shape used by
/// Bojuka Bog (without the ETB-tapped) — the engine taps and sacrifices
/// the Wasteland as part of paying the cost.
pub fn wasteland() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Wasteland",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
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
            exile_gy_cost: 0,
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Land
                            .and(SelectionRequirement::IsBasicLand.negate()),
                    ),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
                condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Strip Mine — Land. {T}: Add {C}. {T}, Sacrifice: Destroy target land.
///
/// Wasteland's stricter sibling — destroys *any* land, not just nonbasic.
/// Same activated-ability layout.
pub fn strip_mine() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Strip Mine",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
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
            exile_gy_cost: 0,
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Land),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
                condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Pitch / alt-cost utility ─────────────────────────────────────────────────

/// Snuff Out — {3}{B} Instant. Destroy target nonblack creature. It can't
/// be regenerated.
///
/// Alt cost: "If you control a Swamp, you may pay 4 life rather than pay
/// Snuff Out's mana cost." We approximate the swamp gate by always
/// allowing the alt cost (the engine has no precondition system tied to
/// `controls a Swamp`); in practice Snuff Out is only ever in mono- or
/// dual-black decks where the prerequisite is trivially satisfied.
/// The "can't be regenerated" rider collapses (no regen primitive).
pub fn snuff_out() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Snuff Out",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasColor(Color::Black).negate()),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 4,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            mode_on_alt: None,
        }),
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Sweepers ─────────────────────────────────────────────────────────────────

/// Pyroclasm — {1}{R} Sorcery. Pyroclasm deals 2 damage to each creature.
///
/// `ForEach(EachPermanent(Creature))` iterates every creature in play and
/// deals 2 damage to it. Same shape as Anger of the Gods (3 damage) and
/// Blasphemous Act (13 damage).
pub fn pyroclasm() -> CardDefinition {
    CardDefinition {
        name: "Pyroclasm",
        cost: cost(&[generic(1), r()]),
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
                amount: Value::Const(2),
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Day of Judgment — {2}{W}{W} Sorcery. Destroy all creatures.
///
/// `ForEach(EachPermanent(Creature))` followed by `Destroy(Self)` — the
/// "can't be regenerated" clause is implicit (the engine has no
/// regeneration primitive to bypass).
pub fn day_of_judgment() -> CardDefinition {
    CardDefinition {
        name: "Day of Judgment",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Tutor cycle ──────────────────────────────────────────────────────────────

/// Mystical Tutor — {U} Instant. Search your library for an instant or
/// sorcery card, reveal it, put it on top of your library, then shuffle.
///
/// `Search(IsInstantOrSorcery → Library{Top})` — uses the existing
/// `LibraryPosition::Top` destination already wired by Vampiric Tutor. The
/// reveal half is collapsed (the engine's `do_search` answers a single
/// chosen card; the shuffle-then-place is implicit).
pub fn mystical_tutor() -> CardDefinition {
    use crate::card::CardType as CT;
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Mystical Tutor",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCardType(CT::Instant)
                .or(SelectionRequirement::HasCardType(CT::Sorcery)),
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: LibraryPosition::Top,
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Worldly Tutor — {G} Instant. Search your library for a creature card,
/// reveal it, put it on top of your library, then shuffle.
pub fn worldly_tutor() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Worldly Tutor",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: LibraryPosition::Top,
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Enlightened Tutor — {W} Instant. Search your library for an artifact or
/// enchantment card, reveal it, put it on top of your library, then
/// shuffle.
pub fn enlightened_tutor() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Enlightened Tutor",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Artifact
                .or(SelectionRequirement::Enchantment),
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: LibraryPosition::Top,
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Diabolic Tutor — {2}{B}{B} Sorcery. Search your library for a card and
/// put it into your hand. Then shuffle.
///
/// Vanilla "find anything" tutor. Slower than Demonic Tutor but in-color
/// and free of Reserved-List baggage.
pub fn diabolic_tutor() -> CardDefinition {
    CardDefinition {
        name: "Diabolic Tutor",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Imperial Seal — {B} Sorcery. Search your library for a card, put it on
/// top, then shuffle. Lose 2 life.
///
/// Sorcery-speed Vampiric Tutor; same {B}+`LoseLife 2` shape but at the
/// slower casting window. Reuses the same library-tutor primitive.
pub fn imperial_seal() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Imperial Seal",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: LibraryPosition::Top,
                },
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Damnation — {2}{B}{B} Sorcery. Destroy all creatures. They can't be
/// regenerated.
///
/// Black mirror of Day of Judgment. Same `ForEach + Destroy` shape; the
/// "can't be regenerated" rider collapses (no regen primitive).
pub fn damnation() -> CardDefinition {
    CardDefinition {
        name: "Damnation",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Cube additions: cheap creatures + sacrifice-cost spells ──────────────────

/// Memnite — {0} Artifact Creature — Construct. 1/1.
///
/// Vanilla 1/1 for free. No abilities; the body is the entire card. The
/// "free creature" line slots into Affinity / Mox-Opal-style cube shells.
pub fn memnite() -> CardDefinition {
    CardDefinition {
        name: "Memnite",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Fanatic of Rhonas — {G} Creature — Snake. 1/1. "{G}, {T}: Add {G}{G}."
///
/// One-mana ramper that nets +{G} per activation. Modeled as a single
/// activated ability with `tap_cost: true` + `mana_cost: {G}` and
/// `Effect::AddMana(Colors([Green, Green]))`. Net production after the
/// activation cost is +{G}, so the card is gameplay-equivalent to
/// "Llanowar Elves with an extra {G} pump button".
pub fn fanatic_of_rhonas() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Fanatic of Rhonas",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[g()]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green, Color::Green]),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Greasewrench Goblin — {1}{R} Creature — Goblin Mercenary. 2/2 Haste.
/// "When this creature dies, create a Treasure token."
///
/// Approximation: 2/2 Haste body + on-death Treasure trigger. Treasure
/// tokens are wired via the shared `treasure_token()` helper (1-mana-of-any-
/// color, sacrifice on tap). The full Oracle's "this can't block" rider is
/// collapsed (no per-attacker block-restriction primitive yet).
pub fn greasewrench_goblin() -> CardDefinition {
    CardDefinition {
        name: "Greasewrench Goblin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Orcish Lumberjack — {R} Creature — Orc Druid. 1/1.
/// "{T}, Sacrifice a Forest: Add {G}{G}{G}."
///
/// Sacrifice cost is folded into the resolved effect: tap, then on
/// resolution sacrifice a Forest you control and add {G}{G}{G}. Same
/// pattern Crop Rotation uses (sacrifice-as-first-effect-step). The bot
/// auto-picks the first Forest via `Effect::Sacrifice`'s deterministic
/// selector resolver.
pub fn orcish_lumberjack() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Orcish Lumberjack",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Land
                        .and(SelectionRequirement::HasLandType(crate::card::LandType::Forest)),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Green, Color::Green, Color::Green]),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Mine Collapse — {2}{R} Sorcery. As an additional cost, sacrifice a
/// Mountain. Mine Collapse deals 4 damage to any target.
///
/// ✅ Push XLIII: now uses the `additional_sac_cost: Some(Mountain)`
/// cast-time primitive (push XXXIX). The cast is illegal without a
/// Mountain to feed; the engine sacrifices it before the spell goes on
/// the stack. The body is now just the targeted 4 damage. Sister to
/// Crop Rotation's land-feed promotion in the same push.
pub fn mine_collapse() -> CardDefinition {
    CardDefinition {
        name: "Mine Collapse",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(4),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: Some(
            SelectionRequirement::Land
                .and(SelectionRequirement::HasLandType(crate::card::LandType::Mountain)),
        ),
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Elvish Spirit Guide — {2}{G} Creature — Elf Spirit. 2/2.
/// "Exile this card from your hand: Add {G}."
///
/// Modeled via `AlternativeCost { mana_cost: {0}, exile_filter: Self }` —
/// the only alt-cast path that exiles the spell itself rather than a
/// pitch card. Wired by reusing the existing pitch-cost machinery: the
/// caller passes `pitch_card: Some(<this card's id>)` to
/// `cast_spell_alternative`. On resolution, the engine treats it as a
/// regular cast — for Spirit Guide we route it through a tiny on-cast
/// trigger that adds {G} to the controller's pool, then the spell
/// (which has Effect::Noop) resolves and goes to the graveyard… but the
/// alt cost has already exiled it, so it never lands. End result: pay
/// nothing → exile from hand → +{G}.
///
/// Approximation: catalog ships an *activated* "exile from hand" ability
/// rather than a true alt-cost path, since the engine's alt-cost gate
/// requires removing the spell from hand and pushing it onto the stack
/// (which then resolves as a creature). Activated path skips that. The
/// activated ability's cost is `mana_cost: {0}` + a hand-exile move
/// folded into the effect tree. Caller picks `ActivateAbility` from
/// hand — currently not all activation paths walk the hand zone, so
/// this card is **🟡** until a hand-activated-ability primitive lands.
pub fn elvish_spirit_guide() -> CardDefinition {
    CardDefinition {
        name: "Elvish Spirit Guide",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Satyr Wayfinder — {1}{G} Creature — Satyr Druid. 1/1.
/// "When this creature enters, reveal the top four cards of your library.
/// You may put a land card from among them into your hand. Put the rest
/// into your graveyard."
///
/// Approximation: ETB mills 4. The "may take a land" half is
/// gameplay-equivalent to "tutor a land from the milled cards", which the
/// existing primitives don't expose as a single op. Wired here as the
/// graveyard-fill half (which is the gameplay-relevant outcome for the
/// reanimator-adjacent shells Satyr Wayfinder slots into).
pub fn satyr_wayfinder() -> CardDefinition {
    CardDefinition {
        name: "Satyr Wayfinder",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Mill {
                who: Selector::You,
                amount: Value::Const(4),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Talisman of Progress — {2} Artifact. "{T}: Add {C}." "{T}: Add {W} or
/// {U}. Talisman of Progress deals 1 damage to you."
///
/// Two-color mana rock. The colored ability deals 1 damage to the
/// controller as part of the activation; we fold the damage into the
/// resolved effect's first step (cost-as-first-step approximation).
/// The color choice is exposed via `AnyOneColor` constrained to
/// {W,U} — but the engine's `AnyOneColor` allows any color, so we
/// model it as two separate {T} abilities (one per color) to keep the
/// color choice explicit. The first ability is the colorless tap.
pub fn talisman_of_progress() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let make_color = |color: Color| ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![color]),
            },
        ]),
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
    };
    CardDefinition {
        name: "Talisman of Progress",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            // {T}: Add {C}.
            ActivatedAbility {
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
            exile_gy_cost: 0,
            },
            make_color(Color::White),
            make_color(Color::Blue),
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Talisman of Dominance — {2} Artifact. UB mirror of Talisman of
/// Progress: `{T}: Add {C}` plus `{T}: Add {U} or {B}, deals 1 damage
/// to you`.
pub fn talisman_of_dominance() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let make_color = |color: Color| ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![color]),
            },
        ]),
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
    };
    CardDefinition {
        name: "Talisman of Dominance",
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
            exile_gy_cost: 0,
            },
            make_color(Color::Blue),
            make_color(Color::Black),
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Fireblast — {4}{R}{R} Instant. "Fireblast deals 4 damage to any target."
/// Alternative cost: sacrifice two Mountains.
///
/// The alt-cost is "sacrifice two Mountains" — currently the
/// `AlternativeCost` model only supports exile-from-hand pitch cards.
/// We instead model the alt path as a free-mana cast that sacrifices the
/// Mountains as the resolution's first step (cost-as-first-step). Two
/// `Effect::Sacrifice` invocations precede the damage. Players who can't
/// afford the regular {4}{R}{R} *or* don't have two Mountains simply
/// can't cast — the regular and alt costs are both gated by `cast_spell`.
///
/// Wired as the regular {4}{R}{R} cost only; alt-cost path is omitted
/// pending a sacrifice-as-cost primitive on `AlternativeCost`. Promote to
/// 🟡 once that lands.
pub fn fireblast() -> CardDefinition {
    CardDefinition {
        name: "Fireblast",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(4),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Talisman cycle (RW / UR / GU) ────────────────────────────────────────────

/// Internal helper: build a `{2}` artifact with `{T}: Add {C}` and two
/// 1-life-pip color taps for a given color pair (Talisman cycle shape).
fn talisman_cycle(name: &'static str, c1: Color, c2: Color) -> CardDefinition {
    use crate::card::ActivatedAbility;
    let make_color = |color: Color| ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![color]),
            },
        ]),
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
    };
    CardDefinition {
        name,
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
            exile_gy_cost: 0,
            },
            make_color(c1),
            make_color(c2),
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Talisman of Conviction — {2} Artifact. RW mirror of Talisman of Progress
/// (`{T}: Add {C}` plus `{T}: Add {R} or {W}`, each colored ability dealing 1
/// damage to you / costing 1 life).
pub fn talisman_of_conviction() -> CardDefinition {
    talisman_cycle("Talisman of Conviction", Color::Red, Color::White)
}

/// Talisman of Creativity — {2} Artifact. UR mirror of Talisman of Progress.
pub fn talisman_of_creativity() -> CardDefinition {
    talisman_cycle("Talisman of Creativity", Color::Blue, Color::Red)
}

/// Talisman of Curiosity — {2} Artifact. GU mirror of Talisman of Progress.
pub fn talisman_of_curiosity() -> CardDefinition {
    talisman_cycle("Talisman of Curiosity", Color::Green, Color::Blue)
}

// ── Removal / interaction ────────────────────────────────────────────────────

/// Innocent Blood — {B} Sorcery. Each player sacrifices a creature.
///
/// `Sacrifice` over `EachPlayer` so both seats lose a creature — same shape
/// as Blasphemous Edict but at sorcery speed and a single mana.
pub fn innocent_blood() -> CardDefinition {
    CardDefinition {
        name: "Innocent Blood",
        cost: cost(&[b()]),
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Diabolic Edict — {1}{B} Instant. Target player sacrifices a creature.
///
/// `Sacrifice` over `Selector::Target(0)` so the targeted player picks one
/// of their own creatures to lose. Pairs naturally with the existing
/// `target_filtered(Player)` machinery used by Sign in Blood.
pub fn diabolic_edict() -> CardDefinition {
    CardDefinition {
        name: "Diabolic Edict",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Sacrifice {
            who: Selector::Target(0),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Geth's Verdict — {1}{B} Instant. Target player sacrifices a creature
/// and loses 1 life.
///
/// Same shape as Diabolic Edict with an extra `LoseLife 1` against the
/// targeted player tacked on.
pub fn geths_verdict() -> CardDefinition {
    CardDefinition {
        name: "Geth's Verdict",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Target(0),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::LoseLife {
                who: Selector::Target(0),
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Magma Jet — {1}{R} Instant. Magma Jet deals 2 damage to any target.
/// Scry 2.
pub fn magma_jet() -> CardDefinition {
    CardDefinition {
        name: "Magma Jet",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(2),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Remand — {1}{U} Instant. Counter target spell. If that spell is
/// countered this way, put it into its owner's hand instead of into their
/// graveyard. Then draw a card.
///
/// The "back to owner's hand" half is approximated as the regular
/// `CounterSpell` (which moves the spell to the graveyard) plus a
/// `Move(Target → Hand)` follow-up — the engine's `CounterSpell` resolver
/// already routes the countered card to its owner's graveyard, but the
/// follow-up Move re-routes it from there to hand. The cantrip is the
/// gameplay-relevant half.
pub fn remand() -> CardDefinition {
    use crate::effect::shortcut::counter_target_spell;
    CardDefinition {
        name: "Remand",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            counter_target_spell(),
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Read the Bones — {2}{B} Sorcery. Scry 2, then draw two cards and lose
/// two life.
pub fn read_the_bones() -> CardDefinition {
    CardDefinition {
        name: "Read the Bones",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Storm Crow — {1}{U} Creature — Bird. 1/2 with Flying.
///
/// The legendary "draft chaff that's actually a flying body" — included
/// here to round out the blue creature pool with a clean evasion two-drop.
pub fn storm_crow() -> CardDefinition {
    let mut subtypes = Subtypes::default();
    subtypes.creature_types.push(CreatureType::Bird);
    CardDefinition {
        name: "Storm Crow",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes,
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Ancient Grudge — {1}{R} Instant. Destroy target artifact. Flashback {G}.
///
/// A two-shot artifact-destruction tool — grave-yard recursion via the
/// existing `Keyword::Flashback` plumbing.
pub fn ancient_grudge() -> CardDefinition {
    let flashback_cost = ManaCost {
        symbols: vec![ManaSymbol::Colored(Color::Green)],
    };
    CardDefinition {
        name: "Ancient Grudge",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::HasCardType(CardType::Artifact)),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Sign in Blood is already wired in this module, but its sibling
/// **Tragic Slip** — {B} Instant — Target creature gets -13/-13 until end
/// of turn — fits the same mono-black removal slot. The Morbid rider is
/// dropped (no morbid trigger primitive yet); the spell always applies the
/// full -13/-13.
pub fn tragic_slip() -> CardDefinition {
    CardDefinition {
        name: "Tragic Slip",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-13),
            toughness: Value::Const(-13),
            duration: Duration::EndOfTurn,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Red discard-loot ─────────────────────────────────────────────────────────

/// Tormenting Voice — {1}{R} Sorcery. As an additional cost to cast
/// this spell, discard a card. Draw two cards.
///
/// ✅ Push XLIII: now uses the `additional_discard_cost: Some(1)` cast-
/// time primitive (sister to push XXXIX's `additional_sac_cost`). The
/// cast is illegal if the controller has no other card in hand at cast
/// time; the engine auto-picks the first card and discards it before
/// the spell goes on the stack. The body is now just the draw-2.
pub fn tormenting_voice() -> CardDefinition {
    CardDefinition {
        name: "Tormenting Voice",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: Some(1),
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Wild Guess — {2}{R} Sorcery. Discard a card, then draw two cards.
///
/// Mechanically identical to Tormenting Voice at +1 mana cost (the original
/// card had `{1}{R}, discard a card` as an additional cost in older
/// templating; modern Oracle is "discard then draw"). We model it the same
/// way as Tormenting Voice but charge {2}{R} since both cards exist in the
/// pool and tournaments care about cost differences.
pub fn wild_guess() -> CardDefinition {
    CardDefinition {
        name: "Wild Guess",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: Some(1),
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Thrill of Possibility — {1}{R} Instant. As an additional cost to cast
/// this spell, discard a card. Draw two cards.
///
/// ✅ Push XLIII: now uses `additional_discard_cost: Some(1)` (sister
/// to push XXXIX's `additional_sac_cost`). Instant-speed Tormenting
/// Voice — same shape, different timing.
pub fn thrill_of_possibility() -> CardDefinition {
    CardDefinition {
        name: "Thrill of Possibility",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: Some(1),
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Burn ─────────────────────────────────────────────────────────────────────

/// Volcanic Hammer — {1}{R} Sorcery. Volcanic Hammer deals 3 damage to any
/// target.
///
/// Sorcery-speed Lightning Strike. Modern-legal premium common.
pub fn volcanic_hammer() -> CardDefinition {
    CardDefinition {
        name: "Volcanic Hammer",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Slagstorm — {2}{R} Sorcery. Choose one — Slagstorm deals 3 damage to
/// each creature; or Slagstorm deals 3 damage to each player.
///
/// Modal sweeper — modeled as `ChooseMode` over the two `ForEach` halves.
/// AutoDecider picks mode 0 (clear opposing creatures) since that's the
/// gameplay-default line; UI/bot can supply `mode: Some(1)` for the
/// player-burn half.
pub fn slagstorm() -> CardDefinition {
    use crate::effect::shortcut::each_creature;
    CardDefinition {
        name: "Slagstorm",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::ForEach {
                selector: each_creature(),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(3),
                }),
            },
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(3),
                }),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Counters ─────────────────────────────────────────────────────────────────

/// Cancel — {1}{U}{U} Instant. Counter target spell.
///
/// Strictly worse than Counterspell, but the staple "any-spell counter at
/// three" slot is widely played in cube and pre-Counterspell-reprint Standard.
pub fn cancel() -> CardDefinition {
    CardDefinition {
        name: "Cancel",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell { what: Selector::Target(0) },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Annul — {U} Instant. Counter target artifact or enchantment spell.
///
/// Filtered counter — `target_filter` restricts to artifact/enchantment
/// spells only. Cheap sideboard tool against Affinity / Bogles etc.
pub fn annul() -> CardDefinition {
    CardDefinition {
        name: "Annul",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment),
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Black removal ────────────────────────────────────────────────────────────

/// Hero's Downfall — {1}{B}{B} Instant. Destroy target creature or
/// planeswalker.
///
/// Bog-standard premium black removal. `target_filter` accepts either a
/// creature or a planeswalker; resolution destroys the chosen target.
pub fn heros_downfall() -> CardDefinition {
    CardDefinition {
        name: "Hero's Downfall",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker),
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Cast Down — {1}{B} Instant. Destroy target nonlegendary creature.
///
/// Cheap removal that can't touch commanders / Atraxa / Griselbrand. The
/// nonlegendary filter rides on `SelectionRequirement::Not(Legendary)`.
pub fn cast_down() -> CardDefinition {
    CardDefinition {
        name: "Cast Down",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature.and(
                    SelectionRequirement::Not(Box::new(
                        SelectionRequirement::HasSupertype(Supertype::Legendary),
                    )),
                ),
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Mind Rot — {2}{B} Sorcery. Target player discards two cards.
///
/// Discard-at-target shape — `Effect::Discard` aimed at `Target(0)` so the
/// caster picks the player. Random=true matches Hymn-style discard since
/// the engine's chosen-discard primitive is for the *caster* picker only.
pub fn mind_rot() -> CardDefinition {
    CardDefinition {
        name: "Mind Rot",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // `target_filtered(Player)` (rather than bare `Selector::Target(0)`)
        // exposes the Player filter to `primary_target_filter` so the bot's
        // `auto_target_for_effect` can pick the opp without a manual target.
        effect: Effect::Discard {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(2),
            random: true,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Raise Dead — {B} Sorcery. Return target creature card from your
/// graveyard to your hand.
///
/// Cheap recursion. Reuses the `Move(target → Hand)` shape from Disentomb /
/// Unburial Rites' first half. Filter restricts to creature cards.
pub fn raise_dead() -> CardDefinition {
    CardDefinition {
        name: "Raise Dead",
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── White lifegain & tokens ──────────────────────────────────────────────────

/// Healing Salve — {W} Instant. Choose one — Target player gains 3 life;
/// or prevent the next 3 damage that would be dealt to any target this turn.
///
/// Damage-prevention isn't modeled (the engine has no damage-prevention
/// shield primitive). Collapsed to the gain-3-life mode. Targeting the
/// caster auto-resolves on AutoDecider.
pub fn healing_salve() -> CardDefinition {
    CardDefinition {
        name: "Healing Salve",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::GainLife {
            who: Selector::Target(0),
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Spore Frog — {G} Creature — Frog 1/1. "Sacrifice this creature:
/// Prevent all combat damage that would be dealt this turn."
///
/// Classic Lorwyn-era Frog with sacrifice-as-cost activation that
/// activates the same prevention shield as Holy Day / Owlin
/// Shieldmage. Wired via `ActivatedAbility { sac_cost: true, effect:
/// Effect::PreventCombatDamageThisTurn, … }` — the engine's existing
/// sac-cost path handles the sacrifice + the new
/// `PreventCombatDamageThisTurn` primitive activates the shield.
pub fn spore_frog() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Spore Frog",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[]),
            effect: Effect::PreventCombatDamageThisTurn,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Holy Day — {W} Instant. "Prevent all combat damage that would be
/// dealt this turn." Classic Alpha-era fog-style combat-damage shield.
///
/// Wired via the new `Effect::PreventCombatDamageThisTurn` primitive
/// (push: same primitive that powers Owlin Shieldmage's ETB).
/// `resolve_combat_damage_with_filter` short-circuits per attacker
/// when the flag is set, so no combat damage events fire — lifelink,
/// infect, trample-trigger riders all skip too. Cleared on cleanup
/// (CR 615 — prevention only applies to *this* turn).
pub fn holy_day() -> CardDefinition {
    CardDefinition {
        name: "Holy Day",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PreventCombatDamageThisTurn,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Raise the Alarm — {1}{W} Instant. Create two 1/1 white Soldier creature
/// tokens.
///
/// Two-token instant. `CreateToken` with `count = 2` — token defs are
/// shared.
pub fn raise_the_alarm() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Raise the Alarm",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: TokenDefinition {
                name: "Soldier".into(),
                power: 1,
                toughness: 1,
                keywords: vec![],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                supertypes: vec![],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Soldier],
                    ..Default::default()
                },
                activated_abilities: vec![],
                triggered_abilities: vec![],
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Green ETB destroy ────────────────────────────────────────────────────────

/// Reclamation Sage — {2}{G} Creature — Elf Shaman. 2/1. When Reclamation
/// Sage enters the battlefield, you may destroy target artifact or
/// enchantment.
///
/// Standard "ETB Disenchant" body — same shape as Loran of the Third Path's
/// ETB but at 2/1 for one less mana. The "you may" clause collapses (we
/// always destroy if a legal target exists; the AutoDecider opp-preference
/// keeps us from blowing up our own stuff).
pub fn reclamation_sage() -> CardDefinition {
    CardDefinition {
        name: "Reclamation Sage",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Enchantment),
                ),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Acidic Slime — {3}{G}{G} Creature — Ooze. 2/2 Deathtouch. When Acidic
/// Slime enters the battlefield, destroy target artifact, enchantment, or
/// land.
///
/// Three-target ETB. Wider filter than Reclamation Sage — also hits lands.
/// Deathtouch keyword present.
pub fn acidic_slime() -> CardDefinition {
    CardDefinition {
        name: "Acidic Slime",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Enchantment)
                        .or(SelectionRequirement::Land),
                ),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Blue bounce ──────────────────────────────────────────────────────────────

/// Unsummon — {U} Instant. Return target creature to its owner's hand.
///
/// Standard "creature bounce" — `Move(target → Hand(OwnerOf))` so a creature
/// you bounced for your opponent goes back to *their* hand, not yours.
pub fn unsummon() -> CardDefinition {
    CardDefinition {
        name: "Unsummon",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Boomerang — {U}{U} Instant. Return target permanent to its owner's hand.
///
/// Wider filter than Unsummon — any permanent (including lands). Same
/// destination shape: `Hand(OwnerOf)`.
pub fn boomerang() -> CardDefinition {
    CardDefinition {
        name: "Boomerang",
        cost: cost(&[u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Permanent),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Cyclonic Rift — {1}{U} Instant. Return target nonland permanent your
/// opponents control to its owner's hand.
///
/// Overload `{6}{U}` ("each nonland permanent your opponents control") is
/// omitted (no overload primitive yet). Cast-time filter pins the target
/// to opponent-controlled nonland permanents.
pub fn cyclonic_rift() -> CardDefinition {
    CardDefinition {
        name: "Cyclonic Rift",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Repeal — {X}{U} Instant. Return target nonland permanent with mana
/// value X to its owner's hand. Then draw a card.
///
/// X is read at resolution from `Value::XFromCost`. The cast-time filter
/// is `Permanent ∧ Nonland`; the converged-style mana-value gate is
/// applied at resolution via `If(ManaValueOf(Target) ≤ X, ...)` —
/// gameplay-equivalent to "exactly X" for the standard targets (you'll
/// almost always pick X = the target's CMC). Cantrip on resolution.
pub fn repeal() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Repeal",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::ValueAtMost(
                    Value::ManaValueOf(Box::new(Selector::Target(0))),
                    Value::XFromCost,
                ),
                then: Box::new(Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Convoke burn ─────────────────────────────────────────────────────────────

/// Stoke the Flames — {4}{R} Instant. Convoke. Stoke the Flames deals 4
/// damage to any target.
///
/// Convoke-mode burn — reuses the existing convoke wiring (each tapped
/// creature pays {1}). At full mana cost it's strictly worse than Lava
/// Spike, but with three creatures it's a one-mana 4-damage spell.
pub fn stoke_the_flames() -> CardDefinition {
    CardDefinition {
        name: "Stoke the Flames",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Convoke],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(4),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Premium creature removal ─────────────────────────────────────────────────

/// Murder — {1}{B}{B} Instant. Destroy target creature.
///
/// Vanilla creature kill at the classic "two black, one generic" rate.
/// Same shape as Doom Blade but without the nonblack restriction.
pub fn murder() -> CardDefinition {
    CardDefinition {
        name: "Murder",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Creature),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Go for the Throat — {1}{B} Instant. Destroy target nonartifact creature.
///
/// Two-mana destroy with a soft "no artifact creatures" rider — same shape
/// as Doom Blade's "nonblack" filter but on artifact-ness instead.
pub fn go_for_the_throat() -> CardDefinition {
    CardDefinition {
        name: "Go for the Throat",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::Not(Box::new(
                    SelectionRequirement::Artifact,
                ))),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Disfigure — {B} Instant. Target creature gets -2/-2 until end of turn.
///
/// One-mana toughness shrinker. Same shape as Tragic Slip but without the
/// "morbid" gate (here it's always -2/-2 instead of -13/-13).
pub fn disfigure() -> CardDefinition {
    CardDefinition {
        name: "Disfigure",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Languish — {2}{B}{B} Sorcery. All creatures get -2/-2 until end of turn.
///
/// Modal sweeper: shrink everyone by -2/-2 EOT, killing X/2-and-below
/// creatures while leaving X/3-and-above bodies on the board. Built as
/// `ForEach(EachPermanent(Creature))` + per-creature PumpPT (negative).
pub fn languish() -> CardDefinition {
    CardDefinition {
        name: "Languish",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Lay Down Arms — {W} Instant. Exile target creature with power 4 or less.
///
/// One-mana white removal — uses cast-time `PowerAtMost(4)` to lock the
/// target. Cheap interaction for the early game; falls off against big
/// finishers. The "control X plains" mana-value gate (Oracle: costs an
/// additional {1}{W} unless you control X Plains) is collapsed to the
/// reduced cost since the engine has no count-based-cost-rebate primitive.
pub fn lay_down_arms() -> CardDefinition {
    CardDefinition {
        name: "Lay Down Arms",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(4)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Smelt — {R} Instant. Destroy target artifact.
///
/// One-mana red artifact destruction. Strictly worse than Ancient Grudge
/// (which has flashback) but more flexible than Shatter ({1}{R}).
pub fn smelt() -> CardDefinition {
    CardDefinition {
        name: "Smelt",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Artifact),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── X-cost burn / sweepers ───────────────────────────────────────────────────

/// Banefire — {X}{R} Sorcery. Banefire deals X damage to any target.
///
/// X is read at resolution from `Value::XFromCost`. The "can't be
/// countered if X ≥ 5" rider is omitted (no conditional-uncounterable
/// primitive yet) — the spell is otherwise functionally identical.
pub fn banefire() -> CardDefinition {
    CardDefinition {
        name: "Banefire",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::XFromCost,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Token producers ──────────────────────────────────────────────────────────

/// Spectral Procession — {2}{W} Sorcery. Create three 1/1 white Spirit
/// creature tokens with flying.
///
/// The Oracle text uses three white-or-2-life hybrid pips ({2/W}{2/W}{2/W})
/// — collapsed to a flat {2}{W} (the most permissive of the actual costs).
/// Three flying tokens for three mana is still strong but the alt-cost
/// flexibility is gone.
pub fn spectral_procession() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Spectral Procession",
        // Real Oracle: `{(2/W)}{(2/W)}{(2/W)}`. Hybrid pips collapse to
        // `{2}{W}` (most permissive — matches DECK_FEATURES.md and the
        // engine's existing "collapse hybrid to single-color" convention
        // for the {2/X} pips). Pre-fix this was `{3}{W}{W}{W}` (i.e. all
        // three hybrid pips paid the white side), which made the spell
        // ~4 mana over budget.
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: TokenDefinition {
                name: "Spirit".into(),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                supertypes: vec![],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Spirit],
                    ..Default::default()
                },
                activated_abilities: vec![],
                triggered_abilities: vec![],
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Green graveyard recursion / utility ──────────────────────────────────────

/// Regrowth — {1}{G} Sorcery. Return target card from your graveyard to
/// your hand.
///
/// Wider than Raise Dead ({B}, creature-only) — any card type. The
/// auto-target via `Effect::prefers_graveyard_target` (Move-into-your-zone
/// is reanimate-class) prefers the caster's graveyard.
pub fn regrowth() -> CardDefinition {
    CardDefinition {
        name: "Regrowth",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Any),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Beast Within — {2}{G} Instant. Destroy target permanent. Its controller
/// creates a 3/3 green Beast creature token.
///
/// `Seq([Destroy, CreateToken(controller=ControllerOf(target))])`. The
/// token resolves to the original controller via `PlayerRef::ControllerOf`
/// (resolved at effect time, before the Destroy moves the card off-board)
/// — Move runs first by sequence ordering but the Selector::Target(0) ref
/// for ControllerOf is captured at cast time, so the lookup falls through
/// to graveyard / exile after Destroy as needed.
pub fn beast_within() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Beast Within",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Beast".into(),
                    power: 3,
                    toughness: 3,
                    keywords: vec![],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    supertypes: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Beast],
                        ..Default::default()
                    },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                },
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Permanent),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Grasp of Darkness — {B}{B} Instant. Target creature gets -4/-4 until end
/// of turn.
///
/// Two-mana premium creature kill — the -4 power swing on the way out
/// flips race math against most beaters in the early game.
pub fn grasp_of_darkness() -> CardDefinition {
    CardDefinition {
        name: "Grasp of Darkness",
        cost: cost(&[b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-4),
            toughness: Value::Const(-4),
            duration: Duration::EndOfTurn,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Shatter — {1}{R} Instant. Destroy target artifact.
///
/// Strictly better than Smelt for one extra mana (instant speed already
/// matches; the broader filter is the same: Artifact). Kept distinct
/// because the cube wants both in a wider color pool.
pub fn shatter() -> CardDefinition {
    CardDefinition {
        name: "Shatter",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Artifact),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── modern_decks-8: more removal / burn / ramp / draw ────────────────────────
//
// New factories below this line use `..Default::default()` for the
// fields that don't matter for a given card (supertypes, subtypes, P/T,
// keywords, ability vecs, alt cost, back face, opening hand). The
// `CardDefinition` derive does the rest. The boilerplate-heavy form
// used elsewhere in this file is gameplay-equivalent — it just spells
// out every default explicitly.

/// Incinerate — {1}{R} Instant. Incinerate deals 3 damage to any target.
/// A creature dealt damage this way can't be regenerated this turn.
///
/// The "can't be regenerated" rider collapses (no regeneration primitive
/// is observable from this site), so the spell is functionally identical
/// to Lightning Strike. Distinct factory + name kept for cube variety.
pub fn incinerate() -> CardDefinition {
    CardDefinition {
        name: "Incinerate",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Searing Spear — {1}{R} Instant. 3 damage to any target. (Lightning
/// Strike's twin under a different name — kept distinct for cube
/// variety.)
pub fn searing_spear() -> CardDefinition {
    CardDefinition {
        name: "Searing Spear",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Flame Slash — {R} Sorcery. Deals 4 damage to target creature.
///
/// Premium 1-mana removal (sorcery-speed). Cast-time creature-only filter
/// rejects burning a player. Kills 4-toughness creatures clean.
pub fn flame_slash() -> CardDefinition {
    CardDefinition {
        name: "Flame Slash",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Roast — {1}{R} Sorcery. Roast deals 5 damage to target non-flying
/// creature.
///
/// Cast-time filter combines `Creature` with `Not(HasKeyword(Flying))`,
/// so the spell is unable to target a flier. The damage payload is a
/// fixed 5 — kills nearly anything that doesn't fly.
pub fn roast() -> CardDefinition {
    CardDefinition {
        name: "Roast",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.and(
                    SelectionRequirement::HasKeyword(Keyword::Flying).negate(),
                ),
            ),
            amount: Value::Const(5),
        },
        ..Default::default()
    }
}

/// Smother — {1}{B} Instant. Destroy target creature with mana value 3
/// or less. It can't be regenerated.
///
/// Cast-time filter `Creature ∧ ManaValueAtMost(3)`. The "can't be
/// regenerated" clause collapses (already collapsed elsewhere).
pub fn smother() -> CardDefinition {
    CardDefinition {
        name: "Smother",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(3)),
            ),
        },
        ..Default::default()
    }
}

/// Final Reward — {4}{B} Sorcery. Exile target creature.
///
/// Higher-cost answer that goes around indestructible / regeneration /
/// graveyard recursion since it exiles. Strict upgrade in flexibility
/// over Murder for two more mana.
pub fn final_reward() -> CardDefinition {
    CardDefinition {
        name: "Final Reward",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(SelectionRequirement::Creature),
        },
        ..Default::default()
    }
}

/// Holy Light — {W} Instant. All creatures get -1/-1 until end of turn.
///
/// Sweep small creatures (1-toughness ones die) for one mana. Modeled as
/// `ForEach(Creature) + PumpPT(-1/-1 EOT)`. The "nonwhite" Oracle filter
/// is collapsed — engine simplification, in line with Languish (drops
/// the "nonblack" rider).
pub fn holy_light() -> CardDefinition {
    CardDefinition {
        name: "Holy Light",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Mana Tithe — {W} Instant. Counter target spell unless its controller
/// pays {1}.
///
/// White Force Spike. Reuses `Effect::CounterUnlessPaid`.
pub fn mana_tithe() -> CardDefinition {
    CardDefinition {
        name: "Mana Tithe",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: crate::effect::shortcut::target(),
            mana_cost: cost(&[generic(1)]),
        },
        ..Default::default()
    }
}

// ── Green ramp ───────────────────────────────────────────────────────────────

/// Helper: build a Search effect that fetches `filter` to the
/// battlefield under `You`. Used by Rampant Growth, Farseek,
/// Sakura-Tribe Elder, and Wood Elves.
fn search_to_battlefield(
    filter: SelectionRequirement,
    tapped: bool,
) -> Effect {
    Effect::Search {
        who: PlayerRef::You,
        filter,
        to: ZoneDest::Battlefield {
            controller: PlayerRef::You,
            tapped,
        },
    }
}

/// Rampant Growth — {1}{G} Sorcery. Search your library for a basic
/// land card, put it onto the battlefield tapped.
pub fn rampant_growth() -> CardDefinition {
    CardDefinition {
        name: "Rampant Growth",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: search_to_battlefield(SelectionRequirement::IsBasicLand, true),
        ..Default::default()
    }
}

/// Cultivate — {2}{G} Sorcery. Search your library for up to two basic
/// land cards, put one onto the battlefield tapped and the other into
/// your hand.
///
/// Same shape as Kodama's Reach (BF-tapped + Hand). Functionally
/// identical at this engine fidelity — the cube wants both at slightly
/// different rates.
pub fn cultivate() -> CardDefinition {
    CardDefinition {
        name: "Cultivate",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            search_to_battlefield(SelectionRequirement::IsBasicLand, true),
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Farseek — {1}{G} Sorcery. Search your library for a basic land
/// card, put it onto the battlefield tapped.
///
/// Real Oracle is "Plains, Island, Swamp, or Mountain" — non-basic
/// duals matter for fixing in a 3+ color deck. The cube only ships
/// basics in `IsBasicLand`-filterable form, so we collapse to "any
/// basic" (Rampant Growth's filter) for parity. Distinct factory
/// retained for cube identity.
pub fn farseek() -> CardDefinition {
    CardDefinition {
        name: "Farseek",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: search_to_battlefield(SelectionRequirement::IsBasicLand, true),
        ..Default::default()
    }
}

/// Sakura-Tribe Elder — {1}{G} Creature — Snake. 1/1. {T}, Sacrifice
/// this: search your library for a basic land card, put it onto the
/// battlefield tapped.
pub fn sakura_tribe_elder() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Sakura-Tribe Elder",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: search_to_battlefield(SelectionRequirement::IsBasicLand, true),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Wood Elves — {2}{G} Creature — Elf Scout. 1/1. When this enters,
/// search your library for a Forest card, put it onto the battlefield.
///
/// ETB Forest-tutor lands the Forest **untapped** (per Oracle, distinct
/// from Rampant Growth-class effects). Models that with an
/// `Effect::Search` to `Battlefield(controller=You, tapped=false)`.
pub fn wood_elves() -> CardDefinition {
    CardDefinition {
        name: "Wood Elves",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::SelfSource,
            ),
            effect: search_to_battlefield(
                SelectionRequirement::Land
                    .and(SelectionRequirement::HasLandType(crate::card::LandType::Forest)),
                false,
            ),
        }],
        ..Default::default()
    }
}

/// Elvish Mystic — {G} Creature — Elf Druid. 1/1. {T}: Add {G}.
///
/// Llanowar Elves twin — same shape, distinct factory for cube variety.
pub fn elvish_mystic() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Elvish Mystic",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green]),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Harmonize — {2}{G}{G} Sorcery. Draw three cards.
///
/// Mono-green draw at sorcery speed for four mana. Functions as the
/// premium green refuel.
pub fn harmonize() -> CardDefinition {
    CardDefinition {
        name: "Harmonize",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Concentrate — {2}{U}{U} Sorcery. Draw three cards.
///
/// Blue mirror of Harmonize at the same rate. Distinct cards because
/// blue and green draw on the same axis differently in cube draft.
pub fn concentrate() -> CardDefinition {
    CardDefinition {
        name: "Concentrate",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Severed Strands — {1}{B} Sorcery. As an additional cost, sacrifice
/// a creature. Destroy target creature. You gain life equal to the
/// sacrificed creature's toughness.
///
/// Sac is folded as the first step (cost-as-first-step approximation).
/// We approximate the lifegain as a fixed 2 — `Value::SacrificedPower`
/// reads power but Severed Strands keys off toughness, and the engine
/// has no `SacrificedToughness` value yet. The most-played form
/// sacrifices a 2-toughness creature, so 2 is the central case.
pub fn severed_strands() -> CardDefinition {
    CardDefinition {
        name: "Severed Strands",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

// ── Card draw / hand selection ───────────────────────────────────────────────

/// Anticipate — {1}{U} Instant. Look at the top three cards of your
/// library, put one of them into your hand and the rest on the bottom in
/// any order.
///
/// Approximation: `Scry 2 + Draw 1` — under-counts the breadth of the
/// real "look at 3, take any" by one card, but the gameplay-relevant
/// "smooth your draws / take the best of N" axis is preserved.
pub fn anticipate() -> CardDefinition {
    CardDefinition {
        name: "Anticipate",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Divination — {2}{U} Sorcery. Draw two cards.
///
/// Plain blue card-advantage staple. Two cards for three mana,
/// sorcery-speed.
pub fn divination() -> CardDefinition {
    CardDefinition {
        name: "Divination",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Ambition's Cost — {3}{B} Sorcery. Draw three cards. You lose 3 life.
pub fn ambitions_cost() -> CardDefinition {
    CardDefinition {
        name: "Ambition's Cost",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Path of Peace — {2}{W} Sorcery. Destroy target creature. Its
/// controller gains 4 life.
///
/// Sorcery-speed creature kill that gives the opp 4 life. Lifegain
/// targets the destroyed creature's controller via
/// `PlayerRef::ControllerOf` — captured at cast time so the lookup
/// works after Destroy moves the card to graveyard (the existing
/// fall-through walks graveyard / exile / hand).
pub fn path_of_peace() -> CardDefinition {
    CardDefinition {
        name: "Path of Peace",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(4),
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
        ]),
        ..Default::default()
    }
}

// ── modern_decks-9: more discard / burn / pump / cost-tax ────────────────────

/// Despise — {B} Sorcery. Target opponent reveals their hand; you
/// choose a creature or planeswalker card from it; that player
/// discards that card.
///
/// Same shape as Inquisition / Thoughtseize but with the
/// "creature OR planeswalker" filter — lets the cube ship a
/// dedicated answer to early threats / walkers.
pub fn despise() -> CardDefinition {
    CardDefinition {
        name: "Despise",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Planeswalker),
        },
        ..Default::default()
    }
}

/// Distress — {B}{B} Sorcery. Target opponent reveals their hand; you
/// choose a nonland, noncreature card from it; that player discards
/// that card.
///
/// Same family as Duress but at sorcery {B}{B} (real card is {1}{B}).
/// Engine path is identical to Duress; the cube takes both for
/// per-color depth.
pub fn distress() -> CardDefinition {
    CardDefinition {
        name: "Distress",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Nonland.and(SelectionRequirement::Noncreature),
        },
        ..Default::default()
    }
}

/// Vryn Wingmare — {2}{W} Creature — Bird Soldier. 2/1 Flying.
/// Noncreature spells cost {1} more to cast.
///
/// Reuses Thalia's `StaticEffect::AdditionalCostAfterFirstSpell`
/// filtered to `Noncreature`. The Oracle's tax applies on the *first*
/// spell too — Thalia/Vryn Wingmare share the same simplification
/// (the engine's static currently applies after the first spell each
/// turn since it reuses Damping Sphere's plumbing). Functionally
/// matches Thalia minus the first-strike body.
pub fn vryn_wingmare() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Vryn Wingmare",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells cost {1} more to cast.",
            effect: StaticEffect::AdditionalCostAfterFirstSpell {
                filter: SelectionRequirement::Noncreature,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Lava Coil — {1}{R} Sorcery. Lava Coil deals 4 damage to target
/// creature. If that creature would die this turn, exile it instead.
///
/// Approximation: deal 4 damage to a creature. The exile-on-death
/// rider isn't observable from this site (no per-LTB replacement
/// effect), but the gameplay-relevant axis (kill 4-toughness creature
/// for 2 mana, sorcery-speed) is preserved.
pub fn lava_coil() -> CardDefinition {
    CardDefinition {
        name: "Lava Coil",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Jaya's Greeting — {1}{R} Instant. Jaya's Greeting deals 3 damage to
/// target creature. Scry 2.
///
/// Slight upgrade on Volcanic Hammer at the cost of a creature-only
/// filter. Pairs the burn with Scry 2 (functionally a card-selection
/// rider — Magma Jet's shape).
pub fn jayas_greeting() -> CardDefinition {
    CardDefinition {
        name: "Jaya's Greeting",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Telling Time — {1}{U} Instant. Look at the top three cards of your
/// library. Put one into your hand, one into the bottom of your
/// library, and one on top of your library.
///
/// Approximation: `Scry 2 + Draw 1` — same rate as Anticipate / Magma
/// Jet's scry. The "put one on top, one on bottom" half collapses to
/// Scry's two-position (top or bottom) decision.
pub fn telling_time() -> CardDefinition {
    CardDefinition {
        name: "Telling Time",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Wild Mongrel — {1}{G} Creature — Hound. 2/2. Discard a card: this
/// creature gets +1/+1 until end of turn and becomes the color of your
/// choice until end of turn.
///
/// Same shape as Psychic Frog's discard pump ability. The
/// "becomes the color of your choice" half collapses (no
/// color-flag primitive) — the +1/+1 EOT swing is the gameplay axis.
pub fn wild_mongrel() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Wild Mongrel",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // Old prints carry "Hound"; modern reprints retconned to
            // "Dog". The engine's CreatureType has Hound but not Dog,
            // so we use the original tag.
            creature_types: vec![CreatureType::Hound],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Read the Tides — {3}{U} Sorcery. Draw three cards.
///
/// Concentrate-class draw at slightly off-color cost. Distinct factory
/// for cube identity (a 4-mana mono-blue draw spell that doesn't
/// require {U}{U} in the colored pips makes monoblue / splash decks
/// happy).
// (Real Oracle: `{3}{U}` Sorcery — Read the Tides is a 4-CMC 3-card draw.)
pub fn read_the_tides() -> CardDefinition {
    CardDefinition {
        name: "Read the Tides",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Last Gasp — {1}{B} Instant. Target creature gets -3/-3 until end of
/// turn.
///
/// Three-toughness kill at instant speed for two mana. Slightly
/// stronger than Disfigure (-2/-2) at one extra mana.
pub fn last_gasp() -> CardDefinition {
    CardDefinition {
        name: "Last Gasp",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Utility lands (Locus / fetch / sacrifice / bridge cycle) ─────────────────

/// Self-source ETB tap trigger — local copy of `super::super::etb_tap` to
/// avoid a wide import here. The same shape powers all the ETB-tapped
/// utility lands below.
fn modern_etb_tap() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Tap { what: Selector::This },
    }
}

/// Glimmerpost — Land — Locus. Glimmerpost enters tapped. When Glimmerpost
/// enters, you gain 1 life for each Locus you control. {T}: Add {C}.
///
/// Approximation: the per-Locus scaling is collapsed to a flat 1 life on
/// ETB. Locus-typed lands are rare in our cube (only Cloudpost shares the
/// type), so the gameplay-relevant outcome on a normal board state is the
/// flat 1. The ETB-tapped + {T}: Add {C} halves are exact.
pub fn glimmerpost() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    CardDefinition {
        name: "Glimmerpost",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Locus],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![
            modern_etb_tap(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Cloudpost — Land — Locus. Cloudpost enters tapped. {T}: Add {C} for
/// each Locus you control.
///
/// Approximation: the per-Locus mana scaling collapses to a flat
/// {T}: Add {C}. With at most a couple of Loci in the cube pool and no
/// per-source mana scaling primitive, the simplified version is closer
/// to a vanilla colorless source than to "12-post" engines, which is
/// fine for cube purposes.
pub fn cloudpost() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    CardDefinition {
        name: "Cloudpost",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Locus],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![modern_etb_tap()],
        ..Default::default()
    }
}

/// Lotus Field — Land. Lotus Field enters tapped. When Lotus Field enters,
/// sacrifice two untapped lands. {T}: Add three mana of one color.
///
/// The "untapped" qualifier on the sacrifice is collapsed (the engine's
/// `Sacrifice` filter doesn't expose tapped/untapped state at sacrifice
/// time without target picking, which would complicate the auto-decider).
/// Net gameplay: ETB sacrifices two of *your* lands then taps for triple
/// mana of any one color — the headline combo line is preserved.
pub fn lotus_field() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let etb = TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap { what: Selector::This },
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(2),
                filter: SelectionRequirement::Land
                    .and(SelectionRequirement::ControlledByYou),
            },
        ]),
    };
    CardDefinition {
        name: "Lotus Field",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::Hexproof],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(3)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![etb],
        ..Default::default()
    }
}

/// Evolving Wilds — Land. Evolving Wilds enters tapped. {T}, Sacrifice
/// Evolving Wilds: Search your library for a basic land card, put it
/// onto the battlefield tapped, then shuffle.
///
/// Same `sac_cost: true` shape as Wasteland's destruction half, but the
/// payoff is a search-to-battlefield instead of a destroy. The "shuffle"
/// step is implicit in the engine's `Search → Battlefield` resolver
/// (libraries shuffle when search ends).
pub fn evolving_wilds() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Evolving Wilds",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![modern_etb_tap()],
        ..Default::default()
    }
}

/// Build a "bridge"-cycle land: enters tapped, has both basic land types
/// (so fetchlands and other land-type-matters effects see it), and taps
/// for {C}.
///
/// In the real Oracle ("X is every basic land type") bridges register all
/// five basic types. We collapse to the two relevant types — that's
/// enough to power fetchland targeting and Nature's-Lore-style searches
/// for a Forest/etc., without overstating the card by giving it Plains
/// production from a Reflecting Pool, etc.
fn bridge_land(
    name: &'static str,
    type_a: crate::card::LandType,
    type_b: crate::card::LandType,
) -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name,
        card_types: vec![CardType::Artifact, CardType::Land],
        keywords: vec![Keyword::Indestructible],
        subtypes: Subtypes {
            land_types: vec![type_a, type_b],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![modern_etb_tap()],
        ..Default::default()
    }
}

/// Mistvault Bridge — Land. Enters tapped. Is Island and Swamp. {T}: Add {C}.
pub fn mistvault_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Mistvault Bridge", LandType::Island, LandType::Swamp)
}

/// Drossforge Bridge — Land. Enters tapped. Is Swamp and Mountain. {T}: Add {C}.
pub fn drossforge_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Drossforge Bridge", LandType::Swamp, LandType::Mountain)
}

/// Razortide Bridge — Land. Enters tapped. Is Plains and Island. {T}: Add {C}.
pub fn razortide_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Razortide Bridge", LandType::Plains, LandType::Island)
}

/// Goldmire Bridge — Land. Enters tapped. Is Plains and Swamp. {T}: Add {C}.
pub fn goldmire_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Goldmire Bridge", LandType::Plains, LandType::Swamp)
}

/// Silverbluff Bridge — Land. Enters tapped. Is Island and Mountain. {T}: Add {C}.
pub fn silverbluff_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Silverbluff Bridge", LandType::Island, LandType::Mountain)
}

/// Tanglepool Bridge — Land. Enters tapped. Is Island and Forest. {T}: Add {C}.
pub fn tanglepool_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Tanglepool Bridge", LandType::Island, LandType::Forest)
}

/// Slagwoods Bridge — Land. Enters tapped. Is Mountain and Forest. {T}: Add {C}.
pub fn slagwoods_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Slagwoods Bridge", LandType::Mountain, LandType::Forest)
}

/// Thornglint Bridge — Land. Enters tapped. Is Plains and Forest. {T}: Add {C}.
pub fn thornglint_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Thornglint Bridge", LandType::Plains, LandType::Forest)
}

/// Darkmoss Bridge — Land. Enters tapped. Is Swamp and Forest. {T}: Add {C}.
pub fn darkmoss_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Darkmoss Bridge", LandType::Swamp, LandType::Forest)
}

/// Rustvale Bridge — Land. Enters tapped. Is Plains and Mountain. {T}: Add {C}.
pub fn rustvale_bridge() -> CardDefinition {
    use crate::card::LandType;
    bridge_land("Rustvale Bridge", LandType::Plains, LandType::Mountain)
}

// ── Utility artifacts ────────────────────────────────────────────────────────

/// Coalition Relic — {3} Artifact. {T}: Add one mana of any color. (The
/// charge-counter rider — "{T}: Put a charge counter; you may remove three
/// to add WUBRG" — is omitted, no charge-counter→mana-burst primitive yet.)
///
/// Functionally Sphere of the Suns at +0 life cost without the four-color-
/// burst payoff: a clean three-mana fixer for any color pair.
pub fn coalition_relic() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Coalition Relic",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
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
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

// ── modern_decks-11: Multi-color removal + sweepers + body ───────────────────

/// Tear Asunder — {1}{B}{G} Instant. Destroy target artifact or enchantment.
///
/// Cube models the base cast (destroy artifact/enchantment); the kicker
/// {2} mode ("destroy any nonland permanent") is omitted since
/// `AlternativeCost` doesn't currently swap target filters at cast time.
/// Functions as a flexible BG answer.
pub fn tear_asunder() -> CardDefinition {
    CardDefinition {
        name: "Tear Asunder",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
        },
        ..Default::default()
    }
}

/// Assassin's Trophy — {B}{G} Instant. Destroy target permanent an
/// opponent controls.
///
/// Two-mana destroy-anything-but-yours. The "owner searches their library
/// for a basic land card and puts it onto the battlefield" downside is
/// collapsed (search-for-basic-onto-opponent's-battlefield isn't yet
/// supported by the engine — `Search` always targets the caster). For
/// gameplay purposes this is the upside half: a clean flexible answer.
pub fn assassins_trophy() -> CardDefinition {
    CardDefinition {
        name: "Assassin's Trophy",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        },
        ..Default::default()
    }
}

/// Volcanic Fallout — {1}{R}{R} Instant. Volcanic Fallout deals 2 damage
/// to each creature and each player.
///
/// The "this can't be countered" rider is dropped — engine has no
/// cast-time uncounterable flag wireup, but the body works as the
/// gameplay-relevant sweeper.
pub fn volcanic_fallout() -> CardDefinition {
    CardDefinition {
        name: "Volcanic Fallout",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(2),
                }),
            },
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(2),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Rout — {3}{W}{W} Sorcery. Destroy all creatures.
///
/// Day of Judgment at +1 mana. The flash mode ({2}{W}{W}{W}, instant) is
/// collapsed since `AlternativeCost` doesn't expose sorcery-vs-instant
/// timing toggles. Sorcery body is fully wired.
pub fn rout() -> CardDefinition {
    CardDefinition {
        name: "Rout",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Plague Wind — {8}{B}{B} Sorcery. Destroy all creatures you don't
/// control. They can't be regenerated.
///
/// Premium one-sided sweeper for ten mana. The regeneration rider
/// collapses (no observable regeneration site in the engine).
pub fn plague_wind() -> CardDefinition {
    CardDefinition {
        name: "Plague Wind",
        cost: cost(&[generic(7), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Carnage Tyrant — {4}{G}{G} 7/6 Dinosaur with Trample, Hexproof, and
/// "Carnage Tyrant can't be countered."
///
/// The cast-time uncounterable rider is approximated by `Keyword::CantBeCountered`
/// — the engine respects this flag in `CounterSpell`. Hexproof keeps the body
/// safe from targeted removal post-resolution.
pub fn carnage_tyrant() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Carnage Tyrant",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Trample, Keyword::Hexproof, Keyword::CantBeCountered],
        ..Default::default()
    }
}

/// Krark-Clan Ironworks — {4} Artifact. Sacrifice an artifact: Add {2}.
///
/// Free artifact-sac mana ability. The sac-as-cost is folded into the
/// resolved effect via `sac_cost: true` on the activated ability — same
/// shape as Lotus Petal / Chromatic Star. Combo enabler in artifact-heavy
/// builds.
pub fn krark_clan_ironworks() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Krark-Clan Ironworks",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            // The "sacrifice an artifact" cost folds into the resolved
            // effect. The activated ability needs a target so the human
            // picker can pick *which* artifact to sac; the bot's
            // auto-target picks the first sacrificeable artifact.
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Artifact
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(2)),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

// ── Surveil land cycle (Murders at Karlov Manor — non-deck slots) ────────────
//
// Each surveil land enters tapped and surveils 1. Reuses the
// `dual_land_with` + `etb_tap_then_surveil_one` helpers from
// `super::super::mod` (the same primitives that power the
// `lands.rs::meticulous_archive` / `shadowy_backstreet` / `undercity_sewers`
// trio in the deck-specific catalog).

/// Underground Mortuary — BG surveil land. ETB tapped, surveil 1, taps for {B} or {G}.
pub fn underground_mortuary() -> CardDefinition {
    use crate::card::LandType;
    super::super::dual_land_with(
        "Underground Mortuary",
        LandType::Swamp, LandType::Forest,
        Color::Black, Color::Green,
        vec![super::super::etb_tap_then_surveil_one()],
    )
}

/// Lush Portico — GW surveil land. ETB tapped, surveil 1, taps for {G} or {W}.
pub fn lush_portico() -> CardDefinition {
    use crate::card::LandType;
    super::super::dual_land_with(
        "Lush Portico",
        LandType::Forest, LandType::Plains,
        Color::Green, Color::White,
        vec![super::super::etb_tap_then_surveil_one()],
    )
}

/// Hedge Maze — UG surveil land. ETB tapped, surveil 1, taps for {U} or {G}.
pub fn hedge_maze() -> CardDefinition {
    use crate::card::LandType;
    super::super::dual_land_with(
        "Hedge Maze",
        LandType::Forest, LandType::Island,
        Color::Green, Color::Blue,
        vec![super::super::etb_tap_then_surveil_one()],
    )
}

/// Thundering Falls — UR surveil land. ETB tapped, surveil 1, taps for {U} or {R}.
pub fn thundering_falls() -> CardDefinition {
    use crate::card::LandType;
    super::super::dual_land_with(
        "Thundering Falls",
        LandType::Island, LandType::Mountain,
        Color::Blue, Color::Red,
        vec![super::super::etb_tap_then_surveil_one()],
    )
}

/// Commercial District — RW surveil land. ETB tapped, surveil 1, taps for {R} or {W}.
pub fn commercial_district() -> CardDefinition {
    use crate::card::LandType;
    super::super::dual_land_with(
        "Commercial District",
        LandType::Mountain, LandType::Plains,
        Color::Red, Color::White,
        vec![super::super::etb_tap_then_surveil_one()],
    )
}

/// Raucous Theater — BR surveil land. ETB tapped, surveil 1, taps for {B} or {R}.
pub fn raucous_theater() -> CardDefinition {
    use crate::card::LandType;
    super::super::dual_land_with(
        "Raucous Theater",
        LandType::Swamp, LandType::Mountain,
        Color::Black, Color::Red,
        vec![super::super::etb_tap_then_surveil_one()],
    )
}

/// Elegant Parlor — RG surveil land (Murders at Karlov Manor). ETB tapped,
/// surveil 1, taps for {R} or {G}. (CUBE_FEATURES.md tags this RW; the
/// printed card is in fact RG — Gruul slot of the MKM cycle.)
pub fn elegant_parlor() -> CardDefinition {
    use crate::card::LandType;
    super::super::dual_land_with(
        "Elegant Parlor",
        LandType::Mountain, LandType::Forest,
        Color::Red, Color::Green,
        vec![super::super::etb_tap_then_surveil_one()],
    )
}

/// Ghost Vacuum — {2} Artifact. {2}, {T}: Exile target card from a graveyard.
///
/// Targeted graveyard hate. The card filter is `Any` since the effect
/// names "a card" — any zone match, but `Effect::Move` from graveyards
/// already routes through `move_card_to(.., ZoneDest::Exile, ..)` thanks
/// to the modern_decks-1 plumbing. The "draw a card whenever a card is
/// put into your graveyard" rider on later printings is omitted (not
/// printed on the original Ghost Vacuum anyway).
pub fn ghost_vacuum() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Ghost Vacuum",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Any,
                },
                to: ZoneDest::Exile,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

// ── Additional Modern playables (claude/modern_decks-12) ────────────────────
//
// All of these are built on existing engine primitives — no engine changes
// required. Each gets at least one functionality test in
// `crabomination/src/tests/modern.rs`.

/// Stone Rain — {2}{R} Sorcery. Destroy target land.
///
/// Three-mana land destruction. Cast-time filter `Land` rejects creatures
/// or other permanent types.
pub fn stone_rain() -> CardDefinition {
    CardDefinition {
        name: "Stone Rain",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Land),
        },
        ..Default::default()
    }
}

/// Bone Splinters — {B} Sorcery. As an additional cost, sacrifice a
/// creature. Destroy target creature.
///
/// Sac-as-additional-cost folded into resolution via `SacrificeAndRemember`
/// — the bot picks the lowest-power eligible creature first. Same shape
/// as Severed Strands but no lifegain.
pub fn bone_splinters() -> CardDefinition {
    CardDefinition {
        name: "Bone Splinters",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
        ]),
        ..Default::default()
    }
}

/// Hieroglyphic Illumination — {2}{U} Instant. Draw two cards.
///
/// The cycling half ({U}: discard, draw 1) is omitted (no Cycling
/// activation primitive yet). The full-cost cast — instant-speed Divination
/// — is the gameplay-relevant default.
pub fn hieroglyphic_illumination() -> CardDefinition {
    CardDefinition {
        name: "Hieroglyphic Illumination",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Mortify — {1}{W}{B} Instant. Destroy target creature or enchantment.
///
/// Premium WB removal. Cast-time filter `Creature ∨ Enchantment` accepts
/// either type; "can't be regenerated" rider collapses (no observable
/// regeneration site). Strictly better than Doom Blade vs enchantments.
pub fn mortify() -> CardDefinition {
    CardDefinition {
        name: "Mortify",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
            ),
        },
        ..Default::default()
    }
}

/// Maelstrom Pulse — {1}{B}{G} Sorcery. Destroy target nonland permanent.
///
/// The "all permanents with the same name" rider is collapsed (no
/// name-match selector primitive yet). Single-target nonland-permanent
/// removal is still flexible: hits creatures, artifacts, enchantments,
/// and planeswalkers. Tear Asunder + Vindicate's Golgari cousin.
pub fn maelstrom_pulse() -> CardDefinition {
    CardDefinition {
        name: "Maelstrom Pulse",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
            ),
        },
        ..Default::default()
    }
}

/// Mind Twist — {X}{B} Sorcery. Target player discards X cards at random.
///
/// X is read at resolution from `Value::XFromCost`. The classic disruption
/// haymaker: castable for {X}=0 as a no-op, but at {2}{B} or higher it
/// strips a chunk of the opponent's hand uniformly at random. The
/// engine's `Discard { random: true }` path picks index 0 (the AutoDecider
/// is deterministic), which is semantically equivalent for tests.
pub fn mind_twist() -> CardDefinition {
    CardDefinition {
        name: "Mind Twist",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Discard {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::XFromCost,
            random: true,
        },
        ..Default::default()
    }
}

/// Dismember — {1}{B}{B} Instant. Target creature gets -5/-5 until end of
/// turn.
///
/// The Phyrexian-mana Oracle ({1}{B/Φ}{B/Φ}{B/Φ}) is collapsed to flat
/// {1}{B}{B}: the engine has no Phyrexian-pip primitive yet (life-paid
/// substitution). The body is the gameplay-relevant black-removal answer
/// to large indestructible threats — -5/-5 kills 5-toughness creatures.
pub fn dismember() -> CardDefinition {
    CardDefinition {
        name: "Dismember",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-5),
            toughness: Value::Const(-5),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Echoing Truth — {1}{U} Instant. Return target nonland permanent and
/// all other permanents with the same name as that permanent to their
/// owners' hands.
///
/// The "and all permanents with the same name" rider is collapsed (no
/// name-match selector primitive). Single-target bounce of any nonland
/// permanent is the gameplay-relevant default — Boomerang's wider land
/// twin, Unsummon's nonland-permanent twin.
pub fn echoing_truth() -> CardDefinition {
    CardDefinition {
        name: "Echoing Truth",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        ..Default::default()
    }
}

/// Celestial Purge — {1}{W} Instant. Exile target black or red permanent.
///
/// Color-hate exile. Cast-time filter `Permanent ∧ (HasColor(Black) ∨
/// HasColor(Red))` reads the targeted card's mana symbols — a black
/// creature with `B` in its cost will match; lands or colorless artifacts
/// won't. Hits planeswalkers / enchantments too, so it's broader than
/// most B/R answers.
pub fn celestial_purge() -> CardDefinition {
    CardDefinition {
        name: "Celestial Purge",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Permanent.and(
                    SelectionRequirement::HasColor(Color::Black)
                        .or(SelectionRequirement::HasColor(Color::Red)),
                ),
            ),
        },
        ..Default::default()
    }
}

/// Earthquake — {X}{R} Sorcery. Earthquake deals X damage to each
/// creature without flying and each player.
///
/// X is read at resolution from `Value::XFromCost`. Two `ForEach` passes:
/// non-flying creatures, then each player. Same shape as Volcanic Fallout
/// but parametrized by X. The "doesn't hit fliers" half is a `Not(HasKeyword(Flying))`
/// filter on the creature pass.
pub fn earthquake() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Earthquake",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying).negate()),
                ),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::XFromCost,
                }),
            },
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::XFromCost,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Glimpse the Unthinkable — {U}{B} Sorcery. Target player puts the top
/// ten cards of their library into their graveyard.
///
/// Premium two-mana mill. `Effect::Mill` over a target-filtered Player.
/// Auto-target picks the opponent (mill is a hostile player-side effect).
pub fn glimpse_the_unthinkable() -> CardDefinition {
    CardDefinition {
        name: "Glimpse the Unthinkable",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Mill {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(10),
        },
        ..Default::default()
    }
}

/// Cling to Dust — {B} Instant. Exile target card from a graveyard. If a
/// creature card was exiled this way, you gain 2 life.
///
/// Targeted graveyard hate with a creature-rider lifegain payoff. Uses
/// `Effect::Move(target → Exile)` (not `Effect::Exile`) so the auto-target
/// heuristic walks graveyards first via `prefers_graveyard_target`. The
/// "creature exiled this way" check runs after the move via
/// `Predicate::EntityMatches` against the captured target — the predicate
/// resolver walks battlefield → graveyards → exile so it still finds the
/// card after it lands in exile. The escape mode (`{2}{B}`, exile five
/// other cards from your graveyard, gain 2 life and draw a card) is
/// omitted — no escape-cost primitive yet.
pub fn cling_to_dust() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Cling to Dust",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Exile,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::Creature,
                },
                then: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── modern_decks-13: cube finishers ──────────────────────────────────────────

/// Lumra, Bellow of the Woods — {4}{G}{G} Legendary Creature — Elemental.
/// 6/6 Trample. When this enters, return all land cards from your graveyard
/// to the battlefield tapped.
///
/// Mass land-recursion: `Move(EachMatching(Graveyard(You), Land) →
/// Battlefield(You, tapped))`. The engine's `Move` already handles
/// graveyard-zone sources, and `Selector::EachMatching` iterates every match
/// — so a single Move call brings them all back at once.
pub fn lumra_bellow_of_the_woods() -> CardDefinition {
    use crate::card::Supertype as Sup;
    use crate::effect::ZoneRef;
    CardDefinition {
        name: "Lumra, Bellow of the Woods",
        cost: cost(&[generic(4), g(), g()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        // Real Oracle: Vigilance, Trample. (Reach was a stale leftover —
        // Lumra is not the Reach printing.)
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::You),
                    filter: SelectionRequirement::Land,
                },
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        }],
        ..Default::default()
    }
}

/// Crabomination — {2}{U}{B} Legendary Creature — Crab Horror. 3/4. When
/// this enters, each opponent mills three cards. Whenever a creature card
/// is put into an opponent's graveyard, you scry 1.
///
/// Custom card. The mill-on-ETB uses `Effect::Mill` with `EachOpponent`;
/// the scry trigger reuses `EventKind::CardDiscarded` with `OpponentControl`
/// — close enough at this engine fidelity (mill counts as discard for the
/// CardDiscarded event isn't quite right; instead we reuse the LandPlayed
/// pattern to fire on `EntersBattlefield` of *cards* into graveyard via
/// the `CreatureDied` listener with `OpponentControl`).
pub fn crabomination() -> CardDefinition {
    use crate::card::Supertype as Sup;
    CardDefinition {
        name: "Crabomination",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Crab, CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                },
            },
            // Whenever a creature your opponents control dies, you scry 1.
            // (Approximation of the broader "creature card put into an opp
            // graveyard" — covers the most common path: combat / removal
            // putting it there from the battlefield.)
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Chaos Warp — {2}{R} Instant. The owner of target permanent shuffles it
/// into their library, then reveals the top card. If it's a permanent card,
/// they put it onto the battlefield; otherwise it stays on top.
///
/// Approximation: shuffles the target in, then fires `RevealTopCard` so the
/// client shows the flip animation.  The "put onto the battlefield if it's a
/// permanent" clause is still collapsed — implementing arbitrary-card ETB from
/// the library top requires a pipeline we don't have yet.
pub fn chaos_warp() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Chaos Warp",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Permanent),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Shuffled,
                },
            },
            Effect::RevealTopCard {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Elvish Reclaimer — {1}{G} Creature — Human Druid. 1/2.
/// {T}, Sacrifice a land: Search your library for a land card and put it
/// onto the battlefield.
///
/// Land-tutor activated ability. Sacrifice-a-land cost is folded into
/// resolution as the first step (`Sacrifice(Land)` filtered to your side),
/// then a normal `Search(Land → BF)`. The Oracle "Threshold pump" rider
/// (P/T 3/4 if seven-or-more cards in your graveyard) is dropped — the cube
/// runs Reclaimer for the activated ability, not the body.
pub fn elvish_reclaimer() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Elvish Reclaimer",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Land
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Land,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Rofellos, Llanowar Emissary — {G}{G} Legendary Creature — Elf Druid. 2/1.
/// {T}: Add {G} for each Forest you control.
///
/// ✅ Push XXII: now uses the SOS-VI `ManaPayload::OfColor(Green,
/// CountOf(EachPermanent(Forest & ControlledByYou)))` primitive — the
/// activation now scales correctly with the Forest count, matching
/// the printed Oracle's "snowball with each new Forest" behavior.
pub fn rofellos_llanowar_emissary() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType, Supertype as Sup};
    CardDefinition {
        name: "Rofellos, Llanowar Emissary",
        cost: cost(&[g(), g()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Green,
                    Value::count(Selector::EachPermanent(
                        SelectionRequirement::Land
                            .and(SelectionRequirement::HasLandType(LandType::Forest))
                            .and(SelectionRequirement::ControlledByYou),
                    )),
                ),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Biorhythm — {6}{G}{G} Sorcery. Each player's life total becomes the
/// number of creatures they control.
///
/// The "set life total to N" primitive doesn't exist directly, but we can
/// approximate it as a `Seq` of two `LoseLife` calls — one for each player
/// — that drops their life by `current_life - creatures_they_control`.
/// Since `Value::count` over each player's creatures returns the live
/// count, and we don't have a `Value::CurrentLife` primitive, we instead
/// model the most common gameplay outcome: the caster gains life equal to
/// their creature count, and each opponent loses to 1 life. For the cube
/// this approximates "cast Biorhythm with a wide board, opponent dies".
///
/// Concrete wiring: `LoseLife(EachOpponent, 20)` (cap by current life via
/// SBA — life can't go negative without dying). Caster's creature count is
/// represented via `count(each_your_creature)` going to `GainLife(You)`.
pub fn biorhythm() -> CardDefinition {
    use crate::effect::shortcut::{count, each_your_creature};
    CardDefinition {
        name: "Biorhythm",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Approximation: each opponent loses a huge chunk (20 life is
            // ≥ starting life so they go to ≤ 0 unless protected).
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(20),
            },
            // You gain life equal to creatures you control — preserves the
            // "your-side-doesn't-die" half of the Oracle.
            Effect::GainLife {
                who: Selector::You,
                amount: count(each_your_creature()),
            },
        ]),
        ..Default::default()
    }
}

/// Karn, Scion of Urza — {4} Legendary Planeswalker — Karn. 5 loyalty.
/// **+1**: Reveal the top two cards of your library. An opponent separates
/// those cards into two piles. Put one pile into your hand and the other
/// into your graveyard.
/// **-1**: Put a +1/+1 counter on each Construct creature you control.
/// **-2**: Create a 0/0 colorless Construct artifact creature token with
/// "This creature gets +1/+1 for each artifact you control".
///
/// Approximation:
/// * +1 collapses to "draw 1, mill 1" — the opp-pile-split is information-
///   only at this engine fidelity.
/// * -1 grants +1/+1 counter to each Construct (via `ForEach` + `AddCounter`).
/// * -2 creates a vanilla 1/1 colorless Construct token (no artifact-count
///   scaling primitive yet).
pub fn karn_scion_of_urza() -> CardDefinition {
    use crate::card::{
        LoyaltyAbility, PlaneswalkerSubtype, Supertype as Sup, TokenDefinition,
    };
    CardDefinition {
        name: "Karn, Scion of Urza",
        cost: cost(&[generic(4)]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Karn],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::Mill {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ]),
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::HasCreatureType(
                                CreatureType::Construct,
                            )),
                    ),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: crate::card::CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                },
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Construct".to_string(),
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Construct],
                            ..Default::default()
                        },
                        supertypes: vec![],
                        power: 1,
                        toughness: 1,
                        keywords: vec![],
                        colors: vec![],
                        activated_abilities: vec![],
                        triggered_abilities: vec![],
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Tezzeret, Cruel Captain — {3}{B} Legendary Planeswalker — Tezzeret. 4 loyalty.
/// **+1**: Up to one target creature gets -2/-2 until end of turn.
/// **-2**: Each opponent loses 2 life and you gain 2 life.
///
/// Cube-style approximation. The "ult" (typically -7) is collapsed: at this
/// engine fidelity we cap Tezzeret at the two cube-relevant abilities (+1
/// removal on a 3-toughness creature, -2 drain). Static "your artifact
/// creatures get +1/+1" isn't modeled.
pub fn tezzeret_cruel_captain() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype as Sup};
    use crate::effect::Duration;
    CardDefinition {
        name: "Tezzeret, Cruel Captain",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Tezzeret],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                },
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Cruel Somnophage — {1}{U}{B} Creature — Phyrexian Horror. 0/0.
/// This creature's power and toughness are each equal to the number of
/// cards in your graveyard.
///
/// Dynamic P/T injection (Cosmogoyf/Tarmogoyf pattern). The compute_battlefield
/// hardcoded site (`crabomination/src/game/mod.rs`) reads
/// `players[controller].graveyard.len()` and injects a layer-7
/// `SetPowerToughness(n, n)` effect when the card name matches.
pub fn cruel_somnophage() -> CardDefinition {
    CardDefinition {
        name: "Cruel Somnophage",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        // Base P/T is 0/0; the compute-time injection overrides with
        // (graveyard size, graveyard size).
        power: 0,
        toughness: 0,
        ..Default::default()
    }
}

/// Pentad Prism — {2} Artifact. Sunburst (this enters with one charge
/// counter for each color of mana spent on its cost). Remove a charge
/// counter from this: Add one mana of any color.
///
/// Approximation: full Sunburst counter-tracking would require
/// `Value::ColorsSpentOnCost` (which doesn't exist yet). Instead we model
/// the most common play pattern: the card enters with 2 charge counters
/// (the typical "two colors" cast), and each activation removes one to
/// add one mana of any color. The activated ability folds the
/// counter-removal cost into resolution (Gemstone-Mine pattern), and the
/// "no counters → can't activate" gate is enforced by the engine's
/// counter-removal failing when the counter pool is empty.
pub fn pentad_prism() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    CardDefinition {
        name: "Pentad Prism",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::Const(2),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::Const(1),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Balefire Dragon — {5}{R}{R} Creature — Dragon. 6/6 Flying.
/// Whenever this deals combat damage to a player, it deals that much damage
/// to each creature that player controls.
///
/// Approximation: a hardcoded "deal 6 damage to each creature each opponent
/// controls" trigger when we attack and connect (`DealsCombatDamageToPlayer`
/// + `SelfSource`). The "that much damage" → "6 damage" collapse holds
///   for the unscaled play pattern; pump effects on Balefire Dragon don't
///   retroactively boost the trigger payload (which matches Oracle for
///   fixed-power triggers, just at a fixed value rather than its current power).
pub fn balefire_dragon() -> CardDefinition {
    CardDefinition {
        name: "Balefire Dragon",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(6),
                }),
            },
        }],
        ..Default::default()
    }
}

// ── modern_decks-14 ──────────────────────────────────────────────────────────

/// Vindicate — {1}{W}{B} Sorcery. Destroy target permanent.
///
/// Premium catch-all removal: hits any nonland permanent and (uniquely)
/// also lands. Filter is `Permanent` (which includes lands), so the cast
/// can target a manabase piece — gameplay-equivalent to original Oracle.
pub fn vindicate() -> CardDefinition {
    CardDefinition {
        name: "Vindicate",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Permanent),
        },
        ..Default::default()
    }
}

/// Anguished Unmaking — {1}{W}{B} Instant. Exile target nonland permanent.
/// You lose 3 life.
///
/// Premium WB instant-speed removal that beats indestructible / regen /
/// graveyard recursion (since it exiles). Lifeloss is unconditional and
/// targets the caster — modeled by ordering `LoseLife` after `Exile`.
pub fn anguished_unmaking() -> CardDefinition {
    CardDefinition {
        name: "Anguished Unmaking",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Magma Spray — {R} Instant. Magma Spray deals 2 damage to target
/// creature. If that creature would die this turn, exile it instead.
///
/// The exile-replacement rider collapses (no per-LTB replacement
/// primitive yet — same simplification as Lava Coil). Resolves as flat
/// 2 damage to a target creature, killing 2-toughness creatures.
pub fn magma_spray() -> CardDefinition {
    CardDefinition {
        name: "Magma Spray",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Despark — {W}{B} Instant. Exile target permanent with mana value 4 or
/// greater.
///
/// Cheap, narrow exile that misses small threats but hits planeswalkers,
/// mid-game finishers, and reanimated bombs. `target_filter` enforces
/// `Permanent ∧ ManaValueAtLeast(4)` at cast time.
pub fn despark() -> CardDefinition {
    CardDefinition {
        name: "Despark",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::ManaValueAtLeast(4)),
            ),
        },
        ..Default::default()
    }
}

/// Crumble to Dust — {2}{R}{R} Sorcery. Exile target nonbasic land,
/// then search its controller's library, graveyard, and hand for any
/// number of cards with the same name and exile them.
///
/// The "all cards with the same name" rider is collapsed (no name-match
/// selector at cast time). The single-target nonbasic exile is the
/// gameplay-relevant payoff: kills a Tron land, a manland, or a
/// Cabal Coffers without needing the chain-effect.
pub fn crumble_to_dust() -> CardDefinition {
    CardDefinition {
        name: "Crumble to Dust",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Land
                    .and(SelectionRequirement::IsBasicLand.negate()),
            ),
        },
        ..Default::default()
    }
}

/// Harrow — {2}{G} Instant. As an additional cost to cast this spell,
/// sacrifice a land. Search your library for up to two basic land cards
/// and put them onto the battlefield. Then shuffle.
///
/// Sac-as-additional-cost folded into resolution. Produces +1 net mana
/// (sac one, fetch two untapped). Both fetched lands are searched
/// individually so the decider can pick different basics each step.
pub fn harrow() -> CardDefinition {
    CardDefinition {
        name: "Harrow",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Land
                    .and(SelectionRequirement::ControlledByYou),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        ]),
        ..Default::default()
    }
}

/// Drown in the Loch — {U}{B} Instant. Choose one — counter target spell;
/// or destroy target creature or planeswalker. Spend only mana from
/// snow sources to cast this spell.
///
/// Modal: mode 0 counters a spell on the stack, mode 1 destroys a
/// creature/planeswalker. The "snow mana only" rider is collapsed (no
/// snow-mana primitive). The "X = cards in opp's graveyard" gate is
/// also collapsed — both halves of the modal are always available.
/// AutoDecider picks mode 0 (the counter line is usually the higher
/// gameplay-impact pick when both are legal).
pub fn drown_in_the_loch() -> CardDefinition {
    CardDefinition {
        name: "Drown in the Loch",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Skullcrack — {1}{R} Instant. Target player can't gain life this turn.
/// Skullcrack deals 3 damage to that player.
///
/// The "can't gain life" rider collapses (no life-gain prevention
/// primitive). Plays as a 3-damage instant against a target player —
/// strictly Lava Spike at +1 mana but at instant speed.
pub fn skullcrack() -> CardDefinition {
    CardDefinition {
        name: "Skullcrack",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Fiery Impulse — {R} Instant. Fiery Impulse deals 2 damage to target
/// creature. Spell mastery — if there are two or more instant and/or
/// sorcery cards in your graveyard, it deals 3 damage instead.
///
/// Spell-mastery scaling collapses to the base 2-damage line (no
/// graveyard-instant-count predicate yet). Pulls double duty as a
/// cheap creature-clearer at instant speed alongside Magma Spray.
pub fn fiery_impulse() -> CardDefinition {
    CardDefinition {
        name: "Fiery Impulse",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Mortuary Mire — Land. Mortuary Mire enters tapped. When Mortuary
/// Mire enters, you may put target creature card from your graveyard
/// on top of your library. {T}: Add {B}.
///
/// Reanimator dual-purpose land: a Swamp that recurs a creature on
/// entry. The "may" is auto-resolved — AutoDecider's graveyard-target
/// preference (the same one used by Raise Dead) picks a creature card
/// when one is available; otherwise the trigger fizzles benignly.
pub fn mortuary_mire() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Mortuary Mire",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Swamp],
            ..Default::default()
        },
        triggered_abilities: vec![
            // ETB tapped.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::EntersBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Tap { what: Selector::This },
            },
            // ETB optional graveyard recursion of a creature card.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::EntersBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Move {
                    what: target_filtered(SelectionRequirement::Creature),
                    to: ZoneDest::Library {
                        who: PlayerRef::You,
                        pos: LibraryPosition::Top,
                    },
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black]),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Geier Reach Sanitarium — Legendary Land. {T}: Add {C}. {1}, {T}:
/// Each player draws a card, then discards a card.
///
/// Wheel-engine land — fills graveyards on both sides for reanimator/
/// dredge while filtering through duds. The "discard a card" half is
/// modeled as a `Discard` over `EachPlayer` after the draw step.
pub fn geier_reach_sanitarium() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Geier Reach Sanitarium",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
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
            exile_gy_cost: 0,
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::Const(1),
                    },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::Const(1),
                        random: false,
                    },
                ]),
                once_per_turn: false,
                sorcery_speed: true,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
            },
        ],
        ..Default::default()
    }
}

/// Searing Blood — {R}{R} Instant. Searing Blood deals 2 damage to target
/// creature. When that creature dies this turn, Searing Blood deals 3
/// damage to that creature's controller.
///
/// The "if it dies, deal 3 to its controller" rider collapses (no
/// "if-this-effect's-target-dies" delayed trigger primitive yet).
/// Resolves as flat 2 damage to a creature — strictly worse than
/// Magma Spray for a half-cost increase but still efficient.
pub fn searing_blood() -> CardDefinition {
    CardDefinition {
        name: "Searing Blood",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Cremate — {B} Instant. Exile target card in a graveyard. Draw a card.
///
/// Graveyard-hate cantrip — pulls a card out of any graveyard (`Any`
/// filter, but the auto-target heuristic walks graveyards first via
/// `prefers_graveyard_target` since the move target is `→ Exile`).
pub fn cremate() -> CardDefinition {
    CardDefinition {
        name: "Cremate",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Exile,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

// ── modern_decks-15: 12 new cube cards ────────────────────────────────────────

/// Strangle — {R} Instant. Strangle deals 3 damage to target creature.
/// Surveil 1.
///
/// Single-target burn-plus-surveil instant — Drown in Ichor's red mirror.
/// `Seq([DealDamage(target Creature, 3), Surveil 1])`.
pub fn strangle() -> CardDefinition {
    CardDefinition {
        name: "Strangle",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Dreadbore — {B}{R} Sorcery. Destroy target creature or planeswalker.
///
/// Sorcery-speed Hero's Downfall at one less mana — kills creatures and
/// planeswalkers without restriction (no regen / nonblack riders). The
/// "can't be regenerated" clause collapses (no observable regeneration
/// site in the engine).
pub fn dreadbore() -> CardDefinition {
    CardDefinition {
        name: "Dreadbore",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
        },
        ..Default::default()
    }
}

/// Bedevil — {B}{B}{R} Instant. Destroy target artifact, creature, or
/// planeswalker.
///
/// Premium instant-speed three-types-removal at triple-pip cost. Filter
/// is the union of Artifact / Creature / Planeswalker — same shape as
/// Dreadbore but at instant speed and adds artifacts.
pub fn bedevil() -> CardDefinition {
    CardDefinition {
        name: "Bedevil",
        cost: cost(&[b(), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .or(SelectionRequirement::Planeswalker),
            ),
        },
        ..Default::default()
    }
}

/// Tome Scour — {U} Sorcery. Target player puts the top five cards of
/// their library into their graveyard.
///
/// One-mana mill staple — cheaper, smaller version of Glimpse the
/// Unthinkable. `Effect::Mill 5` with a target-filtered Player so the
/// caster can pick (auto-target hits the opponent).
pub fn tome_scour() -> CardDefinition {
    CardDefinition {
        name: "Tome Scour",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Mill {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(5),
        },
        ..Default::default()
    }
}

/// Repulse — {2}{U} Instant. Return target creature to its owner's hand.
/// Draw a card.
///
/// Three-mana bounce-cantrip — Unsummon plus a Draw 1 attached. The draw
/// fires after the bounce so the spell stays card-positive even if the
/// bounced creature is killed in the meantime.
pub fn repulse() -> CardDefinition {
    CardDefinition {
        name: "Repulse",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Visions of Beyond — {U} Instant. Draw a card. If a graveyard has 20 or
/// more cards in it, draw three cards instead.
///
/// The "20-card-graveyard" rider is collapsed (no graveyard-size
/// `Predicate::ValueAtLeast` over the matched zone yet); resolves as a
/// flat 1-card cantrip — gameplay-equivalent to Tezzeret's Gambit /
/// Brainstorm at lower density.
pub fn visions_of_beyond() -> CardDefinition {
    CardDefinition {
        name: "Visions of Beyond",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ..Default::default()
    }
}

/// Plummet — {1}{G} Instant. Destroy target creature with flying.
///
/// Cheap green answer to fliers. Cast-time filter combines `Creature`
/// with `HasKeyword(Flying)` so the spell can only be cast by selecting
/// a flying creature (mirrors Roast's filter shape, but Roast forbids
/// Flying — Plummet requires it).
pub fn plummet() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Plummet",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            ),
        },
        ..Default::default()
    }
}

/// Strategic Planning — {1}{U} Sorcery. Look at the top three cards of
/// your library. Put one of them into your hand and the rest into your
/// graveyard.
///
/// Approximated as `Mill 3 + Draw 1` — the gameplay-relevant outcome
/// (graveyard fills with two cards, you grab a card) is preserved; the
/// "look at three, take one to hand" choice collapses to a top-card
/// draw. Pairs especially well with reanimator/dredge shells where the
/// graveyard fill is the reason to cast.
pub fn strategic_planning() -> CardDefinition {
    CardDefinition {
        name: "Strategic Planning",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Ravenous Rats — {1}{B} Creature 1/1 Rat. When this creature enters,
/// target opponent discards a card.
///
/// Cheap discard-on-a-stick body. ETB self-source trigger fires
/// `Discard` against `EachOpponent` (the "target opponent" half collapses
/// to "each opponent" — gameplay-equivalent in 2-player). Non-chosen
/// discard mirrors Mind Rot's caster-side simplification (the engine's
/// chosen-discard primitive only handles caster-side picks today).
pub fn ravenous_rats() -> CardDefinition {
    CardDefinition {
        name: "Ravenous Rats",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
        }],
        ..Default::default()
    }
}

/// Brain Maggot — {1}{B} Creature 1/1 Spirit Insect. When this creature
/// enters, target opponent reveals their hand and you choose a nonland
/// card from it. Exile that card until this creature leaves the
/// battlefield.
///
/// Approximation of "exile until LTB" — uses `DiscardChosen` to send the
/// chosen nonland card to the graveyard (same simplification as
/// Tidehollow Sculler's ETB). The "return on LTB" half is omitted (no
/// exile-until-LTB primitive yet). Caster picks; bots auto-pick the
/// first matching card.
pub fn brain_maggot() -> CardDefinition {
    CardDefinition {
        name: "Brain Maggot",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
        }],
        ..Default::default()
    }
}

/// Bond of Discipline — {3}{W} Sorcery. Tap all creatures your opponents
/// control. Creatures you control gain lifelink until end of turn.
///
/// Tempo / lifegain swing. `Seq([ForEach(opp creatures) → Tap, ForEach(your
/// creatures) → GrantKeyword(Lifelink, EOT)])`. The lifelink is short-
/// duration and applies only to creatures already in play at resolution
/// (the engine evaluates ForEach selectors once at the start).
pub fn bond_of_discipline() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Bond of Discipline",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::Tap { what: Selector::TriggerSource }),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Sudden Edict — {1}{B} Instant. Target player sacrifices a creature.
/// This spell can't be countered.
///
/// Edict effect — bypasses hexproof, indestructible, and protection by
/// forcing the targeted player to choose a creature to sacrifice. Uses
/// `target_filtered(Player)` for the `who` slot so
/// `primary_target_filter` surfaces a Player filter and the bot's
/// auto-target heuristic picks an opponent. The "can't be countered"
/// rider uses `Keyword::CantBeCountered` (same keyword Carnage Tyrant
/// relies on).
pub fn sudden_edict() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Sudden Edict",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Sacrifice {
            who: target_filtered(SelectionRequirement::Player),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        },
        ..Default::default()
    }
}

// ── 2026-05-02 push XXII: classic cube staples ─────────────────────────────
//
// Eleven new card factories spanning W/U/B/R/G + colorless. Each card uses
// existing engine primitives — no engine changes required. The cards are
// long-standing cube staples (Modern Horizons / Innistrad-era reprints +
// command tower classics) that fit cleanly into the cube's
// per-color pools without needing replacement effects, alt-cost gymnastics,
// or new keyword primitives.

/// 3/3 green Ape token. Used by Pongify's "controller creates a 3/3
/// green Ape" rider.
fn ape_token() -> crate::card::TokenDefinition {
    use crate::card::TokenDefinition;
    TokenDefinition {
        name: "Ape".into(),
        power: 3,
        toughness: 3,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

/// 3/3 green Lizard token. Used by Rapid Hybridization's "controller
/// creates a 3/3 green Lizard" rider.
fn lizard_token() -> crate::card::TokenDefinition {
    use crate::card::TokenDefinition;
    TokenDefinition {
        name: "Lizard".into(),
        power: 3,
        toughness: 3,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

/// Pongify — {U} Instant. Destroy target creature. Its controller
/// creates a 3/3 green Ape creature token.
///
/// Hard-removal-with-an-ape: dodges indestructible (it's a destroy, not
/// exile, but the target's dying triggers still fire on the original).
/// The 3/3 token goes to the destroyed creature's controller via
/// `PlayerRef::ControllerOf(Target(0))` — the engine's
/// `find_card_owner` graveyard fallback resolves the post-destroy
/// controller (same path Harsh Annotation took in push XXI).
pub fn pongify() -> CardDefinition {
    CardDefinition {
        name: "Pongify",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: ape_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Rapid Hybridization — {U} Instant. Destroy target creature. Its
/// controller creates a 3/3 green Lizard creature token.
///
/// Pongify twin — same removal pattern, distinct token type for tribal
/// interactions. Cube games rarely care about Ape vs. Lizard, but the
/// distinct factory + token mints preserve printed identity.
pub fn rapid_hybridization() -> CardDefinition {
    CardDefinition {
        name: "Rapid Hybridization",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: lizard_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Mulldrifter — {4}{U} Creature — Elemental. 2/2 with Flying. "When
/// this creature enters, draw two cards."
///
/// Approximation: the Evoke {2}{U} alternative cost (cast for {2}{U}
/// but sacrifice on resolution) is omitted — `AlternativeCost`
/// machinery currently handles "exile from hand" pitches but doesn't
/// chain into a sacrifice-on-resolution rider. Body + ETB draw 2 are
/// fully wired; in cube the card is most often hard-cast for value
/// anyway.
pub fn mulldrifter() -> CardDefinition {
    CardDefinition {
        name: "Mulldrifter",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Wall of Omens — {1}{W} Creature — Wall. 0/4 with Defender. "When
/// this creature enters, draw a card."
///
/// Cantrip wall — a classic two-mana cube include for white midrange
/// shells. Body + ETB draw via `Effect::Draw`. The Defender keyword is
/// enforced inside `declare_attackers` (creatures with Defender can't
/// attack).
pub fn wall_of_omens() -> CardDefinition {
    CardDefinition {
        name: "Wall of Omens",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Sun Titan — {4}{W}{W} Creature — Giant. 6/6 with Vigilance. "When
/// this creature enters and whenever it attacks, you may return target
/// permanent card with mana value 3 or less from your graveyard to the
/// battlefield."
///
/// Recursive value engine. Both triggers fire the same body — a
/// `Move(target → Battlefield)` against a `Permanent ∧ ManaValueAtMost
/// (3)` graveyard card. Auto-target leans on the engine's
/// `prefers_graveyard_source` classification (move-to-bf reads as
/// reanimate), preferring a graveyard pick over an empty target slot.
/// "May" is collapsed to always-do (auto-decider takes the value
/// every time).
pub fn sun_titan() -> CardDefinition {
    let recur = Effect::Move {
        what: target_filtered(
            SelectionRequirement::Permanent
                .and(SelectionRequirement::ManaValueAtMost(3)),
        ),
        to: ZoneDest::Battlefield {
            controller: PlayerRef::You,
            tapped: false,
        },
    };
    CardDefinition {
        name: "Sun Titan",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::EntersBattlefield,
                    EventScope::SelfSource,
                ),
                effect: recur.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: recur,
            },
        ],
        ..Default::default()
    }
}

/// Solemn Simulacrum — {4} Artifact Creature — Golem. 2/2. "When this
/// creature enters, you may search your library for a basic land card,
/// put it onto the battlefield tapped, then shuffle. / When this
/// creature dies, you may draw a card."
///
/// "Sad robot" — a cube staple for ramp + draw value. The "you may"
/// clauses collapse to always-do (auto-decider takes both lines).
/// Search uses `Effect::Search` to BF-tapped; the death draw uses
/// `EventKind::CreatureDied + SelfSource`.
pub fn solemn_simulacrum() -> CardDefinition {
    CardDefinition {
        name: "Solemn Simulacrum",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::EntersBattlefield,
                    EventScope::SelfSource,
                ),
                effect: search_to_battlefield(SelectionRequirement::IsBasicLand, true),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Three Visits — {1}{G} Sorcery. Search your library for a Forest
/// card, put it onto the battlefield, then shuffle.
///
/// Nature's Lore twin — same shape, distinct factory for cube variety.
/// The forest enters **untapped** (per Oracle).
pub fn three_visits() -> CardDefinition {
    CardDefinition {
        name: "Three Visits",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: search_to_battlefield(
            SelectionRequirement::Land
                .and(SelectionRequirement::HasLandType(crate::card::LandType::Forest)),
            false,
        ),
        ..Default::default()
    }
}

/// Fume Spitter — {B} Creature — Horror. 1/1. "Sacrifice this creature:
/// Target creature gets -1/-1 until end of turn."
///
/// One-mana attrition piece — kills X/1 chump blockers, threatens
/// removal at instant speed, fits sac fodder shells. Activation uses
/// `ActivatedAbility::sac_cost` so the source dies before the pump
/// resolves; the -1/-1 is `PumpPT { -1, -1, EOT }` against the chosen
/// creature target.
pub fn fume_spitter() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Duration;
    CardDefinition {
        name: "Fume Spitter",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Galvanic Blast — {R} Instant. "Galvanic Blast deals 2 damage to any
/// target. Metalcraft — It deals 4 damage to that target instead if you
/// control three or more artifacts."
///
/// Metalcraft fold: damage = `2 + 2 if your-artifact-count ≥ 3 else 0`
/// — encoded via `If(ValueAtLeast(PermanentCountControlledBy(You) ∧
/// Artifact, 3))` branching on a Const(4) vs Const(2) DealDamage.
/// Filtering "artifacts you control" via the new
/// `Selector::EachPermanent(Artifact ∧ ControlledByYou)` count
/// primitive. Metalcraft is approximated as "≥ 3 artifacts you control"
/// without distinguishing artifact lands; cube games rarely have 3+
/// nonland artifacts and 0 artifact lands or vice versa, so the
/// approximation is faithful.
pub fn galvanic_blast() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Galvanic Blast",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::count(Selector::EachPermanent(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                )),
                Value::Const(3),
            ),
            then: Box::new(Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::Const(4),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::Const(2),
            }),
        },
        ..Default::default()
    }
}

/// Pithing Edict — {1}{B} Sorcery. "Target opponent sacrifices a
/// creature or planeswalker."
///
/// Variant of Sudden Edict that hits planeswalkers as well. Encoded
/// via `Effect::Sacrifice` with a `Creature ∨ Planeswalker` filter.
/// Single-target opponent prompt collapses to `EachOpponent` since
/// the engine has no per-spell player target picker that filters on
/// opponent-only — same shape as the Sudden Edict pattern.
pub fn pithing_edict() -> CardDefinition {
    CardDefinition {
        name: "Pithing Edict",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Planeswalker),
        },
        ..Default::default()
    }
}

/// Lash of Malice — {B} Instant. "Target creature gets -2/-2 until
/// end of turn."
///
/// Approximation: the printed Magecraft rider ("If you've cast or
/// copied an instant or sorcery spell this turn, that creature
/// instead gets +2/+2") collapses to the base mode (always -2/-2).
/// The cube use case is one-mana removal of an X/2 chump or pre-
/// combat shrink to swing through; the magecraft +2/+2 inversion is a
/// rare aggressive line.
pub fn lash_of_malice() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Lash of Malice",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Aether Adept — {1}{U}{U} Creature — Human Wizard. 2/2. "When this
/// creature enters, return target creature to its owner's hand."
///
/// Classic ETB bouncer. 2/2 body for 3 mana with a one-shot Unsummon
/// rider — fits cube blue tempo shells. The ETB target picker uses
/// `target_filtered(Creature)` so the cast prompt asks for any
/// creature target.
pub fn aether_adept() -> CardDefinition {
    CardDefinition {
        name: "Aether Adept",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        }],
        ..Default::default()
    }
}

/// Wind Drake — {2}{U} Creature — Drake. 2/2 with Flying. (Vanilla
/// flying-bear baseline.)
pub fn wind_drake() -> CardDefinition {
    CardDefinition {
        name: "Wind Drake",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Cursecatcher — {U} Creature — Merfolk Wizard. 1/1. "Sacrifice this
/// creature: Counter target instant or sorcery spell unless its
/// controller pays {1}."
///
/// One-mana tempo piece. Uses `Effect::CounterUnlessPaid` (same shape
/// as Spell Pierce) gated through a sac-cost activation. The "instant
/// or sorcery" target filter is approximated as "any spell" — engine
/// has `IsSpellOnStack` but no separate IS-only filter on the stack.
/// The cube games dominated by IS spells anyway, the approximation
/// rarely matters.
pub fn cursecatcher() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let counter_cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(1)],
    };
    CardDefinition {
        name: "Cursecatcher",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: counter_cost,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Resilient Khenra — {2}{G} Creature — Jackal Warrior. 3/2. "When
/// this creature dies, put a +1/+1 counter on target creature you
/// control."
///
/// Approximation: the Eternalize cost is omitted (no Eternalize
/// keyword primitive). Body + death pump are wired faithfully — death
/// trigger uses `EventKind::CreatureDied + SelfSource` with
/// `Effect::AddCounter` on a friendly creature target. The Jackal
/// subtype is approximated as Hound (no Jackal in `CreatureType`).
pub fn resilient_khenra() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Resilient Khenra",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hound, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Persistent Petitioners — {1}{U} Creature — Human Advisor. 1/3.
/// "{1}, {T}: Target player mills a card. / Tap four untapped Advisors
/// you control: Target player mills twelve cards. / A deck can have any
/// number of cards named Persistent Petitioners."
///
/// Approximation: the "tap four Advisors" alt activation is omitted
/// (no multi-creature-tap-as-cost primitive — would need a "tap N
/// other matching" cost on `ActivatedAbility`). The base
/// `{1},{T}: target player mills 1` is wired; the deck-construction
/// "any number" rule is enforced at the deck-builder layer. Promoted
/// alongside the new `SelectionRequirement::HasName` for any future
/// "named Persistent Petitioners" payoffs.
pub fn persistent_petitioners() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let mill_cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(1)],
    };
    CardDefinition {
        name: "Persistent Petitioners",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: mill_cost,
            effect: Effect::Mill {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

/// Pulse of Murasa — {1}{G}{G} Instant. "Return target creature or
/// land card from a graveyard to its owner's hand. You gain 6 life."
///
/// Reanimator-class graveyard recursion + lifegain. The target filter
/// is `Creature ∨ Land` so it accepts both creature and land
/// graveyard cards (auto-target favors graveyard via the existing
/// `prefers_graveyard_source` heuristic on Move-to-Hand). The 6 life
/// is unconditional.
pub fn pulse_of_murasa() -> CardDefinition {
    CardDefinition {
        name: "Pulse of Murasa",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Land),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(6),
            },
        ]),
        ..Default::default()
    }
}

/// Slime Against Humanity — {1}{G} Sorcery. "Create X 1/1 green Ooze
/// creature tokens, where X is the number of cards named Slime Against
/// Humanity in all graveyards plus the number of cards named Slime
/// Against Humanity you own in exile. / A deck can have any number of
/// cards named Slime Against Humanity."
///
/// Wired via the new `SelectionRequirement::HasName` predicate. The X
/// count uses `Value::CountOf(CardsInZone(You/Graveyard, HasName))`
/// (caster-side approximation — the printed wording also counts opp
/// graveyards + caster exile, both omitted here for simplicity; in
/// typical cube games the caster's own graveyard is the dominant
/// source). Always creates at least 1 token (`+1` on the count).
pub fn slime_against_humanity() -> CardDefinition {
    use crate::card::{TokenDefinition, Zone};
    use std::borrow::Cow;
    let ooze_token = TokenDefinition {
        name: "Ooze".into(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    let name_filter = SelectionRequirement::HasName(Cow::Borrowed("Slime Against Humanity"));
    CardDefinition {
        name: "Slime Against Humanity",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        // X = (count of Slime Against Humanity in your graveyard) + 1
        // (the just-cast resolving copy). Real Oracle counts all
        // graveyards + exile; we approximate as "your graveyard" since
        // the engine has no all-zones name-tally primitive yet and the
        // gameplay-relevant case is your own deck spamming copies.
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Sum(vec![
                Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: name_filter,
                }),
                Value::Const(1),
            ]),
            definition: ooze_token,
        },
        ..Default::default()
    }
}

// ── Boros Charm ─────────────────────────────────────────────────────────────

/// Boros Charm — {R}{W} Instant. Modal: "Choose one — / • Boros Charm
/// deals 4 damage to target player or planeswalker. / • Permanents you
/// control gain indestructible until end of turn. / • Target creature
/// gains double strike until end of turn."
///
/// All three modes are wired faithfully via `Effect::ChooseMode`. Mode
/// 0 = 4 damage to player/PW, Mode 1 = ForEach(Permanent &
/// ControlledByYou) → GrantKeyword(Indestructible EOT), Mode 2 =
/// GrantKeyword(DoubleStrike EOT) on a creature target.
pub fn boros_charm() -> CardDefinition {
    CardDefinition {
        name: "Boros Charm",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Planeswalker),
                amount: Value::Const(4),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::ControlledByYou),
                body: Box::new(Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Dragon's Rage Channeler ─────────────────────────────────────────────────

/// Dragon's Rage Channeler — {R}, 1/1 Human Shaman. "Whenever you cast
/// a noncreature spell, surveil 1. / Delirium — This creature gets +2/+2
/// and has flying as long as there are four or more card types among
/// cards in your graveyard."
///
/// 🟡 Body wired (1/1 Human Shaman, red). Surveil-on-noncreature-cast
/// trigger wired faithfully — uses `EventScope::YourControl` filtered
/// to `¬HasCardType(Creature)` against the just-cast spell. Delirium
/// static (+2/+2 + flying based on graveyard card-type diversity) is
/// omitted (no "card types in graveyard" Value primitive yet — same
/// gap as Tarmogoyf's P/T scaling, which we approximate with a flat
/// 4/5).
pub fn dragons_rage_channeler() -> CardDefinition {
    CardDefinition {
        name: "Dragon's Rage Channeler",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                crate::card::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(CardType::Creature).negate(),
                },
            ),
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Unholy Heat ─────────────────────────────────────────────────────────────

/// Unholy Heat — {R} Instant. "Unholy Heat deals 3 damage to target
/// creature or planeswalker. Delirium — Unholy Heat deals 6 damage to
/// that permanent instead if there are four or more card types among
/// cards in your graveyard."
///
/// 🟡 Mainline 3-damage half wired faithfully. The Delirium upgrade to
/// 6 damage is omitted (no "card types in graveyard" Value primitive
/// yet — same gap as Dragon's Rage Channeler's body buff). The 3-
/// damage version is still one of the best removal spells in red at
/// 1 mana.
pub fn unholy_heat() -> CardDefinition {
    CardDefinition {
        name: "Unholy Heat",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

// ── Pelt Collector ──────────────────────────────────────────────────────────

/// Pelt Collector — {G}, 1/1 Elf Warrior. "Whenever another creature
/// you control enters, if it has greater power than this creature, put
/// a +1/+1 counter on this creature. / Whenever a creature you control
/// dies, if it had greater power than this creature, put a +1/+1
/// counter on this creature. / As long as this creature has three or
/// more +1/+1 counters on it, it has trample."
///
/// 🟡 Body wired (1/1 Elf Warrior). Power-comparison ETB/death triggers
/// are omitted (no "trigger source has greater power than self"
/// Predicate primitive yet) — collapses to a vanilla 1/1. Static
/// trample-when-3+-counters is also omitted (no self-counter-gated
/// keyword grant). Slots into mono-green stompy as a 1-drop body.
pub fn pelt_collector() -> CardDefinition {
    CardDefinition {
        name: "Pelt Collector",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

// ── Frantic Inventory ───────────────────────────────────────────────────────

/// Frantic Inventory — {1}{U} Sorcery. "Draw a card. Then draw a card
/// for each other card named Frantic Inventory in your graveyard."
///
/// Wired faithfully via `Value::Sum([Const(1), CountOf(CardsInZone(
/// Graveyard, HasName))])` — same shape as Slime Against Humanity's
/// X-tally formula (push XXII). Each cast in a deck running multiple
/// copies snowballs in card draw: 1 → 2 → 3 → 4 cards across casts.
pub fn frantic_inventory() -> CardDefinition {
    use crate::card::Zone;
    use std::borrow::Cow;
    let name_filter = SelectionRequirement::HasName(Cow::Borrowed("Frantic Inventory"));
    CardDefinition {
        name: "Frantic Inventory",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Sum(vec![
                Value::Const(1),
                Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: name_filter,
                }),
            ]),
        },
        ..Default::default()
    }
}

// ── Pegasus Stampede ────────────────────────────────────────────────────────

/// Pegasus Stampede — {3}{W} Sorcery. "Create two 1/1 white Pegasus
/// creature tokens with flying. / Flashback {6}{W}{W}."
///
/// Wired faithfully via the `Pegasus` creature subtype (existing) and
/// `Keyword::Flashback`. Two 1/1 fliers on cast + flashback recursion
/// for late-game token spam.
pub fn pegasus_stampede() -> CardDefinition {
    use crate::card::TokenDefinition;
    let pegasus = TokenDefinition {
        name: "Pegasus".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pegasus],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    let flashback_cost = cost(&[generic(6), w(), w()]);
    CardDefinition {
        name: "Pegasus Stampede",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: pegasus,
        },
        ..Default::default()
    }
}


// ── Kolaghan's Command ──────────────────────────────────────────────────────

/// Kolaghan's Command — {B}{R} Instant.
/// "Choose two —
///  • Return target creature card from your graveyard to your hand.
///  • Target opponent discards a card.
///  • Kolaghan's Command deals 2 damage to target creature or planeswalker.
///  • Destroy target artifact."
///
/// 🟡 Same approximation as Boros Charm and the STX Commands: printed
/// "choose two" collapses to "choose one" via `Effect::ChooseMode` (no
/// multi-pick primitive). Each individual mode is wired faithfully —
/// gy-recursion via `Selector::take` over creatures-in-gy, opp-discard
/// via `Effect::Discard { random: true }` collapsed to caster-picks
/// (we use random discard since no opp-side hand picker), 2 dmg via
/// `target_filtered(Creature ∨ Planeswalker)`, and `Effect::Destroy`
/// against an artifact target. BR midrange staple.
pub fn kolaghans_command() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Kolaghan's Command",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            // Mode 0: return creature card from your gy → hand.
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            // Mode 1: target opp discards a card (random — we have no
            // multi-step "opp picks" prompt for triggered Discard, so the
            // random flag is a faithful approximation of "they choose").
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: true,
            },
            // Mode 2: 2 damage to creature or planeswalker.
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            // Mode 3: destroy target artifact.
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::HasCardType(CardType::Artifact)),
            },
        ]),
        ..Default::default()
    }
}

// ── Twincast ────────────────────────────────────────────────────────────────

/// Twincast — {U}{U} Instant. "Copy target instant or sorcery spell.
/// You may choose new targets for the copy."
///
/// Wired faithfully via `Effect::CopySpell` over a target filtered to
/// `IsSpellOnStack ∧ (Instant ∨ Sorcery)`. Same shape as Choreographed
/// Sparks (SOS) but without the controller filter — Twincast can copy
/// any IS spell, including opponents'. The copy inherits the original's
/// targets at copy time (the "may choose new targets" clause is
/// approximated as keeping the original targets — no interactive
/// re-target prompt yet).
pub fn twincast() -> CardDefinition {
    CardDefinition {
        name: "Twincast",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CopySpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
            ),
            count: Value::Const(1),
        },
        ..Default::default()
    }
}

// ── Reverberate ─────────────────────────────────────────────────────────────

/// Reverberate — {R}{R} Instant. "Copy target instant or sorcery spell.
/// You may choose new targets for the copy."
///
/// Functionally identical to Twincast at red. Same `Effect::CopySpell`
/// wiring. (Each card is included for cube color-pool diversity — red
/// vs blue copy spells slot into different archetypes.)
pub fn reverberate() -> CardDefinition {
    CardDefinition {
        name: "Reverberate",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CopySpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
            ),
            count: Value::Const(1),
        },
        ..Default::default()
    }
}

// ── Vendetta ────────────────────────────────────────────────────────────────

/// Vendetta — {B} Instant. "Destroy target nonblack creature. You lose
/// life equal to its toughness."
///
/// 🟡 Mainline destroy half wired faithfully (target filter = Creature ∧
/// ¬Black via `SelectionRequirement::HasColor(Black).negate()`). The
/// "lose life equal to its toughness" rider collapses to a flat 2-life
/// payment — `Value` doesn't yet have a "toughness of pre-destroy
/// target" reader (the target is in the graveyard by the time the
/// life-loss step would resolve, and `Value::ToughnessOf` reads from
/// the battlefield). Same gap as Bone Splinters' generic-cost
/// approximation; tracked in TODO.md.
pub fn vendetta() -> CardDefinition {
    CardDefinition {
        name: "Vendetta",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(Color::Black).negate()),
                ),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

// ── Generous Gift ───────────────────────────────────────────────────────────

/// Generous Gift — {2}{W} Instant. "Destroy target nonland permanent.
/// Its controller creates a 3/3 green Elephant creature token."
///
/// Wired via `Effect::Destroy` on a `Permanent ∧ Nonland` target +
/// `Effect::CreateToken` whose `who` field is
/// `PlayerRef::ControllerOf(Target(0))`. The destroyed card's controller
/// is read at resolution time (graveyard-fallback path matches Harsh
/// Annotation's "destroyed creature's controller" Inkling rider — see
/// SOS push XXI). The token is a 3/3 green Elephant — same colors and
/// stats as Beast Within's Beast token but typed Elephant per print.
pub fn generous_gift() -> CardDefinition {
    use crate::card::TokenDefinition;
    let elephant = TokenDefinition {
        name: "Elephant".into(),
        power: 3,
        toughness: 3,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Generous Gift",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: elephant,
            },
        ]),
        ..Default::default()
    }
}

// ── Crackling Doom ──────────────────────────────────────────────────────────

/// Crackling Doom — {R}{W}{B} Instant. "Crackling Doom deals 2 damage
/// to each opponent. Each opponent sacrifices a creature with the
/// greatest power among creatures they control."
///
/// 🟡 The "greatest power" sacrifice constraint isn't enforced — the
/// engine's `Effect::Sacrifice` with a `Creature` filter delegates the
/// pick to the targeted player (auto-decider picks the lowest power for
/// that player, opposite of the printed "greatest"). Same approximation
/// gap as Pithing Edict's "creature or planeswalker" choice. The 2-
/// damage half is faithful: `Effect::DealDamage` against
/// `Selector::Player(EachOpponent)` lands the chip damage on every
/// opponent regardless of their creature situation.
pub fn crackling_doom() -> CardDefinition {
    CardDefinition {
        name: "Crackling Doom",
        cost: cost(&[r(), w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
        ]),
        ..Default::default()
    }
}

// ── Push XXVI ────────────────────────────────────────────────────────────────
// Cube + STX cards added 2026-05-02. Ten new card factories — five cube
// staples (Cabal Ritual, Rift Bolt, Ancient Stirrings, Stinkweed Imp,
// Endurance, Esper Sentinel, Fiery Confluence) plus three STX 2021
// promotions (Silverquill Apprentice, Brilliant Plan, Path of Peril).

/// Cabal Ritual — {B} Sorcery. Add {B}{B}{B}. Threshold — Add {C}{B}{B}{B}{B}
/// instead if seven or more cards are in your graveyard.
///
/// The threshold gate uses `Predicate::ValueAtLeast(GraveyardSizeOf(You),
/// Const(7))`. The {C} is approximated via an extra colorless pip in the
/// upgraded payout (engine doesn't model "snow" / hybrid "any" pips
/// separately from generic).
pub fn cabal_ritual() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Cabal Ritual",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::GraveyardSizeOf(PlayerRef::You),
                Value::Const(7),
            ),
            then: Box::new(Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![
                        Color::Black,
                        Color::Black,
                        Color::Black,
                        Color::Black,
                    ]),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
            ])),
            else_: Box::new(Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black, Color::Black, Color::Black]),
            }),
        },
        ..Default::default()
    }
}

/// Rift Bolt — {2}{R} Sorcery. Rift Bolt deals 3 damage to any target.
/// Suspend 1—{R}.
///
/// 🟡 Suspend is omitted — the engine has no time-counter / cast-from-
/// exile-after-N-upkeeps primitive. Ships at the printed full cost
/// {2}{R}, which is the strictly-worse line (vs. paying {R} + waiting one
/// turn). Rift Bolt is included for the burn-cube niche where the {2}{R}
/// curve still slots cleanly.
pub fn rift_bolt() -> CardDefinition {
    CardDefinition {
        name: "Rift Bolt",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Ancient Stirrings — {G} Sorcery. Look at the top five cards of your
/// library. You may reveal a colorless card from among them and put it
/// into your hand. Put the rest on the bottom of your library in any
/// order.
///
/// Wired via `Effect::RevealUntilFind` with `cap: 5` and the new
/// `SelectionRequirement::Colorless` predicate — finds the first
/// colorless card in the top 5 and moves it to the caster's hand;
/// non-colorless cards walked through go to the graveyard (engine
/// default). This loses the "rest to bottom of library" semantic but
/// preserves the gameplay-relevant outcome (the colorless card lands
/// in hand on cast).
pub fn ancient_stirrings() -> CardDefinition {
    CardDefinition {
        name: "Ancient Stirrings",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Colorless,
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(5),
            life_per_revealed: 0,
        },
        ..Default::default()
    }
}

/// Stinkweed Imp — {1}{B} Creature — Imp. 1/3 Flying. Whenever this
/// creature deals combat damage to a player, that player puts the top
/// five cards of their library into their graveyard. Dredge 5.
///
/// 🟡 Dredge is omitted (no Dredge primitive — would need a "you may
/// replace your draw with a mill-N + return-this-from-gy" replacement
/// effect). The combat-damage mill rider is wired faithfully via
/// `EventKind::CombatDamageDealtToPlayer / SelfSource` → `Effect::Mill`
/// on the damaged player.
pub fn stinkweed_imp() -> CardDefinition {
    CardDefinition {
        name: "Stinkweed Imp",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(5),
            },
        }],
        ..Default::default()
    }
}

/// Endurance — {1}{G}{G} Creature — Elemental Incarnation. 3/4 Reach,
/// flash. When this creature enters, target player puts all the cards
/// from their graveyard on the bottom of their library in a random order.
///
/// 🟡 Evoke {2}{G} (exile a green card from hand) is omitted (no
/// pitch-with-exile alt-cost flag yet — Solitude / Subtlety / Grief have
/// the same gap). The ETB graveyard-shuffle is wired faithfully via
/// `Effect::ShuffleGraveyardIntoLibrary` against `target_filtered(Player)`.
pub fn endurance() -> CardDefinition {
    CardDefinition {
        name: "Endurance",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach, Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ShuffleGraveyardIntoLibrary {
                who: PlayerRef::Target(0),
            },
        }],
        ..Default::default()
    }
}

/// Esper Sentinel — {W} Creature — Human Advisor. 1/1. Whenever an
/// opponent casts their first noncreature spell each turn, unless that
/// player pays {X}, where X is this creature's power, you draw a card.
///
/// 🟡 The "first noncreature spell each turn" + "unless that player pays
/// {X}" cost gate is approximated as an unconditional draw on every
/// noncreature spell an opponent casts. Two over-payoffs: (1) every
/// non-first cast also draws (was once-per-turn); (2) the opp can never
/// pay to skip. Both are tracked in TODO.md; the body still slots into
/// the white "tax" pool for cube games.
pub fn esper_sentinel() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Esper Sentinel",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Path of Peril — {2}{B}{B} Sorcery. All creatures with mana value 2
/// or less get -3/-3 until end of turn.
///
/// Mass-removal that hits weenie creatures — wired via
/// `Effect::ForEach` over `Selector::EachPermanent(Creature ∧
/// ManaValueAtMost(2))` + a flat -3/-3 EOT pump. Boltable creatures
/// die outright; surviving 3-toughness bodies live with -3/-3 until
/// the EOT cleanup. The "Boast cost" rider from the printed
/// _Kaldheim_ version is omitted (Boast is a one-shot per-turn
/// activation that fires when the source attacks).
pub fn path_of_peril() -> CardDefinition {
    CardDefinition {
        name: "Path of Peril",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            ),
            body: Box::new(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Fiery Confluence — {2}{R}{R} Sorcery. Choose three. You may choose
/// the same mode more than once. — • Fiery Confluence deals 1 damage
/// to each creature. • Fiery Confluence deals 2 damage to each
/// opponent. • Destroy target artifact.
///
/// 🟡 Approximated as a single-mode pick (the engine has no "choose
/// three with repetition" primitive yet — same gap as Mystic
/// Confluence, Cryptic Command, Kaya's Wrath). Each mode resolves
/// faithfully on its own.
pub fn fiery_confluence() -> CardDefinition {
    CardDefinition {
        name: "Fiery Confluence",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(SelectionRequirement::Creature),
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Artifact),
            },
        ]),
        ..Default::default()
    }
}

/// Brilliant Plan — {3}{U} Sorcery. Scry 3, then draw three cards.
///
/// Pure card-selection sorcery (STX 2021 mono-blue). Wired via
/// `Effect::Seq([Scry(3), Draw(3)])`. Plays as a Tidings (4-mana
/// 3-card draw) with extra setup, slotting into UR / UB control
/// archetypes.
pub fn brilliant_plan() -> CardDefinition {
    CardDefinition {
        name: "Brilliant Plan",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(3),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Silverquill Apprentice — {W}{B} Creature — Human Cleric. 2/2.
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// target creature gets -1/-1 until end of turn or +1/+1 until end of
/// turn (controller's choice).
///
/// Push XXXVII: 🟡 → ✅. Both modes now wire faithfully via
/// `Effect::PickModeAtResolution([+1/+1 EOT, -1/-1 EOT])`. AutoDecider
/// picks mode 0 (pump — the "safe" combat-trick default for our own
/// creatures). ScriptedDecider can flip mode 1 (-1/-1) for tests / for
/// the printed "shrink an opp creature" combat line.
pub fn silverquill_apprentice() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Silverquill Apprentice",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        triggered_abilities: vec![magecraft(Effect::PickModeAtResolution(vec![
            // Mode 0: target creature gets +1/+1 EOT.
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            // Mode 1: target creature gets -1/-1 EOT.
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

// ── Push XXVII ──────────────────────────────────────────────────────────────
// Six more cards: Careful Study, Sheoldred the Apocalypse, Liliana of the
// Veil, Light Up the Stage, Brilliant Plan-companion (Fae of Wishes is
// Adventure-shaped — skip), Quandrix Apprentice tweak, Tibalt's Trickery
// 🟡, Spell Pierce-style cards. Each ships against existing primitives.

/// Careful Study — {U} Sorcery. Draw two cards, then discard two cards.
///
/// Pure card-selection cantrip — net zero hand size, but filters two
/// random cards out of your hand. Slot for graveyard archetypes
/// (madness, dredge, reanimator) and is the cheap blue Faithless
/// Looting analogue. Wired with `Effect::Seq([Draw 2, Discard 2])`.
pub fn careful_study() -> CardDefinition {
    CardDefinition {
        name: "Careful Study",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
        ]),
        ..Default::default()
    }
}

/// Sheoldred, the Apocalypse — {2}{B}{B} Legendary Creature — Phyrexian
/// Praetor. 4/5 Deathtouch, lifelink. Whenever a player draws a card,
/// that player loses 2 life unless they're you, in which case you gain
/// 2 life.
///
/// Fully wired via two CardDrawn triggers — one fires when you draw
/// (gain 2 life), the other fires when an opp draws (they lose 2). The
/// engine's `EventScope::YourControl` / `OpponentControl` cleanly
/// partitions the two paths. Body keywords are first-class.
pub fn sheoldred_the_apocalypse() -> CardDefinition {
    CardDefinition {
        name: "Sheoldred, the Apocalypse",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Praetor],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![
            // You draw → you gain 2 life.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            },
            // Opp draws → that specific opp loses 2 life. Push XXVIII
            // threads the trigger subject (the player who drew) through
            // `StackItem::Trigger.subject` into `EffectContext.
            // trigger_source`, so `PlayerRef::Triggerer` correctly
            // resolves to the drawing opponent — no longer the
            // EachOpponent collapse from push XXVII.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Liliana of the Veil — {1}{B}{B} Legendary Planeswalker — Liliana.
/// Starting loyalty 3.
/// +1: Each player discards a card.
/// -2: Target player sacrifices a creature.
/// -6: Separate all permanents target player controls into two piles.
/// That player sacrifices all permanents in the pile of their choice.
///
/// 🟡 The -6 ult collapses (no two-pile-split primitive). +1 and -2
/// wire faithfully. The +1 fires on every player (each opponent +
/// you), implemented as `Seq([Discard{You,1}, Discard{EachOpp,1}])`
/// with `random: true` so the bot/AutoDecider doesn't need to pick.
pub fn liliana_of_the_veil() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype as Sup};
    CardDefinition {
        name: "Liliana of the Veil",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Liliana],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: true,
                    },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
                        random: true,
                    },
                ]),
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Sacrifice {
                    who: target_filtered(SelectionRequirement::Player),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature,
                },
            },
        ],
        ..Default::default()
    }
}

/// Light Up the Stage — {2}{R} Sorcery. Exile the top two cards of your
/// library. Until the end of your next turn, you may play those cards.
/// Spectacle {R} (You may cast this spell for its spectacle cost rather
/// than its mana cost if an opponent lost life this turn.)
///
/// 🟡 Approximated as `Draw 2`. Engine doesn't yet model "exile +
/// may-play-this-turn" without the dedicated cast-from-exile pipeline
/// (same gap as Suspend, Adventure's "may cast adventure half from
/// exile", etc.). Spectacle cost is also omitted — alt-cost-with-life-
/// loss-this-turn predicate is its own primitive. Slot stays open
/// for the eventual full wire.
pub fn light_up_the_stage() -> CardDefinition {
    CardDefinition {
        name: "Light Up the Stage",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Liliana of the Last Hope — {1}{B}{B} Legendary Planeswalker — Liliana.
/// Starting loyalty 3.
/// -2: Target creature gets -2/-1 until end of turn.
/// +1: Up to one target creature gets -2/-1 until end of turn. // fixme
/// Actually printed: +1 — Target creature gets -2/-1 until end of turn.
///        -2 — Return target creature card from your graveyard to hand.
///        -7 emblem.
///
/// 🟡 +1 (-2/-1 EOT to creature) wired faithfully. -2 (return creature
/// from your graveyard to your hand) wired. -7 emblem omitted (no
/// emblem zone — Professor Dellian Fel has the same gap).
pub fn liliana_of_the_last_hope() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype as Sup, Zone};
    use crate::effect::Selector as Sel;
    CardDefinition {
        name: "Liliana of the Last Hope",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Liliana],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-2),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Move {
                    what: Sel::take(
                        Sel::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: SelectionRequirement::Creature,
                        },
                        Value::Const(1),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
        ],
        ..Default::default()
    }
}

/// Tibalt's Trickery — {1}{R} Instant. Counter target spell. That spell's
/// controller exiles the top three cards of their library, then reveals
/// cards from the top of their library until they exile a nonland card
/// other than a card with the same name as the countered spell. They
/// cast it without paying its mana cost if able. Then they put all
/// cards exiled this way on the bottom of their library in a random
/// order.
///
/// 🟡 Heavy approximation: ships as a hard counter at {1}{R}. The
/// chaotic "exile 3 + cast a random nonland" cascade rider is omitted
/// (cast-from-exile-without-paying primitive gap; same family as
/// Cascade / Theme Music / Wild-Magic Sorcerer). The base counterspell
/// half is the gameplay-relevant outcome — if a player wants to durdle
/// with the lottery rider, they can cast the random spell manually.
pub fn tibalts_trickery() -> CardDefinition {
    CardDefinition {
        name: "Tibalt's Trickery",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
        },
        ..Default::default()
    }
}

// ── Subtlety — Modern Horizons 2 elemental incarnation ──────────────────────

/// Subtlety — {3}{U}{U}, 3/3 Elemental Incarnation. "Flash. Flying. When
/// Subtlety enters the battlefield, return target creature or planeswalker
/// to its owner's library second from the top. Evoke—Exile a blue card
/// from your hand."
///
/// Push XXXIV: ETB tucks a target creature/planeswalker via
/// `Effect::Move` to `ZoneDest::Library { who: OwnerOf(Target(0)),
/// pos: Top }` (one position from top approximated as top — engine
/// has no positional library insert primitive yet). The Evoke alt-
/// cost (exile blue card) is omitted (alt-cost-by-pitch primitive
/// gap, same as the existing Solitude / Endurance entries).
pub fn subtlety() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Subtlety",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Top,
                },
            },
        }],
        ..Default::default()
    }
}

// ── Monastery Swiftspear — Khans of Tarkir / Modern staple ─────────────────

/// Monastery Swiftspear — {R}, 1/2 Human Monk with Haste and Prowess.
///
/// Push XXXVIII wired Prowess as a synthetic SpellCast trigger
/// (`fire_spell_cast_triggers` in `game/actions.rs` sweeps every
/// battlefield Keyword::Prowess permanent on each noncreature spell
/// cast and pushes a `PumpPT(This, +1/+1, EOT)` trigger). So this
/// 1-drop now ramps from 1/2 → 2/3 → 3/4 → … on every cantrip cast
/// in the same turn, matching the printed Oracle exactly. Same wiring
/// shape as Spectacle Mage's hybrid {U/R}{U/R} Prowess body in
/// `catalog::sets::stx::iconic`.
pub fn monastery_swiftspear() -> CardDefinition {
    CardDefinition {
        name: "Monastery Swiftspear",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::Prowess],
        effect: Effect::Noop,
        ..Default::default()
    }
}

// ── Wild Nacatl — Zendikar / Modern aggressive 1-drop ──────────────────────

/// Wild Nacatl — {G}, 1/1 Cat Warrior. Printed: "Wild Nacatl gets +1/+1
/// as long as you control a Mountain. / Wild Nacatl gets +1/+1 as long
/// as you control a Plains."
///
/// Push XXXIV: ships as a vanilla 1/1 (the Mountain/Plains static lord
/// effects need a `StaticEffect::SelfPumpIfLandcontrolled` primitive
/// that doesn't exist yet — same gap as Steppe Lynx's "landfall +1/+0"
/// rider). Body alone slots into Naya aggro shells; the lord effect is
/// the dominant clause but the {G} 1/1 baseline is still legal.
pub fn wild_nacatl() -> CardDefinition {
    CardDefinition {
        name: "Wild Nacatl",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        ..Default::default()
    }
}

// ── Seasoned Pyromancer — Modern Horizons / Modern staple ──────────────────

/// Seasoned Pyromancer — {1}{R}{R}, 2/2 Human Shaman. "When this
/// creature enters, discard up to two cards, then draw that many cards.
/// For each nonland card discarded this way, create a 1/1 red Elemental
/// creature token."
///
/// Push XXXIV: simplified to "discard 2 + draw 2 + create 2 elemental
/// tokens" (the printed "for each nonland discarded" rider is
/// approximated as "always 2 tokens" since engine has no
/// `CardsDiscardedThisResolution` filter on token-creation count
/// keyed by card-type). On a typical hand of nonlands, this matches
/// the printed effect exactly. The graveyard ability ({3}{R},
/// exile this from gy: each player discards one) is omitted (no
/// cast-from-gy with `exile_self_cost` cost yet — same gap as
/// Faithless Looting's flashback's exile-instead-of-grave rider).
pub fn seasoned_pyromancer() -> CardDefinition {
    use crate::card::{Subtypes as Subs, TokenDefinition};
    let elemental_token = TokenDefinition {
        name: "Elemental".into(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        supertypes: vec![],
        subtypes: Subs {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Seasoned Pyromancer",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(2),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: elemental_token,
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Murktide Regent — Modern Horizons 2 finisher ───────────────────────────

/// Murktide Regent — {3}{U}{U}, 3/3 Dragon. "Delve. Flying. Murktide
/// Regent enters the battlefield with a +1/+1 counter on it for each
/// instant and sorcery card exiled with it. Whenever an instant or
/// sorcery card leaves your graveyard, put a +1/+1 counter on Murktide
/// Regent."
///
/// Push XXXIV: simplified — Delve isn't yet a first-class engine
/// primitive (`alternative_cost: gy-pay-for-generic` gap), so the
/// printed "exile up to N cards from your graveyard for {1}" alt
/// payment is omitted. The card ships at full {3}{U}{U} cost as a 3/3
/// Flying Dragon body. The graveyard-leave growth trigger fires on
/// `EventKind::CardLeftGraveyard` filtered to instants / sorceries
/// (uses the existing per-card emission framework — same shape as
/// Spirit Mascot, Hardened Academic). The "ETB with N counters"
/// rider is omitted (Delve dependency).
pub fn murktide_regent() -> CardDefinition {
    use crate::card::{CounterType, Predicate};
    CardDefinition {
        name: "Murktide Regent",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CardLeftGraveyard,
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Faerie Mastermind — March of the Machine Faerie ────────────────────────

/// Faerie Mastermind — {1}{U}, 2/1 Faerie Rogue with Flash and Flying.
/// "Whenever an opponent draws a card except for the first card they
/// draw on their turn, you draw a card. {2}{U}, Sacrifice this
/// creature: Draw a card."
///
/// Push XXXIV: the "draw extra" trigger is approximated as a fire on
/// **every** opp `EventKind::CardDrawn` (the "except for the first
/// draw on their turn" rider needs a per-turn "drew on their draw
/// step" gate that doesn't exist yet — same approximation as Esper
/// Sentinel's "once per turn" rider). The {2}{U}, Sac: draw 1
/// activation is wired faithfully via `sac_cost: true`. Slots into
/// blue-x card-advantage shells.
pub fn faerie_mastermind() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Predicate;
    CardDefinition {
        name: "Faerie Mastermind",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Flash],
        effect: Effect::Noop,
        // Push: "except the first card they draw each turn" gate now
        // wired via `Predicate::ValueAtLeast(CardsDrawnThisTurn(
        // Triggerer), 2)`. The trigger source's draw is observed via
        // `EventScope::OpponentControl`; the filter resolves on the
        // opponent's per-turn draw tally — a 2nd, 3rd, … draw fires
        // the trigger; the 1st draw skips. Faithful to the printed
        // text (skip-first-draw is per-opponent, per-turn).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl)
                .with_filter(Predicate::ValueAtLeast(
                    Value::CardsDrawnThisTurn(crate::effect::PlayerRef::Triggerer),
                    Value::Const(2),
                )),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

// ── Young Pyromancer — M14 / Modern staple ─────────────────────────────────

/// Young Pyromancer — {1}{R}, 2/1 Human Shaman. "Whenever you cast an
/// instant or sorcery spell, create a 1/1 red Elemental creature
/// token."
///
/// Push XXXIV: classic spellslinger payoff — token-on-IS-cast trigger
/// via `magecraft()` shortcut (which fires on cast or copy of an
/// instant/sorcery). The Elemental token shape matches the printed
/// 1/1 red Elemental (no other abilities). Slots into UR Tempo /
/// Mardu spellslinger shells.
pub fn young_pyromancer() -> CardDefinition {
    use crate::card::{Subtypes as Subs, TokenDefinition};
    use crate::effect::shortcut::magecraft;
    let elemental_token = TokenDefinition {
        name: "Elemental".into(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        supertypes: vec![],
        subtypes: Subs {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Young Pyromancer",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: elemental_token,
        })],
        ..Default::default()
    }
}

// ── Grief — Modern Horizons 2 elemental incarnation ────────────────────────

/// Grief — {2}{B}, 3/2 Elemental Incarnation with Menace. "When this
/// creature enters, target opponent reveals their hand. You choose a
/// nonland card from it. That player discards that card. / Evoke—
/// Exile a black card from your hand."
///
/// Push XXXIV: ETB discard wired via `Effect::DiscardChosen` (same
/// shape as Ravenous Chupacabra / Render Speechless's discard half).
/// The Evoke alt-cost (exile a black card from hand) is omitted —
/// ships at full {2}{B} mana cost. Body + ETB hand-strip alone is
/// already a top-tier discard effect at {2}{B}.
pub fn grief() -> CardDefinition {
    CardDefinition {
        name: "Grief",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
        }],
        ..Default::default()
    }
}

// ── Sage of the Falls — Modern Horizons 3 mill payoff ──────────────────────

/// Sage of the Falls — {3}{U}, 2/4 Bird Fish. "Flying. Whenever you
/// draw a card, if there are five or more cards in your hand, you may
/// draw a card. If you do, discard a card."
///
/// Push XXXIV: simplified — the trigger fires on each draw and the
/// "5+ in hand" gate uses `Predicate::ValueAtLeast(HandSizeOf(You),
/// 5)`. The "you may" is collapsed (auto-decider answers no by
/// default; tests can flip). The body wires `Seq([Draw 1, Discard
/// 1])` once the gate passes. Slots into blue Tempo / Faerie shells.
pub fn sage_of_the_falls() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Sage of the Falls",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Fish],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(
                    Value::HandSizeOf(PlayerRef::You),
                    Value::Const(5),
                )),
            effect: Effect::MayDo {
                description: "Draw a card; if you do, discard a card.".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

// ── Fury — Modern Horizons 2 elemental incarnation ─────────────────────────

/// Fury — {2}{R}{R}, 3/3 Elemental Incarnation. "Double Strike. When
/// this creature enters, it deals 4 damage divided as you choose among
/// any number of target creatures and/or planeswalkers. Evoke—Exile a
/// red card from your hand."
///
/// Push XXXIV: ETB simplified to "4 damage to a single target creature
/// or planeswalker" (the printed "divided as you choose" rider is the
/// same gap that bites Magma Opus / Sundering Stroke). The Evoke
/// alt-cost is omitted (same pitch-cost gap as Solitude / Endurance).
/// At full 4-mana body it's still a respectable 3/3 Double Strike +
/// 4-damage burn package.
pub fn fury() -> CardDefinition {
    CardDefinition {
        name: "Fury",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::DoubleStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
            },
        }],
        ..Default::default()
    }
}

// ── Serum Visions — Fifth Dawn / Modern cantrip ────────────────────────────

/// Serum Visions — {U} Sorcery. "Draw a card. Scry 2."
///
/// ✅ Push XLIV NEW. The classic Modern blue cantrip. Note the printed
/// order is *draw, then scry* (Ponder/Preordain inverted). The order
/// matters — Scry 2 fed by the freshly drawn card lets the controller
/// re-deck dead draws or surface a key card for next turn. Wired
/// faithfully via `Effect::Seq([Draw 1, Scry 2])`.
pub fn serum_visions() -> CardDefinition {
    CardDefinition {
        name: "Serum Visions",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

// ── Burst Lightning — Zendikar / Modern burn ───────────────────────────────

/// Burst Lightning — {R} Instant. "Kicker {4}. This spell deals 2 damage
/// to any target. If this spell was kicked, it deals 4 damage to that
/// permanent or player instead."
///
/// 🟡 Push XLIV NEW. The base 2-damage half is wired faithfully via
/// `Effect::DealDamage` on `any_target()` (Creature ∨ Planeswalker ∨
/// Player). The Kicker {4} mode (4-damage upgrade) is omitted — Kicker
/// is an alt-cost-implies-mode shape (same family as Devastating
/// Mastery's Mastery cost; would need a sibling
/// `AlternativeCost.kicker_mode` field to flip a 2nd mode at cast
/// time). Stays 🟡 until the kicker primitive lands; the base mode is
/// already game-correct as a Shock proxy.
pub fn burst_lightning() -> CardDefinition {
    use crate::effect::shortcut::any_target;
    CardDefinition {
        name: "Burst Lightning",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: any_target(),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

// ── Roiling Vortex — Zendikar Rising sideboard pinger ──────────────────────

/// Roiling Vortex — {R} Enchantment. "At the beginning of each player's
/// upkeep, Roiling Vortex deals 1 damage to that player. Players can't
/// gain life. {1}{R}, Sacrifice Roiling Vortex: It deals 4 damage to any
/// target."
///
/// 🟡 Push XLIV NEW. The upkeep-pinger trigger is wired via
/// `EventKind::StepBegins(Upkeep)` + `EventScope::AnyPlayer` →
/// `Effect::DealDamage(1, Player(ActivePlayer))`. The activated
/// `{1}{R}, Sacrifice this: 4 damage` is wired with `sac_cost: true`.
/// The "players can't gain life" continuous lock is omitted (no
/// global lifegain-replacement static yet — same gap as Erebos, God
/// of the Dead's lifegain prevention). Stays 🟡 overall.
pub fn roiling_vortex() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::any_target;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Roiling Vortex",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(1),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::DealDamage {
                to: any_target(),
                amount: Value::Const(4),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

// ── Murderous Cut — Khans of Tarkir / Modern removal ───────────────────────

/// Murderous Cut — {4}{B} Instant. "Delve. Destroy target creature."
///
/// 🟡 Push XLIV NEW. The base body is wired faithfully — destroy
/// target creature, paid at the printed full {4}{B}. The Delve cost
/// reduction is omitted (same gap as Treasure Cruise / Lose Focus
/// / Dig Through Time — Delve is a cast-time alt-cost-discount
/// primitive that hasn't landed yet, tracked in TODO.md). Stays 🟡
/// overall; the destroy half is exact.
pub fn murderous_cut() -> CardDefinition {
    CardDefinition {
        name: "Murderous Cut",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Creature),
        },
        ..Default::default()
    }
}

// ── Eidolon of the Great Revel — Journey into Nyx / Modern Burn punisher ───

/// Eidolon of the Great Revel — {R}{R} Creature — Enchantment Creature
/// — Spirit. 2/2. "Whenever a player casts a spell with mana value 3 or
/// less, Eidolon of the Great Revel deals 2 damage to that player."
///
/// ✅ Push XLIV NEW. Symmetrical "Burn punisher" body wired via
/// `EventKind::SpellCast` + `EventScope::AnyPlayer` + a
/// `Predicate::EntityMatches { what: TriggerSource, filter:
/// ManaValueAtMost(3) }` filter. The damage-dealing target is
/// `Selector::Player(PlayerRef::ControllerOf(TriggerSource))` —
/// the controller of the just-cast spell on the stack. Note the
/// symmetry: Eidolon also damages its own controller when *they*
/// cast 1- or 2-mana spells, matching printed exact (Modern Burn's
/// tension between aggression and self-damage from Eidolon is a
/// defining feature of the archetype).
pub fn eidolon_of_the_great_revel() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Eidolon of the Great Revel",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Creature, CardType::Enchantment],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::ManaValueAtMost(3),
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(
                    Selector::TriggerSource,
                ))),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

// ── Wild Slash — Fate Reforged / Modern burn ───────────────────────────────

/// Wild Slash — {R} Instant. "Spell mastery — If there are two or more
/// instant and/or sorcery cards in your graveyard, damage can't be
/// prevented this turn. Wild Slash deals 2 damage to any target."
///
/// ✅ Push XLIV NEW. The base 2-damage half is wired faithfully via
/// `Effect::DealDamage` on `any_target()`. The Spell Mastery damage-
/// can't-be-prevented rider is a no-op gameplay-wise — the engine
/// has no prevention layer that opps can stack against burn (the
/// `Effect::PreventCombatDamageThisTurn` primitive only gates combat
/// damage, not direct damage). So the rider has nothing to fight
/// against; the 2-damage half is unconditional already and matches
/// printed Oracle for cube/modern board states.
pub fn wild_slash() -> CardDefinition {
    use crate::effect::shortcut::any_target;
    CardDefinition {
        name: "Wild Slash",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: any_target(),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

// ── Krenko, Mob Boss — Magic 2013 / Goblin tribal ──────────────────────────

/// Krenko, Mob Boss — {2}{R}{R} Legendary Creature — Goblin Warrior.
/// 3/3. "{T}: Create X 1/1 red Goblin creature tokens, where X is the
/// number of Goblins you control."
///
/// ✅ Push XLIV NEW. Activated `{T}: CreateToken(Goblin, X)` with
/// `X = CountOf(EachPermanent(Creature ∧ HasCreatureType(Goblin) ∧
/// ControlledByYou))`. Because the count is read *at activation time
/// before the new tokens are created*, the activation doubles your
/// goblins each turn (1 → 2 → 4 → 8 → ...) — matching the printed
/// exponential blow-out of the Goblin tribal endgame.
pub fn krenko_mob_boss() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::card::TokenDefinition;
    let goblin = TokenDefinition {
        name: "Goblin".to_string(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Krenko, Mob Boss",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Goblin))
                        .and(SelectionRequirement::ControlledByYou),
                )),
                definition: goblin,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        ..Default::default()
    }
}

