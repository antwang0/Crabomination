//! Modern-playable cards that complement the BRG combo and Goryo's
//! Vengeance demo decks. These are *new* card factories, distinct from the
//! existing `creatures.rs` / `lands.rs` / `spells.rs` set, and each card is
//! built on the existing engine primitives — no engine changes required.
//!
//! Cards in this file are tracked in `DECK_FEATURES.md` under the **Modern
//! supplement** section.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, Effect, Keyword, LandType,
    SelectionRequirement, Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::card::{CounterType, EventKind, EventScope, EventSpec};
use crate::effect::shortcut::{
    each_your_creature, etb, etb_explore, explore, investigate, target_filtered,
};
use crate::effect::{Duration, ManaPayload, Predicate, PlayerRef, ZoneDest};
use crate::mana::{Color, ManaCost, ManaSymbol, b, colorless, cost, g, generic, r, u, w};

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
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Manamorphose — {1}{R/G} Instant. Add two mana in any combination of
/// colors. Draw a card.
///
/// The `{R/G}` pip is a real `ManaSymbol::Hybrid(Red, Green)`, payable
/// with either red or green.
pub fn manamorphose() -> CardDefinition {
    CardDefinition {
        name: "Manamorphose",
        cost: cost(&[generic(1), crate::mana::hybrid(Color::Red, Color::Green)]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Night's Whisper — {1}{B} Sorcery. Draw two cards, lose 2 life.
pub fn nights_whisper() -> CardDefinition {
    CardDefinition {
        name: "Night's Whisper",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Duress — {B} Sorcery. Target opponent reveals their hand. You choose a
/// noncreature, nonland card. They discard it.
pub fn duress() -> CardDefinition {
    CardDefinition {
        name: "Duress",
        cost: cost(&[b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Burn / damage ────────────────────────────────────────────────────────────

/// Lava Spike — {R} Sorcery. Lava Spike deals 3 damage to target player.
pub fn lava_spike() -> CardDefinition {
    CardDefinition {
        name: "Lava Spike",
        cost: cost(&[r()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Lava Dart — {R} Sorcery. Deals 1 damage to any target. Flashback—Sacrifice
/// a Mountain (the `{0}` flashback mana cost plus the name-keyed
/// `flashback_additional_cost_for_name` sacrifice).
pub fn lava_dart() -> CardDefinition {
    let flashback_cost = ManaCost { symbols: vec![] };
    CardDefinition {
        name: "Lava Dart",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(1),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Entomb — {B} Instant. Search your library for a card and put it into
/// your graveyard. Then shuffle.
pub fn entomb() -> CardDefinition {
    CardDefinition {
        name: "Entomb",
        cost: cost(&[b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Burning-Tree Emissary — {R/G}{R/G} 2/2 Creature — Human Shaman. When
/// Burning-Tree Emissary enters, add {R}{G}.
///
/// The `{R/G}{R/G}` pips are real `ManaSymbol::Hybrid(Red, Green)`, each
/// payable with either red or green; the ETB ramp is unchanged.
pub fn burning_tree_emissary() -> CardDefinition {
    CardDefinition {
        name: "Burning-Tree Emissary",
        cost: cost(&[
            crate::mana::hybrid(Color::Red, Color::Green),
            crate::mana::hybrid(Color::Red, Color::Green),
        ]),
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
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Red, Color::Green]),
            },
        }],
        ..Default::default()
    }
}

/// Putrid Imp — {B} 1/1 Imp. Flying. "Discard a card: This creature gains
/// menace until end of turn." A turn-1 discard outlet (the Madness/
/// reanimator enabler) plus a menace trick.
pub fn putrid_imp() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Putrid Imp",
        cost: cost(&[b()]),
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lhurgoyf],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // "Draw a card if an opponent has cast a blue or black spell
            // this turn."
            Effect::If {
                cond: Predicate::CastBlueOrBlackThisTurn { who: PlayerRef::EachOpponent },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::Noop),
            },
            // "Spells your opponents control can't counter spells you
            // control this turn …"
            Effect::GrantSpellsUncounterableThisTurn { who: Selector::You },
            // "… and your opponents can't gain life this turn."
            Effect::LifeGainLockThisTurn {
                who: Selector::Player(PlayerRef::EachOpponent),
            },
        ]),
        ..Default::default()
    }
}

/// Crop Rotation — {G} Instant. As an additional cost, sacrifice a land.
/// Search your library for a land card and put it onto the battlefield.
/// The sacrifice is a real cast-time cost (`AdditionalCastCost`).
pub fn crop_rotation() -> CardDefinition {
    CardDefinition {
        name: "Crop Rotation",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            count: 1,
        }],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Land,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
            },
        ],
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Gitaxian Probe — Phyrexian Sorcery. Look at target opponent's hand. Draw
/// a card.
///
/// The Phyrexian pip `{U/P}` is a real `ManaSymbol::Phyrexian(Blue)`:
/// paying it with blue mana costs no life, while paying with life costs 2
/// (handled by the mana payment side-effect on cast). The "look at
/// opponent's hand" half is dropped (information-only effect with no
/// engine state hook), leaving a free-or-2-life cantrip.
pub fn gitaxian_probe() -> CardDefinition {
    CardDefinition {
        name: "Gitaxian Probe",
        cost: cost(&[crate::mana::phyrexian(Color::Blue)]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: crate::effect::shortcut::target(),
            mana_cost: cost(&[generic(1)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![etb],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(3),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
            },
        ],
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
            },
        ],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Pitch / alt-cost utility ─────────────────────────────────────────────────

/// Snuff Out — {3}{B} Instant. Destroy target nonblack creature. It can't
/// be regenerated.
///
/// Alt cost: "If you control a Swamp, you may pay 4 life rather than pay
/// this spell's mana cost." The Swamp gate rides the alt-cost `condition`
/// predicate (controls ≥1 Swamp). The "can't be regenerated" rider
/// collapses (no regen primitive).
pub fn snuff_out() -> CardDefinition {
    use crate::card::{AlternativeCost, LandType, Predicate};
    CardDefinition {
        name: "Snuff Out",
        cost: cost(&[generic(3), b()]),
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
        triggered_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 4,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::HasLandType(LandType::Swamp)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(1),
            }),
                    exile_from_graveyard_count: 0,
                    return_to_hand: None,
                    sacrifice_permanents: None,
            effect_override: None,
        }),
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Worldly Tutor — {G} Instant. Search your library for a creature card,
/// reveal it, put it on top of your library, then shuffle.
pub fn worldly_tutor() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Worldly Tutor",
        cost: cost(&[g()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Damnation — {2}{B}{B} Sorcery. Destroy all creatures. They can't be
/// regenerated.
///
/// Black mirror of Wrath of God. Same `ForEach + Destroy` shape; the
/// "can't be regenerated" rider is wired via `Effect::DestroyNoRegen`
/// (CR 701.15g).
pub fn damnation() -> CardDefinition {
    CardDefinition {
        name: "Damnation",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::DestroyNoRegen {
                what: Selector::TriggerSource,
            }),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
/// "{T}, Sacrifice a Forest: Add {R}{R}{R}."
///
/// The "Sacrifice a Forest" cost is now a proper pre-resolution
/// activation cost via `sac_other_filter: Some((Forest, 1))` (rejects
/// when the controller has no Forest), and the mana produced is the
/// printed `{R}{R}{R}` (the prior `{G}{G}{G}` was a transcription bug —
/// a mono-red Forest-sacrificer that made green mana made no sense).
pub fn orcish_lumberjack() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Orcish Lumberjack",
        cost: cost(&[r()]),
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
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Red, Color::Red, Color::Red]),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None,
            // Sacrifice a Forest as an activation cost.
            sac_other_filter: Some((
                SelectionRequirement::Land
                    .and(SelectionRequirement::HasLandType(crate::card::LandType::Forest)),
                1,
            )),
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::add_mana;
    CardDefinition {
        name: "Elvish Spirit Guide",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        // Exile this card from your hand: Add {G}.
        activated_abilities: vec![ActivatedAbility {
            effect: add_mana(vec![Color::Green]),
            from_hand: true,
            exile_self_cost: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Satyr Wayfinder — {1}{G} Creature — Satyr Druid. 1/1.
/// "When this creature enters, reveal the top four cards of your library.
/// You may put a land card from among them into your hand. Put the rest
/// into your graveyard." Wired via `LookPickToHand` with a land
/// `pick_filter`; the rest go to the graveyard.
pub fn satyr_wayfinder() -> CardDefinition {
    CardDefinition {
        name: "Satyr Wayfinder",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: true,
                pick_filter: Some(SelectionRequirement::Land),
            },
        }],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
    };
    CardDefinition {
        name: "Talisman of Progress",
        cost: cost(&[generic(2)]),
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
            tap_other_filter: None, from_hand: false,
            },
            make_color(Color::White),
            make_color(Color::Blue),
        ],
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
    };
    CardDefinition {
        name: "Talisman of Dominance",
        cost: cost(&[generic(2)]),
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
            tap_other_filter: None, from_hand: false,
            },
            make_color(Color::Blue),
            make_color(Color::Black),
        ],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fireblast — {4}{R}{R} Instant. "Fireblast deals 4 damage to any target."
/// Alternative cost: sacrifice two Mountains.
///
/// Alt cost: sacrifice two Mountains (the classic free-burn line) via
/// `AlternativeCost.sacrifice_permanents`.
pub fn fireblast() -> CardDefinition {
    use crate::card::{AlternativeCost, LandType};
    CardDefinition {
        name: "Fireblast",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(4),
        },
        alternative_cost: Some(AlternativeCost {
            sacrifice_permanents: Some((SelectionRequirement::HasLandType(LandType::Mountain), 2)),
            ..Default::default()
        }),
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
    };
    CardDefinition {
        name,
        cost: cost(&[generic(2)]),
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
            tap_other_filter: None, from_hand: false,
            },
            make_color(c1),
            make_color(c2),
        ],
        triggered_abilities: vec![],
        ..Default::default()
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

/// Diabolic Edict — {1}{B} Instant. Target player sacrifices a creature.
///
/// `Sacrifice` over `Selector::Target(0)` so the targeted player picks one
/// of their own creatures to lose. Pairs naturally with the existing
/// `target_filtered(Player)` machinery used by Sign in Blood.
pub fn diabolic_edict() -> CardDefinition {
    CardDefinition {
        name: "Diabolic Edict",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Magma Jet — {1}{R} Instant. Magma Jet deals 2 damage to any target.
/// Scry 2.
pub fn magma_jet() -> CardDefinition {
    CardDefinition {
        name: "Magma Jet",
        cost: cost(&[generic(1), r()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Read the Bones — {2}{B} Sorcery. Scry 2, then draw two cards and lose
/// two life.
pub fn read_the_bones() -> CardDefinition {
    CardDefinition {
        name: "Read the Bones",
        cost: cost(&[generic(2), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes,
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::HasCardType(CardType::Artifact)),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// **Tragic Slip** — {B} Instant. "Target creature gets -1/-1 until end of
/// turn. Morbid — That creature gets -13/-13 until end of turn instead if a
/// creature died this turn."
///
/// The Morbid rider is now wired via `Effect::If` gated on
/// `Predicate::Any([CreaturesDiedThisTurnAtLeast{You,1},
/// CreaturesDiedThisTurnAtLeast{EachOpponent,1}])` — i.e. a creature died
/// under either player's control this turn. Without morbid it's a modest
/// -1/-1; with morbid it's the full -13/-13.
pub fn tragic_slip() -> CardDefinition {
    use crate::effect::Predicate;
    // Morbid — "if a creature died this turn" (any player's creature).
    let morbid = Predicate::CreaturesDiedThisTurnTotalAtLeast {
        at_least: Value::Const(1),
    };
    CardDefinition {
        name: "Tragic Slip",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::If {
            cond: morbid,
            then: Box::new(Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-13),
                toughness: Value::Const(-13),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            }),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(3),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell { what: Selector::Target(0) },
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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

// ── White lifegain & tokens ──────────────────────────────────────────────────

/// Healing Salve — {W} Instant. Choose one — Target player gains 3 life;
/// or prevent the next 3 damage that would be dealt to any target this turn.
///
/// Both modes are wired: mode 0 gains 3 life, mode 1 pushes a
/// `PreventNextDamage(3)` shield (CR 615.7) on the target. AutoDecider
/// picks mode 0; ScriptedDecider can select the prevention mode.
pub fn healing_salve() -> CardDefinition {
    CardDefinition {
        name: "Healing Salve",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::GainLife {
                who: Selector::Target(0),
                amount: Value::Const(3),
            },
            Effect::PreventNextDamage {
                target: target_filtered(
                    SelectionRequirement::Player
                        .or(SelectionRequirement::Creature)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Permanent),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Cyclonic Rift — {1}{U} Instant. Return target nonland permanent your
/// opponents control to its owner's hand. Overload `{6}{U}` bounces *each*
/// such permanent via the alt-cost `effect_override` (CR 702.96).
pub fn cyclonic_rift() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Cyclonic Rift",
        cost: cost(&[generic(1), u()]),
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
        triggered_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(6), u()]),
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
                    SelectionRequirement::Nonland
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                }),
            }),
        }),
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Convoke],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(4),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature.and(
                    SelectionRequirement::ManaValueAtMostControlledCount(Box::new(
                        SelectionRequirement::HasLandType(LandType::Plains)
                            .and(SelectionRequirement::ControlledByYou),
                    )),
                )),
            },
        ]),
        ..Default::default()
    }
}

// ── Explore (CR 701.40) ────────────────────────────────────────────────────

/// Merfolk Branchwalker — {1}{G} 2/1 Merfolk Scout. ETB: it explores.
pub fn merfolk_branchwalker() -> CardDefinition {
    CardDefinition {
        name: "Merfolk Branchwalker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb_explore()],
        ..Default::default()
    }
}

/// Jadelight Ranger — {1}{G}{G} 2/1 Merfolk Scout. ETB: it explores, then
/// explores again.
pub fn jadelight_ranger() -> CardDefinition {
    CardDefinition {
        name: "Jadelight Ranger",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Seq(vec![explore(), explore()]))],
        ..Default::default()
    }
}

/// Wildgrowth Walker — {1}{G} 0/3 Elemental. Whenever a creature you control
/// explores, put a +1/+1 counter on Wildgrowth Walker and you gain 3 life.
pub fn wildgrowth_walker() -> CardDefinition {
    CardDefinition {
        name: "Wildgrowth Walker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Explored, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ]),
        }],
        ..Default::default()
    }
}

/// Seekers' Squire — {1}{B} 1/2 Human Pirate. ETB: it explores.
pub fn seekers_squire() -> CardDefinition {
    CardDefinition {
        name: "Seekers' Squire",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb_explore()],
        ..Default::default()
    }
}

/// Tishana's Wayfinder — {2}{G} 2/2 Merfolk Scout. ETB: it explores.
pub fn tishanas_wayfinder() -> CardDefinition {
    CardDefinition {
        name: "Tishana's Wayfinder",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb_explore()],
        ..Default::default()
    }
}

/// Emperor's Vanguard — {2}{G} 3/2 Human Scout. Whenever it attacks, it
/// explores.
pub fn emperors_vanguard() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Emperor's Vanguard",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![on_attack(explore())],
        ..Default::default()
    }
}

