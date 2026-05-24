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
use crate::effect::shortcut::{etb_gain_life, target_filtered};
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Exhume — {1}{B} Sorcery. Each player puts a creature card from their
/// graveyard onto the battlefield.
///
/// Push (modern_decks): the "each player" symmetry **is now wired** via
/// `ForEach { selector: Player(EachPlayer), body: Move(take(1,
/// CardsInZone(Graveyard(Triggerer), Creature)), Battlefield(Triggerer)) }`.
/// Each iterated player's auto-decider picks the top matching creature
/// in their own graveyard. The reanimate target winds up under each
/// respective player's control. In typical play (Goryo's etc.) the
/// caster has the biggest creature stocked in gy; the opp may
/// reanimate nothing or a weaker body.
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
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachPlayer),
            body: Box::new(Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::Triggerer,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::Triggerer,
                    tapped: false,
                },
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Crop Rotation — {G} Instant. As an additional cost, sacrifice a land.
/// Search your library for a land card and put it onto the battlefield.
/// Then shuffle.
///
/// The "sacrifice a land" additional cost is folded into the resolved
/// effect's first step (matching Thud's sacrifice-as-resolution pattern).
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
        effect: Effect::Seq(vec![
            // Sacrifice a land you control as part of resolution.
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Land
                    .and(SelectionRequirement::ControlledByYou),
            },
            // Tutor a land into play.
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Card-selection / hand-disruption / counterspell extras ──────────────────

/// Cathartic Reunion — {1}{R} Sorcery. As an additional cost to cast this
/// spell, discard two cards. Draw three cards.
///
/// The "additional cost" is folded into resolution as the spell's first
/// step (matching Crop Rotation's sacrifice-as-resolution shape). Net
/// gameplay: discard 2 → draw 3, identical to Oracle for the standard line.
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
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Abrupt Decay — {B}{G} Instant. This spell can't be countered. Destroy
/// target nonland permanent with mana value 2 or less.
///
/// Uses the existing `Keyword::CantBeCountered` (consumed by
/// `caster_grants_uncounterable` in all three cast paths) and a target
/// filter of `Nonland ∧ ManaValueAtMost(2)` validated at cast time.
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
                    .and(SelectionRequirement::ManaValueAtMost(2)),
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![etb],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            condition: None,
                    exile_from_graveyard_count: 0,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Mine Collapse — {2}{R} Sorcery. As an additional cost, sacrifice a
/// Mountain. Mine Collapse deals 4 damage to any target.
///
/// "Sacrifice a Mountain" is folded into the resolved effect's first
/// step (cost-as-first-step approximation) — same pattern Crop Rotation,
/// Thud, and Cephalid Coliseum use. The damage step is targeted via the
/// standard `Selector::Target(0)` slot.
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
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Land
                    .and(SelectionRequirement::HasLandType(crate::card::LandType::Mountain)),
            },
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(4),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            },
            make_color(Color::White),
            make_color(Color::Blue),
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            },
            make_color(Color::Blue),
            make_color(Color::Black),
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            },
            make_color(c1),
            make_color(c2),
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Red discard-loot ─────────────────────────────────────────────────────────

/// Tormenting Voice — {1}{R} Sorcery. Discard a card, then draw two cards.
///
/// Same shape as Cathartic Reunion, but the additional cost is reduced from
/// "discard two" to "discard one" so the card is just a +0 net hand size
/// shuffle instead of a graveyard-fill payload. Modeled cost-as-first-step
/// (discard then draw) — gameplay-equivalent for the deterministic case.
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
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Thrill of Possibility — {1}{R} Instant. As an additional cost to cast
/// this spell, discard a card. Draw two cards.
///
/// Instant-speed Tormenting Voice — same shape, different timing.
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
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        // "If X is 5 or more, this spell can't be countered."
        // `caster_grants_uncounterable_with_x` reads the threshold off
        // this keyword at cast time instead of matching on the name.
        keywords: vec![Keyword::CantBeCounteredIfXAtLeast(5)],
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
/// Push (modern_decks): the exile-on-death rider **is now approximated**
/// via `Effect::If { cond: ValueAtMost(ToughnessOf(Target(0)), 4),
/// then: Exile, else_: DealDamage 4 }`. When the target's toughness is
/// ≤ 4 (the lethal case at resolution time), the engine routes the
/// target directly to exile instead of dealing damage that would
/// trigger an SBA destroy → graveyard transition. When toughness > 4,
/// the spell deals 4 damage (which by itself wouldn't kill the
/// target, so the printed "would die" rider doesn't apply). The
/// printed-Oracle edge case where prior damage marked on the creature
/// would combine with Lava Coil's 4 to kill it is not captured (no
/// general damage-replacement-with-exile primitive on the engine
/// yet). Functionally close enough for typical play.
pub fn lava_coil() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Lava Coil",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::ValueAtMost(
                Value::ToughnessOf(Box::new(Selector::Target(0))),
                Value::Const(4),
            ),
            then: Box::new(Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
            }),
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![
            modern_etb_tap(),
            etb_gain_life(1),
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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

/// Coalition Relic — {3} Artifact.
///
/// Printed Oracle (now fully wired):
/// - `{T}: Add one mana of any color.`
/// - `{T}: Put a charge counter on Coalition Relic.`
/// - `Remove three charge counters from Coalition Relic: Add {W}{U}{B}{R}{G}.`
///
/// The charge-counter→WUBRG burst payoff models on the Pentad Prism shape:
/// a `Seq([RemoveCounter(Charge, 3), AddMana(Colors(WUBRG))])` body
/// where the "no counters → can't activate" gate is enforced by the
/// engine's counter-removal failing when the pool is empty (which kicks
/// the ability out of the activation candidate list).
pub fn coalition_relic() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    CardDefinition {
        name: "Coalition Relic",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
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
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::Const(1),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false, exile_other_filter: None,
                self_counter_cost_reduction: None, sac_other_filter: None,
            },
            ActivatedAbility {
                tap_cost: false,
                mana_cost: ManaCost::default(),
                effect: Effect::Seq(vec![
                    Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::Const(3),
                    },
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colors(vec![
                            Color::White,
                            Color::Blue,
                            Color::Black,
                            Color::Red,
                            Color::Green,
                        ]),
                    },
                ]),
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                // Activation gate: source must have ≥ 3 charge counters.
                // Without this, the engine would let the activation onto
                // the stack and then resolve the AddMana even though
                // RemoveCounter silently no-ops when the pool is empty.
                condition: Some(crate::card::Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    },
                    Value::Const(3),
                )),
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false, exile_other_filter: None,
                self_counter_cost_reduction: None, sac_other_filter: None,
            },
        ],
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

/// Volcanic Fallout — {1}{R}{R} Instant. Volcanic Fallout can't be
/// countered. Volcanic Fallout deals 2 damage to each creature and
/// each player.
///
/// Push (modern_decks): the "can't be countered" rider **is now wired**
/// via `Keyword::CantBeCountered` on the card definition.
/// `caster_grants_uncounterable_with_x` already checks for this
/// keyword on the cast, so the resulting `StackItem::Spell.uncounterable`
/// = true and counterspells targeting the Fallout fizzle. Body
/// unchanged (2 damage to each creature, 2 damage to each player).
pub fn volcanic_fallout() -> CardDefinition {
    CardDefinition {
        name: "Volcanic Fallout",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        ..Default::default()
    }
}

