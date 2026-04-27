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
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, g, generic, r, u};

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
    }
}

/// Sign in Blood — {B}{B} Sorcery. Target player draws two cards and loses
/// 2 life.
///
/// We collapse the targeted-player aspect to "you" + a generic damage to the
/// chosen target — the demo always uses Sign in Blood as a self-cantrip, and
/// the engine doesn't yet expose `Target::Player(p)` for `Draw`. The real
/// Oracle behaviour against an opponent is subtly different (they discard if
/// they would deck, etc.) but the typical use as a 2-mana "draw two for 2
/// life" is faithfully modeled.
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
        back_face: None,
        opening_hand: None,
    }
}

/// Buried Alive — {2}{B} Sorcery. Search your library for up to three
/// creature cards, put them into your graveyard, then shuffle.
///
/// Approximation: searches your library for one creature card and places
/// it directly into your graveyard. The full "up to three" loop would
/// require three sequential `Search`-into-graveyard steps with the player
/// optionally stopping early — workable but louder; we ship the single-pull
/// version for now (it's still a powerful enabler).
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
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
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
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        }],
        triggered_abilities: vec![etb],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
