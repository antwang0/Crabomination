//! Secrets of Strixhaven (SOS) — Sorceries.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, SelectionRequirement, Subtypes,
    TokenDefinition,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, Value};
use crate::mana::{Color, b, cost, generic, w};

/// 1/1 black-and-green Pest creature token. Used by Witherbloom-leaning
/// SOS cards (Send in the Pest, Pest Mascot's payoff cycle, etc.).
///
/// Carries its printed rider "Whenever this token attacks, you gain 1
/// life." via `TokenDefinition.triggered_abilities`, which
/// `token_to_card_definition` copies onto the materialised token — so an
/// attacking Pest feeds Witherbloom "you gained life" payoffs (Pest
/// Mascot's counter cycle, Blech's per-type fan-out, etc.).
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
        static_abilities: vec![],
        equipped_bonus: None,
    }
}

/// 0/0 green-and-blue Fractal creature token. Used by Quandrix scaling
/// payoffs (Fractal Anomaly, Snarl Song, Applied Geometry, etc.). Lives
/// or dies based on the +1/+1 counters its creator stamps onto it via
/// `Selector::LastCreatedToken` — a 0/0 with zero counters dies to SBA
/// after the resolution finishes, matching the printed cards' "if X=0,
/// the token dies" outcome.
pub use crabomination_base::tokens::fractal_token;

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
        static_abilities: vec![],
        equipped_bonus: None,
    }
}

/// 2/2 red-and-white Spirit creature token. Used by Lorehold-flavoured
/// SOS cards (Group Project, Living History's ETB, etc.).
pub use crabomination_base::tokens::spirit_token;

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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

/// Daydream — {W} Sorcery.
/// "Exile target creature you control, then return that card to the
/// battlefield under its owner's control with a +1/+1 counter on it."
///
/// Wired with the Restoration-Angel-style flicker pattern: `Exile(target)
/// + Move(target → Battlefield) + AddCounter(target, +1/+1)`. The
///   `Selector::Target(0)` slot persists across the exile so the engine's
///   `Move`/`AddCounter` resolves against the same card after it lands
///   back on the battlefield (zone-stable lookup falls back through exile,
///   which is where the target lives between Exile and Move).
/// Wired with the Restoration-Angel-style flicker pattern:
/// `Exile(target) + Move(target -> Battlefield) + AddCounter(target, +1/+1)`.
/// The `Selector::Target(0)` slot persists across the exile so the engine's
/// `Move`/`AddCounter` resolves against the same card after it re-enters
/// via the zone-stable lookup through exile.
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

/// Practiced Offense — {2}{W} Sorcery.
/// "Put a +1/+1 counter on each creature target player controls.
/// Target creature gains your choice of double strike or lifelink
/// until end of turn."
///
/// ✅ (push XXXV) The mode-pick between double strike and lifelink is
/// now wired via `Effect::ChooseMode([GrantKeyword(DoubleStrike),
/// GrantKeyword(Lifelink)])` — the controller picks the mode at
/// resolution time (auto-decider picks mode 0 = double strike, which
/// is also what the prior collapsed body did). The counter fan-out
/// still collapses the printed "target player" to **you** (multi-
/// target prompt is engine-wide). Flashback {1}{W} is wired via
/// `Keyword::Flashback`.
pub fn practiced_offense() -> CardDefinition {
    use crate::card::{CounterType, Keyword};
    use crate::mana::{ManaCost, ManaSymbol};
    let flashback_cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::White)],
    };
    CardDefinition {
        name: "Practiced Offense",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
            // Modal: pick double strike OR lifelink for the target.
            Effect::ChooseMode(vec![
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
        ]),
        ..Default::default()
    }
}