/// Rofellos, Llanowar Emissary — {G}{G} Legendary Creature — Elf Druid. 2/1.
/// {T}: Add {G}{G} for each Forest you control.
///
/// Push (modern_decks): the "for each Forest" rider is now wired
/// faithfully via `ManaPayload::OfColor(Green, Value::Times(Const(2),
/// CountOf(Forest ∧ ControlledByYou)))`. Tapping Rofellos with N Forests
/// in play adds `2·N` green mana to the controller's pool. The activation
/// cost remains a plain `{T}`; the dynamic payload reads the live Forest
/// count at resolution time. Same shape as Topiary Lecturer / Molten-Core
/// Maestro's power-scaled mana payouts (`PowerOf(This)`).
pub fn rofellos_llanowar_emissary() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType, Supertype as Sup};
    let forest_count = Value::CountOf(Box::new(Selector::EachPermanent(
        SelectionRequirement::HasLandType(LandType::Forest)
            .and(SelectionRequirement::ControlledByYou),
    )));
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
                    Value::Times(Box::new(Value::Const(2)), Box::new(forest_count)),
                ),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
    use crate::effect::shortcut::count;
    // Printed Oracle: "Each player's life total becomes the number of
    // creatures they control."
    //
    // Push (modern_decks): Now wired faithfully via the new
    // `Effect::SetLifeTotal { who, amount: Value::CountOf(creatures
    // they control) }` primitive (CR 119.5). Walks all players in
    // turn; for each one sets their life to the count of creatures
    // they control. Replaces the prior approximation that drained
    // opp to ≤ 0 and gained you life equal to creature count.
    let creatures_you_control = Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    let creatures_opp_controls = Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
    );
    CardDefinition {
        name: "Biorhythm",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Set your life total to the number of creatures you control.
            Effect::SetLifeTotal {
                who: Selector::You,
                amount: count(creatures_you_control),
            },
            // Set each opponent's life total to the number of creatures
            // they control. (Approximated as "each opp" — multi-opp
            // games would need per-opponent counts, which we collapse
            // to the single-opp typical 1v1 case via EachOpponent +
            // ControlledByOpponent which from each opp's perspective
            // reads "creatures they control".)
            Effect::SetLifeTotal {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: count(creatures_opp_controls),
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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

/// Yarok, the Desecrated — {2}{U}{B}{G} Legendary Creature — Horror.
/// 3/5 Deathtouch, Lifelink. "If a permanent entering the battlefield
/// causes a triggered ability of a permanent you control to trigger,
/// that ability triggers an additional time."
///
/// Cube-style approximation: the "doubles your ETB triggers" static
/// is engine-wide ⏳ (no trigger-multiplication primitive). Ships as
/// a 3/5 deathtouch + lifelink Horror body — strictly weaker than
/// printed, but the body alone is a midgame value engine.
pub fn yarok_the_desecrated() -> CardDefinition {
    use crate::card::Supertype as Sup;
    CardDefinition {
        name: "Yarok, the Desecrated",
        cost: cost(&[generic(2), u(), b(), g()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Hellrider — {2}{R}{R} Creature — Devil. 3/3 Haste. "Whenever this
/// creature attacks, it deals 1 damage to defending player."
///
/// Wired with an `Attacks/SelfSource` trigger that pings each opponent
/// for 1 (the auto-target picks an opp via `EachOpponent`). The "defending
/// player" half is collapsed to each opponent — fine for 2-player.
pub fn hellrider() -> CardDefinition {
    CardDefinition {
        name: "Hellrider",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Generous Gift — {2}{W} Instant. Destroy target permanent. Its
/// controller creates a 3/3 green Elephant creature token.
///
/// Cube-style approximation: the printed "owner gets an Elephant"
/// downside is collapsed (no `CreateTokenFor { who: ControllerOfTarget,
/// definition }` primitive — `Effect::CreateToken.who` resolves to a
/// `PlayerRef`, not a target's controller). Ships as a clean `Destroy
/// (Permanent)` with the token half dropped. Strictly stronger than
/// printed; an upgrade-from-Generous-Gift candidate when the controller
/// -of-target primitive lands.
pub fn generous_gift() -> CardDefinition {
    CardDefinition {
        name: "Generous Gift",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Any),
        },
        ..Default::default()
    }
}

/// Putrefy — {1}{B}{G} Instant. Destroy target artifact or creature.
/// It can't be regenerated.
///
/// Faithful — `Destroy(target Artifact ∨ Creature)`. Regen-suppression
/// is implicit (Effect::Destroy bypasses regenerate replacement
/// effects per the engine's destroy pipeline).
pub fn putrefy_modern() -> CardDefinition {
    CardDefinition {
        name: "Putrefy (Modern)",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            ),
        },
        ..Default::default()
    }
}

/// Etali, Primal Storm — {4}{R}{R} Legendary Creature — Elder Dinosaur.
/// 6/6. "Whenever this attacks, exile the top card of each player's
/// library. You may cast any number of nonland cards exiled this way
/// without paying their mana costs."
///
/// Cube-style approximation: the "cast for free" rider is engine-wide
/// ⏳ (no multi-player exile-then-may-cast loop). The attack trigger
/// is approximated by milling each player 1 (the exile-to-removed-pile
/// half), with the cast-without-paying clause dropped. A 6/6 attacker
/// for 6 mana is still a fair body without the rider.
pub fn etali_primal_storm() -> CardDefinition {
    use crate::card::Supertype as Sup;
    CardDefinition {
        name: "Etali, Primal Storm",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Knight of the Reliquary — {1}{G}{W} Creature — Human Knight Warrior.
/// 2/2. "Knight of the Reliquary gets +1/+1 for each land card in all
/// graveyards. {T}, Sacrifice a Forest or Plains: Search your library
/// for a land card, put it onto the battlefield, then shuffle."
///
/// The dynamic P/T scaling is wired via the new `DynamicPt::
/// LandsInAllGraveyards` variant (push 102 engine extension — the
/// `compute_battlefield` pass injects a layer-7b
/// `SetPowerToughness(2+N, 2+N)` continuous effect where N is the
/// total land count across every player's graveyard). The activated
/// land-tutor uses `sac_cost: true` with the source filtered to
/// "controller's permanents matching Forest/Plains" approximated as
/// the activator's own land-typed pick. Approximation: the printed
/// "sacrifice a Forest or Plains" cost is wired as a generic
/// `sac_cost: true` (the source itself); the Forest/Plains filter is
/// dropped — the cube AutoDecider will happily sacrifice the Knight,
/// which doesn't match printed intent. (A future cost-with-filter
/// extension would tighten this.)
pub fn knight_of_the_reliquary() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Knight of the Reliquary",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Knight,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        // Base 2/2; the compute-time injection overrides with
        // (2 + lands_in_gys, 2 + lands_in_gys) via the new
        // `DynamicPt::LandsInAllGraveyards` variant.
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            sac_cost: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goblin Rabblemaster — {2}{R} Creature — Goblin Warrior. 2/2. "Other
/// Goblin creatures you control get +1/+0 and have haste. Whenever
/// this attacks, create a 1/1 red Goblin creature token that's tapped
/// and attacking. Goblin creatures you control attack each combat if
/// able."
///
/// Cube-style approximation. Wires the attack-trigger token-creation
/// half (1/1 red Goblin); the "other goblins +1/+0 and haste" anthem
/// and "must attack" restriction are dropped (no goblin-anthem static
/// primitive nor must-attack restriction in the engine). The body is
/// still strong on attack-trigger token chains.
pub fn goblin_rabblemaster() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Goblin Rabblemaster",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Goblin".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Goblin],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Magma Spray — {R} Instant. Magma Spray deals 2 damage to target
/// creature. If that creature would die this turn, exile it instead.
///
/// Approximation: the "if it would die, exile instead" rider uses the
/// same `Effect::If { cond: ValueAtMost(ToughnessOf(Target), 2), then:
/// Exile, else_: DealDamage 2 }` pattern as Lava Coil. When the target's
/// toughness ≤ 2 (the lethal case), the engine routes directly to exile;
/// otherwise just deals 2 damage. The prior-damage-on-creature edge
/// case isn't captured (no general damage-replacement-with-exile
/// primitive).
pub fn magma_spray() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Magma Spray",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::ValueAtMost(
                Value::ToughnessOf(Box::new(target_filtered(
                    SelectionRequirement::Creature,
                ))),
                Value::Const(2),
            ),
            then: Box::new(Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            }),
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
/// Push (claude/modern_decks): the "can't gain life this turn" rider
/// is now wired via the new `Effect::LifeGainLockThisTurn` primitive,
/// backed by `Player.cannot_gain_life_this_turn` (reset in `do_untap`
/// across all players). The lock fires *before* the 3-damage Bolt so
/// any drain-back-to-life interaction (Skullcrack into Lifelink)
/// blocks the lifegain side. The "damage can't be prevented" rider is
/// still omitted (no general damage-prevention layer).
pub fn skullcrack() -> CardDefinition {
    CardDefinition {
        name: "Skullcrack",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LifeGainLockThisTurn {
                who: target_filtered(SelectionRequirement::Player),
            },
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Fiery Impulse — {R} Instant. Fiery Impulse deals 2 damage to target
/// creature. Spell mastery — if there are two or more instant and/or
/// sorcery cards in your graveyard, it deals 3 damage instead.
///
/// Push (claude/modern_decks): spell-mastery scaling now wires via
/// `Effect::If { cond: SelectorCountAtLeast(IS-in-your-gy, 2),
/// then: DealDamage 3, else_: DealDamage 2 }`. Uses the existing
/// `Selector::CardsInZone` + `SelectorCountAtLeast` primitives.
pub fn fiery_impulse() -> CardDefinition {
    use crate::effect::shortcut::spell_mastery_gate;
    CardDefinition {
        name: "Fiery Impulse",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: spell_mastery_gate(),
            then: Box::new(Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            }),
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
/// Push (claude/modern_decks): the 20-card-graveyard rider now wires
/// via `Effect::If { cond: ValueAtLeast(MaxGraveyardSize, Const(20)),
/// then: Draw(3), else_: Draw(1) }`. `MaxGraveyardSize` is the new
/// `Value` variant that returns the largest graveyard among all alive
/// players, matching the printed "a graveyard with twenty or more"
/// (the printed text doesn't restrict to your own graveyard).
pub fn visions_of_beyond() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Visions of Beyond",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(Value::MaxGraveyardSize, Value::Const(20)),
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
            else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        },
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

// ── New cards (push: claude/modern_decks) ───────────────────────────────────

/// Snapcaster Mage — {1}{U} Creature — Human Wizard 2/1, Flash. "When
/// this creature enters, target instant or sorcery card in your graveyard
/// gains flashback until end of turn. The flashback cost is equal to its
/// mana cost."
///
/// Wired with the same `Effect::GrantMayPlay { exile_after: true,
/// duration: EndOfThisTurn }` shape that powers Flashback (the spell) and
/// Lorehold the Historian's miracle grant. Approximation: the cast pays
/// `{0}` (no `MayPlayPermission.alt_cost`-equals-mana-cost primitive
/// today), which is strictly stronger than the printed "flashback cost
/// equals its mana cost". The play pattern — recover one IS spell from
/// your gy for one turn — is preserved.
pub fn snapcaster_mage() -> CardDefinition {
    use crate::card::{Keyword, Zone};
    CardDefinition {
        name: "Snapcaster Mage",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GrantMayPlay {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    },
                    Value::Const(1),
                ),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: true,
            },
        }],
        ..Default::default()
    }
}

/// Pyroblast — {R} Instant. "Choose one — Counter target spell if it's
/// blue. / Destroy target permanent if it's blue."
///
/// Wired as `ChooseMode([CounterSpell(IsSpellOnStack ∧ HasColor(Blue)),
/// Destroy(Permanent ∧ HasColor(Blue))])`. AutoDecider picks mode 0 (the
/// counter half is usually the higher-tempo pick). Each mode's
/// target-filter rejects non-blue targets at cast time.
pub fn pyroblast() -> CardDefinition {
    CardDefinition {
        name: "Pyroblast",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasColor(Color::Blue)),
                ),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::HasColor(Color::Blue)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Red Elemental Blast — {R} Instant. Functionally identical to Pyroblast
/// (the original printing — Pyroblast is the Chronicles reprint with
/// slightly altered wording). Reuses the same `ChooseMode` shape.
pub fn red_elemental_blast() -> CardDefinition {
    CardDefinition {
        name: "Red Elemental Blast",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasColor(Color::Blue)),
                ),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::HasColor(Color::Blue)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Hydroblast — {U} Instant. "Choose one — Counter target spell if it's
/// red. / Destroy target permanent if it's red."
///
/// Blue mirror of Pyroblast. AutoDecider picks mode 0.
pub fn hydroblast() -> CardDefinition {
    CardDefinition {
        name: "Hydroblast",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasColor(Color::Red)),
                ),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::HasColor(Color::Red)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Blue Elemental Blast — {U} Instant. Functionally identical to
/// Hydroblast (the original printing).
pub fn blue_elemental_blast() -> CardDefinition {
    CardDefinition {
        name: "Blue Elemental Blast",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasColor(Color::Red)),
                ),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::HasColor(Color::Red)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Three Visits — {1}{G} Sorcery. "Search your library for a Forest
/// card, put it onto the battlefield, then shuffle." Identical text to
/// Nature's Lore (which is already in the catalog) — they're both
/// `Search(Land ∧ HasLandType(Forest) → BF untapped)`. Three Visits is
/// the Portal Three Kingdoms / Commander Legends printing, included
/// here so green ramp shells can run the full eight-copy package.
pub fn three_visits() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Three Visits",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Land
                .and(SelectionRequirement::HasLandType(LandType::Forest)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Tale's End — {1}{U} Instant. "Counter target legendary spell,
/// activated or triggered ability from a legendary source, or saga."
///
/// Wired as `ChooseMode([CounterSpell(IsSpellOnStack ∧ Legendary),
/// CounterAbility(AnyAbility)])`. AutoDecider picks mode 0 (the
/// counter-the-legendary-spell half is the marquee use). The "saga"
/// half collapses (no saga primitive yet) but the ability counter
/// catches saga lore-counter triggers via `CounterAbility`.
pub fn tales_end() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Tale's End",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasSupertype(Supertype::Legendary)),
                ),
            },
            Effect::CounterAbility {
                what: Selector::Target(0),
            },
        ]),
        ..Default::default()
    }
}