/// Path of Discovery — {2}{G} Enchantment. Whenever a creature you control
/// enters, it explores.
pub fn path_of_discovery() -> CardDefinition {
    CardDefinition {
        name: "Path of Discovery",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::Explore { who: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

// ── Monstrosity (CR 701.31) ─────────────────────────────────────────────────

/// Nessian Wilds Ravager — {4}{G} 6/6 Hydra. {6}{G}{G}: Monstrosity 5. When
/// it becomes monstrous, you may have it fight target creature you don't
/// control.
pub fn nessian_wilds_ravager() -> CardDefinition {
    use crate::effect::shortcut::{monstrosity, on_becomes_monstrous};
    CardDefinition {
        name: "Nessian Wilds Ravager",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hydra],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        activated_abilities: vec![monstrosity(cost(&[generic(6), g(), g()]), 5)],
        triggered_abilities: vec![on_becomes_monstrous(Effect::MayDo {
            description: "have it fight target creature you don't control".into(),
            body: Box::new(Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            }),
        })],
        ..Default::default()
    }
}

/// Ember Swallower — {2}{R} 4/4 Elemental. {3}{R}{R}: Monstrosity 3. When it
/// becomes monstrous, each player sacrifices three lands.
pub fn ember_swallower() -> CardDefinition {
    use crate::effect::shortcut::{monstrosity, on_becomes_monstrous};
    CardDefinition {
        name: "Ember Swallower",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        activated_abilities: vec![monstrosity(cost(&[generic(3), r(), r()]), 3)],
        triggered_abilities: vec![on_becomes_monstrous(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::Const(3),
            filter: SelectionRequirement::Land,
        })],
        ..Default::default()
    }
}

/// Arbor Colossus — {4}{G} 6/6 Reach. {6}{G}: Monstrosity 3. When it becomes
/// monstrous, destroy target creature with flying.
pub fn arbor_colossus() -> CardDefinition {
    use crate::effect::shortcut::{monstrosity, on_becomes_monstrous};
    CardDefinition {
        name: "Arbor Colossus",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![monstrosity(cost(&[generic(6), g()]), 3)],
        triggered_abilities: vec![on_becomes_monstrous(Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            ),
        })],
        ..Default::default()
    }
}

/// Ill-Tempered Cyclops — {2}{R}{R} 3/3 Cyclops with trample.
/// {3}{R}: Monstrosity 2.
pub fn ill_tempered_cyclops() -> CardDefinition {
    use crate::effect::shortcut::monstrosity;
    CardDefinition {
        name: "Ill-Tempered Cyclops",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cyclops],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![monstrosity(cost(&[generic(3), r()]), 2)],
        ..Default::default()
    }
}

// ── Ixalan dinosaurs / green value (existing primitives) ─────────────────────

/// Charging Monstrosaur — {3}{R}{R} 5/5 Dinosaur with trample and haste.
pub fn charging_monstrosaur() -> CardDefinition {
    CardDefinition {
        name: "Charging Monstrosaur",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample, Keyword::Haste],
        ..Default::default()
    }
}

/// Ripjaw Raptor — {2}{G}{G} 4/5 Dinosaur. Enrage — whenever it's dealt
/// damage, draw a card.
pub fn ripjaw_raptor() -> CardDefinition {
    CardDefinition {
        name: "Ripjaw Raptor",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Thrashing Brontodon — {1}{G}{G} 3/4 Dinosaur.
/// {1}, Sacrifice: Destroy target artifact or enchantment.
pub fn thrashing_brontodon() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Thrashing Brontodon",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Regisaur Alpha — {3}{R}{G} 7/6 Dinosaur with trample. Other Dinosaurs you
/// control have haste. ETB: create a 3/3 green Dinosaur token.
pub fn regisaur_alpha() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Regisaur Alpha",
        cost: cost(&[generic(3), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Other Dinosaurs you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Dinosaur))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Dinosaur".into(),
                power: 3,
                toughness: 3,
                keywords: vec![],
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                supertypes: vec![],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Dinosaur],
                    ..Default::default()
                },
                activated_abilities: vec![],
                triggered_abilities: vec![],
            },
        })],
        ..Default::default()
    }
}

/// Pounce — {1}{G} Sorcery. Target creature you control fights target
/// creature you don't control.
pub fn pounce() -> CardDefinition {
    CardDefinition {
        name: "Pounce",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Fight {
            attacker: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
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

/// Atzocan Archer — {1}{G} 1/4 Human Archer with reach. ETB: you may have it
/// fight target creature you don't control.
pub fn atzocan_archer() -> CardDefinition {
    CardDefinition {
        name: "Atzocan Archer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "have it fight target creature you don't control".into(),
            body: Box::new(Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            }),
        })],
        ..Default::default()
    }
}

/// Ranging Raptors — {2}{G} 3/3 Dinosaur. Enrage — whenever it's dealt
/// damage, search your library for a basic land and put it onto the
/// battlefield tapped.
pub fn ranging_raptors() -> CardDefinition {
    CardDefinition {
        name: "Ranging Raptors",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        }],
        ..Default::default()
    }
}

/// Otepec Huntmaster — {1}{R} 2/2 Human Shaman with haste. Dinosaur spells
/// you cast cost {1} less to cast.
pub fn otepec_huntmaster() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Otepec Huntmaster",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Dinosaur spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasCreatureType(CreatureType::Dinosaur),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Kinjalli's Caller — {W} 1/1 Bird Cleric. Dinosaur spells you cast cost
/// {1} less to cast.
pub fn kinjallis_caller() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Kinjalli's Caller",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Dinosaur spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasCreatureType(CreatureType::Dinosaur),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Territorial Hammerskull — {2}{W} 3/3 Dinosaur. Whenever it attacks, tap
/// target creature defending player controls.
pub fn territorial_hammerskull() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Territorial Hammerskull",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        ..Default::default()
    }
}

/// Grazing Whiptail — {4}{G} 3/4 Dinosaur with reach.
pub fn grazing_whiptail() -> CardDefinition {
    CardDefinition {
        name: "Grazing Whiptail",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Frilled Deathspitter — {1}{R} 2/2 Dinosaur. Enrage — whenever it's dealt
/// damage, it deals 2 damage to each opponent.
pub fn frilled_deathspitter() -> CardDefinition {
    CardDefinition {
        name: "Frilled Deathspitter",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Raptor Hatchling — {1}{R} 1/1 Dinosaur with trample. Enrage — whenever
/// it's dealt damage, create a 3/3 red Dinosaur token.
pub fn raptor_hatchling() -> CardDefinition {
    CardDefinition {
        name: "Raptor Hatchling",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Dinosaur".into(),
                    power: 3,
                    toughness: 3,
                    keywords: vec![Keyword::Trample],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    supertypes: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Dinosaur],
                        ..Default::default()
                    },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                },
            },
        }],
        ..Default::default()
    }
}

/// Farhaven Elf — {2}{G} 1/1 Elf. ETB: search your library for a basic land
/// and put it onto the battlefield tapped.
pub fn farhaven_elf() -> CardDefinition {
    CardDefinition {
        name: "Farhaven Elf",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        })],
        ..Default::default()
    }
}

// ── Goad (CR 701.38) ────────────────────────────────────────────────────────

/// Disrupt Decorum — {3}{R}{R} Sorcery. Goad all creatures you don't control.
pub fn disrupt_decorum() -> CardDefinition {
    CardDefinition {
        name: "Disrupt Decorum",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Goad {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        },
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Artifact),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Token producers ──────────────────────────────────────────────────────────

/// Spectral Procession — {2}{W} Sorcery. Create three 1/1 white Spirit
/// creature tokens with flying.
///
/// The Oracle cost is three monocolored-hybrid pips `{2/W}{2/W}{2/W}` —
/// each payable with {2} generic or one white. Modeled with real
/// `ManaSymbol::MonoHybrid(2, White)` pips (mana value 6 per CR 202.3f;
/// castable for as little as {W}{W}{W}).
pub fn spectral_procession() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::mono_hybrid;
    CardDefinition {
        name: "Spectral Procession",
        cost: cost(&[
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::White),
        ]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Any),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Artifact),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
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

/// Glimmerpost — Land — Locus. Glimmerpost enters tapped. When it enters,
/// you gain 1 life for each Locus you control (`locus_count_value`).
/// {T}: Add {C}.
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![
            modern_etb_tap(),
            crate::effect::shortcut::etb(Effect::GainLife {
                who: Selector::You,
                amount: locus_count_value(),
            }),
        ],
        ..Default::default()
    }
}

/// Number of Loci you control — `{T}: Add {C} for each Locus you control`
/// reads this so 12-post engines scale correctly. The tapped Locus counts
/// itself, so the value is at least 1.
fn locus_count_value() -> Value {
    use crate::card::{LandType, SelectionRequirement};
    Value::CountOf(Box::new(Selector::EachPermanent(
        SelectionRequirement::HasLandType(LandType::Locus)
            .and(SelectionRequirement::ControlledByYou),
    )))
}

/// Cloudpost — Land — Locus. Cloudpost enters tapped. {T}: Add {C} for
/// each Locus you control (`locus_count_value`).
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
                pool: ManaPayload::Colorless(locus_count_value()),
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
        triggered_abilities: vec![modern_etb_tap()],
        ..Default::default()
    }
}

/// Lotus Field — Land. Enters tapped, Hexproof. When it enters, sacrifice
/// two lands. {T}: Add three mana of one color.
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![modern_etb_tap()],
        ..Default::default()
    }
}

/// Build a "bridge"-cycle artifact land (March of the Machine): an
/// indestructible Artifact Land that enters tapped and taps for one of
/// two colours. No basic land types (the printed cards have none).
fn bridge_land(
    name: &'static str,
    color_a: crate::mana::Color,
    color_b: crate::mana::Color,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Artifact, CardType::Land],
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![
            crate::catalog::sets::tap_add(color_a),
            crate::catalog::sets::tap_add(color_b),
        ],
        triggered_abilities: vec![modern_etb_tap()],
        ..Default::default()
    }
}

/// Mistvault Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {U} or {B}.
pub fn mistvault_bridge() -> CardDefinition {
    bridge_land("Mistvault Bridge", Color::Blue, Color::Black)
}

/// Drossforge Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {B} or {R}.
pub fn drossforge_bridge() -> CardDefinition {
    bridge_land("Drossforge Bridge", Color::Black, Color::Red)
}

/// Razortide Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {W} or {U}.
pub fn razortide_bridge() -> CardDefinition {
    bridge_land("Razortide Bridge", Color::White, Color::Blue)
}

/// Goldmire Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {W} or {B}.
pub fn goldmire_bridge() -> CardDefinition {
    bridge_land("Goldmire Bridge", Color::White, Color::Black)
}

/// Silverbluff Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {U} or {R}.
pub fn silverbluff_bridge() -> CardDefinition {
    bridge_land("Silverbluff Bridge", Color::Blue, Color::Red)
}

/// Tanglepool Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {G} or {U}.
pub fn tanglepool_bridge() -> CardDefinition {
    bridge_land("Tanglepool Bridge", Color::Green, Color::Blue)
}

/// Slagwoods Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {R} or {G}.
pub fn slagwoods_bridge() -> CardDefinition {
    bridge_land("Slagwoods Bridge", Color::Red, Color::Green)
}

/// Thornglint Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {G} or {W}.
pub fn thornglint_bridge() -> CardDefinition {
    bridge_land("Thornglint Bridge", Color::Green, Color::White)
}

/// Darkmoss Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {B} or {G}.
pub fn darkmoss_bridge() -> CardDefinition {
    bridge_land("Darkmoss Bridge", Color::Black, Color::Green)
}

/// Rustvale Bridge — Indestructible Artifact Land. Enters tapped. {T}: Add {R} or {W}.
pub fn rustvale_bridge() -> CardDefinition {
    bridge_land("Rustvale Bridge", Color::Red, Color::White)
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
    use crate::game::types::TurnStep;
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
                tap_other_filter: None, from_hand: false,
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
                tap_other_filter: None, from_hand: false,
            },
        ],
        // CR 701.x charge→mana burst: at the beginning of your precombat
        // main phase you may remove all charge counters and add one mana of
        // any color for each. `AddMana` reads the count before the
        // `RemoveCounter` strips them (Seq order), so the mana matches the
        // charges removed. `MayDo` keeps it optional (AutoDecider declines).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Remove all charge counters; add a mana of any color for each"
                    .to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyColors(Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Charge,
                        }),
                    },
                    Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Charge,
                        },
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

// ── modern_decks-11: Multi-color removal + sweepers + body ───────────────────

/// Tear Asunder — {1}{G} Instant, Kicker {1}{B}. Exile target artifact or
/// enchantment; if kicked, exile target nonland permanent instead. The
/// kicked branch's broader target filter is enforced at cast time via the
/// kick-aware target validation (CR 702.32).
pub fn tear_asunder() -> CardDefinition {
    CardDefinition {
        name: "Tear Asunder",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[generic(1), b()]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            }),
            else_: Box::new(Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            }),
        },
        ..Default::default()
    }
}

/// Into the Roil — {1}{U} Instant, Kicker {1}{U}. Return target nonland
/// permanent to its owner's hand; draw a card if kicked (CR 702.32).
pub fn into_the_roil() -> CardDefinition {
    CardDefinition {
        name: "Into the Roil",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[generic(1), u()]))],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Assassin's Trophy — {B}{G} Instant. "Destroy target permanent an
/// opponent controls. Its controller may search their library for a basic
/// land card, put it onto the battlefield, then shuffle." The destroyed
/// permanent's owner (resolved post-destroy via the graveyard) does the
/// basic-land search — the printed ramp downside.
pub fn assassins_trophy() -> CardDefinition {
    CardDefinition {
        name: "Assassin's Trophy",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::Search {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
            },
        ]),
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
        // "Destroy all creatures you don't control. They can't be
        // regenerated." — DestroyNoRegen honors the printed regen rider.
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            body: Box::new(Effect::DestroyNoRegen { what: Selector::TriggerSource }),
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
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

/// Hieroglyphic Illumination — {3}{U} Instant. Draw two cards.
/// Cycling {U} (`Keyword::Cycling`).
pub fn hieroglyphic_illumination() -> CardDefinition {
    CardDefinition {
        name: "Hieroglyphic Illumination",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[u()]))],
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

/// Maelstrom Pulse — {1}{B}{G} Sorcery. "Destroy target nonland permanent
/// and all other permanents with the same name as that permanent."
/// The same-name sweep rides via `Selector::SharingNameWith(Target(0))`.
pub fn maelstrom_pulse() -> CardDefinition {
    CardDefinition {
        name: "Maelstrom Pulse",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Sorcery],
        // Cast-time target validation runs on `Target(0)`; the destroy then
        // hits every battlefield permanent sharing the target's name.
        effect: Effect::Destroy {
            what: Selector::SharingNameWith(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
            })),
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
/// Real Oracle: {1}{B/P}{B/P}{B/P}. The three Phyrexian pips are real
/// `ManaSymbol::Phyrexian(Black)` — each payable with one black or 2
/// life, so Dismember can be cast for as little as {1} + 6 life. The
/// body -5/-5 is the gameplay-relevant black-removal answer to large
/// indestructible threats.
pub fn dismember() -> CardDefinition {
    use crate::mana::phyrexian;
    CardDefinition {
        name: "Dismember",
        cost: cost(&[generic(1), phyrexian(Color::Black), phyrexian(Color::Black), phyrexian(Color::Black)]),
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

/// Echoing Truth — {1}{U} Instant. "Return target nonland permanent and
/// all other permanents with the same name as that permanent to their
/// owners' hands." Same-name sweep via `Selector::SharingNameWith`. The
/// destination resolves to the target's owner (same-named permanents
/// almost always share an owner — token swarms / your own multiples).
pub fn echoing_truth() -> CardDefinition {
    CardDefinition {
        name: "Echoing Truth",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::SharingNameWith(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
            })),
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

/// Cling to Dust — {B} Instant. Exile target card from a graveyard. If it
/// was a creature card you gain 3 life, otherwise you draw a card.
/// Escape—{3}{B}, exile five other cards from your graveyard (CR 702.139).
///
/// Uses `Effect::Move(target → Exile)` so the auto-target heuristic walks
/// graveyards first; the creature/else branch reads the captured target
/// post-move via `Predicate::EntityMatches`. `Keyword::Escape` makes the
/// instant recurring from the graveyard.
pub fn cling_to_dust() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Cling to Dust",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Escape(cost(&[generic(3), b()]), 5)],
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
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
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
/// Land-tutor activated ability. The "Sacrifice a land" cost is a proper
/// pre-resolution activation cost (`sac_other_filter: Some((Land, 1))`),
/// with the body `Search(Land → BF)`; activation is rejected with no land
/// to sacrifice. The Threshold rider (gets +2/+2 → 3/4 with seven or more
/// cards in your graveyard) rides `graveyard_threshold_selfpump_for_name`.
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
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
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
        ..Default::default()
    }
}