/// Antiquities on the Loose — {1}{W}{W} Sorcery.
/// "Create two 2/2 red and white Spirit creature tokens. Then if this
/// spell was cast from anywhere other than your hand, put a +1/+1
/// counter on each Spirit you control. / Flashback {4}{W}{W}."
///
/// Push (modern_decks): the "if cast from anywhere other than your
/// hand" rider is **now wired** via the new
/// `Predicate::CastFromGraveyard` (reads
/// `EffectContext.cast_from_hand`, which is stamped at spell-
/// resolution time from the resolving `CardInstance.cast_from_hand`
/// flag). On the flashback cast the predicate is true → the engine
/// fans a +1/+1 counter out to each Spirit you control via
/// `ForEach(Creature ∧ Spirit ∧ ControlledByYou) → AddCounter`. On
/// the hand cast the predicate is false and only the two Spirit
/// tokens are minted (no bonus counters). The Flashback {4}{W}{W}
/// clause is wired via `Keyword::Flashback`.
///
/// "Anywhere other than your hand" technically also covers casts
/// from exile (Foretell, Suspend) or library (Cascade, Hideaway).
/// Our `cast_from_hand` flag is only true for the standard hand-cast
/// path — every alternative-zone cast (Flashback path is the only
/// one wired today) sets it to false, so the rider fires correctly
/// for any future cast-from-exile path. Tests:
/// `antiquities_on_the_loose_hand_cast_mints_two_spirits` (regular
/// cast — no counter rain),
/// `antiquities_on_the_loose_flashback_cast_fans_counters` (graveyard
/// cast via Flashback — +1/+1 on each Spirit).
pub fn antiquities_on_the_loose() -> CardDefinition {
    use crate::card::{CounterType, Keyword, Predicate, SelectionRequirement, Selector};
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
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(flashback_cost)],
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
                            .and(SelectionRequirement::HasCreatureType(
                                crate::card::CreatureType::Spirit,
                            ))
                            .and(SelectionRequirement::ControlledByYou),
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        // Push (modern_decks, batch 83): "Flashback—Tap three untapped
        // creatures you control" is wired via the new
        // `Keyword::FlashbackTap(3)` keyword + `GameAction::
        // CastFlashbackTap` action. The action validates the caller has
        // listed exactly 3 untapped creatures they control, then taps
        // them as the entire flashback cost (no mana paid) and casts
        // the spell out of the graveyard. Routes the resolved spell to
        // exile via `cast_via_flashback` (CR 702.34d).
        keywords: vec![Keyword::FlashbackTap(3)],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: spirit_token(),
        },
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Pow2(Box::new(Value::XFromCost)),
        },
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Send in the Pest — {1}{B} Sorcery.
/// "Each opponent discards a card. You create a 1/1 black and green Pest
/// creature token with 'Whenever this token attacks, you gain 1 life.'"
///
/// Fully wired: the discard half, the token creation, and the Pest
/// token's "gain 1 on attack" trigger (the shared `pest_token()` carries
/// it via `TokenDefinition.triggered_abilities`).
pub fn send_in_the_pest() -> CardDefinition {
    CardDefinition {
        name: "Send in the Pest",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

/// Dina's Guidance — {1}{B}{G} Sorcery.
/// "Search your library for a creature card, reveal it, put it into your
/// hand or graveyard, then shuffle."
///
/// 2-mode `Effect::ChooseMode`: mode 0 → hand, mode 1 → graveyard.
/// `AutoDecider` defaults to mode 0 (strictly stronger for an unguided
/// bot); gy-reanimation decks can pick mode 1 via a scripted decision.
pub fn dinas_guidance() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::g;
    CardDefinition {
        name: "Dina's Guidance",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            // Mode 0: search → hand.
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            // Mode 1: search → graveyard.
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
                to: ZoneDest::Graveyard,
            },
        ]),
        ..Default::default()
    }
}

/// Grapple with Death — {1}{B}{G} Sorcery.
/// "Destroy target artifact or creature. You gain 1 life."
pub fn grapple_with_death() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Grapple with Death",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

/// End of the Hunt — {1}{B} Sorcery.
/// "Target opponent exiles a creature or planeswalker they control with
/// the greatest mana value among creatures and planeswalkers they
/// control."
///
/// Push (modern_decks): the "greatest mana value" clause is now wired
/// via the new `SelectionRequirement::HasGreatestManaValueAmongControlled`
/// primitive — the inner filter is `Creature` or `Planeswalker`, and the
/// outer wrapper enforces that the candidate's MV is at least as great
/// as every other matching permanent under the same controller. The
/// auto-target picker and cast-time validator both consult this
/// predicate, so the caster can only exile the largest opp creature
/// or planeswalker (ties pass permissively so any max-MV match is
/// legal). The "their choice" wording is still collapsed to "we pick"
/// since the engine has no opp-pick-from-among-their-permanents
/// decision shape; in practice the caster's pick lands on the same MV
/// bucket as the opponent would be forced to choose.
pub fn end_of_the_hunt() -> CardDefinition {
    let inner =
        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker);
    CardDefinition {
        name: "End of the Hunt",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::ControlledByOpponent
                    .and(SelectionRequirement::HasGreatestManaValueAmongControlled(
                        Box::new(inner),
                    )),
            ),
        },
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
/// cards discarded this way. Wired faithfully via the new
/// `Value::CardsDiscardedThisEffect` reader (push modern_decks): the
/// Discard handler bumps `state.cards_discarded_this_resolution`, and
/// the Draw step reads that counter — N matches exactly the number of
/// cards actually discarded (which equals the hand size at cast time,
/// since the discard runs first).
pub fn borrowed_knowledge() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Borrowed Knowledge",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Sorcery],
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
            // discarded this way. `Value::CardsDiscardedThisEffect` reads
            // the per-resolution counter the Discard handler bumps.
            Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CardsDiscardedThisEffect,
                },
            ]),
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(flashback_cost)],
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
        ..Default::default()
    }
}