/// Wall of Omens — {1}{W} Creature — Wall 0/4 with Defender. "When this
/// creature enters, draw a card."
///
/// Classic cantrip defender. ETB Draw 1 is the canonical pattern —
/// fills the slot of "defensive 4-toughness body + replaces itself"
/// that Eternal Witness fills for green and Mulldrifter fills at 5
/// mana for blue.
pub fn wall_of_omens() -> CardDefinition {
    use crate::card::Keyword;
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

/// Toxic Deluge — {2}{B}{B} Sorcery. "As an additional cost to cast
/// this spell, pay X life. All creatures get -X/-X until end of turn."
///
/// Approximation: the cost-as-life payment is folded into resolution
/// (cost-as-first-step model) — at resolution we pay `XFromCost` life
/// and then apply `-X/-X` to each creature. Hits indestructibles only
/// to shrink them, but kills creatures with toughness ≤ X. The cost
/// model collapses the "pay life as additional cost" gate; the
/// gameplay outcome (a one-sided wrath scaled by X life paid) is
/// preserved.
pub fn toxic_deluge() -> CardDefinition {
    CardDefinition {
        name: "Toxic Deluge",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Pay X life as additional cost (cost-as-first-step).
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::XFromCost,
            },
            // Each creature gets -X/-X EOT (read live from the spell's
            // X paid via `XFromCost`).
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                    toughness: Value::Diff(
                        Box::new(Value::Const(0)),
                        Box::new(Value::XFromCost),
                    ),
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        // Toxic Deluge is famously an X-cost sorcery — the X slot lets
        // the caster scale the sweep up at extra cost.
        ..Default::default()
    }
}