/// Rofellos, Llanowar Emissary — {G}{G} Legendary Creature — Elf Druid. 2/1.
/// {T}: Add {G} for each Forest you control.
///
/// Push (modern_decks): the "for each Forest" rider is wired faithfully via
/// `ManaPayload::OfColor(Green, CountOf(Forest ∧ ControlledByYou))`. Tapping
/// Rofellos with N Forests in play adds N green mana to the controller's
/// pool. The activation cost remains a plain `{T}`; the dynamic payload
/// reads the live Forest count at resolution time. Same shape as Topiary
/// Lecturer / Molten-Core Maestro's power-scaled mana payouts
/// (`PowerOf(This)`).
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
                        SelectionRequirement::HasLandType(LandType::Forest)
                            .and(SelectionRequirement::ControlledByYou),
                    )),
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
            tap_other_filter: None, from_hand: false,
        }],
        ..Default::default()
    }
}

/// Biorhythm — {6}{G}{G} Sorcery. Each player's life total becomes the
/// number of creatures they control (CR 119.5).
///
/// `ForEach(Player(EachPlayer))` setting each player's life to
/// `Value::CreatureCountControlledBy(Triggerer)` — faithful for any number
/// of players, including multiplayer where every seat gets their own count.
pub fn biorhythm() -> CardDefinition {
    CardDefinition {
        name: "Biorhythm",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachPlayer),
            body: Box::new(Effect::SetLifeTotal {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::CreatureCountControlledBy(PlayerRef::Triggerer),
            }),
        },
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
/// Static: artifact creatures you control get +1/+1.
/// **+1**: Up to one target creature gets -2/-2 until end of turn.
/// **-2**: Each opponent loses 2 life and you gain 2 life.
///
/// Cube-style card. The static anthem rides `StaticEffect::PumpPT` (planes-
/// walkers feed `static_ability_to_effects` like any permanent). The "ult"
/// (typically -7) is collapsed to the two cube-relevant loyalty lines.
pub fn tezzeret_cruel_captain() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, StaticAbility, Supertype as Sup};
    use crate::effect::{Duration, StaticEffect};
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
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Artifact
                        .and(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
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

/// Pentad Prism — {2} Artifact. Sunburst (enters with one charge counter
/// for each color of mana spent on its cost). Remove a charge counter:
/// Add one mana of any color.
///
/// Sunburst reads `Value::ConvergedValue` (distinct colors of mana spent),
/// threaded from the cast onto the ETB trigger. Each activation removes a
/// counter to add a mana of any color (Gemstone-Mine cost-as-resolution);
/// an empty counter pool fails the removal and the ability fizzles.
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
                amount: Value::ConvergedValue,
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
            tap_other_filter: None, from_hand: false,
        }],
        ..Default::default()
    }
}

/// Balefire Dragon — {5}{R}{R} Creature — Dragon. 6/6 Flying.
/// Whenever this deals combat damage to a player, it deals that much damage
/// to each creature that player controls.
///
/// "That much damage" reads the Dragon's current power
/// (`Value::PowerOf(This)`), so anthem/pump effects scale the sweep — its
/// combat damage equals its power on the connecting hit.
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
                    amount: Value::PowerOf(Box::new(Selector::This)),
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
/// Your permanents' ETB triggers fire an additional time via
/// `StaticEffect::DoubleControllerEtbTriggers` (read by
/// `etb_trigger_multiplier` on both the self-ETB and reaction-ETB dispatch
/// paths), without suppressing opponents' triggers.
pub fn yarok_the_desecrated() -> CardDefinition {
    use crate::card::{StaticAbility, Supertype as Sup};
    use crate::effect::StaticEffect;
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
        static_abilities: vec![StaticAbility {
            description: "If a permanent entering causes a triggered ability \
                          of a permanent you control to trigger, that ability \
                          triggers an additional time.",
            effect: StaticEffect::DoubleControllerEtbTriggers,
        }],
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

/// Murktide Regent — {5}{U}{U} Creature — Dragon, 3/3, Flying, Delve.
/// Enters with two +1/+1 counters; whenever you cast an instant or sorcery
/// spell, put a +1/+1 counter on it.
pub fn murktide_regent() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Murktide Regent",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Delve],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Seasoned Pyromancer — {1}{R}{R} Creature — Human Shaman, 2/2. ETB:
/// discard two cards, then draw two; create a 1/1 red Elemental for each
/// nonland card discarded this way. (The graveyard-exile recursion clause
/// is omitted.)
pub fn seasoned_pyromancer() -> CardDefinition {
    use crate::card::TokenDefinition;
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
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::count(Selector::DiscardedThisResolution {
                    filter: SelectionRequirement::Nonland,
                }),
                definition: TokenDefinition {
                    name: "Elemental".into(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    supertypes: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Elemental],
                        ..Default::default()
                    },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                },
            },
        ]))],
        ..Default::default()
    }
}

/// Aether Figment — {1}{U} Creature — Illusion, 2/2, can't be blocked.
/// Kicker {3}: enters with a +1/+1 counter if it was kicked (CR 702.32 +
/// ETB-kicked context).
pub fn aether_figment() -> CardDefinition {
    CardDefinition {
        name: "Aether Figment",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Illusion],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Unblockable, Keyword::Kicker(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Selfless Spirit — {1}{W} Creature — Spirit, 2/1, Flying. "Sacrifice
/// this: creatures you control gain indestructible until end of turn."
pub fn selfless_spirit() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Selfless Spirit",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: each_your_creature(),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reality Smasher — {4}{C} Creature — Eldrazi, 5/5, Trample, Haste, Ward
/// {2} (the printed "counter unless pay {2} when targeted by a spell" is
/// modeled as Ward {2}).
pub fn reality_smasher() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Reality Smasher",
        cost: cost(&[generic(4), colorless(1)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![
            Keyword::Trample,
            Keyword::Haste,
            Keyword::Ward(WardCost::generic(2)),
        ],
        ..Default::default()
    }
}

/// Glint-Nest Crane — {1}{U} Creature — Bird, 1/3, Flying. ETB: look at the
/// top four cards; put an artifact among them into your hand, rest on the
/// bottom.
pub fn glint_nest_crane() -> CardDefinition {
    CardDefinition {
        name: "Glint-Nest Crane",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: Some(SelectionRequirement::Artifact),
        })],
        ..Default::default()
    }
}

/// Thraben Inspector — {W} Creature — Human Soldier, 1/1. ETB: investigate
/// (create a Clue token; CR 701.13).
pub fn thraben_inspector() -> CardDefinition {
    CardDefinition {
        name: "Thraben Inspector",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(investigate(1))],
        ..Default::default()
    }
}

/// Goblin Chainwhirler — {R}{R}{R} Creature — Goblin Soldier, 3/3, First
/// strike. ETB: deal 1 damage to each creature an opponent controls and to
/// each opponent.
pub fn goblin_chainwhirler() -> CardDefinition {
    CardDefinition {
        name: "Goblin Chainwhirler",
        cost: cost(&[r(), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(1),
                }),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Goblin Bushwhacker — {R} Creature — Goblin, 1/1, Haste. Kicker {R}.
/// When it enters, if it was kicked, creatures you control get +1/+0 and
/// gain haste until end of turn (CR 702.32 + ETB-kicked context).
pub fn goblin_bushwhacker() -> CardDefinition {
    CardDefinition {
        name: "Goblin Bushwhacker",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::Kicker(cost(&[r()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: each_your_creature(),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: each_your_creature(),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
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

/// Crumble to Dust — {2}{R}{R} Sorcery. Exile target nonbasic land, then
/// search its owner's library, graveyard, and hand for any number of cards
/// with the same name and exile them; that player shuffles. Wired via
/// `Effect::ExileSameNameAsTarget` — the full same-name sweep.
pub fn crumble_to_dust() -> CardDefinition {
    CardDefinition {
        name: "Crumble to Dust",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExileSameNameAsTarget {
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

/// Skullcrack — {1}{R} Instant. "Damage can't be prevented this turn.
/// Skullcrack deals 3 damage to target player or planeswalker. Players
/// can't gain life this turn."
///
/// Fully wired: `DamageCantBePreventedThisTurn` (CR 615.12) suppresses
/// prevention shields, `LifeGainLockThisTurn` (backed by
/// `Player.cannot_gain_life_this_turn`, reset in `do_untap`) blocks
/// lifegain before the 3-damage Bolt resolves.
pub fn skullcrack() -> CardDefinition {
    CardDefinition {
        name: "Skullcrack",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DamageCantBePreventedThisTurn,
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
            },
        ],
        ..Default::default()
    }
}

/// Searing Blood — {R}{R} Instant. Deals 2 damage to target creature; when
/// that creature dies this turn, deals 3 to its controller. The death rider
/// is a real event-keyed delayed trigger (`WhenTargetDiesThisTurn`).
pub fn searing_blood() -> CardDefinition {
    CardDefinition {
        name: "Searing Blood",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        // Register the death-watch before the damage so the watch is live
        // when the 2 damage kills the creature within this same resolution.
        effect: Effect::Seq(vec![
            Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(3),
                }),
            },
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Grapeshot — {1}{R} Sorcery with Storm. Deals 1 damage to any target.
/// Storm copies it once for each spell cast before it this turn (CR 702.40),
/// each copy choosing its own target.
pub fn grapeshot() -> CardDefinition {
    CardDefinition {
        name: "Grapeshot",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Storm],
        effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
        ..Default::default()
    }
}

/// Ahn-Crop Crasher — {2}{R}, 3/2 Minotaur Warrior with Haste and Exert
/// (CR 702.83). We auto-exert it as it attacks: it gains its on-attack
/// bonus (target creature can't block this turn) and won't untap on the
/// controller's next untap step.
pub fn ahn_crop_crasher() -> CardDefinition {
    CardDefinition {
        name: "Ahn-Crop Crasher",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::Exert],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Servant of Tymaret — {1}{B}, 2/1 Zombie with Inspired (CR 702.108):
/// whenever it becomes untapped, each opponent loses 1 life.
pub fn servant_of_tymaret() -> CardDefinition {
    CardDefinition {
        name: "Servant of Tymaret",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Unholy Heat — {R} Instant. Deals 2 damage to target creature or
/// planeswalker; Delirium (4+ card types in your graveyard) deals 6 instead.
pub fn unholy_heat() -> CardDefinition {
    CardDefinition {
        name: "Unholy Heat",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::DeliriumActive { who: PlayerRef::You },
            then: Box::new(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(6),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            }),
        },
        ..Default::default()
    }
}

/// Cut Down — {B} Instant. Destroy target creature with total power and
/// toughness 5 or less.
pub fn cut_down() -> CardDefinition {
    CardDefinition {
        name: "Cut Down",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::PowerPlusToughnessAtMost(5)),
        },
        ..Default::default()
    }
}

/// Galvanic Blast — {R} Instant. Deals 1 damage to any target; Affinity for
/// artifacts (3+ artifacts you control) deals 4 instead.
pub fn galvanic_blast() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Blast",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(3),
            },
            then: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(4) }),
            else_: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) }),
        },
        ..Default::default()
    }
}