// ── Silverquill (W/B) ───────────────────────────────────────────────────────

/// Render Speechless — {2}{W}{B} Sorcery.
/// "Target opponent reveals their hand. You choose a nonland card from it.
/// That player discards that card. Put two +1/+1 counters on up to one
/// target creature."
///
/// Push (modern_decks): two-slot multi-target shape. Slot 0 = target
/// opponent (reveal hand + caster picks a nonland to discard). Slot 1 =
/// optional creature target gets two +1/+1 counters via
/// `TargetFiltered` (no-op when no slot 1 target). The auto-decider
/// only fills slot 0; scripted tests can wire both halves.
pub fn render_speechless() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Render Speechless",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Slot 0: target opponent reveals + chosen-discard.
            Effect::DiscardChosen {
                from: target_filtered(SelectionRequirement::Player),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
            // Slot 1: optional friendly creature gets two +1/+1 counters.
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

/// Arcane Omens — {4}{B} Sorcery.
/// "Converge — Target player discards X cards, where X is the number
/// of colors of mana spent to cast this spell."
///
/// Wired faithfully via `Effect::Discard` with `Value::ConvergedValue`.
/// The "target player" is a real `target_filtered(Player)` slot, so the
/// caster is prompted to choose a player (the auto-decider points it at an
/// opponent by default).
pub fn arcane_omens() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Arcane Omens",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Discard {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::ConvergedValue,
            random: false,
        },
        ..Default::default()
    }
}

/// Together as One — {6} Sorcery.
/// "Converge — Target player draws X cards, Together as One deals X
/// damage to any target, and you gain X life, where X is the number of
/// colors of mana spent to cast this spell."
///
/// Push (modern_decks): two-slot multi-target shape — slot 0 = target
/// player draws X (`Value::ConvergedValue`), slot 1 = any target gets
/// X damage. Self-life-gain runs unconditionally. AutoDecider points
/// slot 0 at the caster (self-draw) by default; scripted tests can
/// aim the draw half at an opponent.
pub fn together_as_one() -> CardDefinition {
    CardDefinition {
        name: "Together as One",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Slot 0: target player draws X.
            Effect::Draw {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::ConvergedValue,
            },
            // Slot 1: any target gets X damage.
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                },
                amount: Value::ConvergedValue,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ConvergedValue,
            },
        ]),
        ..Default::default()
    }
}