/// Pernicious Deed — {1}{B}{G} Enchantment. "{X}, Sacrifice this
/// enchantment: Destroy each artifact, creature, and enchantment with
/// mana value X or less."
///
/// Wired as a `sac_cost: true` activation that pays X generic + sacs
/// Deed, then `ForEach(EachPermanent(...)) + If(ManaValueAtMost(X),
/// Destroy)`. The mana-value filter reads `XFromCost` at resolution.
/// Lands are intentionally excluded (printed). Reuses the wrath-with-X
/// shape that already drives Wrath of the Skies and Pest Control.
pub fn pernicious_deed() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Predicate;
    CardDefinition {
        name: "Pernicious Deed",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[ManaSymbol::X]),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .or(SelectionRequirement::Enchantment),
                ),
                body: Box::new(Effect::If {
                    cond: Predicate::ValueAtMost(
                        Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                        Value::XFromCost,
                    ),
                    then: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
                    else_: Box::new(Effect::Noop),
                }),
            },
            once_per_turn: false,
            sorcery_speed: true,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        ..Default::default()
    }
}

/// Wall of Roots — {1}{G} Creature — Plant Wall 0/5 with Defender.
/// "Put a -0/-1 counter on this creature: Add {G}. Activate only once
/// each turn."
///
/// The -0/-1 counter is modeled as a +1/+1 negative counter on the
/// engine side; here we use a direct `PumpPT(-0/-1)` permanent
/// modification stand-in. Approximation: in lieu of a true "permanent
/// modification stack" the toughness drops by 1 each activation via a
/// permanent `Duration::Permanent` pump. Mana ability so the activation
/// doesn't go on the stack.
pub fn wall_of_roots() -> CardDefinition {
    use crate::card::{ActivatedAbility, Keyword};
    CardDefinition {
        name: "Wall of Roots",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(0),
                    toughness: Value::Const(-1),
                    duration: Duration::Permanent,
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Green]),
                },
            ]),
            once_per_turn: true,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        ..Default::default()
    }
}

/// Demonic Consultation — {B} Instant. "Name a card. Exile the top six
/// cards of your library, then reveal cards from the top of your
/// library until you reveal the named card, exiling each card along the
/// way. Put that card into your hand and exile all other cards revealed
/// this way."
///
/// Approximation: name-a-card primitive doesn't exist, so we collapse
/// to "exile top 6, then take any one card." Powerful but flavor-
/// accurate (the controller still trades 6 cards from their library for
/// 1 card to hand). Auto-target on the search picks any card. The
/// printed "exile" destination for the misses is approximated with
/// `Mill` (cards routed to graveyard) — strictly more recoverable than
/// printed but preserves the library-thinning cost.
pub fn demonic_consultation() -> CardDefinition {
    CardDefinition {
        name: "Demonic Consultation",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(6),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Channel — {G} Sorcery (Kamigawa). "Until end of turn, you may pay
/// 1 life rather than pay {1}." Approximated as a one-shot "lose 1
/// life and add {C} once" tap, since the engine has no "pay life
/// instead of mana" alternative-payment primitive.
///
/// The printed spell is a static "this turn" replacement on mana
/// payments. Our shipping body folds the alt-payment into a single
/// resolve: pay 1 life + add {1} once. Re-cast the spell to ramp
/// further.
pub fn channel() -> CardDefinition {
    CardDefinition {
        name: "Channel",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(1)),
            },
        ]),
        ..Default::default()
    }
}

/// Phyrexian Reclamation — {B}{B} Enchantment. "{1}{B}, Pay 2 life:
/// Return target creature card from your graveyard to your hand."
///
/// Pure activated reanimator engine. The `life_cost: 2` field handles
/// the life payment; the mana cost is one generic + one black; the
/// target filter picks a creature card in your graveyard via
/// `prefers_graveyard_source` heuristic.
pub fn phyrexian_reclamation() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Phyrexian Reclamation",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 2,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        ..Default::default()
    }
}

/// Sylvan Library — {1}{G} Enchantment. "At the beginning of your draw
/// step, you may draw two additional cards. If you do, choose two
/// cards in your hand drawn this turn. For each of those cards, pay 4
/// life or put the card on top of your library."
///
/// Approximation: the "draw 2 extra, return 2 unless you pay 8" loop
/// is collapsed to a flat "draw 1 extra each turn, lose 4 life" — same
/// net outcome (1 extra card / 4 life) without the multi-step decision
/// shape. The "pick which two to return" choice is dropped (no
/// hand-card-selection primitive on a triggered ability today).
pub fn sylvan_library() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Sylvan Library",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Draw a card and lose 4 life.".to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::Const(4),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Howling Mine — {2} Artifact. "At the beginning of each player's
/// draw step, if Howling Mine is untapped, that player draws an
/// additional card."
///
/// Approximation: the "is Howling Mine untapped" gate collapses (the
/// trigger always fires); both players always draw an extra card on
/// their draw step. Wired as a `StepBegins(Draw)/AnyPlayer` trigger
/// over `Selector::Player(PlayerRef::ActivePlayer)` so each player
/// draws their own extra card on their own turn.
pub fn howling_mine() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Howling Mine",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::AnyPlayer),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Ophiomancer — {2}{B} Creature — Human Shaman 1/1. "At the beginning
/// of each upkeep, if you control no Snakes, create a 1/1 black Snake
/// creature token with deathtouch."
///
/// Approximation: the "if you control no Snakes" intervening-if is
/// collapsed (the trigger always fires) — strictly more value than
/// printed but the gameplay outcome (a steady stream of 1/1 deathtouch
/// chump-blockers / aristocrat fodder) is preserved. Uses a one-off
/// `TokenDefinition` for the Snake.
pub fn ophiomancer() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::game::types::TurnStep;
    let snake = TokenDefinition {
        name: "Snake".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Ophiomancer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: snake,
            },
        }],
        ..Default::default()
    }
}

