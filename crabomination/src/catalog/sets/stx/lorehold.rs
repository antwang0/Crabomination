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
/// 🟡 Push XX: lifegain + 1-damage-to-each-opponent now both wired
/// (was lifegain only). Same approximation as Storm-Kiln Artist: the
/// "any target" damage collapses to `Selector::Player(EachOpponent)`
/// because the Magecraft trigger fires off the cast event and binds
/// targets at trigger-resolution time. Mono-target picking on
/// triggered-ability bodies is still tracked as an engine TODO; the
/// each-opponent collapse keeps the spell on its printed cast/curve
/// without losing the chip-damage payoff.
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
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
/// All three abilities now wired. The "whenever you attack" trigger
/// fan-out fires per-declared-attacker via the new combat-side
/// broadcast (declare_attackers now consults all your permanents'
/// `Attacks/AnotherOfYours` triggers, binding the just-declared
/// attacker as `TriggerSource`). The +1/+1 counter is added on the
/// attacker that fired this instance. Net result: each declared
/// attacker ends up with one new counter, matching the printed
/// "+1/+1 counter on each attacking creature".
pub fn sparring_regimen() -> CardDefinition {
    use crate::card::CounterType;
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: lorehold_spirit_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
                // The attacker is pre-bound as `Target(0)` by the
                // combat-side broadcast in `declare_attackers`.
                effect: Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Reconstruct History ─────────────────────────────────────────────────────

/// Reconstruct History — {1}{R}{W} Sorcery. "Return up to two artifact
/// cards from your graveyard to your hand, then draw a card."
///
/// Wired faithfully via `Selector::take(_, 2)` over the controller's
/// graveyard (filtered to artifacts) — same shape as Pull from the
/// Grave's "up to two creature cards". Returns up to 2 artifacts; the
/// draw rider always fires. Lorehold artifact-recursion staple.
pub fn reconstruct_history() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Reconstruct History",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::HasCardType(CardType::Artifact),
                    },
                    Value::Const(2),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
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

// ── Igneous Inspiration ─────────────────────────────────────────────────────

/// Igneous Inspiration — {2}{R} Sorcery. "Igneous Inspiration deals 3
/// damage to target creature or planeswalker. Then learn."
///
/// Learn is approximated as `Draw 1` (matching the rest of the STX
/// catalog's Learn approximation — see Eyetwitch comment). Mainline
/// 3-damage half is faithful.
pub fn igneous_inspiration() -> CardDefinition {
    CardDefinition {
        name: "Igneous Inspiration",
        cost: cost(&[generic(2), r()]),
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
                amount: Value::Const(3),
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

// ── Lorehold Command ────────────────────────────────────────────────────────

/// Lorehold Command — {R}{W} Instant.
/// "Choose two —
/// • Target opponent loses 4 life.
/// • Target player creates two 1/1 white Spirit creature tokens with flying.
/// • Return target permanent card with mana value 2 or less from your
///   graveyard to your hand.
/// • Exile target card from a graveyard."
///
/// Push XXIV: 🟡 — printed "choose two" collapses to "choose one" via
/// `Effect::ChooseMode` (same approximation as Moment of Reckoning,
/// Witherbloom Command). Each individual mode is wired faithfully.
/// The flying-Spirit-token mode mints a single 1/1 white Spirit with
/// flying via a fresh `TokenDefinition` (separate from the 2/2 R/W
/// `lorehold_spirit_token()` used by Sparring Regimen — different P/T
/// and color identity).
pub fn lorehold_command() -> CardDefinition {
    use crate::card::{TokenDefinition, Zone};
    use crate::effect::shortcut::target_filtered;
    use crate::effect::ZoneDest;
    let flying_spirit = TokenDefinition {
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
    };
    let mv_at_most_2 = SelectionRequirement::Permanent
        .and(SelectionRequirement::ManaValueAtMost(2));
    CardDefinition {
        name: "Lorehold Command",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: target opponent loses 4 life (collapse to each opp).
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(4),
            },
            // Mode 1: create two 1/1 white Spirit tokens with flying.
            // Printed: "Target player creates …"; we collapse to You,
            // matching the auto-target framework's typical own-side bias
            // (the printed mode is hardly ever used to give an opponent
            // tokens — same approximation as similar "target player"
            // modes elsewhere).
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: flying_spirit,
            },
            // Mode 2: gy → hand on permanent card with MV ≤ 2.
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: mv_at_most_2,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            // Mode 3: exile target card from a graveyard.
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Any),
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
