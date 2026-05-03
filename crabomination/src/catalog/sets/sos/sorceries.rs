//! Secrets of Strixhaven (SOS) — Sorceries.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, SelectionRequirement, Subtypes,
    TokenDefinition,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, Value};
use crate::mana::{Color, b, cost, generic, w};

/// 1/1 black-and-green Pest creature token. Used by Witherbloom-leaning
/// SOS cards (Send in the Pest, Pest Mascot's payoff cycle, etc.).
///
/// Approximation: the printed Oracle is "Whenever this token attacks,
/// you gain 1 life." Token-side triggered abilities aren't materialised
/// through `token_to_card_definition` yet (the function copies P/T,
/// keywords, subtypes, colors, and *activated* abilities, but not
/// triggered ones). Until that lands, the Pest token enters with no
/// rider — Witherbloom payoffs that key off "you gained life" lose this
/// passive trickle, which is tracked in `STRIXHAVEN2.md` as 🟡 for the
/// Pest-creating cards.
pub fn pest_token() -> TokenDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    TokenDefinition {
        name: "Pest".into(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        activated_abilities: vec![],
        // "Whenever this token attacks, you gain 1 life." Wired via the
        // newly-plumbed `TokenDefinition.triggered_abilities` field —
        // `token_to_card_definition` copies these triggers onto the
        // resulting token's `CardDefinition`, so an attacking Pest now
        // correctly trickles 1 life into the controller's Witherbloom
        // payoff engine (Pest Mascot's lifegain → +1/+1 counter chain,
        // Blech's per-creature-type counter fan-out, etc.).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
    }
}