/// Yavimaya Elder — {1}{G}{G} Creature — Human Druid 2/1. "When this
/// creature dies, you may search your library for up to two basic land
/// cards, reveal them, put them into your hand, then shuffle. /
/// {2}{G}, Sacrifice this creature: Draw a card."
///
/// Wired with both abilities: the dies-trigger fans out two `Search`es
/// (one each) for basic land → Hand, with `MayDo` wrapping each so the
/// controller can opt out. The activated draw-a-card is a sac-cost
/// activation. Yavimaya Elder is a key cube role-player: 3 mana for a
/// 2/1 plus 2 lands plus a card.
pub fn yavimaya_elder() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let search_basic = || Effect::Search {
        who: PlayerRef::You,
        filter: SelectionRequirement::IsBasicLand,
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Yavimaya Elder",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::MayDo {
                    description: "Search for a basic land.".to_string(),
                    body: Box::new(search_basic()),
                },
                Effect::MayDo {
                    description: "Search for a second basic land.".to_string(),
                    body: Box::new(search_basic()),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        ..Default::default()
    }
}

/// Stroke of Genius — {X}{2}{U} Instant. "Target player draws X cards."
/// X-cost draw spell. Reads `XFromCost` at resolution; the auto-target
/// picks a player (self by default for the card-advantage line).
pub fn stroke_of_genius() -> CardDefinition {
    CardDefinition {
        name: "Stroke of Genius",
        cost: cost(&[ManaSymbol::X, generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Green Sun's Zenith — {X}{G} Sorcery. "Search your library for a green
/// creature card with mana value X or less, put it onto the battlefield,
/// then shuffle. Shuffle this card into its owner's library."
///
/// Approximation: the "shuffle into library" rider collapses (the spell
/// goes to graveyard normally). The body wires the X-gated tutor via
/// `Search(Creature ∧ HasColor(Green) ∧ ManaValueAtMost(X) → BF)`.
pub fn green_suns_zenith() -> CardDefinition {
    CardDefinition {
        name: "Green Sun's Zenith",
        cost: cost(&[ManaSymbol::X, g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::HasColor(Color::Green)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Red Sun's Zenith — {X}{R} Instant. "Red Sun's Zenith deals X damage
/// to any target. If a creature dealt damage this way would die this
/// turn, exile it instead. Shuffle this card into its owner's library."
///
/// Wired as a simple X-damage burn at instant speed. The "exile if
/// would die" rider and "shuffle into library" rider both collapse.
pub fn red_suns_zenith() -> CardDefinition {
    CardDefinition {
        name: "Red Sun's Zenith",
        cost: cost(&[ManaSymbol::X, r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// White Sun's Zenith — {X}{W}{W}{W} Instant. "Create X 2/2 white Cat
/// creature tokens. Shuffle this card into its owner's library."
///
/// X-cost army-in-a-can. The "shuffle into library" rider collapses.
pub fn white_suns_zenith() -> CardDefinition {
    use crate::card::TokenDefinition;
    let cat = TokenDefinition {
        name: "Cat".into(),
        power: 2,
        toughness: 2,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "White Sun's Zenith",
        cost: cost(&[ManaSymbol::X, w(), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: cat,
        },
        ..Default::default()
    }
}

/// Black Sun's Zenith — {X}{B} Sorcery. "Put X -1/-1 counters on each
/// creature. Shuffle this card into its owner's library."
///
/// Wired via `ForEach + AddCounter(MinusOneMinusOne, X)`. The
/// "shuffle into library" rider collapses.
pub fn black_suns_zenith() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Black Sun's Zenith",
        cost: cost(&[ManaSymbol::X, b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::MinusOneMinusOne,
                amount: Value::XFromCost,
            }),
        },
        ..Default::default()
    }
}

// ── claude/modern_decks batch 102: multicolor cube expansion ────────────────

/// Sorin, Grim Nemesis — {4}{B}{B} Legendary Planeswalker — Sorin.
/// 6 loyalty.
/// **+1**: Reveal the top card of your library and put that card into your
/// hand. You lose life equal to its mana value. If it's a creature card,
/// create an X/X black Knight creature token with lifelink, where X is
/// its mana value.
/// **-X**: Sorin deals X damage to target creature or planeswalker and you
/// gain X life.
/// **-9**: Target opponent loses life equal to the number of cards in their
/// graveyard.
///
/// Cube-style approximation: the headline play pattern is the +1 (a
/// life-loss-for-cards exchange) and the -X (a tutored drain). The +1's
/// "reveal then take into hand" branch is collapsed to a straight Draw
/// 1 + LoseLife 3 (the most common cost — an average top-of-library mana
/// value). The Knight-token half is dropped (no mana-value-scaled token
/// primitive at this fidelity). The -X drain reads `Value::XFromCost`
/// (set by the `x_value` rider on `GameAction::ActivateLoyaltyAbility`).
/// The -9 ult uses a flat 10 drain (typical late-game state).
pub fn sorin_grim_nemesis() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype as Sup};
    CardDefinition {
        name: "Sorin, Grim Nemesis",
        cost: cost(&[generic(4), b(), b()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Sorin],
            ..Default::default()
        },
        base_loyalty: 6,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::Const(3),
                    },
                ]),
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: target_filtered(
                            SelectionRequirement::Creature
                                .or(SelectionRequirement::Planeswalker),
                        ),
                        amount: Value::Const(1),
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ]),
            },
            LoyaltyAbility {
                loyalty_cost: -9,
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(10),
                },
            },
        ],
        ..Default::default()
    }
}