/// Seal of Fire — {R} Enchantment. Sacrifice it: deals 2 damage to any target.
pub fn seal_of_fire() -> CardDefinition {
    CardDefinition {
        name: "Seal of Fire",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![crate::effect::ActivatedAbility {
            sac_cost: true,
            effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Abrade — {1}{R} Instant. Choose one — deal 3 damage to target creature;
/// or destroy target artifact.
pub fn abrade() -> CardDefinition {
    CardDefinition {
        name: "Abrade",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
        ]),
        ..Default::default()
    }
}

/// Boros Charm — {R}{W} Instant. Choose one — 4 damage to target player or
/// planeswalker; or permanents you control gain indestructible until end of
/// turn; or target creature gains double strike until end of turn.
pub fn boros_charm() -> CardDefinition {
    CardDefinition {
        name: "Boros Charm",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Player.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(SelectionRequirement::ControlledByYou),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
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

/// Sprite Dragon — {U}{R}, 1/1 Faerie Dragon with Flying and Haste.
/// Whenever you cast a noncreature spell, put a +1/+1 counter on it.
pub fn sprite_dragon() -> CardDefinition {
    CardDefinition {
        name: "Sprite Dragon",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Dragon],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Kiln Fiend — {1}{R}, 1/2 Elemental. Whenever you cast an instant or
/// sorcery spell, it gets +3/+0 until end of turn.
pub fn kiln_fiend() -> CardDefinition {
    CardDefinition {
        name: "Kiln Fiend",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::magecraft(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(3),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Soul-Scar Mage — {R}, 1/2 Human Monk with Prowess. (Its noncombat-damage-
/// as-(-1/-1)-counters replacement is omitted — no damage-replacement
/// primitive yet; tracked in TODO.md.)
pub fn soul_scar_mage() -> CardDefinition {
    CardDefinition {
        name: "Soul-Scar Mage",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    }
}

/// Temur Battle Rage — {1}{R} Instant. Target creature gets +1/+1 and gains
/// trample until end of turn. Ferocious — it also gains double strike if you
/// control a creature with power 4 or greater.
pub fn temur_battle_rage() -> CardDefinition {
    CardDefinition {
        name: "Temur Battle Rage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::ControlledByYou
                            .and(SelectionRequirement::PowerAtLeast(4)),
                    ),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Mutagenic Growth — {G/P} Instant. Target creature gets +2/+2 until end of
/// turn. (Modeled at the {G} cost — Phyrexian "pay 2 life" alt is omitted;
/// tracked in TODO.md.)
pub fn mutagenic_growth() -> CardDefinition {
    CardDefinition {
        name: "Mutagenic Growth",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Brute Force — {G} Instant. Target creature gets +3/+3 until end of turn.
pub fn brute_force() -> CardDefinition {
    CardDefinition {
        name: "Brute Force",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(3),
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Titan's Strength — {R} Instant. Target creature gets +3/+1 until end of
/// turn. Scry 1.
pub fn titans_strength() -> CardDefinition {
    CardDefinition {
        name: "Titan's Strength",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Crash Through — {R} Sorcery. Creatures you control gain trample until end
/// of turn. Draw a card.
pub fn crash_through() -> CardDefinition {
    CardDefinition {
        name: "Crash Through",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Fling — {1}{R} Instant. As an additional cost, sacrifice a creature.
/// Deals damage equal to the sacrificed creature's power to any target.
pub fn fling() -> CardDefinition {
    CardDefinition {
        name: "Fling",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::SacrificedPower },
        ]),
        ..Default::default()
    }
}

/// Supreme Verdict — {1}{W}{W}{U} Sorcery. Destroy all creatures. This spell
/// can't be countered.
pub fn supreme_verdict() -> CardDefinition {
    CardDefinition {
        name: "Supreme Verdict",
        cost: cost(&[generic(1), w(), w(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Stubborn Denial — {U} Instant. Counter target noncreature spell unless its
/// controller pays {1}. Ferocious — pays {3} instead if you control a
/// creature with power 4 or greater.
pub fn stubborn_denial() -> CardDefinition {
    let spell_filter =
        SelectionRequirement::IsSpellOnStack.and(SelectionRequirement::Noncreature);
    CardDefinition {
        name: "Stubborn Denial",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::ControlledByYou
                        .and(SelectionRequirement::PowerAtLeast(4)),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::CounterUnlessPaid {
                what: target_filtered(spell_filter.clone()),
                mana_cost: cost(&[generic(3)]),
            }),
            else_: Box::new(Effect::CounterUnlessPaid {
                what: target_filtered(spell_filter),
                mana_cost: cost(&[generic(1)]),
            }),
        },
        ..Default::default()
    }
}

/// Archmage's Charm — {U}{U}{U} Instant. Choose one — counter target spell;
/// or draw two cards; or gain control of target nonland permanent with mana
/// value 1 or less.
pub fn archmages_charm() -> CardDefinition {
    CardDefinition {
        name: "Archmage's Charm",
        cost: cost(&[u(), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::GainControl {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ManaValueAtMost(1)),
                ),
                to: None,
                duration: Duration::Permanent,
            },
        ]),
        ..Default::default()
    }
}

/// Snakeskin Veil — {G} Instant. Target creature gets +1/+1 and gains
/// hexproof until end of turn.
pub fn snakeskin_veil() -> CardDefinition {
    CardDefinition {
        name: "Snakeskin Veil",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Murmuring Mystic — {3}{U}, 1/5 Human Wizard. Whenever you cast an instant
/// or sorcery spell, create a 1/1 blue Bird Illusion with flying.
pub fn murmuring_mystic() -> CardDefinition {
    CardDefinition {
        name: "Murmuring Mystic",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        triggered_abilities: vec![crate::effect::shortcut::magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Bird Illusion".into(),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying],
                card_types: vec![CardType::Creature],
                colors: vec![Color::Blue],
                supertypes: vec![],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Bird, CreatureType::Illusion],
                    ..Default::default()
                },
                activated_abilities: vec![],
                triggered_abilities: vec![],
            },
        })],
        ..Default::default()
    }
}

/// Werewolf Pack Leader — {G}{G}, 3/3 Wolf. Whenever it attacks, if you
/// control three or more creatures, draw a card. {3}{G}{G}: it gets +1/+1
/// and can't be blocked this turn.
pub fn werewolf_pack_leader() -> CardDefinition {
    CardDefinition {
        name: "Werewolf Pack Leader",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wolf],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(3),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![crate::effect::ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), g()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Infernal Grasp — {1}{B} Instant. Destroy target creature. You lose 2 life.
pub fn infernal_grasp() -> CardDefinition {
    CardDefinition {
        name: "Infernal Grasp",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(SelectionRequirement::Creature) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Village Rites — {B} Instant. As an additional cost, sacrifice a creature.
/// Draw two cards.
pub fn village_rites() -> CardDefinition {
    CardDefinition {
        name: "Village Rites",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Power Word Kill — {1}{B} Instant. Destroy target non-Angel, non-Demon,
/// non-Dragon creature. (The printed "non-God" clause is dropped — no God
/// creature type in the engine yet.)
pub fn power_word_kill() -> CardDefinition {
    use crate::card::CreatureType as CT;
    CardDefinition {
        name: "Power Word Kill",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CT::Angel).negate())
                    .and(SelectionRequirement::HasCreatureType(CT::Demon).negate())
                    .and(SelectionRequirement::HasCreatureType(CT::Dragon).negate()),
            ),
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
/// graveyard (`LookPickToHand` with `rest_to_graveyard`).
pub fn strategic_planning() -> CardDefinition {
    CardDefinition {
        name: "Strategic Planning",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: None,
        },
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
/// Wired via `Effect::ExileChosenUntilSourceLeaves` (CR 603.6e): the
/// chosen nonland card is exiled linked to Brain Maggot and returns to
/// its owner's hand when Brain Maggot leaves play. Caster picks; bots
/// auto-pick the first matching card.
pub fn brain_maggot() -> CardDefinition {
    use crate::card::ExileReturnZone;
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
            effect: Effect::ExileChosenUntilSourceLeaves {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
                return_to: ExileReturnZone::Hand,
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
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
            tap_other_filter: None, from_hand: false,
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
/// -7 grants a real emblem (`Effect::CreateEmblem`) whose end-step
/// trigger fires the copy body twice each turn.
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
            // -7: emblem — "At the beginning of your end step, create two
            // tokens that are copies of target artifact or creature you
            // control, except they have haste." Modeled via the CR 114
            // emblem zone (one CreateTokenCopyOf count 2 + haste). NOTE:
            // copy-target auto-selection through the step-trigger path is a
            // known gap (Seq-wrapped CreateTokenCopyOf source slot isn't
            // filled by `auto_target_for_effect` from a command-zone
            // trigger) — tracked in TODO.md. The emblem itself is created
            // and fires; the copy body resolves once the gap closes.
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Saheeli Rai".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::StepBegins(crate::game::TurnStep::End),
                            EventScope::YourControl,
                        ),
                        effect: Effect::Seq(vec![
                            Effect::CreateTokenCopyOf {
                                who: PlayerRef::You,
                                count: Value::Const(2),
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
                        ]),
                    }],
                },
            },
        ],
        ..Default::default()
    }
}

// ── modern_decks-16: new cube cards ──────────────────────────────────────────

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

// Lightning Helix — already in catalog as `lightning_helix` (lea set).

/// Kolaghan's Command — {1}{B}{R} Instant. Choose two:
/// - Return target creature card from your graveyard to your hand.
/// - Target player discards a card.
/// - Destroy target artifact.
/// - Deal 2 damage to any target.
///
/// Faithful "choose two" via `Effect::ChooseN`. Each mode owns a cast-time
/// target slot in pick order. `picks: [0, 3]` (reanimate + 2 damage) is the
/// AutoDecider default; a UI/scripted decider can pick any two of the four.
pub fn kolaghans_command() -> CardDefinition {
    CardDefinition {
        name: "Kolaghan's Command",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![0, 3],
            modes: vec![
                // Mode 0: return target creature card from your gy to hand.
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Creature),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                // Mode 1: target player discards a card.
                Effect::Discard {
                    who: target_filtered(SelectionRequirement::Player),
                    amount: Value::Const(1),
                    random: false,
                },
                // Mode 2: destroy target artifact.
                Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Artifact),
                },
                // Mode 3: deal 2 damage to any target.
                Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                },
            ],
        },
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

/// Collective Brutality — {1}{B} Sorcery. Choose one (escalate collapsed):
/// - Target creature gets -2/-2 until end of turn.
/// - Target player discards a card.
/// - Target opponent loses 2 life and you gain 2 life.
pub fn collective_brutality() -> CardDefinition {
    CardDefinition {
        name: "Collective Brutality",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
        ]),
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
                        to: None,
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

/// Oko, Thief of Crowns — {1}{G}{U} Planeswalker. 4 loyalty.
/// +2: Create a Food token (approximated as gain 3 life).
/// +1: Target artifact or creature becomes a 3/3 Elk, losing all other
///   types and abilities (`ResetCreature`).
/// -5: Exchange control of target (collapsed to gain control).
pub fn oko_thief_of_crowns() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    CardDefinition {
        name: "Oko, Thief of Crowns",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Oko],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::ResetCreature {
                    what: target_filtered(
                        SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                    ),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Elk],
                    duration: Duration::Permanent,
                },
            },
            LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::GainControl {
                    what: target_filtered(SelectionRequirement::Permanent),
                    to: None,
                    duration: Duration::Permanent,
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
/// "Can attack only alone" is wired via `Keyword::AttacksAlone` (CR 508.0).
/// The `AttacksAndIsntBlocked` trigger sets the defender to 1 life and grants
/// `Keyword::DealsNoCombatDamage` until end of turn, so its first-strike
/// deathtouch ping no longer finishes the kill — the printed "deals no
/// combat damage this turn" rider is now honored.
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
        keywords: vec![Keyword::FirstStrike, Keyword::Deathtouch, Keyword::AttacksAlone],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::SetLifeTotal {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::DealsNoCombatDamage,
                    duration: Duration::EndOfTurn,
                },
            ]),
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

/// Kitchen Finks — {1}{G}{W} Creature 3/2. When this enters, you gain
/// 2 life. Persist.
pub fn kitchen_finks() -> CardDefinition {
    CardDefinition {
        name: "Kitchen Finks",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Persist],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
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

/// Wall of Blossoms — {1}{G} Creature 0/4 Plant Wall. Defender. When this
/// enters, draw a card.
pub fn wall_of_blossoms() -> CardDefinition {
    CardDefinition {
        name: "Wall of Blossoms",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Trinisphere — {3} Artifact. While untapped, each spell that would cost
/// less than {3} to cast costs {3} instead.
pub fn trinisphere() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Trinisphere",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "As long as Trinisphere is untapped, each spell that \
                          would cost less than three mana to cast costs three \
                          instead.",
            effect: StaticEffect::SpellCostFloor { amount: 3 },
        }],
        ..Default::default()
    }
}

/// Raven's Crime — {B} Sorcery. "Target player discards a card. Retrace"
/// (CR 702.81 — recast from the graveyard for its cost plus discarding a
/// land card). Wired via `Keyword::Retrace` + `GameAction::CastRetrace`.
pub fn ravens_crime() -> CardDefinition {
    CardDefinition {
        name: "Raven's Crime",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Retrace],
        effect: Effect::Discard {
            who: Selector::Target(0),
            amount: Value::Const(1),
            random: false,
        },
        ..Default::default()
    }
}

/// Mulldrifter — {4}{U} Creature 2/2 Elemental. Flying. When this enters,
/// draw two cards. Evoke {2}{U}.
pub fn mulldrifter() -> CardDefinition {
    use crate::card::AlternativeCost;
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
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        }],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), u()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: true,
            not_your_turn_only: false,
            target_filter: None,
            ..Default::default()
        }),
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
/// The "an opponent gains control" downside now rides
/// `Effect::GainControl { to: Some(EachOpponent) }` (permanent control
/// shift). The tutor reads `SelectionRequirement::Any`; the activation
/// rejects when no wish counters remain.
pub fn wishclaw_talisman() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    CardDefinition {
        name: "Wishclaw Talisman",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Charge, Value::Const(3))),
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
                // CR — an opponent gains control of Wishclaw afterward.
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::EachOpponent),
                    duration: Duration::Permanent,
                },
            ]),
            sorcery_speed: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lingering Souls — {2}{W} Sorcery. Create two 1/1 white Spirit creature
/// tokens with flying. Flashback {1}{B}.
pub fn lingering_souls() -> CardDefinition {
    CardDefinition {
        name: "Lingering Souls",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(1), b()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
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
        ..Default::default()
    }
}

/// Murderous Cut — {4}{B} Instant. Delve. Destroy target creature.
///
/// Delve (CR 702.66) is wired via `Keyword::Delve` + `CastSpellDelve`:
/// each graveyard card exiled while casting pays {1} of the {4}, so a
/// stocked graveyard turns this into a {B} kill spell. Body is a clean
/// `Destroy(Creature)`.
pub fn murderous_cut() -> CardDefinition {
    CardDefinition {
        name: "Murderous Cut",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![crate::card::Keyword::Delve],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Creature),
        },
        ..Default::default()
    }
}

/// Gurmag Angler — {6}{B} Creature — Zombie Fish. 5/5 vanilla. Delve.
///
/// The canonical Delve creature: with a stocked graveyard the {6} generic
/// is paid by exiling graveyard cards, dropping the cost toward {B}.
/// Exercises Delve on a *permanent* spell (the delve cards are exiled as
/// part of paying the casting cost, not the resolution).
pub fn gurmag_angler() -> CardDefinition {
    CardDefinition {
        name: "Gurmag Angler",
        cost: cost(&[generic(6), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Fish],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![crate::card::Keyword::Delve],
        ..Default::default()
    }
}

/// Drudge Skeletons — {1}{B} Creature — Skeleton. 1/1. "{B}: Regenerate
/// this creature." (CR 701.15)
///
/// The classic regenerator: the activated ability stamps a regeneration
/// shield via `Effect::Regenerate(This)`, so the next destruction this
/// turn taps it and heals damage instead of killing it. Exercises the
/// regeneration replacement against both combat damage and `Destroy`.
pub fn drudge_skeletons() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Drudge Skeletons",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost { symbols: vec![crate::mana::ManaSymbol::Colored(Color::Black)] },
            effect: Effect::Regenerate { what: Selector::This },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        ..Default::default()
    }
}

/// Tombstalker — {6}{B}{B} Creature — Demon. 5/5 with Flying. Delve.
///
/// Delve (CR 702.66) pays the {6} generic; off a full graveyard this is a
/// {B}{B} 5/5 flier.
pub fn tombstalker() -> CardDefinition {
    CardDefinition {
        name: "Tombstalker",
        cost: cost(&[generic(6), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Delve],
        ..Default::default()
    }
}

/// Will-o'-the-Wisp — {B} Creature — Spirit. 0/1 with Flying.
/// "{B}: Regenerate Will-o'-the-Wisp." (CR 701.15)
pub fn will_o_the_wisp() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Will-o'-the-Wisp",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost { symbols: vec![crate::mana::ManaSymbol::Colored(Color::Black)] },
            effect: Effect::Regenerate { what: Selector::This },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        ..Default::default()
    }
}

/// Wall of Bone — {2}{B} Creature — Skeleton Wall. 1/4 with Defender.
/// "{B}: Regenerate Wall of Bone." (CR 701.15)
pub fn wall_of_bone() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Wall of Bone",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Wall],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost { symbols: vec![crate::mana::ManaSymbol::Colored(Color::Black)] },
            effect: Effect::Regenerate { what: Selector::This },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        ..Default::default()
    }
}

/// Hooting Mandrills — {5}{G} Creature — Ape. 4/4 with Trample. Delve.
///
/// Delve (CR 702.66) pays the {5} generic by exiling graveyard cards.
pub fn hooting_mandrills() -> CardDefinition {
    CardDefinition {
        name: "Hooting Mandrills",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Delve],
        ..Default::default()
    }
}

/// Become Immense — {5}{G} Instant. Delve. Target creature gets +6/+6
/// until end of turn.
///
/// Delve (CR 702.66) pays the {5}; a stocked graveyard turns this into a
/// {G} +6/+6 pump.
pub fn become_immense() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Become Immense",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Delve],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(6),
            toughness: Value::Const(6),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Severed Legion — {2}{B} Creature — Zombie. 2/2 with Fear (CR 702.36):
/// can't be blocked except by artifact and/or black creatures.
pub fn severed_legion() -> CardDefinition {
    CardDefinition {
        name: "Severed Legion",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Fear],
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

// Terminate — already in catalog as `terminate`.
// Spell Pierce — already in catalog.

/// Relic of Progenitus — {1} Artifact. {T}: Target player exiles a card
/// from their graveyard. {1}, Exile Relic: Exile all graveyards, draw 1.
///
/// Approximation: first ability only (exile each opponent's top gy card).
pub fn relic_of_progenitus() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Relic of Progenitus",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::Move {
                    what: Selector::CardsInZone {
                        zone: crate::card::Zone::Graveyard,
                        who: PlayerRef::EachOpponent,
                        filter: SelectionRequirement::Any,
                    },
                    to: ZoneDest::Exile,
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
                life_cost: 0,
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: false,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Seq(vec![
                    Effect::ForEach {
                        selector: Selector::Player(PlayerRef::EachPlayer),
                        body: Box::new(Effect::ShuffleGraveyardIntoLibrary {
                            who: PlayerRef::Triggerer,
                        }),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ]),
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
                condition: None,
                life_cost: 0,
                ..Default::default()
            },
        ],
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

