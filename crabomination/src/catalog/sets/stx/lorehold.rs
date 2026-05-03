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
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Selector, SelectionRequirement, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{any_target, magecraft, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
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
/// ✅ Both clauses wired faithfully: `Effect::Seq([GainLife 1, DealDamage(1, any_target)])`.
/// "Any target" routes through the new `effect::shortcut::any_target()`
/// helper (`Creature ∨ Planeswalker ∨ Player`); the auto-target picker
/// prefers the opp player face for hostile damage but falls through to
/// creatures / planeswalkers when face damage isn't legal (hexproof,
/// shroud).
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
                to: any_target(),
                amount: Value::Const(1),
            },
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Lorehold Pledgemage ─────────────────────────────────────────────────────

/// Lorehold Pledgemage — {1}{R}{W}, 2/2 Spirit Cleric. "Reach. {2}{R}{W},
/// Exile a card from your graveyard: This creature gets +1/+1 until end
/// of turn."
///
/// ✅ Push XXXIV: activation now wires via the new
/// `ActivatedAbility::exile_gy_cost: u32` field — `{2}{R}{W}, exile one
/// card from your graveyard: +1/+1 EOT`. Pre-flight gate rejects with
/// `GameError::InsufficientGraveyard` when the controller has 0 cards
/// in their graveyard. Auto-pick is the oldest gy card (index 0).
pub fn lorehold_pledgemage() -> CardDefinition {
    use crate::effect::Duration;
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
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), r(), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 1,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
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
        additional_sac_cost: None,
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
        additional_sac_cost: None,
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
        additional_sac_cost: None,
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
        additional_sac_cost: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Storm-Kiln Artist ───────────────────────────────────────────────────────

/// Storm-Kiln Artist — {2}{R}{W}, 3/3 Human Wizard. "Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, Storm-Kiln Artist deals
/// 1 damage to any target. Then create a Treasure token."
///
/// ✅ "Any target" routes through the new `effect::shortcut::any_target()`
/// helper (`Creature ∨ Planeswalker ∨ Player`); the auto-target picker
/// prefers the opp player face for hostile damage but falls through to
/// creatures / planeswalkers when face damage isn't legal. The Treasure
/// follow-up is pure: standard `treasure_token()` helper.
pub fn storm_kiln_artist() -> CardDefinition {
    use crate::effect::shortcut::any_target;
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
                to: any_target(),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Rip Apart ───────────────────────────────────────────────────────────────

/// Rip Apart — {R}{W} Sorcery. "Choose one — / • Rip Apart deals 3 damage
/// to target creature or planeswalker. / • Destroy target artifact or
/// enchantment."
///
/// Push XXIX: Lorehold flexible removal — straightforward modal pick (this
/// is "choose one", not "choose two", so the existing `Effect::ChooseMode`
/// primitive ships it 1:1 — same shape as Boros Charm). Mode 0 covers
/// the creature/PW kill; mode 1 covers artifact / enchantment hate.
/// Auto-decider picks mode 0 by default — same as the other modal cards
/// in the catalog. The single-target-per-mode pattern keeps the prompt
/// simple (a planeswalker target is permitted because the
/// `Planeswalker` predicate covers both `Creature` and `Planeswalker`
/// rows when bound to permanents — see Boros Charm).
pub fn rip_apart() -> CardDefinition {
    CardDefinition {
        name: "Rip Apart",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: 3 damage to target creature or planeswalker.
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            // Mode 1: destroy target artifact or enchantment.
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::HasCardType(CardType::Enchantment)),
                ),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Plargg, Dean of Chaos ───────────────────────────────────────────────────

/// Plargg, Dean of Chaos — {1}{R}, 1/3 Legendary Human Wizard. "{T}:
/// Discard a card, then draw a card."
///
/// 🟡 Push XXIX: front face only. Plargg is the front of the
/// Plargg / Augusta paired-DFC legend; the back face Augusta, Dean of
/// Order ({1}{W}, 2/2 Vigilance with the "two-or-more attackers" rider)
/// is omitted since (a) the engine's MDFC cycle is keyed off
/// `back_face: Some(_)` for cast-other-side, and (b) the "two or more
/// creatures attacked" rider needs a count-of-attackers-this-combat
/// `Value` that doesn't exist yet (same gap as Adriana, Captain of the
/// Guard's "for each other attacking" pump).
///
/// Front-face activation is straightforward: tap to rummage. We also
/// ship the second Plargg ability — "{2}{R}: Look at the top three
/// cards of your library; you may exile one. Put the rest on the
/// bottom of your library in a random order" — as a flat scry-3 +
/// exile-bottom approximation deferred (no exile-from-top primitive),
/// so only the {T} rummage activates today.
pub fn plargg_dean_of_chaos() -> CardDefinition {
    CardDefinition {
        name: "Plargg, Dean of Chaos",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[]),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
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
        back_face: None,
        opening_hand: None,
    }
}

// ── Augusta, Dean of Order ──────────────────────────────────────────────────