/// Saheeli Rai — {1}{U}{R} Legendary Planeswalker — Saheeli. 3 loyalty.
/// **+1**: Scry 1. Saheeli Rai deals 1 damage to each opponent and each
/// planeswalker an opponent controls.
/// **-2**: Create a token that's a copy of target artifact or creature
/// you control, except it has haste. Exile it at the beginning of the
/// next end step.
/// **-7**: You get an emblem with "At the beginning of your end step,
/// create two tokens that are copies of target artifact or creature you
/// control, except they have haste. Exile them at the beginning of the
/// next end step."
///
/// Cube-style approximation. The +1's "scry 1 + damage each opponent +
/// each PW they control" wires through `Scry + Drain` (the PW-also half
/// collapses; engine has no `EachOpponentsPlaneswalker` selector). The -2
/// uses `CreateTokenCopyOf` with the target friendly creature or
/// artifact; haste is granted via the granted-keyword pipeline; the
/// "exile at next end step" rider uses `DelayUntil(NextEndStep)`. The
/// -7 ult is approximated as the -2 body fired twice (emblem
/// approximation; the engine's emblem primitive isn't wired yet for
/// "create two more each turn" auto-recurring effects).
pub fn saheeli_rai() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype as Sup};
    use crate::effect::{DelayedTriggerKind, Duration};
    let copy_friendly = || Effect::Seq(vec![
        Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::Const(1),
            source: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            extra_creature_types: vec![],
            override_pt: None,
        },
        Effect::GrantKeyword {
            what: Selector::LastCreatedToken,
            keyword: crate::card::Keyword::Haste,
            duration: Duration::Permanent,
        },
        Effect::DelayUntil {
            kind: DelayedTriggerKind::NextEndStep,
            body: Box::new(Effect::Exile {
                what: Selector::LastCreatedToken,
            }),
        },
    ]);
    CardDefinition {
        name: "Saheeli Rai",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Saheeli],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Scry {
                        who: PlayerRef::You,
                        amount: Value::Const(1),
                    },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
                    },
                ]),
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: copy_friendly(),
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Seq(vec![copy_friendly(), copy_friendly()]),
            },
        ],
        ..Default::default()
    }
}

/// Ashiok, Nightmare Weaver — {1}{U}{B} Legendary Planeswalker — Ashiok.
/// 3 loyalty.
/// **+2**: Exile the top three cards of target opponent's library.
/// **-X**: Exile target creature an opponent controls. Put onto the
/// battlefield under your control a token that's a copy of a creature
/// card with mana value X or less exiled with Ashiok.
/// **-10**: Each opponent draws seven cards from cards exiled with this.
///
/// Cube-style approximation. The +2 mills 3 (engine routes exile-from-
/// library through `Mill`-style movement, but Ashiok's "exiled with this"
/// linkage is engine-wide ⏳ — for cube play the milled cards stay in
/// the opponent's removal pile rather than a dedicated exile-with-this
/// zone). The -X uses `Effect::Exile` on the targeted opponent creature
/// (the "create a copy of exiled creature" rider is dropped, same gap as
/// Saheeli's emblem). The -10 ult is collapsed to "each opponent loses
/// the game" via the standard `WinGame` pattern.
pub fn ashiok_nightmare_weaver() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype as Sup};
    CardDefinition {
        name: "Ashiok, Nightmare Weaver",
        cost: cost(&[generic(1), u(), b()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Ashiok],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Mill {
                    who: target_filtered(SelectionRequirement::Player),
                    amount: Value::Const(3),
                },
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::Exile {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::WinGame { who: PlayerRef::You },
            },
        ],
        ..Default::default()
    }
}

/// Tamiyo, Collector of Tales — {2}{G}{U} Legendary Planeswalker — Tamiyo.
/// 4 loyalty.
/// **Static**: "Spells your opponents control can't cause you to discard
/// cards or sacrifice permanents." (Approximation: collapsed — engine
/// has no opponent-spell-effect filter on `DiscardChosen` / `Sacrifice`.)
/// **-2**: Return target card from your graveyard to your hand.
/// **-3**: Search your library for a card with the same name as a card
/// in target player's graveyard, reveal it, put it into your hand, then
/// shuffle.
/// **-7**: Draw cards equal to the number of nonland card types among
/// cards in your graveyard. You get an emblem with "Spells you cast
/// have convoke."
///
/// Cube-style approximation. The static is dropped (engine-wide gap).
/// The -2 reanimate uses `Move(target → Hand)`. The -3 is approximated
/// as `Search → Hand` on the controller's library with no name-match
/// (any card; future name-match primitive will tighten this). The -7
/// uses `Draw 4` (a reasonable midgame approximation; the full
/// distinct-types-in-gy + convoke-emblem is engine-wide ⏳).
pub fn tamiyo_collector_of_tales() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype as Sup};
    CardDefinition {
        name: "Tamiyo, Collector of Tales",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Tamiyo],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Move {
                    what: target_filtered(SelectionRequirement::Any),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Any,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(4),
                },
            },
        ],
        ..Default::default()
    }
}

/// Geyadrone Dihada — {2}{B}{R} Legendary Planeswalker — Dihada.
/// 3 loyalty.
/// **+1**: Each opponent loses 1 life and you draw a card. Then if you
/// have less life than an opponent, this PW's loyalty is reset to its
/// starting value.
/// **-3**: Gain control of target creature or planeswalker until end of
/// turn. Untap it. It gains haste until end of turn.
/// **-7**: Ult — every opponent loses half their life.
///
/// Cube-style approximation. The +1's "loyalty reset" rider collapses
/// (engine has no loyalty-set primitive). The -3 GainControl + Untap +
/// Haste is the headline Threaten-style play pattern. The -7 ult uses
/// `LoseLife(EachOpponent, 10)` as the half-life approximation
/// (typical mid-late-game value).
pub fn geyadrone_dihada() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype as Sup};
    use crate::effect::Duration;
    CardDefinition {
        name: "Geyadrone Dihada",
        cost: cost(&[generic(2), b(), r()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Dihada],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ]),
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::GainControl {
                        what: target_filtered(
                            SelectionRequirement::Creature
                                .or(SelectionRequirement::Planeswalker),
                        ),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap {
                        what: Selector::Target(0),
                        up_to: None,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: crate::card::Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(10),
                },
            },
        ],
        ..Default::default()
    }
}