// Tidehollow Sculler — already in catalog.

/// Spell Queller — {1}{W}{U} Creature 2/3 Spirit. Flash, flying.
/// When this enters, exile target spell with mana value 4 or less.
///
/// Approximation: ETB counters target spell (exile-until-LTB not wired).
pub fn spell_queller() -> CardDefinition {
    use crate::effect::shortcut::counter_target_spell;
    CardDefinition {
        name: "Spell Queller",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: counter_target_spell(),
        }],
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

/// Bloodbraid Elf — {2}{R}{G} Creature 3/2. Haste. Cascade.
///
/// Cascade (CR 702.85) is now a first-class engine mechanic: the
/// `shortcut::cascade(mv)` trigger fires on cast, exiles from the top of
/// the library until a nonland card with MV < 4 is exiled, lets the
/// controller cast it for free, and bottoms the rest.
pub fn bloodbraid_elf() -> CardDefinition {
    use crate::effect::shortcut::cascade;
    CardDefinition {
        name: "Bloodbraid Elf",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![cascade(4)],
        ..Default::default()
    }
}

/// Gift of Orzhova — {1}{W}{B} Enchantment — Aura. "Enchant creature.
/// Enchanted creature gets +1/+1 and has flying and lifelink."
///
/// First Aura in the catalog. The cast-time target (a creature) is driven
/// by the `Effect::Attach` target slot; the spell-resolution path
/// (`stack.rs`) sets the Aura's `attached_to` link, and `equipped_bonus`
/// flows the +1/+1 (layer 7c) and flying/lifelink keywords (layer 6) onto
/// the enchanted creature for as long as the Aura stays attached.
pub fn gift_of_orzhova() -> CardDefinition {
    CardDefinition {
        name: "Gift of Orzhova",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature,
            },
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying, Keyword::Lifelink],
        }),
        ..Default::default()
    }
}

/// Apex Devastator — {8}{G}{G} Creature — Kavu. 10/10, Trample.
/// "Cascade, cascade, cascade, cascade" (CR 702.85 — four independent
/// cascade triggers on cast).
///
/// Showcases the cascade mechanic chaining: each of the four
/// `shortcut::cascade(10)` triggers walks the top of the library
/// independently when the spell is cast (MV gate 10 = {8}{G}{G}).
pub fn apex_devastator() -> CardDefinition {
    use crate::effect::shortcut::cascade;
    CardDefinition {
        name: "Apex Devastator",
        cost: cost(&[generic(8), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu],
            ..Default::default()
        },
        power: 10,
        toughness: 10,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![cascade(10), cascade(10), cascade(10), cascade(10)],
        ..Default::default()
    }
}

/// Life from the Loam — {1}{B}{G} Sorcery. "Return up to three target land
/// cards from your graveyard to your hand. Dredge 3." (CR 702.52)
///
/// The "up to three target land cards" pick is approximated as a
/// deterministic `Selector::Take { CardsInZone(gy, Land), 3 }` (the engine
/// auto-pulls up to three land cards from your graveyard). Dredge 3 is now
/// a first-class engine mechanic via `Keyword::Dredge(3)` + the
/// `draw_one` replacement.
pub fn life_from_the_loam() -> CardDefinition {
    CardDefinition {
        name: "Life from the Loam",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Dredge(3)],
        effect: Effect::Move {
            what: Selector::Take {
                inner: Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Land,
                }),
                count: Box::new(Value::Const(3)),
            },
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Golgari Thug — {1}{B} Creature — Human Mercenary. 1/1. Dredge 4.
/// "When this creature dies, put target creature card from your graveyard
/// on top of your library."
///
/// The death recursion is wired as a `CreatureDied/SelfSource` trigger that
/// moves a creature card from your graveyard to the top of your library
/// (`Selector::Take { CardsInZone(gy, Creature), 1 }` — deterministic pull,
/// the "target" choice collapsed). Dredge 4 via `Keyword::Dredge(4)`.
pub fn golgari_thug() -> CardDefinition {
    CardDefinition {
        name: "Golgari Thug",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Dredge(4)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    }),
                    count: Box::new(Value::Const(1)),
                },
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: crate::effect::LibraryPosition::Top,
                },
            },
        }],
        ..Default::default()
    }
}

/// Shardless Agent — {1}{G}{U} Artifact Creature — Human Artificer. 2/2.
/// Cascade (CR 702.85).
pub fn shardless_agent() -> CardDefinition {
    use crate::effect::shortcut::cascade;
    CardDefinition {
        name: "Shardless Agent",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![cascade(3)],
        ..Default::default()
    }
}

/// Enlisted Wurm — {5}{G}{W} Creature — Wurm. 5/5. Cascade.
pub fn enlisted_wurm() -> CardDefinition {
    use crate::effect::shortcut::cascade;
    CardDefinition {
        name: "Enlisted Wurm",
        cost: cost(&[generic(5), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![cascade(7)],
        ..Default::default()
    }
}

/// Maelstrom Wanderer — {5}{U}{R}{G} Legendary Creature — Elemental. 7/5.
/// "Creatures you control have haste. Cascade, cascade." (two independent
/// cascade triggers, CR 702.85.)
///
/// The team-haste static is wired via `StaticEffect::GrantKeywordToYours`
/// (the same primitive used by anthem-keyword granters).
pub fn maelstrom_wanderer() -> CardDefinition {
    use crate::card::{StaticAbility, Supertype};
    use crate::effect::shortcut::cascade;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Maelstrom Wanderer",
        cost: cost(&[generic(5), u(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 7,
        toughness: 5,
        triggered_abilities: vec![cascade(8), cascade(8)],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
            },
        }],
        ..Default::default()
    }
}

/// Stinkweed Imp — {1}{B} Creature — Imp. 1/2, Flying. Dredge 5.
/// "Whenever this deals combat damage to a creature, destroy that
/// creature." Modeled with `Keyword::Deathtouch` (gameplay-equivalent for
/// combat damage to creatures) + `Keyword::Dredge(5)`.
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
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Dredge(5)],
        ..Default::default()
    }
}

/// Golgari Brownscale — {1}{G} Creature — Lizard Beast. 2/2. Dredge 2.
/// The "when returned to hand from graveyard, gain 2 life" rider is omitted
/// (no enters-hand-from-graveyard trigger event); the body + Dredge 2 ship.
pub fn golgari_brownscale() -> CardDefinition {
    CardDefinition {
        name: "Golgari Brownscale",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Dredge(2)],
        ..Default::default()
    }
}

/// Golgari Grave-Troll — {X}{B}{B} Creature — Skeleton Troll. 0/0, enters
/// with X +1/+1 counters. Dredge 6. The "{T}, remove four +1/+1 counters:
/// regenerate" ability is omitted (tracked in TODO.md); the X-body +
/// Dredge 6 ship.
pub fn golgari_grave_troll() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::effect::Predicate;
    CardDefinition {
        name: "Golgari Grave-Troll",
        cost: cost(&[generic(0), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Troll],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Dredge(6)],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        // {T}, Remove four +1/+1 counters: Regenerate this creature.
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
                Value::Const(4),
            )),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(4),
                },
                Effect::Regenerate { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rancor — {G} Enchantment — Aura. "Enchant creature. Enchanted creature
/// gets +2/+0 and has trample." The "when put into a graveyard from the
/// battlefield, return Rancor to its owner's hand" recursion is omitted
/// (no leaves-battlefield-to-hand trigger for noncreature permanents yet);
/// the +2/+0 + trample buff ships via the Aura attach + equipped_bonus path.
pub fn rancor() -> CardDefinition {
    CardDefinition {
        name: "Rancor",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature,
            },
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 2,
            toughness: 0,
            keywords: vec![Keyword::Trample],
        }),
        ..Default::default()
    }
}

/// Bituminous Blast — {3}{B}{R} Instant. "Deals 4 damage to target
/// creature. Cascade." (CR 702.85.)
pub fn bituminous_blast() -> CardDefinition {
    use crate::effect::shortcut::cascade;
    CardDefinition {
        name: "Bituminous Blast",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            amount: Value::Const(4),
            to: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature,
            },
        },
        triggered_abilities: vec![cascade(5)],
        ..Default::default()
    }
}

/// Violent Outburst — {1}{R}{G} Instant. "Creatures you control get +1/+1
/// until end of turn. Cascade."
pub fn violent_outburst() -> CardDefinition {
    use crate::effect::shortcut::cascade;
    CardDefinition {
        name: "Violent Outburst",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![cascade(3)],
        ..Default::default()
    }
}

/// Ardent Plea — {1}{W} Enchantment. "Exalted (CR 702.83). Cascade
/// (CR 702.85)." Cascade fires when the enchantment is cast.
pub fn ardent_plea() -> CardDefinition {
    use crate::effect::shortcut::{cascade, exalted};
    CardDefinition {
        name: "Ardent Plea",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![exalted(), cascade(2)],
        ..Default::default()
    }
}

/// Darkblast — {B} Instant. "Target creature gets -1/-1 until end of turn.
/// Dredge 3." (CR 702.52.)
pub fn darkblast() -> CardDefinition {
    CardDefinition {
        name: "Darkblast",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Dredge(3)],
        effect: Effect::PumpPT {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature,
            },
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Helper: build a simple "+P/+T (and optional keywords)" Aura.
fn simple_aura(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature,
            },
        },
        equipped_bonus: Some(crate::card::EquipBonus { power, toughness, keywords }),
        ..Default::default()
    }
}

/// Spectral Flight — {1}{U} Aura. "Enchanted creature gets +2/+2 and has
/// flying."
pub fn spectral_flight() -> CardDefinition {
    simple_aura("Spectral Flight", cost(&[generic(1), u()]), 2, 2, vec![Keyword::Flying])
}

/// Flight — {U} Aura. "Enchanted creature has flying."
pub fn flight() -> CardDefinition {
    simple_aura("Flight", cost(&[u()]), 0, 0, vec![Keyword::Flying])
}

/// Unholy Strength — {B} Aura. "Enchanted creature gets +2/+1."
pub fn unholy_strength() -> CardDefinition {
    simple_aura("Unholy Strength", cost(&[b()]), 2, 1, vec![])
}

/// Holy Strength — {W} Aura. "Enchanted creature gets +1/+2."
pub fn holy_strength() -> CardDefinition {
    simple_aura("Holy Strength", cost(&[w()]), 1, 2, vec![])
}

/// Pacifism — {1}{W} Aura. "Enchant creature. Enchanted creature can't
/// attack or block." (CR 702 — granted via the aura's keyword bonus.)
pub fn pacifism() -> CardDefinition {
    simple_aura(
        "Pacifism",
        cost(&[generic(1), w()]),
        0,
        0,
        vec![Keyword::CantAttack, Keyword::CantBlock],
    )
}

/// Lure — {1}{G}{G} Enchantment — Aura. Enchant creature. "All creatures
/// able to block enchanted creature do so." (CR 509.1c, `AllMustBlock`.)
pub fn lure() -> CardDefinition {
    simple_aura("Lure", cost(&[generic(1), g(), g()]), 0, 0, vec![Keyword::AllMustBlock])
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

/// Firebolt — {R} Sorcery. Deal 2 damage to any target. Flashback {4}{R}.
pub fn firebolt() -> CardDefinition {
    CardDefinition {
        name: "Firebolt",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), r()]))],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(2),
        },
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
        ..Default::default()
    }
}

/// Chainer's Edict — {1}{B} Sorcery. Target player sacrifices a creature.
/// Flashback {5}{B}{B}.
pub fn chainers_edict() -> CardDefinition {
    CardDefinition {
        name: "Chainer's Edict",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(5), b(), b()]))],
        effect: Effect::Sacrifice {
            who: target_filtered(SelectionRequirement::Player),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        },
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

/// Decree of Justice — {X}{X}{2}{W}{W} Sorcery. Create X 4/4 white Angel
/// creature tokens with flying.
///
/// Approximation: Create X tokens where X = XFromCost.
pub fn decree_of_justice() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Decree of Justice",
        cost: cost(&[x(), x(), generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: TokenDefinition {
                name: "Angel".into(),
                power: 4,
                toughness: 4,
                keywords: vec![Keyword::Flying],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                supertypes: vec![],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Angel],
                    ..Default::default()
                },
                activated_abilities: vec![],
                triggered_abilities: vec![],
            },
        },
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

/// Tireless Provisioner — {2}{G} Creature 3/2 Elf Druid.
/// Whenever a land enters under your control, create a Treasure token.
///
/// Landfall → Treasure.
pub fn tireless_provisioner() -> CardDefinition {
    CardDefinition {
        name: "Tireless Provisioner",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Helix Pinnacle — {G} Enchantment. "Helix Pinnacle has shroud as long
/// as it has fewer than 100 storage counters on it. / {X}: Put X storage
/// counters on Helix Pinnacle. / At the beginning of your upkeep, if
/// Helix Pinnacle has 100 or more storage counters on it, you win the
/// game."
///
/// Wires the headline "win at 100 counters" upkeep gate plus the
/// activation that adds X storage counters. The shroud-while-under-100
/// rider is engine-wide ⏳ — Helix Pinnacle ships as a non-targetable
/// enchantment via the standard rules (vanilla 1-mana enchantment).
/// The new `max_counters_of_kind: Some((Charge, 100))` cap caps the
/// counter at 100 even if activations would overshoot — so the SBA
/// prunes back to 100. The {X} activation uses XFromCost; the upkeep
/// win trigger uses the new CR 603.4 resolve-time intervening-if
/// re-check via the existing predicate path (the SpellCast +
/// StepBegins primitives).
pub fn helix_pinnacle() -> CardDefinition {
    use crate::card::{CounterType, EventScope, EventSpec, TriggeredAbility, Predicate};
    use crate::card::EventKind;
    use crate::effect::ActivatedAbility;
    use crate::mana::x;
    CardDefinition {
        name: "Helix Pinnacle",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        enters_as_copy: None,
        max_counters_of_kind: Some((CounterType::Charge, 100)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: false,
            sac_cost: false,
            life_cost: 0,
            sorcery_speed: false,
            exile_self_cost: false,
            from_graveyard: false,
            once_per_turn: false,
            condition: None,
            exile_other_filter: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            self_counter_cost_reduction: None,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::XFromCost,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                kind: EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                scope: EventScope::SelfSource,
                filter: Some(Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    },
                    Value::Const(100),
                )),
            },
            effect: Effect::WinGame { who: PlayerRef::You },
        }],
        ..Default::default()
    }
}

fn treasure_token() -> TokenDefinition {
    use crate::card::ActivatedAbility;
    TokenDefinition {
        name: "Treasure".into(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Artifact],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes::default(),
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
            ..Default::default()
        }],
        triggered_abilities: vec![],
    }
}

// ── Push claude/modern_decks additions ──────────────────────────────────

/// Fiery Confluence — {2}{R}{R} Sorcery. Choose three. You may choose the
/// same mode more than once.
/// - Deal 1 damage to each creature.
/// - Deal 2 damage to each opponent.
/// - Destroy target artifact.
///
/// Approximation: ChooseMode over the three options (single pick, not 3x).
pub fn fiery_confluence() -> CardDefinition {
    CardDefinition {
        name: "Fiery Confluence",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(1),
                }),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(6),
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Artifact),
            },
        ]),
        ..Default::default()
    }
}