/// 0/0 green-and-blue Fractal creature token. Used by Quandrix scaling
/// payoffs (Fractal Anomaly, Snarl Song, Applied Geometry, etc.). Lives
/// or dies based on the +1/+1 counters its creator stamps onto it via
/// `Selector::LastCreatedToken` — a 0/0 with zero counters dies to SBA
/// after the resolution finishes, matching the printed cards' "if X=0,
/// the token dies" outcome.
pub fn fractal_token() -> TokenDefinition {
    use crate::mana::Color;
    TokenDefinition {
        name: "Fractal".into(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

/// 3/3 blue-and-red flying Elemental creature token. Used by Prismari-
/// flavoured SOS cards (Visionary's Dance, Muse's Encouragement, etc.).
pub fn elemental_token() -> TokenDefinition {
    use crate::card::Keyword;
    TokenDefinition {
        name: "Elemental".into(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
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

/// 2/2 red-and-white Spirit creature token. Used by Lorehold-flavoured
/// SOS cards (Group Project, Living History's ETB, etc.).
pub fn spirit_token() -> TokenDefinition {
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

// ── White ───────────────────────────────────────────────────────────────────

/// Dig Site Inventory — {W} Sorcery.
/// "Put a +1/+1 counter on target creature you control. It gains vigilance
/// until end of turn."
///
/// Approximation: the Flashback {W} clause ("you may cast this card from
/// your graveyard for its flashback cost. Then exile it.") is omitted —
/// the engine has no cast-from-graveyard pipeline yet. The face-up effect
/// is wired faithfully.
pub fn dig_site_inventory() -> CardDefinition {
    use crate::card::{CounterType, Keyword};
    use crate::effect::Duration;
    use crate::mana::ManaCost;
    let flashback_cost = ManaCost {
        symbols: vec![crate::mana::ManaSymbol::Colored(Color::White)],
    };
    CardDefinition {
        name: "Dig Site Inventory",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
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

/// Daydream — {W} Sorcery.
/// "Exile target creature you control, then return that card to the
/// battlefield under its owner's control with a +1/+1 counter on it."
///
/// Wired with the Restoration-Angel-style flicker pattern: `Exile(target)
/// + Move(target → Battlefield) + AddCounter(target, +1/+1)`. The
/// `Selector::Target(0)` slot persists across the exile so the engine's
/// `Move`/`AddCounter` resolves against the same card after it lands
/// back on the battlefield (zone-stable lookup falls back through exile,
/// which is where the target lives between Exile and Move).
///
/// The Flashback {2}{W} clause is wired via `Keyword::Flashback`; the
/// engine's existing `cast_flashback` path replays the body identically.
pub fn daydream() -> CardDefinition {
    use crate::card::{CounterType, Keyword};
    use crate::effect::ZoneDest;
    use crate::mana::{ManaCost, ManaSymbol};
    let flashback_cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(2), ManaSymbol::Colored(Color::White)],
    };
    CardDefinition {
        name: "Daydream",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
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

/// Practiced Offense — {2}{W} Sorcery.
/// "Put a +1/+1 counter on each creature target player controls.
/// Target creature gains your choice of double strike or lifelink
/// until end of turn. / Flashback {1}{W}"
///
/// Push: ✅ (was 🟡, "DS-or-Lifelink mode pick collapsed to DS"). The
/// printed mode pick is now wired as a top-level `Effect::ChooseMode`:
/// mode 0 = +1/+1 fan-out + double strike grant; mode 1 = +1/+1 fan-out
/// + lifelink grant. Cast-time `mode` argument (`Some(0)` / `Some(1)`)
/// flips between the two; `mode: None` defaults to mode 0 (DS, the
/// strictly more aggressive pick). Player-target slot still collapses
/// to "you" (the +1/+1 fan-out lands on every creature you control —
/// engine has no multi-target prompt for "any player" + "any
/// creature"). Flashback {1}{W} unchanged.
pub fn practiced_offense() -> CardDefinition {
    use crate::card::{CounterType, Keyword};
    use crate::mana::{ManaCost, ManaSymbol};
    let flashback_cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::White)],
    };
    let fan_out_pump = Effect::ForEach {
        selector: Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        body: Box::new(Effect::AddCounter {
            what: Selector::TriggerSource,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        }),
    };
    CardDefinition {
        name: "Practiced Offense",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::ChooseMode(vec![
            // Mode 0: +1/+1 fan-out + grant double strike EOT.
            Effect::Seq(vec![
                fan_out_pump.clone(),
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
            // Mode 1: +1/+1 fan-out + grant lifelink EOT.
            Effect::Seq(vec![
                fan_out_pump,
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
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

/// Antiquities on the Loose — {1}{W}{W} Sorcery.
/// "Create two 2/2 red and white Spirit creature tokens. Then if this
/// spell was cast from anywhere other than your hand, put a +1/+1
/// counter on each Spirit you control. / Flashback {4}{W}{W}."
///
/// The Flashback {4}{W}{W} clause is wired via `Keyword::Flashback`. The
/// "if cast from anywhere other than your hand, +1/+1 counter on each
/// Spirit you control" rider is still omitted (no `cast_from_zone`
/// snapshot on `StackItem` yet) — the flashback cast still mints the
/// two Spirits but skips the bonus counter shower.
pub fn antiquities_on_the_loose() -> CardDefinition {
    use crate::card::{CounterType, Keyword, Predicate};
    use crate::mana::{ManaCost, ManaSymbol};
    let flashback_cost = ManaCost {
        symbols: vec![
            ManaSymbol::Generic(4),
            ManaSymbol::Colored(Color::White),
            ManaSymbol::Colored(Color::White),
        ],
    };
    CardDefinition {
        name: "Antiquities on the Loose",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        // Mainline: create two 2/2 R/W Spirit tokens. Then if the spell
        // was cast from anywhere other than your hand (i.e. from the
        // graveyard via flashback), put a +1/+1 counter on each Spirit
        // you control. The cast-from-graveyard rider uses the new
        // `Predicate::CastFromGraveyard` (push reading
        // `EffectContext.cast_face`) — true iff the StackItem::Spell's
        // face was `Flashback`.
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: spirit_token(),
            },
            Effect::If {
                cond: Predicate::CastFromGraveyard,
                then: Box::new(Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::HasCreatureType(
                                crate::card::CreatureType::Spirit,
                            )),
                    ),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                }),
                else_: Box::new(Effect::Noop),
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

/// Group Project — {1}{W} Sorcery.
/// "Create a 2/2 red and white Spirit creature token."
///
/// Approximation: the Flashback "Tap three untapped creatures you
/// control" clause is omitted (the engine has no cast-from-graveyard
/// pipeline, and the engine has no "tap three creatures as additional
/// cost" primitive yet). The face-up token-creation half is wired
/// faithfully.
pub fn group_project() -> CardDefinition {
    CardDefinition {
        name: "Group Project",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: spirit_token(),
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

// ── Blue ────────────────────────────────────────────────────────────────────

/// Procrastinate — {X}{U} Sorcery.
/// "Tap target creature. Put twice X stun counters on it."
pub fn procrastinate() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::{u, x};
    CardDefinition {
        name: "Procrastinate",
        cost: cost(&[x(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::XFromCost),
                ),
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

/// Mathemagics — {X}{X}{U}{U} Sorcery.
/// "Target player draws 2ˣ cards. (2⁰ = 1, 2¹ = 2, 2² = 4, 2³ = 8, 2⁴ = 16,
/// 2⁵ = 32, and so on.)"
///
/// Approximation: the target-player slot is collapsed to "you" — the
/// engine has no multi-target-player prompt for sorceries, and casting
/// Mathemagics on yourself is the typical play pattern. Powered by the
/// engine's new `Value::Pow2(XFromCost)` primitive. The Pow2 evaluator
/// caps the exponent at 30 so absurd X values can't deck the player out
/// or overflow.
pub fn mathemagics() -> CardDefinition {
    use crate::mana::{u, x};
    CardDefinition {
        name: "Mathemagics",
        cost: cost(&[x(), x(), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Pow2(Box::new(Value::XFromCost)),
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

// ── Green ───────────────────────────────────────────────────────────────────

/// Oracle's Restoration — {G} Sorcery.
/// "Target creature you control gets +1/+1 until end of turn. You draw a
/// card and gain 1 life."
pub fn oracles_restoration() -> CardDefinition {
    use crate::card::Effect as E;
    use crate::effect::Duration;
    use crate::mana::g;
    CardDefinition {
        name: "Oracle's Restoration",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            E::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            E::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            E::GainLife {
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

// ── Black ───────────────────────────────────────────────────────────────────

/// Send in the Pest — {1}{B} Sorcery.
/// "Each opponent discards a card. You create a 1/1 black and green Pest
/// creature token with 'Whenever this token attacks, you gain 1 life.'"
///
/// Approximation: the Pest token's "gain 1 on attack" trigger is omitted
/// (token-side triggered abilities aren't materialised through
/// `token_to_card_definition` yet). The discard half and token creation
/// are wired faithfully.
pub fn send_in_the_pest() -> CardDefinition {
    CardDefinition {
        name: "Send in the Pest",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: pest_token(),
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

/// Dina's Guidance — {1}{B}{G} Sorcery.
/// "Search your library for a creature card, reveal it, put it into your
/// hand or graveyard, then shuffle."
///
/// Push: ✅ (was 🟡, "destination collapsed to hand"). The hand-or-
/// graveyard destination prompt is wired as `Effect::ChooseMode` with
/// two modes — mode 0 searches the creature to your hand, mode 1
/// searches it to your graveyard. Both modes use the existing
/// `Effect::Search` primitive (which already routes to any `ZoneDest`,
/// including `Graveyard`). The auto-decider picks mode 0 (hand) by
/// default — the strictly stronger pick in 95%+ of board states.
/// Reanimator decks (Goryo's Vengeance / Animate Dead / Reanimate)
/// can flip to mode 1 via the cast-time `mode` argument.
pub fn dinas_guidance() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::g;
    CardDefinition {
        name: "Dina's Guidance",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: search a creature card to hand (the default).
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            // Mode 1: search a creature card directly to your graveyard
            // (reanimator-fuel mode — Goryo's Vengeance / Animate Dead /
            // Reanimate downstream).
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
                to: ZoneDest::Graveyard,
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

/// Grapple with Death — {1}{B}{G} Sorcery.
/// "Destroy target artifact or creature. You gain 1 life."
pub fn grapple_with_death() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Grapple with Death",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::Creature),
                ),
            },
            Effect::GainLife {
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

/// Pull from the Grave — {2}{B} Sorcery.
/// "Return up to two target creature cards from your graveyard to your
/// hand. You gain 2 life."
///
/// Approximation: the printed card has two target slots; the engine
/// has no multi-target prompt yet, so we auto-pick the top two creature
/// cards from the controller's graveyard via `Selector::take(_, 2)`.
/// This is faithful to the printed scaling (caster gets up to two) and
/// strictly closer than the previous single-card cap. The lifegain
/// half resolves regardless.
pub fn pull_from_the_grave() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Pull from the Grave",
        cost: cost(&[generic(2), b()]),
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
                        filter: SelectionRequirement::Creature,
                    },
                    Value::Const(2),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::GainLife {
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

/// End of the Hunt — {1}{B} Sorcery.
/// "Target opponent exiles a creature or planeswalker they control with
/// the greatest mana value among creatures and planeswalkers they
/// control."
///
/// Approximation: the "greatest mana value" clause is approximated by
/// targeting any opponent-controlled creature/planeswalker — the
/// auto-decider picks the first eligible permanent (typically the
/// largest cost one in our cube pools, since they're listed in roughly
/// curve order). The "their choice" wording is collapsed to "we exile
/// the chosen target", which is strictly better for the caster but
/// matches the spell's role as targeted edict-removal.
pub fn end_of_the_hunt() -> CardDefinition {
    CardDefinition {
        name: "End of the Hunt",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::ControlledByOpponent.and(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
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
    }
}

/// Vicious Rivalry — {2}{B}{G} Sorcery.
/// "As an additional cost to cast this spell, pay X life. / Destroy
/// all artifacts and creatures with mana value X or less."
///
/// Approximation: the engine has no per-cast "pay X life" additional
/// cost primitive yet, so X is read off the spell's cost (`{X}` slot)
/// rather than a separate life payment, and the caster doesn't lose
/// X life on cast. Net: a flexible mass removal that scales with X
/// — Wrath shape at higher X, Pyroclasm-on-artifacts shape at lower
/// X. Tracked under TODO.md "X-life additional cost primitive".
pub fn vicious_rivalry() -> CardDefinition {
    use crate::mana::{ManaSymbol, g};
    let mut spell_cost = cost(&[generic(2), b(), g()]);
    spell_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Vicious Rivalry",
        cost: spell_cost,
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            // Approximate "pay X life" additional cost.
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::XFromCost,
            },
            // Destroy all matching permanents whose mana value ≤ X.
            // We run two `ForEach` passes — one for artifacts, one for
            // creatures — each gated by a per-iteration mana-value
            // check via `Effect::If` + `Predicate::ValueAtMost`.
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::Creature),
                ),
                body: Box::new(Effect::If {
                    cond: crate::effect::Predicate::ValueAtMost(
                        Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                        Value::XFromCost,
                    ),
                    then: Box::new(Effect::Destroy {
                        what: Selector::TriggerSource,
                    }),
                    else_: Box::new(Effect::Noop),
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
    }
}

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

/// Borrowed Knowledge — {2}{R}{W} Sorcery.
/// "Choose one —
/// • Discard your hand, then draw cards equal to the number of cards
///   in target opponent's hand.
/// • Discard your hand, then draw cards equal to the number of cards
///   discarded this way."
///
/// Mode 0: discard your hand, then draw a number of cards equal to the
/// (chosen) opponent's hand size. Wired faithfully — the spell's target
/// slot picks the opponent (auto-decider grabs an opponent), and
/// `Value::HandSizeOf(PlayerRef::Target(0))` reads that opponent's
/// hand at resolution time.
///
/// Mode 1: discard your hand, then draw cards equal to the number of
/// cards discarded this way — wired faithfully via the new
/// `Value::CardsDiscardedThisResolution` primitive (a per-resolution
/// counter bumped by every `Effect::Discard` in the same `Seq`).
pub fn borrowed_knowledge() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Borrowed Knowledge",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: discard hand, then draw = target opponent's hand size.
            Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::Target(0)),
                },
            ]),
            // Mode 1: discard hand, then draw cards equal to the number
            // of cards discarded this way — wired faithfully via
            // `Value::CardsDiscardedThisResolution`.
            Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CardsDiscardedThisResolution,
                },
            ]),
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

/// Pursue the Past — {R}{W} Sorcery.
/// "You gain 2 life. You may discard a card. If you do, draw two cards."
///
/// Approximation: the "may discard" optionality is dropped — picking
/// this card commits to the discard-then-draw-2 (engine has no in-effect
/// optionality primitive yet). The auto-decider has the controller pick
/// the lowest-priority card for discard. The Flashback {2}{R}{W} clause
/// is wired via `Keyword::Flashback` so the body replays from the
/// graveyard.
pub fn pursue_the_past() -> CardDefinition {
    use crate::card::Keyword;
    use crate::mana::{ManaCost, ManaSymbol, r, w as w_mana};
    let flashback_cost = ManaCost {
        symbols: vec![
            ManaSymbol::Generic(2),
            ManaSymbol::Colored(Color::Red),
            ManaSymbol::Colored(Color::White),
        ],
    };
    CardDefinition {
        name: "Pursue the Past",
        cost: cost(&[r(), w_mana()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        // Push XV: the printed "you may discard a card. If you do, draw
        // two cards" is now wired via `Effect::MayDo`. The 2-life gain
        // is unconditional. The discard+draw chain is gated on a single
        // yes/no decision (auto-decider says no by default; the
        // ScriptedDecider can flip it for tests).
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::MayDo {
                description: "Pursue the Past: discard a card, then draw two?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                ])),
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

/// Molten Note — {X}{R}{W} Lorehold Sorcery.
/// "Molten Note deals damage to target creature equal to the amount of
/// mana spent to cast this spell. Untap all creatures you control. /
/// Flashback {6}{R}{W} (You may cast this card from your graveyard for
/// its flashback cost. Then exile it.)"
///
/// Wired faithfully:
/// - Damage = mana spent. Engine has no first-class "mana spent to cast"
///   value, so we branch on `Predicate::CastFromGraveyard` (push XVIII)
///   to reproduce both halves exactly:
///   * Hand cast ({X}{R}{W}): damage = X + 2 (the X plus the {R}{W}
///     portion). `Value::Sum(XFromCost, Const(2))` reads the actual paid
///     X from `EffectContext.x_value`.
///   * Flashback cast ({6}{R}{W}, fixed): damage = 8 (the {6} + {R}{W}).
///     The fixed-cost branch is `Value::Const(8)`.
/// - "Untap all creatures you control" → `Effect::Untap` on each
///   creature you control (no `up_to` cap, mirrors the printed wording).
/// - Flashback wired via `Keyword::Flashback({6}{R}{W})`. The "Then
///   exile it" tail is the engine's default for flashback-cast cards
///   (the engine routes flashbacked spells to exile after resolution).
pub fn molten_note() -> CardDefinition {
    use crate::card::Keyword;
    use crate::effect::Predicate;
    use crate::mana::{ManaCost, ManaSymbol};
    let flashback_cost = ManaCost {
        symbols: vec![
            ManaSymbol::Generic(6),
            ManaSymbol::Colored(Color::Red),
            ManaSymbol::Colored(Color::White),
        ],
    };
    let damage_target = target_filtered(SelectionRequirement::Creature);
    CardDefinition {
        name: "Molten Note",
        // {X}{R}{W} — the X is the cast-time variable.
        cost: ManaCost {
            symbols: vec![
                ManaSymbol::X,
                ManaSymbol::Colored(Color::Red),
                ManaSymbol::Colored(Color::White),
            ],
        },
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            // Damage half — branch on cast-face so the "amount of mana
            // spent" formula matches the actual cost paid.
            Effect::If {
                cond: Predicate::CastFromGraveyard,
                then: Box::new(Effect::DealDamage {
                    to: damage_target.clone(),
                    amount: Value::Const(8),
                }),
                else_: Box::new(Effect::DealDamage {
                    to: damage_target,
                    amount: Value::Sum(vec![Value::XFromCost, Value::Const(2)]),
                }),
            },
            // Untap-all-creatures-you-control half.
            Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
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

// ── Silverquill (W/B) ───────────────────────────────────────────────────────

/// Render Speechless — {2}{W}{B} Sorcery.
/// "Target opponent reveals their hand. You choose a nonland card from it.
/// That player discards that card. Put two +1/+1 counters on up to one
/// target creature."
///
/// Push: ✅ (was 🟡, "up to one creature target was required, blocking
/// the cast when no creature existed"). The +1/+1 counter half now
/// auto-picks a friendly creature via `Selector::one_of(EachPermanent(
/// Creature ∧ ControlledByYou))` — same multi-target collapse as
/// Cost of Brilliance / Vibrant Outburst's tap half. Cast is now
/// legal even when you control no creatures (just discard half fires).
/// `DiscardChosen` on `EachOpponent` handles the reveal-and-discard
/// half (auto-decider picks the first matching nonland card).
pub fn render_speechless() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Render Speechless",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
            Effect::AddCounter {
                what: Selector::one_of(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                )),
                kind: CounterType::PlusOnePlusOne,
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

/// Moment of Reckoning — {3}{W}{W}{B}{B} Sorcery.
/// "Choose up to four. You may choose the same mode more than once.
/// • Destroy target nonland permanent.
/// • Return target nonland permanent card from your graveyard to the
///   battlefield."
///
/// Approximation: the printed card lets the controller pick the same
/// mode more than once and target up to four different permanents.
/// The engine's `ChooseMode` only picks one mode and one target per
/// resolution, so we collapse to a single-target two-mode picker that
/// runs *both* halves at full power: the first mode destroys a target
/// nonland permanent (auto-targets an opponent's), and the second mode
/// returns a nonland permanent card from your graveyard. Net play: the
/// card swings the board exactly the way a 1×destroy / 1×return play
/// would in a real game, just without the optional 3rd/4th invocation.
pub fn moment_of_reckoning() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Moment of Reckoning",
        cost: cost(&[generic(3), w(), w(), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: destroy target nonland permanent.
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
            // Mode 1: return target nonland permanent card from your
            // graveyard to the battlefield.
            Effect::Move {
                what: target_filtered(SelectionRequirement::Nonland),
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

/// Social Snub — {1}{W}{B} Silverquill Sorcery.
/// "When you cast this spell while you control a creature, you may
/// copy this spell. / Each player sacrifices a creature of their
/// choice. Each opponent loses 1 life and you gain 1 life."
///
/// Wired faithfully on the play-pattern halves; the on-cast may-copy
/// rider is omitted (no copy-spell primitive yet — same gap as
/// Silverquill the Disputant, Choreographed Sparks, etc.).
///
/// - Mass sacrifice: `Effect::Sacrifice` is fanned across **each
///   player** (you + each opponent) via a `Seq` of two sacrifice
///   effects. Each player picks their own sac via the auto-decider's
///   first-matching-creature heuristic, mirroring the printed "of
///   their choice" wording.
/// - Drain: `Effect::Drain` from each opponent → you. The {1}{W}{B}
///   rate of one-life drain is unconditional (vs. the on-cast may-copy
///   rider that conditionally doubles the body).
///
/// Now fully wired (post-XX): the on-cast `may copy this spell while
/// you control a creature` rider is wired via a `SelfSource +
/// SpellCast` triggered ability filtered on `SelectorExists(Creature &
/// ControlledByYou)`. The trigger body asks the controller via
/// `Effect::MayDo` and on yes copies the spell with `CopySpell {
/// what: CastSpellSource, count: 1 }`. The copy resolves first,
/// applying mass-sacrifice and drain a second time — matching the
/// printed semantic of "copy this spell" (each copy resolves its own
/// body independently).
pub fn social_snub() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Predicate, TriggeredAbility};
    CardDefinition {
        name: "Social Snub",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            // Each player sacrifices a creature of their choice. Two
            // separate sac effects so each player's auto-decider gets
            // its own selection (one for the controller, one for the
            // EachOpponent fan-out which dispatches per-opp).
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            // Drain 1 from each opponent to you.
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource).with_filter(
                Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                )),
            ),
            effect: Effect::MayDo {
                description: "Copy this spell".to_string(),
                body: Box::new(Effect::CopySpell {
                    what: Selector::CastSpellSource,
                    count: Value::Const(1),
                }),
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

/// Arcane Omens — {4}{B} Sorcery.
/// "Converge — Target player discards X cards, where X is the number
/// of colors of mana spent to cast this spell."
///
/// Wired faithfully via `Effect::Discard` with `Value::ConvergedValue`.
/// The "target player" target slot is approximated by `EachOpponent`
/// — the auto-decider always targets an opponent, which matches the
/// printed-card use case (no one targets themselves with discard).
pub fn arcane_omens() -> CardDefinition {
    CardDefinition {
        name: "Arcane Omens",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ConvergedValue,
            random: false,
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

/// Together as One — {6} Sorcery.
/// "Converge — Target player draws X cards, Together as One deals X
/// damage to any target, and you gain X life, where X is the number of
/// colors of mana spent to cast this spell."
///
/// Approximation: the "target player draws X" half is collapsed to
/// "you draw X" — same engine multi-target gap that's blocked Cost of
/// Brilliance. The damage half targets any-target via the spell's
/// primary target slot. Self-life-gain runs unconditionally.
pub fn together_as_one() -> CardDefinition {
    CardDefinition {
        name: "Together as One",
        cost: cost(&[generic(6)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::ConvergedValue,
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::ConvergedValue,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ConvergedValue,
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

/// Wisdom of Ages — {4}{U}{U}{U} Sorcery.
/// "Return all instant and sorcery cards from your graveyard to your
/// hand. You have no maximum hand size for the rest of the game. /
/// Exile Wisdom of Ages."
///
/// Approximation: the "no maximum hand size" clause is omitted (the
/// engine has no hand-size cap or "remove cap" primitive yet — there's
/// no functional difference in 2-player matches without a discard
/// trigger that fires off hand size). The graveyard recursion uses
/// `Selector::CardsInZone { zone: Graveyard }` with a card-type
/// filter and a `ZoneDest::Hand` destination. The "exile this" rider
/// is omitted (sorceries already go to graveyard on resolution; the
/// special exile clause is for replay-prevention which we don't have
/// any payoff for yet).
pub fn wisdom_of_ages() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    use crate::mana::u;
    CardDefinition {
        name: "Wisdom of Ages",
        cost: cost(&[generic(4), u(), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            },
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
    }
}

/// Rapturous Moment — {4}{U}{R} Sorcery.
/// "Draw three cards, then discard two cards. Add {U}{U}{R}{R}{R}."
///
/// Wired faithfully — three-step `Effect::Seq` of Draw 3, Discard 2,
/// AddMana with the printed UU/RRR pool. The mana addition uses
/// `ManaPayload::Colors` so it adds each color symbol independently
/// (no auto-color picker required).
pub fn rapturous_moment() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::mana::{r, u, Color};
    CardDefinition {
        name: "Rapturous Moment",
        cost: cost(&[generic(4), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![
                    Color::Blue,
                    Color::Blue,
                    Color::Red,
                    Color::Red,
                    Color::Red,
                ]),
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

/// Visionary's Dance — {5}{U}{R} Sorcery.
/// "Create two 3/3 blue and red Elemental creature tokens with flying.
/// {2}, Discard this card: Look at the top two cards of your library.
/// Put one of them into your hand and the other into your graveyard."
///
/// Approximation: the spell-side resolution (two 3/3 flying Elemental
/// tokens) is wired faithfully via the new `elemental_token()` helper.
/// The `{2}, Discard this card:` activated ability lives on the card
/// while in **hand**, which the engine's `do_activate_ability` walker
/// doesn't yet visit (it iterates the battlefield only). Tracked under
/// TODO.md ("Mana Ability from Non-Battlefield Zone" / "activated
/// abilities from hand"). Until that primitive lands, the discard
/// half is silently dropped.
pub fn visionarys_dance() -> CardDefinition {
    use crate::mana::{r, u};
    CardDefinition {
        name: "Visionary's Dance",
        cost: cost(&[generic(5), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: elemental_token(),
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

/// Splatter Technique — {1}{U}{U}{R}{R} Sorcery. Choose one —
/// • Draw four cards.
/// • Splatter Technique deals 4 damage to each creature and planeswalker.
pub fn splatter_technique() -> CardDefinition {
    use crate::mana::{r, u};
    CardDefinition {
        name: "Splatter Technique",
        cost: cost(&[generic(1), u(), u(), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: draw 4.
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(4),
            },
            // Mode 1: 4 damage to each creature and planeswalker.
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
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
    }
}

/// Cost of Brilliance — {2}{B} Sorcery.
/// "Target player draws two cards and loses 2 life. Put a +1/+1 counter
/// on up to one target creature."
///
/// Push: ✅ (was 🟡, "+1/+1 counter required a creature target —
/// blocked the cast when no creature existed"). Promotion: the
/// +1/+1 half now uses `Selector::one_of(EachPermanent(Creature ∧
/// ControlledByYou))` — auto-picks a friendly creature, no-ops
/// cleanly when none exist. The "target player" prompt for the
/// draws/loses-life half is still collapsed to "you" (caster
/// self-loots) — engine has no multi-target prompt for sorceries.
pub fn cost_of_brilliance() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Cost of Brilliance",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: Selector::one_of(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                )),
                kind: CounterType::PlusOnePlusOne,
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

/// Mind Roots — {1}{B}{G} Sorcery.
/// "Target player discards two cards. Put up to one land card discarded
/// this way onto the battlefield tapped under your control."
///
/// The "land card discarded this way → battlefield" rider is now wired
/// via the new `Selector::DiscardedThisResolution(IsLand)` primitive
/// + `Selector::one_of(...)` to clamp to one land. The discard half
/// uses `EachOpponent` so the caster never targets themselves — keeps
/// the spell aligned with its hand-disruption role even though the
/// printed card lets the caster target any player.
pub fn mind_roots() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::g;
    CardDefinition {
        name: "Mind Roots",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
                random: false,
            },
            // "Put up to one land card discarded this way onto the
            // battlefield tapped under your control." Filter the
            // per-resolution discard list to lands and pick one.
            Effect::Move {
                what: Selector::one_of(Selector::DiscardedThisResolution(
                    SelectionRequirement::Land,
                )),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
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

/// Mind into Matter — {X}{G}{U} Sorcery.
/// "Draw X cards. Then you may put a permanent card with mana value X
/// or less from your hand onto the battlefield tapped."
///
/// Approximation: the "may put a permanent ≤ X onto the battlefield"
/// half is omitted (the engine has no hand-to-battlefield primitive
/// gated on a card's mana value yet). The Draw X half is wired
/// faithfully via `Value::XFromCost`.
pub fn mind_into_matter() -> CardDefinition {
    use crate::mana::{ManaSymbol, g, u};
    let mut spell_cost = cost(&[g(), u()]);
    spell_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Mind into Matter",
        cost: spell_cost,
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
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
    }
}

/// Growth Curve — {G}{U} Sorcery.
/// "Put a +1/+1 counter on target creature you control, then double the
/// number of +1/+1 counters on that creature."
///
/// Wired faithfully: step 1 adds 1 +1/+1 counter; step 2 adds
/// `CountersOn(Target(0), +1/+1)` more counters, which equals the new
/// count after step 1. Net: a creature with N +1/+1 counters before the
/// spell ends with 2*(N+1) counters after, matching the printed
/// "double" behaviour.
pub fn growth_curve() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::{g, u};
    CardDefinition {
        name: "Growth Curve",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::Target(0)),
                    kind: CounterType::PlusOnePlusOne,
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

/// Killian's Confidence — {W}{B} Sorcery.
/// "Target creature gets +1/+1 until end of turn. Draw a card."
/// "Whenever one or more creatures you control deal combat damage to a
/// player, you may pay {W/B}. If you do, return this card from your
/// graveyard to your hand."
///
/// Mainline pump+draw + the gy-recursion trigger are now both wired:
/// - Pump+draw via `Effect::PumpPT` + `Effect::Draw`.
/// - The "creatures you control deal combat damage to a player" trigger
///   uses `EventScope::FromYourGraveyard` (so it only fires while
///   Killian's Confidence sits in its owner's graveyard) +
///   `EventKind::DealsCombatDamageToPlayer`. The combat-damage event
///   broadcaster (`fire_combat_damage_to_player_triggers`) walks the
///   attacker's controller's graveyard for `FromYourGraveyard`-scoped
///   triggers, so the card's owner's own attacking creature dealing
///   combat damage fires this trigger. The pay-{W/B}-to-return body is
///   wired via `Effect::MayPay` with a hybrid `{W/B}` cost; declining
///   skips silently. Per-card emission means the trigger fires once per
///   attacker that connects (printed: "one or more creatures" — close
///   enough; the controller can decline the second+ trigger).
pub fn killians_confidence() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::ZoneDest;
    use crate::mana::{ManaCost, ManaSymbol};
    let recursion_cost = ManaCost {
        symbols: vec![ManaSymbol::Hybrid(Color::White, Color::Black)],
    };
    CardDefinition {
        name: "Killian's Confidence",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::FromYourGraveyard,
            ),
            effect: Effect::MayPay {
                description:
                    "Killian's Confidence: pay {W/B} to return from graveyard to hand?".into(),
                mana_cost: recursion_cost,
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                }),
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

/// Planar Engineering — {3}{G} Sorcery.
/// "Sacrifice two lands. Search your library for four basic land cards,
/// put them onto the battlefield tapped, then shuffle."
///
/// Wired with `Effect::Sacrifice` (lands, count=2) followed by an
/// `Effect::Repeat` of 4 `Effect::Search` invocations putting basic
/// lands onto the battlefield tapped. Each `Search` is a separate
/// suspend point in the engine's interactive flow; for the bot harness
/// the auto-decider picks each basic in turn.
pub fn planar_engineering() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::g as gm;
    CardDefinition {
        name: "Planar Engineering",
        cost: cost(&[generic(3), gm()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(2),
                filter: SelectionRequirement::Land,
            },
            Effect::Repeat {
                count: Value::Const(4),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
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
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Pox Plague — {B}{B}{B}{B}{B} Sorcery.
/// "Each player loses half their life, then discards half the cards in
/// their hand, then sacrifices half the permanents they control of
/// their choice. Round down each time."
///
/// Wired via a `ForEach Selector::Player(EachPlayer)` whose body uses
/// `PlayerRef::Triggerer` (the iterated player) for each clause. Each
/// of the three "half" amounts is computed via the new
/// `Value::HalfDown(...)` primitive against the iterated player's life,
/// hand size, and `PermanentCountControlledBy` count. Sacrifice uses
/// `who: Triggerer` so the existing per-controller candidate filter in
/// `Effect::Sacrifice` correctly limits each player to *their* permanents,
/// and the `Permanent` filter picks any nonland-or-land permanent.
pub fn pox_plague() -> CardDefinition {
    CardDefinition {
        name: "Pox Plague",
        cost: cost(&[b(), b(), b(), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachPlayer),
            body: Box::new(Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::HalfDown(Box::new(Value::LifeOf(PlayerRef::Triggerer))),
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::HalfDown(Box::new(Value::HandSizeOf(PlayerRef::Triggerer))),
                    random: false,
                },
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Triggerer),
                    count: Value::HalfDown(Box::new(Value::PermanentCountControlledBy(
                        PlayerRef::Triggerer,
                    ))),
                    filter: SelectionRequirement::Permanent,
                },
            ])),
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

/// Withering Curse — {1}{B}{B} Sorcery.
/// "All creatures get -2/-2 until end of turn. / Infusion — If you
/// gained life this turn, destroy all creatures instead."
///
/// Resolves as an `If`-gated branch on `LifeGainedThisTurnAtLeast`: the
/// Infusion path runs `Destroy` over `EachPermanent(Creature)`; the
/// mainline hands every creature a -2/-2 EOT pump (matching `Languish`'s
/// shape). Tracked life is reset on the controller's untap.
pub fn withering_curse() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Withering Curse",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::If {
            cond: Predicate::LifeGainedThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(1),
            },
            then: Box::new(Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::Destroy {
                    what: Selector::TriggerSource,
                }),
            }),
            else_: Box::new(Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                }),
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
    }
}

// ── Witherbloom (B/G) ───────────────────────────────────────────────────────

/// Root Manipulation — {3}{B}{G} Sorcery.
/// "Until end of turn, creatures you control get +2/+2 and gain menace
/// and 'Whenever this creature attacks, you gain 1 life.'"
///
/// The +2/+2 pump and Menace grant land on every creature controlled at
/// resolution. The "whenever this creature attacks → gain 1 life" rider
/// is omitted (the engine has no temporary-trigger-grant primitive for
/// pump spells; granting an attack-trigger needs installing a transient
/// trigger per-creature). Tracked in TODO.md.
pub fn root_manipulation() -> CardDefinition {
    use crate::card::Keyword;
    use crate::mana::g;
    CardDefinition {
        name: "Root Manipulation",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ])),
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

// ── Green ───────────────────────────────────────────────────────────────────

/// Chelonian Tackle — {2}{G} Sorcery.
/// "Target creature you control gets +0/+10 until end of turn. Then it
/// fights up to one target creature an opponent controls."
///
/// Wired against the new `Effect::Fight` primitive. The chosen target
/// (a creature you control) gets +0/+10 EOT, then fights the
/// auto-selected first opponent creature via
/// `EachPermanent(Creature & ControlledByOpponent)`. The "up to one"
/// half is preserved naturally — Fight no-ops if the defender
/// selector resolves to no permanent (no opp creature on bf).
///
/// Note: the engine currently doesn't expose a multi-target prompt,
/// so the defender is auto-picked rather than chosen. A future
/// multi-target enhancement could let the controller pick which opp
/// creature to fight.
pub fn chelonian_tackle() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Chelonian Tackle",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(0),
                toughness: Value::Const(10),
                duration: Duration::EndOfTurn,
            },
            Effect::Fight {
                attacker: Selector::Target(0),
                defender: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
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

// ── Red ─────────────────────────────────────────────────────────────────────

/// Steal the Show — {2}{R} Sorcery. Choose one or both —
/// • Target player discards any number of cards, then draws that many
///   cards.
/// • Steal the Show deals damage equal to the number of instant and
///   sorcery cards in your graveyard to target creature or planeswalker.
///
/// Approximation: the engine has no "choose one or both" multi-mode
/// primitive, so this is wired as a normal `ChooseMode` (pick one).
/// Mode 0 collapses "any number of cards" to "discard 2, draw 2" — the
/// engine has no controller-picks-N-from-hand affordance for the
/// targeted player; the most common play pattern is "discard your
/// hand to refill". Mode 1 reads the IS-graveyard count from the
/// caster's graveyard via `Value::CountOf(EachMatching)` and damages
/// the target creature/PW.
pub fn steal_the_show() -> CardDefinition {
    use crate::card::Zone;
    use crate::mana::r;
    let is_graveyard_count = Value::count(Selector::CardsInZone {
        who: PlayerRef::You,
        zone: Zone::Graveyard,
        filter: SelectionRequirement::HasCardType(CardType::Instant)
            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
    });
    CardDefinition {
        name: "Steal the Show",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: target player discards 2, then draws 2 ("any
            // number of cards" approximation).
            Effect::Seq(vec![
                Effect::Discard {
                    who: target_filtered(SelectionRequirement::Player),
                    amount: Value::Const(2),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::Target(0),
                    amount: Value::Const(2),
                },
            ]),
            // Mode 1: damage = # of instant/sorcery cards in your
            // graveyard, to target creature or planeswalker.
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: is_graveyard_count,
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

// ── More Green ──────────────────────────────────────────────────────────────

/// Snarl Song — {5}{G} Sorcery (Converge).
/// "Converge — Create two 0/0 green and blue Fractal creature tokens. Put
/// X +1/+1 counters on each of them and you gain X life, where X is the
/// number of colors of mana spent to cast this spell."
///
/// Converge X is read at resolution time from `Value::ConvergedValue`,
/// which the engine populates from the spell's cast-time color set
/// (already plumbed for Rancorous Archaic / Together as One). We mint
/// two Fractal tokens via two `CreateToken` calls, stamping each with
/// X +1/+1 counters via `Selector::LastCreatedToken` between mints.
/// At a 6-color cost this is a {5}{G} for 2 X/X bodies + X life.
pub fn snarl_song() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::g;
    let x = || Value::ConvergedValue;
    CardDefinition {
        name: "Snarl Song",
        cost: cost(&[generic(5), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: x(),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: x(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: x(),
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

/// Wild Hypothesis — {X}{G} Sorcery.
/// "Create a 0/0 green and blue Fractal creature token. Put X +1/+1
/// counters on it. / Surveil 2."
///
/// Mints a Fractal token then adds X (= XFromCost) counters via
/// `Selector::LastCreatedToken`, then `Effect::Surveil 2`. Surveil is
/// a first-class engine effect so this resolves end-to-end with no
/// approximation. At X=0 the token enters as 0/0 and dies to SBA before
/// surveil resolves, matching printed semantics.
pub fn wild_hypothesis() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::{ManaSymbol, g};
    let mut spell_cost = cost(&[g()]);
    spell_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Wild Hypothesis",
        cost: spell_cost,
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            },
            Effect::Surveil {
                who: PlayerRef::You,
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




/// Zimone's Experiment — {3}{G} Sorcery.
/// "Look at the top five cards of your library. You may reveal up to two
/// creature and/or land cards from among them, then put the rest on the
/// bottom of your library in a random order. Put all land cards revealed
/// this way onto the battlefield tapped and put all creature cards
/// revealed this way into your hand."
///
/// Approximation: the engine has no "look at top N, choose ≤K matching
/// among them" primitive that splits hits between two destinations
/// (hand vs. tapped battlefield). We collapse to the most common play
/// pattern: a single `RevealUntilFind` for a creature to hand (cap 5),
/// then a `Search` for a basic land into play tapped — the cards-not-
/// chosen-on-the-reveal half lands on the bottom of the library
/// implicitly. The "reveal up to two" cap is approximated as "find
/// one of each" (typical pull is 1 creature + 1 land); the random-
/// bottom sort happens via the natural reveal-rest-go-to-bottom path.
/// Artistic Process — {3}{R}{R} Sorcery.
/// "Choose one — / • Artistic Process deals 6 damage to target creature.
/// / • Artistic Process deals 2 damage to each creature you don't
/// control. / • Create a 3/3 blue and red Elemental creature token with
/// flying. It gains haste until end of turn."
///
/// All three modes wired on existing primitives:
/// - Mode 0 fires a 6-damage hit at a single creature target via
///   `target_filtered(Creature)`.
/// - Mode 1 deals 2 damage to each creature *not controlled by the
///   caster* — `Selector::EachPermanent(Creature & ControlledByOpponent)`.
///   In a 2-player match this is exactly "each opponent's creature".
/// - Mode 2 mints an Elemental via the shared `elemental_token()` helper
///   then grants haste EOT to the freshly-minted token via
///   `Selector::LastCreatedToken`.
pub fn artistic_process() -> CardDefinition {
    use crate::card::Keyword;
    use crate::mana::r;
    CardDefinition {
        name: "Artistic Process",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: 6 damage to target creature.
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(6),
            },
            // Mode 1: 2 damage to each creature you don't control.
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(2),
            },
            // Mode 2: 3/3 blue-and-red Elemental flier with haste EOT.
            Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: elemental_token(),
                },
                Effect::GrantKeyword {
                    what: Selector::LastCreatedToken,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
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

/// Decorum Dissertation — {3}{B}{B} Sorcery — Lesson.
/// "Target player draws two cards and loses 2 life. / Paradigm (Then
/// exile this spell. After you first resolve a spell with this name,
/// you may cast a copy of it from exile without paying its mana cost
/// at the beginning of each of your first main phases.)"
///
/// Approximation: the printed effect is "*target player* draws two and
/// loses 2 life" — Diabolic Sleight-style asymmetric lesson. Today the
/// engine has no multi-target prompt, so we collapse the target slot
/// to **the caster** (you draw two; you lose 2 — same net "draw 2 lose
/// 2"). The Paradigm rider is omitted (no copy-spell-from-exile-at-
/// upkeep primitive). The Lesson tag is informational; learn pulls in
/// SOS just exile the lesson and resolve it as a sorcery, which is
/// exactly the same flow as casting it from hand.
pub fn decorum_dissertation() -> CardDefinition {
    CardDefinition {
        name: "Decorum Dissertation",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::LoseLife {
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

/// Germination Practicum — {3}{G}{G} Sorcery — Lesson.
/// "Put two +1/+1 counters on each creature you control. / Paradigm
/// (...)"
///
/// Wired as a `ForEach` over your creatures with a per-iteration
/// `AddCounter +1/+1 ×2` body — the printed +2/+2 anthem-style fan-out.
/// The Paradigm rider is omitted (no copy-spell-from-exile-at-upkeep
/// primitive yet — same gap as Decorum Dissertation, Improvisation
/// Capstone, Echocasting Symposium).
pub fn germination_practicum() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::g;
    CardDefinition {
        name: "Germination Practicum",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
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
    }
}

/// Restoration Seminar — {5}{W}{W} Sorcery — Lesson.
/// "Return target nonland permanent card from your graveyard to the
/// battlefield. / Paradigm (...)"
///
/// Mode 0 (the on-cast effect) wired faithfully: graveyard → bf with a
/// `Nonland` filter, controller `You`. Untapped (printed default — the
/// printed wording doesn't say "tapped"). The Paradigm rider is omitted
/// (same gap as Decorum Dissertation et al — copy-spell-from-exile-at-
/// upkeep primitive missing).
pub fn restoration_seminar() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Restoration Seminar",
        cost: cost(&[generic(5), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Nonland),
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

pub fn zimones_experiment() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::g;
    CardDefinition {
        name: "Zimone's Experiment",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            // First: look at top 5, find a creature → hand. Misses go to
            // the bottom of the library (engine default for unmatched).
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(5),
                life_per_revealed: 0,
            },
            // Then: search for a basic land → battlefield tapped. The real
            // card pulls a non-basic land too if it's in the top 5;
            // approximating with a basic-land tutor is close enough at
            // typical cube fidelity (most non-basic lands ETB tapped
            // anyway under our school-land template).
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
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

/// Flow State — {1}{U} Sorcery.
/// "Look at the top three cards of your library. Put one of them into
/// your hand and the rest on the bottom of your library in any order.
/// If there is an instant card and a sorcery card in your graveyard,
/// instead put two of those cards into your hand and the third on the
/// bottom of your library."
///
/// Approximated as `Scry 3 + Draw 1` — the controller orders the top
/// three, then draws the chosen one. The conditional "instead pick 2"
/// upgrade rider when both an instant and a sorcery sit in the
/// graveyard is omitted (no per-zone presence-pair predicate yet, and
/// no "look-and-distribute-by-count" primitive). Net play: the
/// controller filters their next draw, exact-match against the
/// printed mainline mode for typical cast paths.
pub fn flow_state() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Flow State",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
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

/// Follow the Lumarets — {1}{G} Sorcery (push XV).
/// "Infusion — Look at the top four cards of your library. You may
/// reveal a creature or land card from among them and put it into your
/// hand. If you gained life this turn, you may instead reveal two
/// creature and/or land cards from among them and put them into your
/// hand. Put the rest on the bottom of your library in a random
/// order."
///
/// Wired as a conditional `Effect::If(LifeGainedThisTurn)`:
/// - Mainline (no life gain this turn): one `RevealUntilFind` over
///   the top 4 cards, find a creature OR land → hand.
/// - Infusion (life gained this turn): two `RevealUntilFind` calls
///   back-to-back, each over the top 4 (after the first miss-mill).
///
/// Approximations:
/// - Misses go to the graveyard (engine default for `RevealUntilFind`),
///   not the bottom of the library — this is mildly worse for the
///   controller but unlocks gy interactions; tracked under the
///   `look-and-distribute-by-count` engine gap in TODO.md.
/// - The "you may reveal" optionality is collapsed to always-do (the
///   `MayDo` wrapping would just mill 4 cards on a "no" answer, which
///   is strictly worse).
pub fn follow_the_lumarets() -> CardDefinition {
    use crate::effect::{Predicate, ZoneDest};
    use crate::mana::g;
    let creature_or_land =
        SelectionRequirement::Creature.or(SelectionRequirement::Land);
    let single_pull = Effect::RevealUntilFind {
        who: PlayerRef::You,
        find: creature_or_land.clone(),
        to: ZoneDest::Hand(PlayerRef::You),
        cap: Value::Const(4),
        life_per_revealed: 0,
    };
    CardDefinition {
        name: "Follow the Lumarets",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::If {
            cond: Predicate::LifeGainedThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(1),
            },
            // Infusion: pull two creature/land cards.
            then: Box::new(Effect::Seq(vec![
                single_pull.clone(),
                single_pull.clone(),
            ])),
            // Mainline: pull one creature/land card.
            else_: Box::new(single_pull),
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