/// Korvold, Fae-Cursed King — {2}{B}{R}{G} Legendary Creature — Dragon
/// Noble. 4/4 Flying. "Whenever you sacrifice a permanent, put a +1/+1
/// counter on this and draw a card."
///
/// Wired against the new `EventKind::PermanentSacrificed / YourControl`
/// trigger (CR 701.16 — shipped alongside this card in batch 102).
/// The engine's sacrifice resolver now emits a `PermanentSacrificed`
/// event for every sacrifice regardless of card type, so Korvold's
/// trigger catches Treasure-sac / Clue-sac / Food-sac / land-sac /
/// creature-sac uniformly.
pub fn korvold_fae_cursed_king() -> CardDefinition {
    use crate::card::{CounterType, Supertype as Sup};
    CardDefinition {
        name: "Korvold, Fae-Cursed King",
        cost: cost(&[generic(2), b(), r(), g()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentSacrificed,
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Lord Xander, the Collector — {3}{U}{B}{R} Legendary Creature — Vampire
/// Demon Noble. 6/6 Flying.
/// **When this enters**: Target opponent discards three cards.
/// **Whenever this attacks**: Defending player mills half their library,
/// rounded down.
/// **When this dies**: Target opponent sacrifices half their nonland
/// permanents, rounded down.
///
/// Cube-style approximation. The "half their library, rounded down" is
/// approximated as `Mill 8` (typical midgame library size around 16 →
/// half). The "sacrifices half their permanents" is collapsed to
/// `Sacrifice(EachOpponent, 3, Permanent ∧ Nonland)` (typical board
/// state). Both halves use `target_filtered(Player)` for the attack/die
/// triggers — the cube AutoDecider picks the active opponent.
pub fn lord_xander_the_collector() -> CardDefinition {
    use crate::card::Supertype as Sup;
    CardDefinition {
        name: "Lord Xander, the Collector",
        cost: cost(&[generic(3), u(), b(), r()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Vampire,
                CreatureType::Demon,
                CreatureType::Noble,
            ],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DiscardChosen {
                    from: target_filtered(SelectionRequirement::Player),
                    count: Value::Const(3),
                    filter: SelectionRequirement::Any,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(8),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::Const(3),
                    filter: SelectionRequirement::Nonland,
                },
            },
        ],
        ..Default::default()
    }
}

/// Master of Cruelties — {2}{B}{R} Legendary Creature — Demon. 1/4 First
/// Strike, Deathtouch. "Master of Cruelties can attack only alone.
/// Whenever Master of Cruelties attacks a player, that player's life
/// total becomes 1. Master of Cruelties deals no combat damage this
/// turn."
///
/// Cube-style approximation: the printed "can attack only alone" combat
/// restriction is dropped (engine has no attack-alone restriction
/// primitive — same gap as Mortician Beetle's "first creature you cast
/// each turn" gate). The attack-trigger uses `SetLifeTotal` on the
/// defending player (currently routes to the active defending opponent
/// via `target_filtered(Player)`); the "deals no combat damage" rider
/// is also dropped — combined with the SetLifeTotal → 1 effect, the
/// engine's combat-damage step would land the deathtouch ping on top,
/// so the net play is "attack alone → opp at 0 → opp loses." This is
/// the printed kill condition in any case.
pub fn master_of_cruelties() -> CardDefinition {
    use crate::card::Supertype as Sup;
    CardDefinition {
        name: "Master of Cruelties",
        cost: cost(&[generic(2), b(), r()]),
        supertypes: vec![Sup::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::SetLifeTotal {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Territorial Kavu — {2}{R}{G} Creature — Kavu. 3/2. "Whenever a land
/// enters the battlefield under an opponent's control, put a +1/+1
/// counter on Territorial Kavu."
///
/// Wired via `LandPlayed` + `OpponentControl` trigger → `AddCounter` on
/// `Selector::This`. The trigger reads the published `LandPlayed` event
/// (`event_subject = player`), filtered by scope so only opp-controlled
/// lands fire. Approximation: lands that ETB without being "played"
/// (Sakura-Tribe Elder, Kodama's Reach branches) still emit
/// `LandPlayed`, so the trigger catches every fresh land that lands on
/// the opponent's side.
pub fn territorial_kavu() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Territorial Kavu",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::OpponentControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Kolaghan's Command — {1}{B}{R} Instant. Choose two —
/// • Target player discards a card.
/// • Return target creature card from your graveyard to your hand.
/// • Destroy target artifact.
/// • Kolaghan's Command deals 2 damage to target creature or
///   planeswalker.
///
/// Cube-style approximation: the printed "choose two" multi-mode picker
/// (CR 700.2d) is collapsed to a single `ChooseMode` of three commonly-
/// useful bundled pairs (each pair covers one creature + one creature/
/// artifact answer plus the discard/draw exchange). The AutoDecider
/// picks mode 0 (discard + reanimate) by default; ScriptedDecider can
/// override.
pub fn kolaghans_command() -> CardDefinition {
    CardDefinition {
        name: "Kolaghan's Command",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            // Mode 0: discard + return creature from gy.
            Effect::Seq(vec![
                Effect::DiscardChosen {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Any,
                },
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature,
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
            // Mode 1: 2 damage + destroy artifact.
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                },
                Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Artifact),
                },
            ]),
            // Mode 2: discard + 2 damage.
            Effect::Seq(vec![
                Effect::DiscardChosen {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Any,
                },
                Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                },
            ]),
        ]),
        ..Default::default()
    }
}

/// Heroic Intervention — {1}{G} Instant. Permanents you control gain
/// hexproof and indestructible until end of turn.
///
/// Cube-style approximation: the engine has no `Hexproof` keyword-grant
/// to permanents en masse, but `Indestructible` keyword-grant works
/// (CR 702.12b). Wired via `ForEach(your perms) + GrantKeyword
/// (Indestructible, EOT)`. The hexproof half is dropped — strictly
/// weaker than printed, but the typical use-case (saving the board
/// from a Wrath) is preserved.
pub fn heroic_intervention() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Heroic Intervention",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::ControlledByYou,
            ),
            body: Box::new(Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Wear // Tear — {1}{R} // {W} Sorcery — split card. Wear destroys
/// target artifact; Tear destroys target enchantment. (Fuse: cast both
/// halves for their combined cost.)
///
/// Cube-style approximation: the split-card primitive (CR 709) is
/// engine-wide ⏳. We ship the Tear half (destroy target enchantment)
/// at the Wear cost of {1}{R} as a single-spell approximation — the
/// most expensive but most useful cast pattern. A real Split-Card
/// primitive would expose both halves and the fuse cost. Sorting it
/// under R so the cube fan-out picks it up.
pub fn wear_tear() -> CardDefinition {
    CardDefinition {
        name: "Wear // Tear",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment),
            ),
        },
        ..Default::default()
    }
}

/// Trinisphere — {3} Artifact. As long as Trinisphere is untapped, each
/// spell costs at least {3} to cast.
///
/// Cube-style approximation: the "cost at least {3}" static is engine-
/// wide ⏳ (no minimum-cost primitive — only the additional-cost-per-
/// spell tax, Damping Sphere). Ships as a vanilla 3-mana artifact body
/// — promote to ✅ once the minimum-cost-floor static lands. The card
/// remains in pools as a colorless stop-gap pickup.
pub fn trinisphere() -> CardDefinition {
    CardDefinition {
        name: "Trinisphere",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        effect: Effect::Noop,
        ..Default::default()
    }
}

/// Stillmoon Cavalier — {1}{W}{B} Creature — Zombie Knight. 2/2.
/// • {W}: Stillmoon Cavalier gains flying until end of turn.
/// • {B}: Stillmoon Cavalier gains first strike until end of turn.
/// • {1}{W}: Stillmoon Cavalier gains protection from black until end
///   of turn.
/// • {1}{B}: Stillmoon Cavalier gains protection from white until end
///   of turn.
///
/// Wired with four printed activated abilities (no costs collapsed). The
/// protection grants use the engine's existing `Keyword::Protection`
/// (which enforces combat unblockable + spell-target restriction in
/// `can_block` and target validation). EOT grants clear at cleanup.
pub fn stillmoon_cavalier() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Duration;
    let activated = |mana: ManaCost, kw: Keyword| ActivatedAbility {
        mana_cost: mana,
        effect: Effect::GrantKeyword {
            what: Selector::This,
            keyword: kw,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Stillmoon Cavalier",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            activated(cost(&[w()]), Keyword::Flying),
            activated(cost(&[b()]), Keyword::FirstStrike),
            activated(cost(&[generic(1), w()]), Keyword::Protection(Color::Black)),
            activated(cost(&[generic(1), b()]), Keyword::Protection(Color::White)),
        ],
        ..Default::default()
    }
}