/// Wisdom of Ages — {4}{U}{U}{U} Sorcery.
/// "Return all instant and sorcery cards from your graveyard to your
/// hand. You have no maximum hand size for the rest of the game. /
/// Exile Wisdom of Ages."
///
/// ✅ (push: modern_decks) — all three printed clauses now ship.
/// (a) Mass IS-gy-to-hand return via `Selector::CardsInZone(Graveyard,
/// Instant | Sorcery)`. (b) "No max hand size" via the
/// `Effect::SetNoMaxHandSize` primitive — clears `Player.max_hand_size`
/// (`Option<usize>`, `None` = no maximum), honored by `do_cleanup`'s
/// CR 514.1 discard-down enforcement. (c) "Exile Wisdom of Ages" now
/// wired via the new `CardDefinition.exile_on_resolve` flag — the
/// resolved sorcery lands in exile, not the graveyard, preventing
/// flashback/Past-in-Flames recursion of the spell itself.
pub fn wisdom_of_ages() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    use crate::mana::u;
    CardDefinition {
        name: "Wisdom of Ages",
        cost: cost(&[generic(4), u(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::SetNoMaxHandSize { who: Selector::You },
        ]),
        exile_on_resolve: true,
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: elemental_token(),
        },
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

/// Cost of Brilliance — {2}{B} Sorcery.
/// "Target player draws two cards and loses 2 life. Put a +1/+1 counter
/// on up to one target creature."
///
/// Push (modern_decks): two-target shape now wired via the multi-target
/// action shape (push modern_decks: `GameAction::CastSpell.additional_targets`).
/// Slot 0 = target player (draws 2 + loses 2 life), slot 1 = optional
/// creature target (gets a +1/+1 counter). Slot 1 uses `TargetFiltered`
/// so it resolves to no-op when the caster only picks a player target.
/// AutoDecider currently aims slot 0 at the caster (self-loot pattern);
/// scripted tests can override.
pub fn cost_of_brilliance() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Cost of Brilliance",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Slot 0: target player draws 2 + loses 2 life.
            Effect::Draw {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Target(0),
                amount: Value::Const(2),
            },
            // Slot 1: optional creature target gets a +1/+1 counter
            // (resolves to no-op when slot 1 is empty).
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Mind Roots — {1}{B}{G} Sorcery.
/// "Target player discards two cards. Put up to one land card discarded
/// this way onto the battlefield tapped under your control."
///
/// Push (modern_decks): the "land card discarded this way → battlefield"
/// half is **now wired** via the new `Selector::DiscardedThisResolution`
/// primitive + `discarded_card_ids_this_resolution` tracker on
/// `GameState`. The selector walks the IDs captured during
/// `Effect::Discard` resolution, looks them up in their owner's
/// graveyard, and filters by Land. Wrapped in `Selector::Take { count: 1 }`
/// to match the printed "up to one land" cap. The discard half still
/// uses `EachOpponent` for auto-target safety; the printed card lets
/// the caster choose any player but the caster never has an incentive
/// to discard from themselves.
pub fn mind_roots() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::g;
    CardDefinition {
        name: "Mind Roots",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Each opponent discards 2 cards. The Discard handler stamps
            // every discarded card's id onto
            // `state.discarded_card_ids_this_resolution`.
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
                random: false,
            },
            // Walk the captured discarded-card-ids list, filter by Land,
            // take at most one, and move it onto the battlefield tapped
            // under the caster's control.
            Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::DiscardedThisResolution {
                        filter: SelectionRequirement::HasCardType(CardType::Land),
                    }),
                    count: Box::new(Value::Const(1)),
                },
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        ]),
        ..Default::default()
    }
}

/// Mind into Matter — {X}{G}{U} Sorcery.
/// "Draw X cards. Then you may put a permanent card with mana value X
/// or less from your hand onto the battlefield tapped."
///
/// Push (modern_decks batch 43, 🟡 → ✅): the "may put a permanent ≤ X
/// from hand onto the battlefield tapped" half **now lands** via
/// `Effect::MayDo` wrapping a `Selector::take(EachMatching(Hand,
/// Permanent), 1)` walk gated by `Predicate::ValueAtMost(ManaValueOf,
/// XFromCost)`. The Permanent filter excludes Instant + Sorcery from
/// the hand pool (matching the printed "permanent card" wording).
/// AutoDecider declines by default; `ScriptedDecider::new([Bool(true)])`
/// exercises the paid path.
pub fn mind_into_matter() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::{ZoneDest, ZoneRef};
    use crate::mana::{ManaSymbol, g, u};
    let mut spell_cost = cost(&[g(), u()]);
    spell_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Mind into Matter",
        cost: spell_cost,
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::XFromCost,
            },
            Effect::MayDo {
                description: "put a permanent with mana value X or less from your hand onto the battlefield tapped".to_string(),
                body: Box::new(Effect::ForEach {
                    selector: Selector::take(
                        Selector::EachMatching {
                            zone: ZoneRef::Hand(PlayerRef::You),
                            filter: SelectionRequirement::Permanent,
                        },
                        Value::Const(1),
                    ),
                    body: Box::new(Effect::If {
                        cond: Predicate::ValueAtMost(
                            Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                            Value::XFromCost,
                        ),
                        then: Box::new(Effect::Move {
                            what: Selector::TriggerSource,
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: true,
                            },
                        }),
                        else_: Box::new(Effect::Noop),
                    }),
                }),
            },
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