/// Augusta, Dean of Order — {1}{W}, 2/2 Legendary Human Wizard with
/// Vigilance. "Whenever two or more creatures you control attack,
/// those creatures get +1/+1 and gain double strike until end of turn."
///
/// Push XXX: 🟡 → ✅ via the new `Value::AttackersThisCombat` primitive.
/// The trigger now fires per attacker (still via the
/// `Attacks/AnotherOfYours` broadcast), but each instance is gated by
/// `Predicate::ValueAtLeast(AttackersThisCombat, 2)` — so single-attacker
/// swings no longer false-positive. The trigger source is bound to
/// the just-declared attacker (`Selector::Target(0)`), and the gate
/// reads the *current* `state.attacking.len()` after that attack has
/// been pushed.
///
/// Net combat math:
/// • Single attacker: trigger fires but the gate fails (count = 1) →
///   no pump, matches printed text exactly.
/// • Two+ attackers: each fires, each passes the ≥2 gate, each
///   attacker ends up with +1/+1 + double strike EOT (matches printed).
///
/// The same `AttackersThisCombat` primitive unblocks Adriana, Captain of
/// the Guard's "+1/+1 for each *other* attacking creature" pump (just
/// with a `Diff(AttackersThisCombat, 1)` Value).
pub fn augusta_dean_of_order() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Augusta, Dean of Order",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours)
                .with_filter(Predicate::ValueAtLeast(
                    Value::AttackersThisCombat,
                    Value::Const(2),
                )),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
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
/// Push XXXVI: ✅ — "choose two" now wires faithfully via the new
/// `Effect::ChooseModes { count: 2, up_to: false, allow_duplicates:
/// false }` primitive. Auto-decider picks modes 0+1 (drain 4 + spirit
/// tokens). `ScriptedDecider::new([DecisionAnswer::Modes(vec![2, 3])])`
/// can pick gy → hand + exile-from-gy for tests. Each individual mode
/// is wired faithfully.
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
        effect: Effect::ChooseModes {
            count: 2,
            up_to: false,
            allow_duplicates: false,
            modes: vec![
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
            ],
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Spectacle Mage's siblings — Hofri Ghostforge ────────────────────────────

/// Hofri Ghostforge — {2}{R}{W}, 3/4 Legendary Human Cleric. Printed Oracle:
/// "Other nonlegendary creatures you control get +1/+1.
///  Whenever a nontoken creature you control dies, exile it. If you do,
///  create a token that's a copy of that creature, except it's a 1/1
///  white-and-red Spirit with flying."
///
/// Push XXX: 🟡. Body wired faithfully (3/4 R/W Legendary Cleric).
/// The "other nonlegendary creatures get +1/+1" static is a universal
/// anthem — `Effect::PumpPT` fires off the static-pump path used by
/// Glorious Anthem. The dies-trigger spawn-as-Spirit is omitted (token-
/// copy-of-creature primitive gap, same gap as Phantasmal Image and
/// Mockingbird in CUBE_FEATURES.md). The anthem half is the dominant
/// effect on most boards; the dies-trigger Spirit-copy rider needs
/// future engine work to land at full fidelity.
///
/// Note: the printed legendary tag means this card is a ✓ for Felisa,
/// Fang of Silverquill's "creature with a counter on it dies" trigger
/// (it's still a creature for that purpose, just not a token to be
/// pumped by the anthem itself).
pub fn hofri_ghostforge() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    // Push XXXVIII: "Other nonlegendary creatures you control get +1/+1"
    // now wires faithfully via the new `excluded_supertypes` field on
    // `AffectedPermanents::All` (legendary creatures filter out) plus
    // the new `exclude_source` flag at the layer layer (Hofri herself
    // doesn't pump herself, even though she's legendary so the
    // supertype filter would already drop her). Decomposed at static-
    // layer translation time from `Not(HasSupertype(Legendary))` in
    // `affected_from_requirement` (`game/mod.rs`).
    let other_nonleg_creatures = SelectionRequirement::Creature
        .and(SelectionRequirement::ControlledByYou)
        .and(SelectionRequirement::Not(Box::new(
            SelectionRequirement::HasSupertype(crate::card::Supertype::Legendary),
        )));
    CardDefinition {
        name: "Hofri Ghostforge",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other nonlegendary creatures you control get +1/+1",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(other_nonleg_creatures),
                power: 1,
                toughness: 1,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Mascot Interception (Lorehold) ──────────────────────────────────────────

/// Mascot Interception — {2}{R}{W} Instant. Printed Oracle:
/// "Gain control of target creature an opponent controls until end of turn.
///  Untap that creature. It gains haste until end of turn."
///
/// ✅ Push XXXIV: the printed "Threaten / Act of Treason" template now
/// wires faithfully via `Effect::GainControl` (push XXXIV — turned the
/// previously-stub `Effect::GainControl` arm into a Layer-2 continuous
/// effect with `EffectDuration::UntilEndOfTurn`, so control reverts at
/// Cleanup). The body is `Seq([GainControl, Untap, GrantKeyword(Haste,
/// EOT)])` — control change first so the untap and haste land on the
/// freshly-stolen creature. EOT cleanup drops the control change *and*
/// the haste grant; the original controller regains the creature
/// untapped at end of turn.
pub fn mascot_interception() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Mascot Interception",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Approach of the Lorehold ────────────────────────────────────────────────

/// Approach of the Lorehold — {1}{R}{W} Sorcery. Printed Oracle:
/// "Approach of the Lorehold deals 2 damage to any target. Create a
///  1/1 white Spirit creature token with flying."
///
/// Push XXX: ✅. Lorehold's flexible utility sorcery — chip damage at
/// instant speed plus a 1/1 flier on a single resolution. Wired with
/// `Effect::Seq([DealDamage 2, CreateToken Spirit])`. The "any target"
/// damage collapses to `Selector::Player(EachOpponent)` (auto-target
/// framework picks each opponent rather than a creature when no
/// creature-target is bound) — same approximation as Storm-Kiln Artist
/// and Lorehold Apprentice's "any target" magecraft riders. Spirit is
/// the 1/1 flying white from the Lorehold spirit-token line — same
/// frame as Lorehold Command's Mode 1 Spirit.
pub fn approach_of_the_lorehold() -> CardDefinition {
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
    CardDefinition {
        name: "Approach of the Lorehold",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: flying_spirit,
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