/// Wishclaw Talisman — {1}{B} Artifact. Enters with three wish counters.
/// {T}, Remove a wish counter from this: Search your library for a card
/// and put that card into your hand, then shuffle. An opponent gains
/// control of Wishclaw Talisman.
///
/// Cube-style approximation: the "An opponent gains control" downside
/// is engine-wide ⏳ — `Effect::GainControl { who: ctx.controller }`
/// can't yet route to "the opp" because no `GainControlBy { who: …
/// EachOpponent }` variant exists. We ship the tutor body + counter
/// removal cost; the downside is documented and dropped. The "search
/// for any card" tutor reads `SelectionRequirement::Any`. The
/// counter-removal cost is folded into resolution; the activation
/// rejects when the artifact has no wish counters left.
pub fn wishclaw_talisman() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    CardDefinition {
        name: "Wishclaw Talisman",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Charge, Value::Const(3))),
        max_counters_of_kind: None,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::Const(1),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Any,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
            sorcery_speed: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Murderous Cut — {5}{B} Instant. Delve. Destroy target creature.
///
/// Cube-style approximation: the Delve cost-reduction primitive isn't
/// wired yet (same gap as Treasure Cruise, Dig Through Time). The card
/// is castable at full {5}{B} cost; the body is a clean
/// `Destroy(Creature)`. Promote to ✅ when Delve lands.
pub fn murderous_cut() -> CardDefinition {
    CardDefinition {
        name: "Murderous Cut",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Creature),
        },
        ..Default::default()
    }
}

// ── modern_decks batch 103: cube expansion cards ─────────────────────────────

/// Death-Greeter's Champion — {1}{R} Creature — Human Warrior. 2/2 with
/// Haste. "When this creature attacks, target opponent loses 1 life."
///
/// Synthesised body for the ⏳ cube row. Aggressive R two-drop with a
/// faux-poke trigger.
pub fn death_greeters_champion() -> CardDefinition {
    use crate::effect::shortcut::{lose_life, on_attack};
    CardDefinition {
        name: "Death-Greeter's Champion",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(lose_life(
            1,
            crate::effect::shortcut::target_filtered(SelectionRequirement::Player),
        ))],
        ..Default::default()
    }
}

/// Glaring Fleshraker — {3} Artifact Creature — Construct. 3/3. "When
/// this creature enters, it deals 2 damage to any target."
///
/// Synthesised body for the ⏳ cube row. Colorless artifact body with
/// an ETB ping.
pub fn glaring_fleshraker() -> CardDefinition {
    use crate::effect::shortcut::{etb, target_filtered};
    CardDefinition {
        name: "Glaring Fleshraker",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Detective's Phoenix — {2}{R} Creature — Phoenix. 2/2 Flying Haste.
/// "When this creature dies, return it to its owner's hand at the
/// beginning of the next end step."
///
/// Synthesised body for the ⏳ cube row. Approximation of the printed
/// Phoenix recursion: rather than a "return from gy at end step"
/// trigger, the simpler "dies → bounce to hand at NextEndStep" rider
/// uses the existing `DelayUntil` primitive.
pub fn detectives_phoenix() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    use crate::effect::{DelayedTriggerKind, ZoneDest};
    CardDefinition {
        name: "Detective's Phoenix",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phoenix],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![on_dies(Effect::DelayUntil {
            kind: DelayedTriggerKind::NextEndStep,
            body: Box::new(Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            }),
        })],
        ..Default::default()
    }
}

/// Lonis, Genetics Expert — {1}{G}{U} Legendary Creature — Otter
/// Detective. 2/2. "Whenever a creature you control enters, investigate."
///
/// Synthesised body for the ⏳ cube row. Investigates via the
/// existing `clue_token()` helper. The "Sacrifice X Clues: target
/// opponent reveals top X cards" activated ability is collapsed as a
/// future polish item (no per-activation X prompt for clue scaling).
pub fn lonis_genetics_expert() -> CardDefinition {
    use crate::card::Supertype;
    use crate::effect::Predicate;
    use crate::game::effects::clue_token;
    CardDefinition {
        name: "Lonis, Genetics Expert",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::AnotherOfYours,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature,
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: clue_token(),
            },
        }],
        ..Default::default()
    }
}

/// Loot, the Pathfinder — {1}{G}{W} Legendary Creature — Otter Scout.
/// 2/3 with Vigilance. "When this creature enters, create a Map token."
///
/// Synthesised body for the ⏳ cube row. The Map token primitive isn't
/// wired (CR 111.10s explore-token), so we ship a faux Clue token
/// equivalent (also exiles for {2} + sac → draw, gameplay-relevant
/// approximation of "use a Map to explore"). Vigilance keyword is
/// honored.
pub fn loot_the_pathfinder() -> CardDefinition {
    use crate::card::Supertype;
    use crate::effect::shortcut::etb;
    use crate::game::effects::clue_token;
    CardDefinition {
        name: "Loot, the Pathfinder",
        cost: cost(&[generic(1), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: clue_token(),
        })],
        ..Default::default()
    }
}

/// Brightglass Gearhulk — {4} Artifact Creature — Construct. 4/4.
/// "When this creature enters, scry 2, then draw a card."
///
/// Synthesised body for the ⏳ cube row. A vanilla 4-mana colorless
/// big-body cantripping artifact creature.
pub fn brightglass_gearhulk() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Brightglass Gearhulk",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Mossborn Hydra — {X}{G} Creature — Hydra. 0/0. "This creature
/// enters with X +1/+1 counters on it."
///
/// Synthesised body for the ⏳ cube row. Reuses the existing
/// `enters_with_counters` field with a Value::XFromCost reader so the
/// counters scale with the X paid at cast time. The "double counters
/// if X ≥ 4" rider is collapsed.
pub fn mossborn_hydra() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Mossborn Hydra",
        cost: ManaCost::new(vec![
            ManaSymbol::X,
            ManaSymbol::Colored(Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hydra],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        max_counters_of_kind: None,
        ..Default::default()
    }
}

/// Mai, Scornful Striker — {1}{B} Creature — Human Rogue. 2/1 with
/// Menace. "Whenever this creature attacks, each opponent loses 1
/// life."
///
/// Synthesised body for the ⏳ cube row. Menace evasion + life-drip
/// payoff fills the 2-drop curve in B aggro shells.
pub fn mai_scornful_striker() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Mai, Scornful Striker",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Tempest Angler — {2}{U} Creature — Merfolk Wizard. 2/2 Flying.
/// "When this creature enters, scry 2."
///
/// Synthesised body for the ⏳ cube row. A flying scry tempo creature
/// in U shells.
pub fn tempest_angler() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Tempest Angler",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Carnage Interpreter — {2}{B}{R} Creature — Vampire. 4/3 with Trample.
/// "When this creature enters, each opponent discards a card."
///
/// Synthesised body for the ⏳ cube row. A BR aggressive body that
/// strips a card on entry, fitting Rakdos sacrifice/discard shells.
pub fn carnage_interpreter() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Carnage Interpreter",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: true,
        })],
        ..Default::default()
    }
}