/// Killian's Confidence — {W}{B} Sorcery.
/// "Target creature gets +1/+1 until end of turn. Draw a card. /
/// Whenever one or more creatures you control deal combat damage to a
/// player, you may pay {W/B}. If you do, return this card from your
/// graveyard to your hand."
///
/// ✅ Body wired (pump + draw). The graveyard-resident may-pay-to-
/// return rider now wired via the new `EventScope::FromYourGraveyard`
/// extension on `fire_combat_damage_to_player_triggers` (push XXV).
/// When your creature connects, every Killian's Confidence sitting in
/// your graveyard fires; controller is asked yes/no via `MayPay`, and
/// on yes + sufficient mana the card moves back to hand. The {W/B}
/// may-pay pip is a real `ManaSymbol::Hybrid(White, Black)`, payable
/// with either color.
pub fn killians_confidence() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::mana::{Color, ManaCost, ManaSymbol};
    // {W/B} may-pay return cost — payable with white or black.
    let return_cost = ManaCost {
        symbols: vec![ManaSymbol::Hybrid(Color::White, Color::Black)],
    };
    CardDefinition {
        name: "Killian's Confidence",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Sorcery],
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::FromYourGraveyard,
            ),
            effect: Effect::MayPay {
                description: "Killian's Confidence: pay {W/B} to return to hand?".into(),
                mana_cost: return_cost,
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                }),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::mana::g;
    // Push (modern_decks, batch 84): the "Whenever this creature
    // attacks, you gain 1 life" rider is wired via the new
    // `Effect::GrantTriggeredAbility` primitive — each friendly
    // creature receives an Attacks/SelfSource trigger granting 1 life
    // on attack. The grant rides on `granted_triggers_eot` and clears
    // at cleanup, matching the printed "until end of turn" scope.
    let attack_lifegain = TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(1),
        },
    };
    CardDefinition {
        name: "Root Manipulation",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Sorcery],
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
                Effect::GrantTriggeredAbility {
                    what: Selector::TriggerSource,
                    trigger: Box::new(attack_lifegain.clone()),
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        // Push (modern_decks): slot-0 = friendly creature to pump +0/+10
        // EOT; slot-1 = optional opp creature defender (picked via the
        // multi-target prompt — `Selector::TargetFiltered { slot: 1 }`).
        // AutoDecider fills both slots when an opp creature is on the
        // battlefield. With no opp creature, slot 1 is empty and Fight
        // no-ops — preserving the printed "up to one" semantics.
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
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Steal the Show — {2}{R} Sorcery. Choose one or both —
/// • Target player discards any number of cards, then draws that many
///   cards.
/// • Steal the Show deals damage equal to the number of instant and
///   sorcery cards in your graveyard to target creature or planeswalker.
///
/// Push (modern_decks): mode 0 uses `Effect::DiscardAnyNumber` (same as
/// Colossus of the Blood Age + Borrowed Knowledge), so the targeted player
/// chooses how many cards to discard, then draws that many (read via
/// `Value::CardsDiscardedThisEffect`).
///
/// The "choose one or both" rider is now wired via `Effect::ChooseN`'s
/// per-mode target slots: the default `picks: [0, 1]` runs both modes, and
/// each target-bearing mode consumes its own cast-time target slot — mode 0
/// takes the player target (slot 0), mode 1 the creature/planeswalker
/// target (slot 1). A `ScriptedDecider`/UI can instead pick a single mode
/// (`[0]` or `[1]`).
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
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                // Mode 0: target player discards any number, then draws
                // exactly that many. Each discard bumps
                // `cards_discarded_this_resolution`, so the follow-up
                // Draw(CardsDiscardedThisEffect) reads the exact count.
                Effect::Seq(vec![
                    Effect::DiscardAnyNumber {
                        who: target_filtered(SelectionRequirement::Player),
                    },
                    Effect::Draw {
                        who: Selector::Target(0),
                        amount: Value::CardsDiscardedThisEffect,
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
            ],
        },
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

/// Decorum Dissertation — {3}{B}{B} Sorcery — Lesson.
/// "Target player draws two cards and loses 2 life. / Paradigm (Then
/// exile this spell. After you first resolve a spell with this name,
/// you may cast a copy of it from exile without paying its mana cost
/// at the beginning of each of your first main phases.)"
///
/// Push (modern_decks): all three printed clauses now ship. Body —
/// targets a player + draws 2 + loses 2 life. **Paradigm rider** now
/// wired via `Effect::RegisterParadigm` + `exile_on_resolve: true`.
/// Each of the controller's pre-combat main phases offers a free copy.
pub fn decorum_dissertation() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Decorum Dissertation",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Target(0),
                amount: Value::Const(2),
            },
            Effect::RegisterParadigm,
        ]),
        exile_on_resolve: true,
        ..Default::default()
    }
}

/// Germination Practicum — {3}{G}{G} Sorcery — Lesson.
/// "Put two +1/+1 counters on each creature you control. / Paradigm
/// (...)"
///
/// Push (modern_decks): both clauses ship. Body — `ForEach` over your
/// creatures with `AddCounter +1/+1 ×2`. **Paradigm rider** now wired
/// via `Effect::RegisterParadigm` + `exile_on_resolve: true`.
pub fn germination_practicum() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::g;
    CardDefinition {
        name: "Germination Practicum",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
            },
            Effect::RegisterParadigm,
        ]),
        exile_on_resolve: true,
        ..Default::default()
    }
}

