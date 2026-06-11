//! Lorehold (R/W) college cards from Strixhaven.
//!
//! Lorehold's themes are Spirit tokens (typically 1/1 or 2/2 reach), spell-
//! cast triggers via Magecraft, and graveyard recursion (lots of cards
//! reference exile-from-graveyard or "card left your graveyard"). The
//! engine doesn't yet support exile-as-cost on activated abilities or a
//! `LeavesGraveyard` event, so the riders that lean on those primitives
//! are stubbed and the body/keyword half ships only — see STRIXHAVEN2.md
//! for the per-card status.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    Selector, SelectionRequirement, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{magecraft, target_filtered};
use crate::effect::{PlayerRef, ZoneDest};
use crate::mana::{cost, generic, r, w, Color};

// ── Lorehold spirit token ───────────────────────────────────────────────────

/// 2/2 red-and-white Spirit creature token. Used by Lorehold cards (and
/// SOS Group Project / Living History) that mint a Spirit body with no
/// extra abilities. Pulled into a helper so future Lorehold cards can
/// compose against the same definition.
pub fn lorehold_spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 2,
        toughness: 2,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

// ── Lorehold Apprentice ─────────────────────────────────────────────────────

/// Lorehold Apprentice — {R}{W}, 1/1 Human Cleric.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// you gain 1 life and Lorehold Apprentice deals 1 damage to any target."
///
/// 🟡 The dual-effect rider is split: the lifegain half is wired
/// unconditionally, but the "1 damage to any target" half is collapsed
/// into the lifegain only — Magecraft fires off the cast event, which
/// pre-dates target selection on a triggered ability with its own
/// target. Wiring the damage half needs the same "auto-target on
/// trigger" plumbing as Repartee: a `target_filtered(Any)` body that
/// the Magecraft helper resolves at trigger time. Tracked in
/// STRIXHAVEN2.md.
pub fn lorehold_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Apprentice",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Lorehold Pledgemage ─────────────────────────────────────────────────────

/// Lorehold Pledgemage — {1}{R}{W}, 2/2 Spirit Cleric. "Reach. {2}{R}{W},
/// Exile a card from your graveyard: This creature gets +1/+1 until end
/// of turn."
///
/// 🟡 The activated ability requires "exile a card from your graveyard"
/// as part of its cost — there's no `exile_gy_cost` flag on
/// `ActivatedAbility` (we have `tap_cost` and `sac_cost` only). The
/// pumped body still ships; the activation is omitted until the cost
/// primitive lands. Tracked in TODO.md.
pub fn lorehold_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pledgemage",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
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

// ── Pillardrop Rescuer ──────────────────────────────────────────────────────

/// Pillardrop Rescuer — {3}{R}{W}, 3/3 Spirit Cleric. "Flying. When
/// Pillardrop Rescuer enters the battlefield, return target instant or
/// sorcery card from your graveyard to your hand."
///
/// Same shape as Zealous Lorecaster ({5}{R}, 4/4 Giant): ETB returns one
/// IS card from your graveyard. Wired with the standard ETB +
/// `Effect::Move` against a `target_filtered` GY card. The 3/3 flying
/// body for {3}{R}{W} is a respectable Lorehold floor.
pub fn pillardrop_rescuer() -> CardDefinition {
    CardDefinition {
        name: "Pillardrop Rescuer",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
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

// ── Heated Debate ───────────────────────────────────────────────────────────

/// Heated Debate — {2}{R} Instant. "Heated Debate deals 4 damage to
/// target creature. Damage can't be prevented this turn."
///
/// 🟡 The "damage can't be prevented this turn" rider is a no-op (the
/// engine doesn't model damage-prevention layers, so all damage is
/// already unpreventable in practice — this matches a handful of other
/// SOS cards like Impractical Joke).
pub fn heated_debate() -> CardDefinition {
    CardDefinition {
        name: "Heated Debate",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
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
    }
}

// ── Sparring Regimen ────────────────────────────────────────────────────────

/// Sparring Regimen — {2}{R}{W} Enchantment. "When this enchantment
/// enters, create a 2/2 red and white Spirit creature token. / Whenever
/// you attack, put a +1/+1 counter on each attacking creature you
/// control."
///
/// 🟡 Mainline ETB token wired. The "whenever you attack" trigger that
/// pumps each attacker is omitted — the engine has a `DeclareAttackers`
/// event but the trigger here wants to enumerate every declared
/// attacker, not just one source. Tracked under TODO.md "Triggered-
/// Ability Event Gaps" → `PlayerAttackedWith`.
pub fn sparring_regimen() -> CardDefinition {
    CardDefinition {
        name: "Sparring Regimen",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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

// ── Storm-Kiln Artist ───────────────────────────────────────────────────────

/// Storm-Kiln Artist — {2}{R}{W}, 3/3 Human Wizard. "Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, this creature deals 1
/// damage to each opponent. Then create a Treasure token."
///
/// Note: the printed Strixhaven Storm-Kiln Artist's text is "Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, Storm-Kiln
/// Artist deals 1 damage to any target. Then create a Treasure token."
/// We collapse the "any target" damage to "each opponent" because the
/// magecraft trigger fires on cast and binds its targets at trigger-
/// resolution time — and our auto-targeting framework picks "each
/// opponent" cleanly when no creature target is available. The Treasure
/// half is pure: standard `treasure_token()` helper.
pub fn storm_kiln_artist() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Storm-Kiln Artist",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