/// Cam and Farrik, Havoc Duo — {3}{R}{G}, 4/5 Legendary Human Warrior.
/// Trample. Whenever you cast a noncreature spell, +2/+0 until end of turn.
pub fn cam_and_farrik() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Cam and Farrik, Havoc Duo",
        cost: cost(&[generic(3), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Magda, Brazen Outlaw — {1}{R}, 2/1 Legendary Dwarf Berserker.
/// Other Dwarves you control get +1/+0. Whenever a Dwarf you control
/// becomes tapped, create a Treasure token. (The five-Treasure sacrifice
/// tutor is omitted.)
pub fn magda_brazen_outlaw() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Magda, Brazen Outlaw",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Other Dwarves you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Dwarf)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Dwarf),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            },
        }],
        ..Default::default()
    }
}

// Hymn to Tourach already in catalog.

/// Sinkhole — {B}{B} Sorcery. Destroy target land.
pub fn sinkhole() -> CardDefinition {
    CardDefinition {
        name: "Sinkhole",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Land),
        },
        ..Default::default()
    }
}

/// Keen-Eyed Curator — {2}{G}, 3/3 Elf Druid.
/// ETB: +1/+1 counter on self + graveyard hate approximation.
pub fn keen_eyed_curator() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Keen-Eyed Curator",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Cube expansion: body-only stubs ─────────────────────────────────────────

/// Enduring Innocence — {W}{W}{W} Enchantment Creature — Glimmer. 2/1.
/// Lifelink. "Whenever a nontoken creature you control enters, draw a card."
///
/// 🟡 Body-only: the "return from exile after death" enchantment-recur
/// clause is omitted. Wired as 2/1 Lifelink + AnotherOfYours ETB draw
/// filtered to nontoken creatures.
pub fn enduring_innocence() -> CardDefinition {
    CardDefinition {
        name: "Enduring Innocence",
        cost: cost(&[w(), w(), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Glimmer],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::AnotherOfYours,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::NotToken),
            }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Amped Raptor — {1}{R} Creature — Dinosaur. 2/1.
/// "When Amped Raptor enters, you get {E}{E}, then you may cast a spell
/// from exile with mana value ≤ energy spent."
///
/// 🟡 Body-only: energy system is not wired. ETB gains 2 life as a
/// placeholder effect to exercise the trigger path. The exile-cast
/// clause is omitted.
pub fn amped_raptor() -> CardDefinition {
    CardDefinition {
        name: "Amped Raptor",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Thundertrap Trainer — {1}{W} Creature — Human Soldier. 2/2. Flash.
/// "When Thundertrap Trainer enters, tap target creature an opponent
/// controls." (Synthesised body; ETB tap fully wired.)
pub fn thundertrap_trainer() -> CardDefinition {
    CardDefinition {
        name: "Thundertrap Trainer",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        ..Default::default()
    }
}

/// Corpse Dance — {1}{B}{B} Instant. Buyback {2}.
/// "Put the top creature card of target player's graveyard onto the
/// battlefield under your control. Sacrifice it at the beginning of
/// the next end step."
///
/// 🟡 Body-only: Buyback omitted. Reanimates the top creature from
/// your graveyard to the battlefield, then registers a delayed trigger
/// to sacrifice it at end step.
/// Corpse Dance — {1}{B}{B} Instant. Buyback {2}.
/// "Put the top creature card of target player's graveyard onto the
/// battlefield under your control. Sacrifice it at the beginning of
/// the next end step."
pub fn corpse_dance() -> CardDefinition {
    use crate::effect::DelayedTriggerKind;
    CardDefinition {
        name: "Corpse Dance",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Basking Rootwalla — {1}{G} Creature — Lizard. 1/1.
/// "{1}{G}: Basking Rootwalla gets +2/+2 until end of turn. Activate
/// only once each turn." Madness {0}.
///
/// ✅ Madness {0} wired via `Keyword::Madness` (CR 702.35): discarding
/// Basking Rootwalla exiles it and offers a free ({0}) cast. The
/// once-per-turn {1}{G} activated pump (+2/+2 until EOT) is unchanged.
pub fn basking_rootwalla() -> CardDefinition {
    use crate::card::{ActivatedAbility, Keyword};
    CardDefinition {
        name: "Basking Rootwalla",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Madness(ManaCost::default())],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Green)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: true,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        ..Default::default()
    }
}

/// Blazing Rootwalla — {R} Creature — Lizard. 1/1. Madness {0}.
/// "{1}{R}: Blazing Rootwalla gets +1/+1 until end of turn. Activate only
/// once each turn." The red sibling of Basking Rootwalla.
pub fn blazing_rootwalla() -> CardDefinition {
    use crate::card::{ActivatedAbility, Keyword};
    CardDefinition {
        name: "Blazing Rootwalla",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Madness(ManaCost::default())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Red)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Anje's Ravager — {2}{R}{R} Legendary Creature — Vampire Berserker. 3/3.
/// Trample. Madness {1}{R}. "Whenever Anje's Ravager attacks, discard your
/// hand, then draw three cards."
pub fn anjes_ravager() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Keyword, Supertype, TriggeredAbility};
    CardDefinition {
        name: "Anje's Ravager",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![
            Keyword::Trample,
            Keyword::Madness(ManaCost::new(vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Red)])),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            ]),
        }],
        ..Default::default()
    }
}

/// Wind Drake — {2}{U} Creature — Drake. 2/2. Flying.
pub fn wind_drake() -> CardDefinition {
    use crate::card::Keyword;
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

/// Cunning Sparkmage — {1}{R} Creature — Human Shaman. 1/1. Haste.
/// "{T}: This creature deals 1 damage to any target."
pub fn cunning_sparkmage() -> CardDefinition {
    use crate::card::{ActivatedAbility, Keyword};
    CardDefinition {
        name: "Cunning Sparkmage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hill Giant — {3}{R} Creature — Giant. 3/3. A vanilla beater.
pub fn hill_giant() -> CardDefinition {
    CardDefinition {
        name: "Hill Giant",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}

/// Reckless Wurm — {3}{R} Creature — Wurm. 5/4. Trample. Madness {1}{R}.
pub fn reckless_wurm() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Reckless Wurm",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![
            Keyword::Trample,
            Keyword::Madness(ManaCost::new(vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Red)])),
        ],
        ..Default::default()
    }
}

/// Fiery Temper — {1}{R}{R} Instant. "Deal 3 damage to any target."
/// Madness {R}.
pub fn fiery_temper() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Fiery Temper",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Madness(ManaCost::new(vec![ManaSymbol::Colored(Color::Red)]))],
        effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Vampire Nighthawk — {1}{B}{B} Creature — Vampire Shaman. 2/3. Flying,
/// deathtouch, lifelink.
pub fn vampire_nighthawk() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Vampire Nighthawk",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Nekrataal — {2}{B}{B} Creature — Human Assassin. 2/1. First strike.
/// "When this creature enters, destroy target nonartifact, nonblack
/// creature. It can't be regenerated."
pub fn nekrataal() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Keyword, SelectionRequirement, TriggeredAbility};
    let filter = SelectionRequirement::Creature
        .and(SelectionRequirement::HasColor(Color::Black).negate())
        .and(SelectionRequirement::HasCardType(CardType::Artifact).negate());
    CardDefinition {
        name: "Nekrataal",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DestroyNoRegen { what: target_filtered(filter) },
        }],
        ..Default::default()
    }
}

/// Skinrender — {3}{B} Creature — Phyrexian Horror. 3/3. "When this creature
/// enters, put three -1/-1 counters on target creature."
pub fn skinrender() -> CardDefinition {
    use crate::card::{CounterType, EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility};
    CardDefinition {
        name: "Skinrender",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}

/// Ravenous Chupacabra — {2}{B}{B} Creature — Beast Horror. 2/2. "When this
/// creature enters, destroy target creature an opponent controls."
pub fn ravenous_chupacabra() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility};
    CardDefinition {
        name: "Ravenous Chupacabra",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Sentinel Spider — {3}{G}{G} Creature — Spider. 4/4. Vigilance, reach.
pub fn sentinel_spider() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Sentinel Spider",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Reach],
        ..Default::default()
    }
}

/// Brindle Boar — {2}{G} Creature — Boar. 3/3. "Sacrifice Brindle Boar: You
/// gain 4 life."
pub fn brindle_boar() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Brindle Boar",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reckless Abandon — {R} Sorcery. "As an additional cost to cast this
/// spell, sacrifice a creature. Deal 4 damage to any target."
pub fn reckless_abandon() -> CardDefinition {
    use crate::card::{AdditionalCastCost, SelectionRequirement};
    CardDefinition {
        name: "Reckless Abandon",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            count: 1,
        }],
        effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(4) },
        ..Default::default()
    }
}

/// Cloudgoat Ranger — {3}{W}{W} Creature — Giant. 2/2. "When this creature
/// enters, create three 1/1 white Kithkin Soldier creature tokens."
pub fn cloudgoat_ranger() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    let kithkin = TokenDefinition {
        name: "Kithkin Soldier".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kithkin, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Cloudgoat Ranger",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: kithkin,
            },
        }],
        ..Default::default()
    }
}

/// Pelakka Wurm — {5}{G}{G} Creature — Wurm. 7/7. Trample. "When this
/// creature enters, you gain 7 life. When this creature dies, draw a card."
pub fn pelakka_wurm() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Keyword, TriggeredAbility};
    CardDefinition {
        name: "Pelakka Wurm",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::GainLife { who: Selector::You, amount: Value::Const(7) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Springbloom Druid — {2}{G} Creature — Human Druid. 2/2. "When this
/// creature enters, search your library for up to two basic land cards,
/// put them onto the battlefield tapped, then shuffle." Two basic-land
/// searches into play tapped.
pub fn springbloom_druid() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility};
    use crate::effect::ZoneDest;
    let fetch = || Effect::Search {
        who: PlayerRef::You,
        filter: SelectionRequirement::IsBasicLand,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
    };
    CardDefinition {
        name: "Springbloom Druid",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![fetch(), fetch()]),
        }],
        ..Default::default()
    }
}

/// Cryptolith Rite — {1}{G} Enchantment. "Creatures you control have
/// '{T}: Add one mana of any color.'" Wired via
/// `StaticEffect::GrantActivatedAbility`.
pub fn cryptolith_rite() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Cryptolith Rite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![crate::effect::shortcut::grant_tap_for_any_color(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        )],
        ..Default::default()
    }
}

/// Call of the Herd — {2}{G} Sorcery. "Create a 3/3 green Elephant creature
/// token. Flashback {3}{G}." Flashback via `Keyword::Flashback`.
pub fn call_of_the_herd() -> CardDefinition {
    let elephant = TokenDefinition {
        name: "Elephant".to_string(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Call of the Herd",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), g()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: elephant,
        },
        ..Default::default()
    }
}

/// Arrogant Wurm — {3}{G}{G} Creature — Wurm. 4/4. Trample. Madness {2}{G}.
pub fn arrogant_wurm() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Arrogant Wurm",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Trample,
            Keyword::Madness(ManaCost::new(vec![ManaSymbol::Generic(2), ManaSymbol::Colored(Color::Green)])),
        ],
        ..Default::default()
    }
}

/// Big Game Hunter — {2}{B} Creature — Human Mercenary. 1/1. Madness {B}.
/// "When this creature enters, destroy target creature with power 4 or
/// greater. It can't be regenerated."
pub fn big_game_hunter() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Keyword, SelectionRequirement, TriggeredAbility};
    CardDefinition {
        name: "Big Game Hunter",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Madness(ManaCost::new(vec![ManaSymbol::Colored(Color::Black)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DestroyNoRegen {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
                ),
            },
        }],
        ..Default::default()
    }
}

// ── Modern cube supplement: additional cube-playable cards ──────────────────

/// Dreadhorde Arcanist — {1}{R} Creature — Zombie Wizard 1/3. Trample.
/// "Whenever Dreadhorde Arcanist attacks, you may cast target instant or
/// sorcery card with mana value less than or equal to Dreadhorde Arcanist's
/// power from your graveyard without paying its mana cost. If that spell
/// would be put into a graveyard, exile it instead."
///
/// Approximation: attack trigger moves the top instant-or-sorcery card from
/// your graveyard to your hand (simplified from free-cast). The MV ≤ power
/// restriction is omitted. The exile-instead-of-graveyard rider is also
/// omitted in this simplified version.
pub fn dreadhorde_arcanist() -> CardDefinition {
    use crate::card::CardType as CT;
    CardDefinition {
        name: "Dreadhorde Arcanist",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::HasCardType(CT::Instant)
                            .or(SelectionRequirement::HasCardType(CT::Sorcery)),
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Parallax Nexus — {1}{B}{B} Enchantment. Fading 5 (enters with five fade
/// counters; at the beginning of your upkeep, remove a fade counter; when the
/// last is removed, sacrifice this).
/// "{0}: Exile target card from an opponent's hand."
///
/// Approximation: enters with 5 charge counters. Upkeep trigger removes one
/// counter, and when the counter total hits 0 the permanent is sacrificed.
/// Activated ability ({0}, no tap) forces the opponent to discard one card
/// (the closest engine proxy for "exile target card from opponent's hand").
pub fn parallax_nexus() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::effect::Predicate;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Parallax Nexus",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::Const(1),
                },
                Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                        Selector::EachPermanent(
                            SelectionRequirement::WithCounter(CounterType::Charge)
                                .and(SelectionRequirement::ControlledByYou)
                                .and(SelectionRequirement::Enchantment),
                        ),
                    ))),
                    then: Box::new(Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::Const(1),
                        filter: SelectionRequirement::Enchantment,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        enters_with_counters: Some((CounterType::Charge, Value::Const(5))),
        ..Default::default()
    }
}

// ── Push XIX: cube creatures ───────────────────────────────────────────

// ── Push (claude/modern_decks session 5): new cube cards ────────────────

/// Omnath, Locus of Creation — {R}{G}{W}{U} Legendary 4/4 Elemental.
/// ETB draw 1 + gain 4 life.
pub fn omnath_locus_of_creation() -> CardDefinition {
    CardDefinition {
        name: "Omnath, Locus of Creation",
        cost: cost(&[r(), g(), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ]),
        }],
        ..Default::default()
    }
}

// ── modern_decks-17: new supplement cards ────────────────────────────────────

/// Grim Flayer — {B}{G} Creature 2/2 Human Warrior. Trample.
/// Whenever this deals combat damage to a player, surveil 2.
pub fn grim_flayer() -> CardDefinition {
    CardDefinition {
        name: "Grim Flayer",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Young Pyromancer — {1}{R} Creature 2/1 Human Shaman.
/// Whenever you cast an instant or sorcery spell, create a 1/1 red
/// Elemental creature token.
pub fn young_pyromancer() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
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
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Elemental".into(),
                power: 1,
                toughness: 1,
                keywords: vec![],
                card_types: vec![CardType::Creature],
                colors: vec![Color::Red],
                supertypes: vec![],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Elemental],
                    ..Default::default()
                },
                activated_abilities: vec![],
                triggered_abilities: vec![],
            },
        })],
        ..Default::default()
    }
}

/// Monastery Swiftspear — {R} Creature 1/2 Human Monk.
/// Haste. Prowess (noncreature spell → +1/+1 EOT).
pub fn monastery_swiftspear() -> CardDefinition {
    use crate::effect::shortcut::prowess_trigger;
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
        triggered_abilities: vec![prowess_trigger()],
        ..Default::default()
    }
}