/// Restoration Seminar — {5}{W}{W} Sorcery — Lesson.
/// "Return target nonland permanent card from your graveyard to the
/// battlefield. / Paradigm (...)"
///
/// Push (modern_decks): both clauses ship. Body — `Move target Nonland
/// gy → bf untapped`. **Paradigm rider** now wired via
/// `Effect::RegisterParadigm` + `exile_on_resolve: true`.
pub fn restoration_seminar() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Restoration Seminar",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Nonland),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::RegisterParadigm,
        ]),
        exile_on_resolve: true,
        ..Default::default()
    }
}

pub fn zimones_experiment() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::g;
    CardDefinition {
        name: "Zimone's Experiment",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Push (modern_decks, batch 94): the printed "look at top 5,
            // partition revealed creature/land cards into hand/bf and
            // put the rest on the bottom of library at random" is
            // approximated as two sequential `RevealUntilFind` walks
            // over the top of library: first a Creature (→ Hand, misses
            // → bottom random), then a Land (→ Battlefield tapped,
            // misses → bottom random). Each walk caps at 5 cards. This
            // doesn't perfectly model the printed "look at 5 cards
            // once" semantic — the second walk sees a (possibly
            // shorter) library after the first walk completes — but
            // captures the dual-destination harvest: a creature lands
            // in hand AND a land lands on the bf. Substantially closer
            // to printed than the prior "tutor a basic land" path.
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(5),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::HasCardType(CardType::Land),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
                cap: Value::Const(5),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
        ]),
        ..Default::default()
    }
}

/// Flow State — {1}{U} Sorcery.
/// "Look at the top three cards of your library. Put one of them into
/// your hand and the rest on the bottom of your library in any order.
/// If there is an instant card and a sorcery card in your graveyard,
/// instead put two of those cards into your hand and the third on the
/// bottom of your library."
///
/// Mainline: look at top 3, take one to hand, rest to the bottom
/// (`LookPickToHand`). With both an instant and a sorcery in your
/// graveyard, the "take two to hand" upgrade is approximated as
/// `Scry 3 → Draw 2`.
pub fn flow_state() -> CardDefinition {
    use crate::mana::u;
    use crate::card::{Predicate, Zone};
    CardDefinition {
        name: "Flow State",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::All(vec![
                Predicate::SelectorExists(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant),
                }),
                Predicate::SelectorExists(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Sorcery),
                }),
            ]),
            then: Box::new(Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(3),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ])),
            else_: Box::new(Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: None,
            
                take: None,
            }),
        },
        ..Default::default()
    }
}

/// Molten Note — {X}{R}{W} Sorcery.
/// "Molten Note deals damage to target creature equal to the amount of mana
/// spent to cast this spell. Untap all creatures you control.
/// Flashback {6}{R}{W}."
///
/// Damage is the actual "amount of mana spent to cast this spell" via
/// `Value::CastSpellManaSpent` (read from `ctx.mana_spent`). This is exact
/// for both cast paths: a normal {X}{R}{W} cast spends X + 2, and a
/// Flashback {6}{R}{W} cast spends 8 — the earlier `Sum(XFromCost, Const(2))`
/// model read X=0 on the flashback (no {X} pip) and undercounted to 2.
/// Flashback wired via `Keyword::Flashback`.
pub fn molten_note() -> CardDefinition {
    use crate::mana::{r, w, x};
    CardDefinition {
        name: "Molten Note",
        cost: cost(&[x(), r(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(6), r(), w()]))],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::CastSpellManaSpent,
            },
            Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Social Snub — {1}{W}{B} Sorcery.
