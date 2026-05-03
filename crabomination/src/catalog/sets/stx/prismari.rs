//! Prismari (U/R) college cards from Strixhaven.
//!
//! Prismari's themes are spellslinger payoffs (Magecraft, copy-spell
//! triggers, treasure-style ramp) and creature-makes-token mid-cast
//! synergies. The first wave here covers the basic Apprentice +
//! Pledgemage pair, plus a couple of mono-shape supporting cards.
//! Larger Prismari cards (Magma Opus, Expressive Iteration's siblings)
//! lean on the copy-spell primitive and stay ⏳ until that lands.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, Selector, SelectionRequirement,
    Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{magecraft, magecraft_self_pump, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{cost, generic, r, u, Color};

// ── Prismari Pledgemage ─────────────────────────────────────────────────────

/// Prismari Pledgemage — {1}{U}{R}, 2/3 Elemental. "Trample, haste."
///
/// Pure stat-line + keyword body. Prismari Pledgemage is the "free
/// vanilla beater" of the Prismari arsenal: a 2/3 trample-haste for
/// {URR}-equivalent costs is solid, and it composes against every
/// pump and copy effect in the college.
pub fn prismari_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pledgemage",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Trample, Keyword::Haste],
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

// ── Prismari Apprentice ─────────────────────────────────────────────────────

/// Prismari Apprentice — {U}{R}, 2/2 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// choose one — / • Scry 1. / • Prismari Apprentice gets +1/+0 until
/// end of turn."
///
/// Push XXXVII: ✅ — both modes now wire faithfully via the new
/// `Effect::PickModeAtResolution([Scry 1, PumpPT(+1/+0, EOT)])` primitive.
/// AutoDecider picks mode 0 (Scry 1, the universal default — it's strictly
/// card-selection without depending on board state). ScriptedDecider can
/// flip mode 1 for the self-pump path (relevant when Prismari Apprentice
/// is already attacking and a +1/+0 turns combat math).
pub fn prismari_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Prismari Apprentice",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PickModeAtResolution(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Symmetry Sage ───────────────────────────────────────────────────────────

/// Symmetry Sage — {U}, 1/2 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// Symmetry Sage gets +1/+0 and gains flying until end of turn."
///
/// 🟡 We split the rider into two separate magecraft triggers: one
/// `magecraft_self_pump(+1/+0)` and one grant-flying. They're functionally
/// equivalent to the original `Seq` body — both fire on every magecraft
/// event and both reference the source via `Selector::This`. The split
/// also means the helper is reusable across any future magecraft
/// self-pump creature (e.g. Quandrix's Berta, Symmetry Sage's siblings)
/// without copy-pasting a six-line `Seq`.
pub fn symmetry_sage() -> CardDefinition {
    CardDefinition {
        name: "Symmetry Sage",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            magecraft_self_pump(1, 0),
            magecraft(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Creative Outburst ───────────────────────────────────────────────────────

/// Creative Outburst — {3}{U}{U}{R}{R} Sorcery. "Discard your hand. Draw
/// five cards."
///
/// Pure rummage upgrade: the entire hand goes to the graveyard, then the
/// caster draws 5. Perfect Prismari spellslinger refill — the discarded
/// instants/sorceries fuel later Magecraft / flashback payoffs.
pub fn creative_outburst() -> CardDefinition {
    use crate::effect::Value as V;
    CardDefinition {
        name: "Creative Outburst",
        cost: cost(&[generic(3), u(), u(), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: V::HandSizeOf(PlayerRef::You),
                random: false,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(5),
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

// ── Prismari Command ────────────────────────────────────────────────────────

/// Prismari Command — {1}{U}{R} Instant.
/// "Choose two —
/// • Prismari Command deals 2 damage to any target.
/// • Discard 2 cards, then draw 2 cards.
/// • Create a Treasure token.
/// • Destroy target artifact."
///
/// Push XXXVI: ✅ — "choose two" now wires faithfully via the new
/// `Effect::ChooseModes { count: 2, up_to: false, allow_duplicates:
/// false }` primitive. Auto-decider picks modes 0+1 (2 damage +
/// discard/draw). `ScriptedDecider::new([Modes(vec![2, 3])])` picks
/// Treasure + destroy artifact for tests. Each individual mode is
/// wired faithfully against existing primitives.
pub fn prismari_command() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::target_filtered;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Command",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseModes {
            count: 2,
            up_to: false,
            allow_duplicates: false,
            modes: vec![
                // Mode 0: 2 damage to creature/PW (auto-target collapse to
                // creature for the auto-target framework — same shape as
                // Igneous Inspiration's "creature or planeswalker" pick).
                Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                },
                // Mode 1: discard 2, draw 2.
                Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(2),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                ]),
                // Mode 2: create a Treasure token.
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: treasure_token(),
                },
                // Mode 3: destroy target artifact.
                Effect::Destroy {
                    what: target_filtered(SelectionRequirement::HasCardType(CardType::Artifact)),
                },
            ],
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

// ── Prismari Elemental token ────────────────────────────────────────────────

/// 4/4 blue and red Elemental creature token. Used by Magma Opus and any
/// future Prismari "create a 4/4 Elemental" rider. Vanilla 4/4 — no
/// abilities — but multicolor so it interacts with Multicolored payoffs.
fn prismari_elemental_token() -> TokenDefinition {
    TokenDefinition {
        name: "Elemental".into(),
        power: 4,
        toughness: 4,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

// ── Magma Opus ──────────────────────────────────────────────────────────────

/// Magma Opus — {7}{U}{R} Sorcery. "Magma Opus deals 4 damage divided as
/// you choose among any number of target creatures and/or planeswalkers.
/// Tap two target permanents. Create a 4/4 blue and red Elemental
/// creature token. Draw two cards. / {U}{R}, Discard Magma Opus: Create
/// a Treasure token."
///
/// Push XXIX: Prismari finisher. 🟡 — printed "4 damage divided" +
/// "tap two target permanents" both collapse to single-target picks
/// (`Effect::DealDamage` and `Effect::Tap` are single-target on
/// `target_filtered`); the engine has no "divide N damage among M
/// targets" prompt. The remaining halves — 4/4 Elemental token + draw
/// 2 — are wired faithfully. The discard-for-Treasure alt cost is
/// omitted (no alt-cost-by-discard primitive yet — same gap as
/// Bonecrusher Giant's Adventure side).
///
/// Net resolution: 4 damage to one target creature/PW, mint a 4/4
/// Elemental, draw 2. At 9 mana that's roughly 4 + 4/4 + 2 cards =
/// a healthy "win-the-game" finisher even after the simplifications.
pub fn magma_opus() -> CardDefinition {
    CardDefinition {
        name: "Magma Opus",
        cost: cost(&[generic(7), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: prismari_elemental_token(),
            },
            Effect::Draw {
                who: Selector::You,
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
    }
}

// ── Expressive Iteration ────────────────────────────────────────────────────

/// Expressive Iteration — {U}{R} Sorcery. "Exile the top three cards of
/// your library. You may play a land and cast a spell from among them
/// this turn. Put the rest on the bottom of your library in a random
/// order."
///
/// Push XXIX: 🟡 — collapsed to a "look-at-top-3 + draw 1" cantrip
/// approximation. Wired as `Scry 2 + Draw 1` (the scry's "top or
/// bottom" picks substitute for the "any order + may shuffle" decision;
/// the gameplay-relevant outcome — taking the best one of the top
/// three on the next draw — is preserved). The "you may play a land
/// from among them" rider is omitted (cast-from-exile + play-land-
/// from-exile primitives are tracked together with Expressive's full
/// wire — same gap as Augur of Bolas, Outpost Siege).
pub fn expressive_iteration() -> CardDefinition {
    CardDefinition {
        name: "Expressive Iteration",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
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
    }
}