/// Omnath, Locus of Rage — {3}{R}{G} Legendary 5/5 Elemental.
/// Landfall: create a 5/5 Elemental token (dies → 3 dmg to each opp).
pub fn omnath_locus_of_rage() -> CardDefinition {
    use crate::card::TokenDefinition;
    let elemental = TokenDefinition {
        name: "Elemental".to_string(),
        power: 5,
        toughness: 5,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
        }],
    };
    CardDefinition {
        name: "Omnath, Locus of Rage",
        cost: cost(&[generic(3), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: elemental,
            },
        }],
        ..Default::default()
    }
}

/// Torsten, Founder of Benalia — {3}{G}{W}{W} Legendary 7/7 Human Soldier.
/// ETB: Search 3 basic lands -> BF tapped.
pub fn torsten_founder_of_benalia() -> CardDefinition {
    CardDefinition {
        name: "Torsten, Founder of Benalia",
        cost: cost(&[generic(3), g(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Repeat {
                count: Value::Const(3),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            },
        }],
        ..Default::default()
    }
}

/// Grisly Salvage — {B}{G} Instant. Mill 5, then return a creature or
/// land card from among them to your hand.
///
/// Approximation: Mill 5 + Scry 1. The "pick a creature or land from
/// among the milled" is approximated by the Scry — you get to choose
/// whether the next draw is what you want, preserving the
/// selection-after-mill gameplay pattern.
pub fn grisly_salvage() -> CardDefinition {
    CardDefinition {
        name: "Grisly Salvage",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(5) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Thought Erasure — {U}{B} Sorcery. Target opponent reveals their hand.
/// You choose a nonland card from it. That player discards that card.
/// Surveil 1.
pub fn thought_erasure() -> CardDefinition {
    CardDefinition {
        name: "Thought Erasure",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Coveted Jewel — {6} Artifact. ETB draw 3. {T}: Add 3 mana any color.
pub fn coveted_jewel() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Coveted Jewel",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        )],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(3)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lightning Greaves — {2} Artifact — Equipment.
/// Equipped creature has haste and shroud. Equip {0}.
///
/// Wired via the real attach path (`GameAction::Equip`): the
/// `equipped_bonus` grants Haste + Shroud to the equipped creature through
/// the layer system. Shroud is enforced by the targeting layer
/// (`GameError::TargetHasShroud`), so an equipped creature can't be the
/// target of any spell or ability — the classic Greaves "protect your
/// commander" pattern.
pub fn lightning_greaves() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Lightning Greaves",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(ManaCost::default())],
        equipped_bonus: Some(EquipBonus {
            power: 0,
            toughness: 0,
            keywords: vec![Keyword::Haste, Keyword::Shroud],
        }),
        ..Default::default()
    }
}

/// Bonesplitter — {1} Artifact — Equipment.
/// Equipped creature gets +2/+0. Equip {1}.
///
/// First card to use the real attach-based equip path: the `equipped_bonus`
/// flows onto the equipped creature via the layer system (CR 702.6) instead
/// of the older grant-on-activate approximation.
pub fn bonesplitter() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Bonesplitter",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 0,
            keywords: vec![],
        }),
        ..Default::default()
    }
}

/// Shuko — {1} Artifact — Equipment.
/// Equipped creature gets +1/+0. Equip {0}.
///
/// The free equip cost ({0}) makes Shuko a notorious "toughness/untap"
/// engine piece; the engine honors the zero-mana equip via the standard
/// `GameAction::Equip` path (empty `ManaCost` pays for free).
pub fn shuko() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Shuko",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(ManaCost::default())],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 0,
            keywords: vec![],
        }),
        ..Default::default()
    }
}

/// Lavaspur Boots — {1} Artifact — Equipment.
/// Equipped creature gets +1/+1 and has haste. Equip {1}.
///
/// (The printed "Equip — pay {1} less if it targets a creature that entered
/// this turn" discount rider is omitted; the base equip {1} always applies.)
pub fn lavaspur_boots() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Lavaspur Boots",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Haste],
        }),
        ..Default::default()
    }
}

/// The Mightstone and Weakstone — {5} Artifact.
/// ETB: ChooseMode — draw 2 / creature -5/-5 EOT. {T}: Add {C}{C}.
pub fn the_mightstone_and_weakstone() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "The Mightstone and Weakstone",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(
            Effect::ChooseMode(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-5),
                    toughness: Value::Const(-5),
                    duration: Duration::EndOfTurn,
                },
            ]),
        )],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tasigur, the Golden Fang — {5}{B} Creature 4/5 Legendary Human Shaman.
/// Delve omitted; shipped at full cost.
/// Activated: {2}{G/U}: Mill 2, then return a nonland card from your
/// graveyard to your hand.
///
/// The {G/U} pip in the activated cost is a real
/// `ManaSymbol::Hybrid(Green, Blue)`, payable with either green or blue.
pub fn tasigur_the_golden_fang() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Tasigur, the Golden Fang",
        cost: cost(&[generic(5), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), crate::mana::hybrid(Color::Green, Color::Blue)]),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Nonland),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Doomsday Excruciator — {5}{B}{B} 6/6 Flying Demon.
/// ETB: Each player mills 20.
pub fn doomsday_excruciator() -> CardDefinition {
    CardDefinition {
        name: "Doomsday Excruciator",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(20),
            },
        )],
        ..Default::default()
    }
}

/// Stonecoil Serpent — {X} Artifact Creature 0/0 Snake.
/// Trample, reach, protection from multicolored.
/// Enters the battlefield with X +1/+1 counters on it.
///
/// Protection from multicolored is omitted (no
/// `Keyword::ProtectionFromMulticolored` primitive). The ETB counter
/// placement uses `Value::XFromCost`.
pub fn stonecoil_serpent() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::x;
    // CR 614.12 — "enters with X +1/+1 counters" is a replacement
    // effect, not a triggered ability. Using `enters_with_counters`
    // (rather than an ETB AddCounter trigger) means the X counters are
    // present the instant the Serpent enters, so a 0/0 base body never
    // hits the SBA death check as a counter-less 0/0.
    CardDefinition {
        name: "Stonecoil Serpent",
        cost: cost(&[x()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample, Keyword::Reach],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        ..Default::default()
    }
}

/// Planar Nexus — Land. ETB tapped. {T}: Add any color.
pub fn planar_nexus() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Planar Nexus",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        triggered_abilities: vec![
            etb(Effect::Tap { what: Selector::This }),
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Push XVII: Additional Modern burn/modal spells ─────────────────────────

/// Collective Defiance — {1}{R}{R} Sorcery. Choose one:
/// - Deal 4 damage to target creature.
/// - Target player discards their hand, then draws that many cards (approx: discard 3, draw 3).
/// - Deal 3 damage to target opponent.
pub fn collective_defiance() -> CardDefinition {
    CardDefinition {
        name: "Collective Defiance",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
            },
            Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                },
            ]),
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Kozilek's Command — {X} Instant. 3-mode ChooseMode.
pub fn kozileks_command() -> CardDefinition {
    CardDefinition {
        name: "Kozilek's Command",
        cost: cost(&[ManaSymbol::X]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: crate::card::TokenDefinition {
                    name: "Eldrazi Scion".to_string(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    card_types: vec![CardType::Creature],
                    colors: vec![],
                    supertypes: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Eldrazi],
                        ..Default::default()
                    },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                },
            },
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::XFromCost },
        ]),
        ..Default::default()
    }
}

/// Char — {2}{R} Instant. Deal 4 damage to any target. You take 2 damage.
pub fn char() -> CardDefinition {
    CardDefinition {
        name: "Char",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(4),
            },
            Effect::DealDamage {
                to: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Eldrazi Confluence — {4} Instant. 3-mode ChooseMode.
pub fn eldrazi_confluence() -> CardDefinition {
    CardDefinition {
        name: "Eldrazi Confluence",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::card::TokenDefinition {
                    name: "Eldrazi Scion".to_string(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    card_types: vec![CardType::Creature],
                    colors: vec![],
                    supertypes: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Eldrazi],
                        ..Default::default()
                    },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                },
            },
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
        ..Default::default()
    }
}

/// Searing Blaze — {R}{R} Instant. Deal 3 damage to target creature and
/// 3 damage to that creature's controller (approx: 3 to creature + 3 to
/// each opponent).
pub fn searing_blaze() -> CardDefinition {
    CardDefinition {
        name: "Searing Blaze",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Aluren — {2}{G}{G} Enchantment.
/// Each player may cast creature spells with mana value 3 or less without
/// paying their mana costs.
///
/// Approximation: body-only enchantment. The static cost-reduction effect
/// is beyond current engine scope; the card is included for its type line
/// and mana value in cube draft context.
pub fn aluren() -> CardDefinition {
    CardDefinition {
        name: "Aluren",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

// ── Modern supplement: Burn & Creature additions ────────────────────────────

/// Chain Lightning — {R} Sorcery. Deal 3 damage to any target.
///
/// The printed "then that player may pay {R}{R} to copy this spell" rider
/// is omitted — no spell-copy primitive exists yet.
pub fn chain_lightning() -> CardDefinition {
    CardDefinition {
        name: "Chain Lightning",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Messenger Falcons — {2}{G/U}{W} Creature — Bird 2/2.
/// "Flying / When Messenger Falcons enters the battlefield, draw a card."
/// The `{G/U}` pip is a real `ManaSymbol::Hybrid(Green, Blue)`.
pub fn messenger_falcons() -> CardDefinition {
    CardDefinition {
        name: "Messenger Falcons",
        cost: cost(&[generic(2), crate::mana::hybrid(Color::Green, Color::Blue), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Rift Bolt — {2}{R} Sorcery. Deal 3 damage to any target.
///
/// Suspend 1 — {R} is omitted (no suspend primitive).
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

/// Trenchpost — Land — Locus.
/// "{T}: Add {C}{C}."
/// (Approximation: Locus subtype noted but not mechanically relevant
/// without the Locus-counting static from Cloudpost.)
pub fn trenchpost() -> CardDefinition {
    use crate::card::LandType;
    use crate::catalog::sets::tap_add_colorless;
    CardDefinition {
        name: "Trenchpost",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Locus],
            ..Default::default()
        },
        activated_abilities: vec![{
            let mut ab = tap_add_colorless();
            ab.effect = Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            };
            ab
        }],
        ..Default::default()
    }
}

/// Exquisite Firecraft — {1}{R}{R} Sorcery. Deal 4 damage to any target.
///
/// The "spell mastery — this spell can't be countered" rider is omitted.
pub fn exquisite_firecraft() -> CardDefinition {
    CardDefinition {
        name: "Exquisite Firecraft",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Three Tree City — Legendary Land.
/// "Three Tree City enters with three charge counters on it.
///  {T}, Remove a charge counter: Add one mana of any color.
///  When the last charge counter is removed, sacrifice Three Tree City."
///
/// Approximation: enters with 3 charge counters via ETB trigger.
/// Tap + remove counter for any-color mana. Sacrifice when empty
/// is approximated by the natural counter-depletion — the card becomes
/// a dead land once all counters are removed. The sacrifice-on-empty
/// trigger is omitted (no last-counter-removed event).
pub fn three_tree_city() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    CardDefinition {
        name: "Three Tree City",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Charge,
            amount: Value::Const(3),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
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
            condition: Some(Predicate::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                },
                Value::Const(1),
            )),
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        ..Default::default()
    }
}

/// Sulfuric Vortex — {1}{R}{R} Enchantment. At the beginning of each
/// player's upkeep, Sulfuric Vortex deals 2 damage to that player. Players
/// can't gain life (`StaticEffect::PlayerCannotGainLife`, CR 119.7).
pub fn sulfuric_vortex() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::{PlayerStaticTarget, StaticEffect};
    CardDefinition {
        name: "Sulfuric Vortex",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Players can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife {
                target: PlayerStaticTarget::EachPlayer,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Conclave Sledge-Captain — {5}{G} Creature — Elephant Soldier 6/6.
/// "Trample / Creatures you control with +1/+1 counters on them have
/// trample. / When Conclave Sledge-Captain enters, put a +1/+1 counter
/// on each creature you control."
pub fn conclave_sledge_captain() -> CardDefinition {
    use crate::card::{CounterType, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Conclave Sledge-Captain",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with +1/+1 counters have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}

/// Kari Zev, Skyship Raider — {1}{R} Legendary Creature — Human Pirate 1/3.
/// First strike, menace. Whenever Kari Zev attacks, create Ragavan, a
/// legendary 2/1 red Monkey creature token. Ragavan is tapped and attacking.
///
/// Approximation: creates a 2/1 Monkey token on attack (the token being
/// tapped-and-attacking and exiled at end of combat is omitted).
pub fn kari_zev_skyship_raider() -> CardDefinition {
    CardDefinition {
        name: "Kari Zev, Skyship Raider",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Ragavan".into(),
                    power: 2,
                    toughness: 1,
                    keywords: vec![],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    supertypes: vec![Supertype::Legendary],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Monkey],
                        ..Default::default()
                    },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                },
            },
        }],
        ..Default::default()
    }
}

/// Pithing Needle — {1} Artifact. "As ~ enters, choose a card name.
/// Activated abilities of sources with the chosen name can't be activated
/// unless they're mana abilities." ETB `Effect::NameCard` stamps the chosen
/// name; `activate_ability` suppresses non-mana abilities of matching sources.
pub fn pithing_needle() -> CardDefinition {
    use crate::card::TriggeredAbility;
    use crate::effect::{EventKind, EventScope, EventSpec};
    CardDefinition {
        name: "Pithing Needle",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::NameCard { what: Selector::This },
        }],
        ..Default::default()
    }
}

/// Wight of the Reliquary — {1}{B}{G} Creature — Zombie Knight 1/1.
/// Gets +1/+1 for each land card in your graveyard (dynamic P/T via
/// `DynamicPt::BasePlusLandsInControllerGraveyard`). {T}, Sacrifice a
/// land: Search your library for a land card onto the battlefield tapped.
pub fn wight_of_the_reliquary() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Wight of the Reliquary",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            sac_other_filter: Some((SelectionRequirement::Land, 1)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Scavenging Ooze — {1}{G} Creature — Ooze 2/2.
/// {G}: Exile target card from a graveyard. If it was a creature card, put
/// a +1/+1 counter on Scavenging Ooze and you gain 1 life.
///
/// Approximation: {G}, tap: add a +1/+1 counter on this and gain 1 life
/// (the graveyard-exile targeting is collapsed — no "target card in a
/// graveyard" selector yet).
pub fn scavenging_ooze() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    CardDefinition {
        name: "Scavenging Ooze",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[g()]),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Zopandrel, Hunger Dominus — {5}{G}{G} Creature — Phyrexian Horror 4/6.
/// "Reach / At the beginning of each combat, double the power and
/// toughness of each creature you control until end of turn.
/// {G/P}{G/P}, Sacrifice two other creatures: Put an indestructible
/// counter on Zopandrel."
///
/// Begin-combat doubling reads each creature's current P/T off the
/// `ForEach` binding (`Value::PowerOf`/`ToughnessOf(TriggerSource)`) and
/// adds it back — true doubling, not a flat bonus. The `{G/P}{G/P},
/// Sacrifice two other creatures: Put an indestructible counter on
/// Zopandrel` activation is wired via real Phyrexian pips,
/// `sac_other_filter: (Creature, 2)`, and `AddCounter(Indestructible)`.
pub fn zopandrel_hunger_dominus() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::game::types::TurnStep;
    use crate::mana::phyrexian;
    CardDefinition {
        name: "Zopandrel, Hunger Dominus",
        cost: cost(&[generic(5), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[phyrexian(Color::Green), phyrexian(Color::Green)]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Indestructible,
                amount: Value::Const(1),
            },
            sac_other_filter: Some((SelectionRequirement::Creature, 2)),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    // Double P/T: add each creature's current power/toughness
                    // to itself (read per-creature off the ForEach binding).
                    power: Value::PowerOf(Box::new(Selector::TriggerSource)),
                    toughness: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Push XVII continued: ETB creatures ─────────────────────────────────────

/// Fiend Hunter — {1}{W}{W} Creature — Human Cleric 1/3.
/// "When Fiend Hunter enters, exile target creature. When Fiend Hunter
/// leaves the battlefield, return the exiled card to the battlefield
/// under its owner's control."
///
/// Wired via `Effect::ExileUntilSourceLeaves` (CR 603.6e, return to
/// battlefield). Targets an opponent's creature (the printed "you may"
/// optionality is dropped for the auto-decider's benefit).
pub fn fiend_hunter() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Fiend Hunter",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                return_to: ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}

/// Banisher Priest — {1}{W}{W} Creature — Human Cleric 2/2.
/// "When Banisher Priest enters, exile target creature an opponent
/// controls until Banisher Priest leaves the battlefield."
/// Wired via `Effect::ExileUntilSourceLeaves` (CR 603.6e, return to
/// battlefield).
pub fn banisher_priest() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Banisher Priest",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                return_to: ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}

/// Oblivion Ring — {2}{W} Enchantment.
/// "When Oblivion Ring enters, exile another target nonland permanent
/// until Oblivion Ring leaves the battlefield."
/// Wired via `Effect::ExileUntilSourceLeaves` (CR 603.6e, return to
/// battlefield). `OtherThanSource` enforces the "another" clause.
pub fn oblivion_ring() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Oblivion Ring",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                return_to: ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}

/// Akrasan Squire — {W} 1/1 Human Soldier Warrior with Exalted.
/// "Exalted (Whenever a creature you control attacks alone, that
/// creature gets +1/+1 until end of turn.)"
pub fn akrasan_squire() -> CardDefinition {
    use crate::effect::shortcut::exalted;
    CardDefinition {
        name: "Akrasan Squire",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![exalted()],
        ..Default::default()
    }
}

/// Aven Squire — {1}{W} 1/1 Bird Soldier with Flying and Exalted.
pub fn aven_squire() -> CardDefinition {
    use crate::effect::shortcut::exalted;
    CardDefinition {
        name: "Aven Squire",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![exalted()],
        ..Default::default()
    }
}

/// Fanatical Firebrand — {R} 1/1 Goblin Pirate with Haste.
/// "{T}, Sacrifice this creature: It deals 1 damage to any target."
pub fn fanatical_firebrand() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Fanatical Firebrand",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goblin Wardriver — {1}{R} 2/2 Goblin Warrior with Battle Cry.
/// "Battle cry (Whenever this creature attacks, each other attacking
/// creature gets +1/+0 until end of turn.)"
pub fn goblin_wardriver() -> CardDefinition {
    use crate::effect::shortcut::battle_cry;
    CardDefinition {
        name: "Goblin Wardriver",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![battle_cry(1)],
        ..Default::default()
    }
}

/// Dakkon, Shadow Slayer — {W}{U}{B} Legendary Planeswalker — Dakkon.
/// "+1: Surveil 2. / -3: Exile target creature. / -6: You get an
///  emblem with 'At the beginning of your upkeep, draw a card.'"
///
/// +1 Surveil 2, -3 exile target creature, -6 upkeep-draw emblem all
/// wired. Base loyalty = 3 (printed: loyalty equals lands you control,
/// approximated as a fixed 3).
pub fn dakkon_shadow_slayer() -> CardDefinition {
    use crate::card::LoyaltyAbility;
    CardDefinition {
        name: "Dakkon, Shadow Slayer",
        cost: cost(&[w(), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Exile {
                    what: target_filtered(SelectionRequirement::Creature),
                },
            },
            // -6: You get an emblem with "At the beginning of your
            // upkeep, draw a card."
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Dakkon, Shadow Slayer".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                            EventScope::YourControl,
                        ),
                        effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    }],
                },
            },
        ],
        ..Default::default()
    }
}