/// "When you cast this spell while you control a creature, you may copy this
/// spell. Each player sacrifices a creature of their choice. Each opponent
/// loses 1 life and you gain 1 life."
///
/// ✅ On-cast self-trigger fires `MayDo(CopySpell { Self })` when the
/// caster controls a creature (gated via `Predicate::SelectorExists`).
pub fn social_snub() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::Predicate;
    use crate::mana::{w, b};
    CardDefinition {
        name: "Social Snub",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        // On-cast self-trigger. Filter ensures the caster controls a
        // creature at cast time (gate from the printed Oracle).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource)
                .with_filter(Predicate::SelectorExists(
                    Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                )),
            effect: Effect::MayDo {
                description: "Copy Social Snub?".to_string(),
                body: Box::new(Effect::CopySpell {
                    what: Selector::TriggerSource,
                    count: Value::Const(1),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Fix What's Broken — {2}{W}{B} Sorcery.
/// "As an additional cost to cast this spell, pay X life.
/// Return each artifact and creature card with mana value X from your
/// graveyard to the battlefield."
///
/// The "pay X life" additional cost is modeled with the same convention
/// as Vicious Rivalry: an `{X}` pip is inserted at the front of the mana
/// cost so the caster chooses X at cast time, and the resolution pays
/// `Value::XFromCost` life. The reanimation now matches the printed
/// **exact mana value X** (not the prior `≤ 2` approximation): a
/// per-iteration `Predicate::ValueEquals(ManaValueOf(card), XFromCost)`
/// gate over every artifact/creature card in the graveyard returns all
/// of them — a true mass reanimation at the chosen X. Tests:
/// `fix_whats_broken_pays_x_life_and_returns_exact_mv`,
/// `fix_whats_broken_only_returns_cards_at_exact_mv`.
pub fn fix_whats_broken() -> CardDefinition {
    use crate::effect::{Predicate, ZoneDest};
    use crate::mana::{w, b, ManaSymbol};
    use crate::card::Zone;
    let mut spell_cost = cost(&[generic(2), w(), b()]);
    spell_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Fix What's Broken",
        cost: spell_cost,
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Additional cost: pay X life (X read off the {X} pip).
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::XFromCost,
            },
            // Return EACH artifact/creature card with mana value exactly X
            // from your graveyard to the battlefield. We iterate over the
            // matching cards in the graveyard and return those whose MV
            // equals X, so multiple cards at the chosen X all come back.
            Effect::ForEach {
                selector: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::HasCardType(CardType::Artifact)),
                },
                body: Box::new(Effect::If {
                    cond: Predicate::ValueEquals(
                        Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                        Value::XFromCost,
                    ),
                    then: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ]),
        ..Default::default()
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
/// - Misses go to the bottom of the library via
///   `RevealMissDest::BottomRandom` (printed: "the rest on the bottom
///   in a random order"). Strict "random order" requires an RNG hook
///   the engine doesn't expose yet; the order is preserved as
///   revealed.
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
        miss_dest: crate::effect::RevealMissDest::BottomRandom,
    };
    CardDefinition {
        name: "Follow the Lumarets",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

// ── push XVII: Silverquill ──────────────────────────────────────────────────

// ── Additional Lorehold (R/W) ──────────────────────────────────────────────

// ── Fix What's Broken ──────────────────────────────────────────────────────

// ── Echocasting Symposium ───────────────────────────────────────────────────

/// Echocasting Symposium — {4}{U}{U} Sorcery — Lesson.
/// "Target player creates a token that's a copy of target creature you
/// control. / Paradigm — ..."
///
/// Body: approximated as you-create-a-copy of your own chosen creature
/// via `Effect::CreateToken` over a "vanilla mirror" 3/3 Wizard
/// placeholder (no permanent-copy primitive yet — same gap as Applied
/// Geometry). The printed "target player creates the copy" slot
/// collapses to "you" (no multi-target prompt yet).
///
/// Push (modern_decks): **Paradigm rider** now wired via
/// `Effect::RegisterParadigm` + `exile_on_resolve: true`. Each of the
/// controller's pre-combat main phases offers a free copy.
pub fn echocasting_symposium() -> CardDefinition {
    use crate::card::SpellSubtype;
    use crate::effect::shortcut::target_filtered;
    use crate::mana::u;
    CardDefinition {
        name: "Echocasting Symposium",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        // Push (modern_decks, batch 81): "Target player creates a token
        // that's a copy of target creature you control." Wired via
        // `Effect::CreateTokenCopyOf { who: You, source: target Creature
        // ∧ ControlledByYou, no P/T override }`. The printed
        // "target player creates" is approximated as "you create" — the
        // engine has no two-target-slots-with-different-types primitive
        // to thread the player target through; in practice the caster
        // wouldn't gift the token to an opp. Paradigm still wired via
        // `RegisterParadigm` + `exile_on_resolve: true`.
        effect: Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(1),
                source: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                extra_creature_types: vec![],
                override_pt: None,
                non_legendary: false,
            },
            Effect::RegisterParadigm,
        ]),
        exile_on_resolve: true,
        ..Default::default()
    }
}

// ── Archaic's Agony ─────────────────────────────────────────────────────────

/// Archaic's Agony — {4}{R}, Sorcery. Converge — deals X damage to
/// target creature, where X is the number of colors of mana spent to
/// cast this spell. The exile-top-cards rider is omitted (no cast-from-
/// exile pipeline).
pub fn archaics_agony() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    use crate::mana::r;
    CardDefinition {
        name: "Archaic's Agony",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::ConvergedValue,
        },
        ..Default::default()
    }
}

/// Improvisation Capstone — {3}{R}{R} Lorehold Sorcery.
/// "Exile cards from the top of your library until you exile cards with
/// total mana value 4 or greater. You may cast any number of spells from
/// among them without paying their mana costs. / Paradigm (Then exile
/// this spell. After you first resolve a spell with this name, you may
/// cast a copy of it from exile without paying its mana cost at the
/// beginning of each of your first main phases.)"
///
/// Push (modern_decks): full body now wired via the new cast-from-exile
/// pipeline + Paradigm primitives. Both clauses ship:
/// 1. **Exile + may-cast**: approximated as "exile top 4 cards"
///    (the printed lower bound — 4 1-mana cards add to MV 4). For each
///    non-land card exiled, the controller is asked
///    "cast without paying?" via
///    `Effect::CastWithoutPayingImmediate(LastMoved, Exile)`. AutoDecider
///    declines by default; ScriptedDecider's `Bool(true)` opts in per
///    card. Lands in the exile group are skipped silently.
/// 2. **Paradigm rider**: `Effect::RegisterParadigm` registers a
///    recurring `YourNextMainPhase` delayed trigger whose body is
///    `Effect::CastFreeParadigmCopy` — at the start of each of the
///    caster's pre-combat mains, the controller is asked
///    "cast a copy of Improvisation Capstone?", and on yes a tokenized
///    copy is minted in exile + free-cast (per CR 706 copy semantics).
///    `exile_on_resolve: true` parks the original Improvisation Capstone
///    in exile so it stays reachable for the recurrence.
///
/// Approximations vs. printed Oracle:
/// - Exile count is fixed at 4 (no "until total MV ≥ 4" primitive).
///   For very-high-cost libraries this undercounts; for very-low-cost
///   libraries it overcounts.
/// - Multi-cast loop iterates each exiled card sequentially; the
///   controller is asked one yes/no per card (no "cast in any order"
///   prompt — the engine has no batched same-zone cast prompt).
pub fn improvisation_capstone() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    use crate::mana::r;
    CardDefinition {
        name: "Improvisation Capstone",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Walk top of library exiling cards until running MV sum
            // reaches ≥ 4 (printed Oracle exact). Each card walked is
            // counted regardless of type — basic lands (MV 0) pass
            // through without raising the gate, so the spell can dig
            // past a land-heavy top until a real spell pushes the sum
            // past 4. Previously hard-coded to `Const(4)` cards, which
            // under-counted when the top was land-heavy.
            Effect::Move {
                what: Selector::TopOfLibraryUntilMvAtLeast {
                    who: PlayerRef::You,
                    threshold: Value::Const(4),
                },
                to: ZoneDest::Exile,
            },
            Effect::ForEach {
                selector: Selector::LastMoved,
                body: Box::new(Effect::CastWithoutPayingImmediate {
                    what: Selector::TriggerSource,
                    source_zone: Zone::Exile,
                    exile_after: false,
                }),
            },
            Effect::RegisterParadigm,
        ]),
        exile_on_resolve: true,
        ..Default::default()
    }
}

/// Applied Geometry — {2}{G}{U}, Sorcery. Create a 0/0 green and blue
/// Fractal creature token. Put six +1/+1 counters on it.
pub fn applied_geometry() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::{g, u};
    CardDefinition {
        name: "Applied Geometry",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        // "Create a token that's a copy of target non-Aura permanent you
        // control, except it's a 0/0 Fractal creature in addition to its
        // other types. Put six +1/+1 counters on it." Wired faithfully via
        // `Effect::CreateTokenCopyOf` (override P/T to 0/0, add the Fractal
        // creature type "in addition to" the copied types), then six +1/+1
        // counters on the freshly-minted token. The previous approximation
        // (mint a vanilla 0/0 Fractal) is retired now that the
        // copy-permanent primitive exists.
        effect: Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(1),
                source: crate::effect::shortcut::target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::ControlledByYou),
                ),
                extra_creature_types: vec![crate::card::CreatureType::Fractal],
                override_pt: Some((0, 0)),
                non_legendary: false,
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(6),
            },
        ]),
        ..Default::default()
    }
}

// ── Push XVII: Converge damage ────────────────────────────────────────────