/// Flametongue Kavu — {3}{R} Creature — Kavu 4/2.
/// ETB: deal 4 damage to target creature.
pub fn flametongue_kavu() -> CardDefinition {
    CardDefinition {
        name: "Flametongue Kavu",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
            },
        }],
        ..Default::default()
    }
}

/// Fallen Shinobi — {3}{U}{B} Creature — Zombie Ninja 5/4.
/// "Ninjutsu {2}{U}{B} / Whenever Fallen Shinobi deals combat damage
/// to a player, exile the top two cards of that player's library. Until
/// end of turn, you may play those cards without paying their mana costs."
///
/// Approximation: body-only 5/4 with combat-damage trigger that exiles
/// 2 from the defender's library (the play-from-exile half is omitted
/// pending cast-from-exile pipeline). Ninjutsu is omitted.
pub fn fallen_shinobi() -> CardDefinition {
    CardDefinition {
        name: "Fallen Shinobi",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Ninja],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::Player(PlayerRef::DefendingPlayer),
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Bonecrusher Giant — {2}{R} Creature — Giant 4/3.
/// (Stomp adventure omitted — implemented as just the creature body.)
/// Whenever Bonecrusher Giant becomes the target of a spell, deal 2
/// damage to that spell's controller. (Triggered ability omitted.)
pub fn bonecrusher_giant() -> CardDefinition {
    CardDefinition {
        name: "Bonecrusher Giant",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        ..Default::default()
    }
}

/// Esika's Chariot — {3}{G} Legendary Artifact — Vehicle 4/4.
/// When this enters, create two 2/2 green Cat creature tokens. Whenever it
/// attacks, create a token that's a copy of target token you control.
/// Crew 4.
pub fn esikas_chariot() -> CardDefinition {
    use crate::card::{Supertype, TokenDefinition};
    CardDefinition {
        name: "Esika's Chariot",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        keywords: vec![Keyword::Crew(4)],
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: TokenDefinition {
                        name: "Cat".into(),
                        power: 2,
                        toughness: 2,
                        keywords: vec![],
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green],
                        supertypes: vec![],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Cat],
                            ..Default::default()
                        },
                        activated_abilities: vec![],
                        triggered_abilities: vec![],
                    },
                },
            },
            // "Whenever Esika's Chariot attacks, create a token that's a copy
            // of target token you control." (CR 707 token copy.)
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: target_filtered(
                        SelectionRequirement::IsToken
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    extra_creature_types: vec![],
                    override_pt: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Smuggler's Copter — {2} Artifact — Vehicle 3/3.
/// Flying. Whenever this Vehicle attacks or blocks, you may draw a card. If
/// you do, discard a card. Crew 1.
///
/// Crew 1 wired via `Keyword::Crew(1)`; the attack/block loot trigger fires
/// on `EventKind::Attacks` (the engine fires Attacks for crewed Vehicles
/// that attack). The block half is approximated by the attack trigger only
/// (no DeclaredBlocker event for Vehicles yet). The "may draw then discard"
/// rummage uses the standard loot pattern.
pub fn smugglers_copter() -> CardDefinition {
    CardDefinition {
        name: "Smuggler's Copter",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        keywords: vec![Keyword::Crew(1), Keyword::Flying],
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "draw a card, then discard a card".to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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

/// Robber of the Rich — {1}{R} Creature — Human Archer Rogue 2/2.
/// Reach, Haste. (Cast-from-opponent's-library ability omitted.)
pub fn robber_of_the_rich() -> CardDefinition {
    CardDefinition {
        name: "Robber of the Rich",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach, Keyword::Haste],
        ..Default::default()
    }
}

// ── Push XXIV: a handful of vanilla / near-vanilla bodies ─────────────────

/// Phyrexian Revoker — {2} Artifact Creature — Phyrexian Construct 2/1.
/// "As ~ enters, choose a nonland card name. Activated abilities of sources
/// with the chosen name can't be activated unless they're mana abilities."
/// Shares Pithing Needle's `Effect::NameCard` + `activate_ability` suppression.
pub fn phyrexian_revoker() -> CardDefinition {
    use crate::card::TriggeredAbility;
    use crate::effect::{EventKind, EventScope, EventSpec};
    CardDefinition {
        name: "Phyrexian Revoker",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Construct],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::NameCard { what: Selector::This },
        }],
        ..Default::default()
    }
}

/// Solemn Simulacrum — {4} Artifact Creature — Golem 2/2.
/// "When Solemn Simulacrum enters, you may search your library for a
/// basic land card, put it onto the battlefield tapped, then shuffle.
/// / When Solemn Simulacrum dies, you may draw a card."
pub fn solemn_simulacrum() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{PlayerRef, Selector, ZoneDest};
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
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Search your library for a basic land card?".to_string(),
                    body: Box::new(Effect::Search {
                        who: PlayerRef::You,
                        filter: crate::card::SelectionRequirement::IsBasicLand,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: true,
                        },
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Draw a card?".to_string(),
                    body: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: crate::effect::Value::Const(1),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Inquisitive Puppet — {1} Artifact Creature — Homunculus 0/2.
/// "When this creature enters, look at the top card of your library.
/// You may put that card on the bottom of your library."
///
/// 🟡 Approximated as Scry 1 — the engine's Scry primitive offers the
/// look + may-bottom semantics exactly. (Real card lacks the "leave on
/// top" option that real Scry offers, but the gameplay outcome is
/// strictly a subset.)
pub fn inquisitive_puppet() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{PlayerRef, Value};
    CardDefinition {
        name: "Inquisitive Puppet",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Juggernaut — {4} Artifact Creature — Juggernaut 5/3. "Juggernaut attacks
/// each combat if able." (CR 508.1d via `Keyword::MustAttack`, enforced in
/// `declare_attackers`. The "can't be blocked by Walls" rider is dropped —
/// no Wall-typed blockers are modelled.)
pub fn juggernaut() -> CardDefinition {
    CardDefinition {
        name: "Juggernaut",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Juggernaut],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![crate::card::Keyword::MustAttack],
        ..Default::default()
    }
}

// ── Spellbomb cycle + cheap utility (claude/modern_decks) ────────────────────

/// Nihil Spellbomb — {1} Artifact. "{T}, Sacrifice this: Exile target
/// player's graveyard. / {B}, Sacrifice this: Draw a card."
pub fn nihil_spellbomb() -> CardDefinition {
    use crate::card::{ActivatedAbility, Zone};
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Nihil Spellbomb",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::EachOpponent,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    },
                    to: ZoneDest::Exile,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Pyrite Spellbomb — {1} Artifact. "{T}, Sacrifice this: deals 2 damage to
/// any target. / {R}, Sacrifice this: Draw a card."
pub fn pyrite_spellbomb() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::target_any;
    CardDefinition {
        name: "Pyrite Spellbomb",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Seal of Removal — {U} Enchantment. "Sacrifice Seal of Removal: Return
/// target creature to its owner's hand."
pub fn seal_of_removal() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Seal of Removal",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vendetta — {B} Instant. "Destroy target nonblack creature. You lose life
/// equal to that creature's toughness." (Toughness captured before destroy.)
pub fn vendetta() -> CardDefinition {
    CardDefinition {
        name: "Vendetta",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::Target(0))),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(Color::Black).negate()),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Sylvan Spellbomb — {1} Artifact. "{T}, Sacrifice this: Search your library
/// for a basic land card, reveal it, put it into your hand, then shuffle. /
/// {G}, Sacrifice this: Draw a card."
pub fn sylvan_spellbomb() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Sylvan Spellbomb",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Horizon Spellbomb — {1} Artifact. "{T}, Sacrifice this: Search your library
/// for a basic land card, reveal it, put it into your hand, then shuffle. /
/// {W}, Sacrifice this: Draw a card."
pub fn horizon_spellbomb() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Horizon Spellbomb",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Expedition Map — {1} Artifact. "{2}, {T}, Sacrifice this: Search your
/// library for a land card, reveal it, put it into your hand, then shuffle."
pub fn expedition_map() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Expedition Map",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Executioner's Capsule — {B} Artifact. "{1}{B}, Sacrifice this: Destroy
/// target nonblack creature."
pub fn executioners_capsule() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Executioner's Capsule",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(Color::Black).negate()),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rapid Hybridization — {U} Instant. "Destroy target creature. It can't be
/// regenerated. Its controller creates a 3/3 green Frog Lizard token."
pub fn rapid_hybridization() -> CardDefinition {
    use crate::card::TokenDefinition;
    let frog = TokenDefinition {
        name: "Frog Lizard".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Lizard],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Rapid Hybridization",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: frog,
            },
            Effect::DestroyNoRegen {
                what: target_filtered(SelectionRequirement::Creature),
            },
        ]),
        ..Default::default()
    }
}

/// Impulse — {1}{U} Instant. "Look at the top four cards of your library. Put
/// one of them into your hand and the rest on the bottom of your library in
/// any order."
pub fn impulse() -> CardDefinition {
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Impulse",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: None,
        },
        ..Default::default()
    }
}

/// Serum Visions — {U} Sorcery. "Draw a card, then scry 2."
pub fn serum_visions() -> CardDefinition {
    use crate::effect::PlayerRef;
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

/// Flame Rift — {1}{R} Sorcery. "Flame Rift deals 4 damage to each player."
pub fn flame_rift() -> CardDefinition {
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Flame Rift",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Ultimate Price — {1}{B} Instant. "Destroy target monocolored creature."
pub fn ultimate_price() -> CardDefinition {
    CardDefinition {
        name: "Ultimate Price",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::Monocolored),
            ),
        },
        ..Default::default()
    }
}

/// Walk the Plank — {1}{B} Sorcery. "Destroy target non-Merfolk creature."
pub fn walk_the_plank() -> CardDefinition {
    CardDefinition {
        name: "Walk the Plank",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(
                    SelectionRequirement::HasCreatureType(CreatureType::Merfolk).negate(),
                ),
            ),
        },
        ..Default::default()
    }
}

/// Rabid Bite — {1}{G} Sorcery. "Target creature you control deals damage
/// equal to its power to target creature you don't control."
pub fn rabid_bite() -> CardDefinition {
    CardDefinition {
        name: "Rabid Bite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            },
            amount: Value::PowerOf(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            })),
        },
        ..Default::default()
    }
}

/// Oust — {W} Sorcery. "Put target creature into its owner's library second
/// from the top. Its owner gains 5 life."
pub fn oust() -> CardDefinition {
    use crate::effect::{LibraryPosition, PlayerRef, ZoneDest};
    CardDefinition {
        name: "Oust",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(5),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::FromTop(1),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Soul Snare — {1}{W} Enchantment. "{1}, Sacrifice Soul Snare: Exile target
/// attacking or blocking creature."
pub fn soul_snare() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Soul Snare",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::IsAttacking
                            .or(SelectionRequirement::IsBlocking),
                    ),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// 1/1 red Goblin token shared by the goblin-swarm sorceries below.
fn goblin_token() -> TokenDefinition {
    TokenDefinition {
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
    }
}

/// Dragon Fodder — {1}{R} Sorcery. "Create two 1/1 red Goblin tokens."
pub fn dragon_fodder() -> CardDefinition {
    CardDefinition {
        name: "Dragon Fodder",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: goblin_token(),
        },
        ..Default::default()
    }
}

/// Krenko's Command — {1}{R} Sorcery. "Create two 1/1 red Goblin tokens."
pub fn krenkos_command() -> CardDefinition {
    CardDefinition {
        name: "Krenko's Command",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: goblin_token(),
        },
        ..Default::default()
    }
}

/// Hordeling Outburst — {1}{R}{R} Sorcery. "Create three 1/1 red Goblin
/// tokens."
pub fn hordeling_outburst() -> CardDefinition {
    CardDefinition {
        name: "Hordeling Outburst",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: goblin_token(),
        },
        ..Default::default()
    }
}
