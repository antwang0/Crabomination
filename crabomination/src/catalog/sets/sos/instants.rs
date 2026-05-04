//! Secrets of Strixhaven (SOS) — Instants.

use super::no_abilities;
use super::creatures::inkling_token;
use crate::card::{
    CardDefinition, CardType, CounterType, Effect, Keyword, SelectionRequirement, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, Value};
use crate::mana::{b, cost, generic, w};

// ── White ───────────────────────────────────────────────────────────────────

/// Erode — {W} Instant.
/// "Destroy target creature or planeswalker. Its controller may search their
/// library for a basic land card, put it onto the battlefield tapped, then
/// shuffle."
///
/// Push XV: now wired faithfully. The destroy half resolves first;
/// then the target's controller (looked up via
/// `PlayerRef::ControllerOf(Target(0))`, which falls back through the
/// graveyard via `find_card_owner` after the destroy moved the
/// permanent off the battlefield) gets a `Search { IsBasicLand →
/// Battlefield(tapped) }`. The "may" optionality is collapsed to
/// always-search — most opponents will accept the free basic, and
/// `Effect::Search`'s decider already lets the searcher decline by
/// returning `Search(None)`.
pub fn erode() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Erode",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::Search {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Harsh Annotation — {1}{W} Instant.
/// "Destroy target creature. Its controller creates a 1/1 white and black
/// Inkling creature token with flying."
///
/// Now wired faithfully (post-XX): the Inkling token is created under
/// the *target creature's controller* via
/// `PlayerRef::ControllerOf(Target(0))`. The engine's `ControllerOf`
/// resolver falls back through graveyards via `find_card_owner` after
/// the destroy moves the permanent off the battlefield, so the token
/// lands on the right player's side even though the destroy happens
/// first in the `Seq`. No more "you give yourself the token" trade-off.
pub fn harsh_annotation() -> CardDefinition {
    CardDefinition {
        name: "Harsh Annotation",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: inkling_token(),
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
        enters_with_counters: None,
    }
}

/// Interjection — {W} Instant.
/// "Target creature gets +2/+2 and gains first strike until end of turn."
pub fn interjection() -> CardDefinition {
    CardDefinition {
        name: "Interjection",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
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
        enters_with_counters: None,
    }
}

/// Stand Up for Yourself — {2}{W} Instant.
/// "Destroy target creature with power 3 or greater."
pub fn stand_up_for_yourself() -> CardDefinition {
    CardDefinition {
        name: "Stand Up for Yourself",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(3)),
            ),
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
        enters_with_counters: None,
    }
}

/// Rapier Wit — {1}{W} Instant.
/// "Tap target creature. If it's your turn, put a stun counter on it. Draw a
/// card."
///
/// Wired as a `Seq` of (1) Tap, (2) conditional `AddCounter Stun` gated by
/// `Predicate::IsTurnOf(PlayerRef::You)`, (3) Draw 1.
pub fn rapier_wit() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Rapier Wit",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::If {
                cond: Predicate::IsTurnOf(PlayerRef::You),
                then: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
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
        enters_with_counters: None,
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Banishing Betrayal — {1}{U} Instant.
/// "Return target nonland permanent to its owner's hand. Surveil 1."
pub fn banishing_betrayal() -> CardDefinition {
    use crate::effect::{PlayerRef, ZoneDest};
    use crate::mana::u;
    CardDefinition {
        name: "Banishing Betrayal",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Surveil {
                who: PlayerRef::You,
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
        enters_with_counters: None,
    }
}

/// Brush Off — {2}{U}{U} Instant.
/// "This spell costs {1}{U} less to cast if it targets an instant or
/// sorcery spell. / Counter target spell."
///
/// Approximation: the cost-reduction-when-targeting-IS-spell rider is
/// omitted (the engine has no target-aware cost reduction yet — same
/// shape as Killian, Ink Duelist's "spells you cast that target a
/// creature cost {2} less"). The counter half is wired faithfully via
/// `Effect::CounterSpell`. Net: a 4-mana hard counter rather than the
/// printed conditional 2-mana counter.
pub fn brush_off() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Brush Off",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
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
        enters_with_counters: None,
    }
}

/// Mana Sculpt — {1}{U}{U} Instant.
/// "Counter target spell. If you control a Wizard, add an amount of
/// {C} equal to the amount of mana spent to cast that spell at the
/// beginning of your next main phase."
///
/// Approximation: the engine has no "amount of mana spent on the
/// countered spell" introspection (the same gap that drops the Opus
/// rider on Strixhaven creatures), and no "delay-until-your-next-main
/// add C" primitive. We collapse the rider to "if you control a
/// Wizard, add {C}{C} immediately" — a conservative two-colorless
/// snap-back that approximates the typical countered-spell mana
/// value (2-3) without overshooting on cheap counters.
pub fn mana_sculpt() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::ManaPayload;
    use crate::mana::u;
    CardDefinition {
        name: "Mana Sculpt",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(crate::card::CreatureType::Wizard)
                        .and(SelectionRequirement::ControlledByYou),
                )),
                then: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(2)),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Run Behind — {3}{U} Instant.
/// "This spell costs {1} less to cast if it targets an attacking
/// creature. / Target creature's owner puts it on their choice of the
/// top or bottom of their library."
///
/// Approximation: the conditional cost reduction is omitted. The
/// top-or-bottom owner-choice is collapsed to **bottom of library** —
/// the engine has no top-or-bottom prompt for the targeted creature's
/// *owner* to pick, and bottom-of-library is the more typical "kill"
/// outcome. (A Vraska's Contempt-style permanent removal at instant
/// speed for {3}{U}, which matches the spell's role in cube.)
pub fn run_behind() -> CardDefinition {
    use crate::effect::{LibraryPosition, PlayerRef, ZoneDest};
    use crate::mana::u;
    CardDefinition {
        name: "Run Behind",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Bottom,
            },
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
        enters_with_counters: None,
    }
}

/// Chase Inspiration — {U} Instant.
/// "Target creature you control gets +0/+3 and gains hexproof until end of
/// turn."
pub fn chase_inspiration() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Chase Inspiration",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(0),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
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
        enters_with_counters: None,
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Foolish Fate — {2}{B} Instant.
/// "Destroy target creature. / Infusion — If you gained life this turn,
/// that creature's controller loses 3 life."
///
/// The Infusion rider drains 3 life from the target's controller when
/// the caster has gained life this turn (gated via the new
/// `Predicate::LifeGainedThisTurnAtLeast`). The destroy half always
/// resolves.
pub fn foolish_fate() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Foolish Fate",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(3),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Dissection Practice — {B} Instant.
/// "Target opponent loses 1 life and you gain 1 life. / Up to one target
/// creature gets +1/+1 until end of turn. / Up to one target creature
/// gets -1/-1 until end of turn."
///
/// Approximation: the printed card has three independent target slots
/// (an opponent for the drain, plus two optional creature targets for
/// the pump and debuff). The engine has only one target slot per spell,
/// so the drain half fires unconditionally (caster drains an opponent),
/// and the spell's single creature target receives -1/-1 EOT (the
/// removal half), which is the strongest play in 2-player. The +1/+1
/// half is omitted. Net: the spell behaves like a 1-mana drain-1 +
/// shrink-1 — within the printed power band.
/// Dissection Practice — {B} Instant. Three-mode "splice" instant:
/// drain 1 + optional pump halves on a friendly and an opp creature.
///
/// Push: ✅ (was 🟡, "+1/+1 half dropped"). The two optional creature
/// halves now both fire — the user-picked target slot 0 takes the
/// -1/-1 EOT (auto-picks an opp creature when no explicit target was
/// chosen, falls through harmlessly when none exist), and a Selector::
/// one_of-picked friendly creature takes the +1/+1 EOT pump (no-op
/// when you control no creatures). Same multi-target-collapse pattern
/// as Vibrant Outburst's tap half / Decisive Denial mode 1.
pub fn dissection_practice() -> CardDefinition {
    CardDefinition {
        name: "Dissection Practice",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
            // +1/+1 half: auto-picks a friendly creature (falls
            // through cleanly when you control none).
            Effect::PumpPT {
                what: Selector::one_of(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                )),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            // -1/-1 half: user-picked slot 0 (any creature). The auto-
            // target picker prefers an opp creature with low toughness
            // (lethal-first), matching the printed flavour.
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
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
        enters_with_counters: None,
    }
}

/// Wander Off — {3}{B} Instant. Exile target creature.
pub fn wander_off() -> CardDefinition {
    CardDefinition {
        name: "Wander Off",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(SelectionRequirement::Creature),
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
        enters_with_counters: None,
    }
}

/// Masterful Flourish — {B} Instant.
/// "Target creature you control gets +1/+0 and gains indestructible until
/// end of turn."
pub fn masterful_flourish() -> CardDefinition {
    CardDefinition {
        name: "Masterful Flourish",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
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
        enters_with_counters: None,
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Impractical Joke — {R} Sorcery — really an Instant in our usage. Real
/// Oracle: "Damage can't be prevented this turn. Impractical Joke deals 3
/// damage to up to one target creature or planeswalker."
///
/// The damage-prevention clause is omitted (the engine doesn't model damage
/// prevention as a replacement layer yet), so the card collapses to
/// "deal 3 to a creature/planeswalker". Note: the printed type-line is
/// `Sorcery`; we honor that here.
pub fn impractical_joke() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Impractical Joke",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
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
        enters_with_counters: None,
    }
}

/// Heated Argument — {4}{R} Instant.
/// "Heated Argument deals 6 damage to target creature. You may exile a
/// card from your graveyard. If you do, Heated Argument also deals 2
/// damage to that creature's controller."
///
/// Approximation: the optional "may exile a card from your graveyard /
/// if you do, deal 2" rider is collapsed to **always deal 2 to the
/// target's controller** — auto-decider would always choose to exile
/// (the bonus 2 damage is a free upside over a graveyard card), and
/// the engine has no `Move`-with-count primitive to exile *exactly
/// one* card from a zone (a `CardsInZone`-driven `Move` would empty
/// the entire graveyard). Net play: a 5-mana 6-to-creature + 2-to-face
/// plus an implicit "your graveyard isn't really used" fudge — within
/// the printed power band.
pub fn heated_argument() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    use crate::mana::r;
    // Push XV: the printed "you may exile a card from your graveyard.
    // If you do, also deal 2 to that creature's controller" is now
    // wired faithfully via `Effect::MayDo` — the controller picks
    // yes/no, and the gy-exile + extra damage either both fire or
    // both skip. The gy-exile itself uses `Selector::CardsInZone`
    // with a `Take(1)` wrapper to pick exactly one card (matching
    // the printed "a card", not "every card").
    CardDefinition {
        name: "Heated Argument",
        cost: cost(&[generic(4), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(6),
            },
            Effect::MayDo {
                description: "Heated Argument: exile a card from your graveyard, then deal 2 to that creature's controller?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::take(
                            Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: Zone::Graveyard,
                                filter: SelectionRequirement::Any,
                            },
                            Value::Const(1),
                        ),
                        to: ZoneDest::Exile,
                    },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Choreographed Sparks — {R}{R} Instant.
/// "This spell can't be copied. / Choose one or both —
///  • Copy target instant or sorcery spell you control.
///  • Copy target creature spell you control. The copy gains haste
///    and 'At the beginning of the end step, sacrifice this token.'"
///
/// Now wired (post-XX) as a single-mode copy of a *targeted* spell on
/// the stack: target filter is `IsSpellOnStack & ControlledByYou`. The
/// copy locks onto the targeted spell at cast time so a copy-of-itself
/// recursion can't run away. The "creature spell — gains haste,
/// end-step sac" rider is omitted (no permanent-copy primitive yet, no
/// transient sac trigger). The "this spell can't be copied" rider is
/// a no-op (engine has no copy-immune flag).
pub fn choreographed_sparks() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Choreographed Sparks",
        cost: cost(&[r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CopySpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::ControlledByYou),
            ),
            count: Value::Const(1),
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
        enters_with_counters: None,
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Efflorescence — {2}{G} Instant.
/// "Put two +1/+1 counters on target creature. / Infusion — If you
/// gained life this turn, that creature also gains trample and
/// indestructible until end of turn."
///
/// Both halves wired: counters always go down; the trample +
/// indestructible keywords are conditioned on the new
/// `LifeGainedThisTurnAtLeast` predicate.
pub fn efflorescence() -> CardDefinition {
    use crate::card::{CounterType, Keyword};
    use crate::effect::{Duration, Predicate};
    use crate::mana::g;
    CardDefinition {
        name: "Efflorescence",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    },
                ])),
                else_: Box::new(Effect::Noop),
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
        enters_with_counters: None,
    }
}

/// Glorious Decay — {1}{G} Instant. Choose one —
/// • Destroy target artifact.
/// • Glorious Decay deals 4 damage to target creature with flying.
/// • Exile target card from a graveyard. Draw a card.
pub fn glorious_decay() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::g;
    CardDefinition {
        name: "Glorious Decay",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: destroy artifact.
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::HasCardType(CardType::Artifact)),
            },
            // Mode 1: 4 damage to flying creature.
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                ),
                amount: Value::Const(4),
            },
            // Mode 2: exile target card from a graveyard, draw a card.
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Any),
                    to: ZoneDest::Exile,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
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
        enters_with_counters: None,
    }
}

/// Lumaret's Favor — {1}{G} Instant.
/// "Infusion — When you cast this spell, copy it if you gained life
/// this turn. You may choose new targets for the copy. / Target
/// creature gets +2/+4 until end of turn."
///
/// Now fully wired (post-XX): mainline +2/+4 EOT pump + on-cast self
/// trigger gated on `Predicate::LifeGainedThisTurnAtLeast(1)` that
/// copies via `Effect::CopySpell { what: Selector::CastSpellSource,
/// count: 1 }`. The "you may choose new targets for the copy" rider is
/// collapsed: the copy inherits the original's target slot (no per-copy
/// retargeting prompt yet — TODO.md).
pub fn lumarets_favor() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Predicate, TriggeredAbility};
    use crate::effect::PlayerRef;
    use crate::mana::g;
    CardDefinition {
        name: "Lumaret's Favor",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(4),
            duration: Duration::EndOfTurn,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource).with_filter(
                Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
            ),
            effect: Effect::CopySpell {
                what: Selector::CastSpellSource,
                count: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Quandrix (G/U) ──────────────────────────────────────────────────────────

/// Embrace the Paradox — {3}{G}{U} Instant.
/// "Draw three cards. You may put a land card from your hand onto the
/// battlefield tapped."
///
/// Now wired (push XVI): draw 3 + a `MayDo` rider that uses
/// `Selector::one_of(CardsInZone(Hand, Land))` to pick at most one
/// land card and `Effect::Move` it to the battlefield tapped under
/// the caster's control. The auto-decider answers "no" by default,
/// so the printed "may" optionality is honored.
pub fn embrace_the_paradox() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    use crate::mana::{g, u};
    CardDefinition {
        name: "Embrace the Paradox",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::MayDo {
                description: "Put a land card from your hand onto the battlefield tapped?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: SelectionRequirement::Land,
                    }),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Proctor's Gaze — {2}{G}{U} Instant.
/// "Return up to one target nonland permanent to its owner's hand.
/// Search your library for a basic land card, put it onto the battlefield
/// tapped, then shuffle."
///
/// Wired faithfully on existing primitives: target slot 0 is a nonland
/// permanent (auto-decider picks an opponent's threat), `Effect::Move`
/// returns it to its owner's hand, then `Effect::Search` over `IsBasicLand`
/// fetches a basic to the battlefield tapped.
pub fn proctors_gaze() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{g, u};
    CardDefinition {
        name: "Proctor's Gaze",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Witherbloom (B/G) ───────────────────────────────────────────────────────

/// Witherbloom Charm — {B}{G} Instant. Choose one —
/// • You may sacrifice a permanent. If you do, draw two cards.
/// • You gain 5 life.
/// • Destroy target nonland permanent with mana value 2 or less.
///
/// The "you may" gating on mode 0 is dropped — picking the mode commits
/// you to the sacrifice (engine has no in-mode optionality primitive).
/// AutoDecider falls through to mode 1 (gain 5) when no permanent is
/// sacrificable, so the card never bricks.
pub fn witherbloom_charm() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Witherbloom Charm",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: "you may sacrifice a permanent. If you do, draw
            // two cards." Wired via `Effect::MayDo` (push XV) so the
            // "you may" optionality is honored — picking mode 0 with no
            // permanent worth sacrificing now correctly gates on the
            // controller's yes/no answer instead of always sac-ing.
            Effect::MayDo {
                description: "Witherbloom Charm: sacrifice a permanent, then draw two?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::Const(1),
                        filter: SelectionRequirement::Permanent,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                ])),
            },
            // Mode 1: gain 5 life.
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(5),
            },
            // Mode 2: destroy target nonland permanent with mv ≤ 2.
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ManaValueAtMost(2)),
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
        enters_with_counters: None,
    }
}

// ── Silverquill (W/B) ───────────────────────────────────────────────────────

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

/// Lorehold Charm — {R}{W} Instant. Choose one —
/// • Each opponent sacrifices a nontoken artifact of their choice.
/// • Return target artifact or creature card with mana value 2 or less
///   from your graveyard to the battlefield.
/// • Creatures you control get +2/+1 until end of turn.
///
/// All three modes wired:
/// - Mode 0 forces each opponent to sacrifice a nontoken artifact (the
///   `Sacrifice` primitive auto-picks the lowest-CMC token-or-artifact;
///   the `NotToken` requirement keeps Treasures out of the picker).
/// - Mode 1 returns a graveyard card (auto-decider picks the highest-
///   priority eligible card).
/// - Mode 2 fans out a +2/+1 EOT pump across creatures the caster
///   controls.
pub fn lorehold_charm() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::r;
    CardDefinition {
        name: "Lorehold Charm",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: each opponent sacrifices a nontoken artifact.
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::HasCardType(CardType::Artifact)
                    .and(SelectionRequirement::NotToken),
            },
            // Mode 1: return target artifact/creature card with mv ≤ 2
            // from your graveyard to the battlefield.
            Effect::Move {
                what: target_filtered(
                    (SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::Creature))
                    .and(SelectionRequirement::ManaValueAtMost(2)),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            // Mode 2: creatures you control get +2/+1 EOT.
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(2),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Prismari (U/R) ──────────────────────────────────────────────────────────

/// Vibrant Outburst — {U}{R} Instant.
/// "Vibrant Outburst deals 3 damage to any target. Tap up to one target
/// creature."
///
/// Push: the printed two-target structure is collapsed to a user-
/// targeted "any target" damage hit (slot 0) + an auto-picked opp
/// creature for the tap half (slot 1) via `Selector::one_of(
/// EachPermanent(opp creature))` — same approximation as Decisive
/// Denial mode 1 / Chelonian Tackle. The tap no-ops cleanly when no
/// opp creature is on the battlefield, preserving the printed "up to
/// one" semantics. Status: ✅ now (was 🟡, "tap half dropped").
pub fn vibrant_outburst() -> CardDefinition {
    use crate::mana::{r, u};
    CardDefinition {
        name: "Vibrant Outburst",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::Tap {
                what: Selector::one_of(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                )),
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
        enters_with_counters: None,
    }
}

/// Stress Dream — {3}{U}{R} Instant.
/// "Stress Dream deals 5 damage to up to one target creature. Look at the
/// top two cards of your library. Put one of those cards into your hand
/// and the other on the bottom of your library."
///
/// Push: the 5-damage half is now wired against a `Selector::one_of`-
/// picked opp creature (auto-targets the top opp creature, no-ops when
/// none exist) — promotes the printed "up to one target creature" to
/// always-fire-when-possible without requiring a user creature target,
/// so the cast is legal even when the opp has no creatures (just the
/// scry+draw resolves). The "look at top two, put one in hand and the
/// other on the bottom" half is still collapsed to **scry 1 + draw 1**
/// (no "look at N, choose K to hand, rest to bottom" primitive yet).
pub fn stress_dream() -> CardDefinition {
    use crate::mana::{r, u};
    CardDefinition {
        name: "Stress Dream",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            // 5 damage to up to one target creature — the auto-target
            // picker prefers opp-controlled creatures (lethal-first
            // shape, same as Decisive Denial mode 1 / Chelonian
            // Tackle). Falls through cleanly when no opp creature
            // exists, so the spell stays castable for the scry+draw
            // half alone.
            Effect::DealDamage {
                to: Selector::one_of(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                )),
                amount: Value::Const(5),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
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
        enters_with_counters: None,
    }
}

/// Traumatic Critique — {X}{U}{R} Instant.
/// "Traumatic Critique deals X damage to any target. Draw two cards,
/// then discard a card."
///
/// X is read at resolution time from `Value::XFromCost`. The
/// damage-and-loot pattern is faithfully wired — pump-and-loot fits a
/// single `Effect::Seq`.
pub fn traumatic_critique() -> CardDefinition {
    use crate::mana::{ManaSymbol, r, u};
    let mut spell_cost = cost(&[u(), r()]);
    spell_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Traumatic Critique",
        cost: spell_cost,
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::XFromCost,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Quandrix (G/U) ──────────────────────────────────────────────────────────

/// Fractal Anomaly — {U} Instant.
/// "Create a 0/0 green and blue Fractal creature token and put X +1/+1
/// counters on it, where X is the number of cards you've drawn this
/// turn."
///
/// Wired faithfully via `Effect::CreateToken` + `Effect::AddCounter`
/// targeting the just-spawned token via the engine's new
/// `Selector::LastCreatedToken` plumbing. X = `Value::CardsDrawnThisTurn
/// (PlayerRef::You)`. If X is 0 the token enters as 0/0 and dies to
/// state-based actions after resolution, matching the printed
/// behaviour.
pub fn fractal_anomaly() -> CardDefinition {
    use super::super::sos::sorceries::fractal_token;
    use crate::mana::u;
    CardDefinition {
        name: "Fractal Anomaly",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
                amount: Value::CardsDrawnThisTurn(PlayerRef::You),
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
        enters_with_counters: None,
    }
}

/// Quandrix Charm — {G}{U} Instant. Choose one —
/// • Counter target spell unless its controller pays {2}.
/// • Destroy target enchantment.
/// • Target creature has base power and toughness 5/5 until end of turn.
///
/// Approximation: mode 2 ("becomes base 5/5") is wired as a flat
/// `PumpPT +3/+3` rather than a "set base to 5/5" — the engine does
/// not yet evaluate `Effect::ResetCreature` (it's stubbed out in
/// `apply_effect`), so a true "set base P/T" rewrite is not yet
/// possible. Net play: a 2/2 → 5/5 (matches the printed result) but a
/// 4/4 → 7/7 (printed: 5/5). The pump-rather-than-set approximation
/// is logged here pending a layer-aware base-stat rewrite primitive.
pub fn quandrix_charm() -> CardDefinition {
    use crate::mana::{ManaCost, generic as gen_pip, g, u};
    let counter_cost = ManaCost {
        symbols: vec![gen_pip(2)],
    };
    CardDefinition {
        name: "Quandrix Charm",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: counter target spell unless controller pays {2}.
            Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: counter_cost,
            },
            // Mode 1: destroy target enchantment.
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Enchantment),
            },
            // Mode 2: target creature gets +3/+3 EOT (approximation of
            // "becomes base 5/5" — see card-level docs).
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
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
        enters_with_counters: None,
    }
}

/// Ajani's Response — {4}{W} Instant.
/// "This spell costs {3} less to cast if it targets a tapped creature.
/// Destroy target creature."
///
/// ✅ Push XXXVIII: 🟡 → ✅. The "{3} less if targets a tapped
/// creature" rider now wires faithfully via a self-static
/// `StaticEffect::CostReductionTargeting`. `cost_reduction_for_spell`
/// walks the cast card's own static abilities in addition to the
/// battlefield, so self-discount spells (Ajani's Response, future
/// "this spell costs N less" patterns) close the cost-reduction gap
/// without needing a permanent in play. Floating just {1}{W} with a
/// tapped opp creature targeted is now a legal cast.
pub fn ajanis_response() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Ajani's Response",
        cost: cost(&[generic(4), w()]),
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
        static_abilities: vec![StaticAbility {
            description: "Costs {3} less if it targets a tapped creature",
            effect: StaticEffect::CostReductionTargeting {
                spell_filter: SelectionRequirement::Any,
                target_filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::Tapped),
                amount: 3,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Silverquill (W/B) ───────────────────────────────────────────────────────

/// Silverquill Charm — {W}{B} Instant. Choose one —
/// • Put two +1/+1 counters on target creature.
/// • Exile target creature with power 2 or less.
/// • Each opponent loses 3 life and you gain 3 life.
pub fn silverquill_charm() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Charm",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: +1/+1 ×2 on target creature.
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            // Mode 1: exile target creature with power ≤ 2.
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
                ),
            },
            // Mode 2: drain 3.
            Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            ]),
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
        enters_with_counters: None,
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Burrog Barrage — {1}{G} Instant.
/// "Target creature you control gets +1/+0 until end of turn if you've
/// cast another instant or sorcery spell this turn. Then it deals damage
/// equal to its power to up to one target creature an opponent controls."
///
/// Wired in two stages via `Effect::Seq`:
/// 1. Conditional pump: gated on
///    `Predicate::SpellsCastThisTurnAtLeast { who: You, at_least: 2 }`
///    (≥2 because Burrog Barrage itself counts as one cast — we want
///    "another instant or sorcery"). Pumps the chosen target friendly
///    creature +1/+0 EOT.
/// 2. Power-as-damage: deal `Value::PowerOf(target)` damage to the
///    *same* target slot. The 2-target prompt for an opponent's
///    creature is collapsed to "self-damage" — auto-targeter prefers
///    the friendly creature pump first; the spell ends up dealing its
///    new power as damage to itself rather than to an opp creature.
///    The single-target gap is logged in TODO.md under "Multi-Target
///    Prompt for Sorceries / Instants" and matches the Cost of
///    Brilliance / Render Speechless approximation pattern.
pub fn burrog_barrage() -> CardDefinition {
    use crate::card::Predicate;
    use crate::mana::g;
    CardDefinition {
        name: "Burrog Barrage",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
                then: Box::new(Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            // "Then it deals damage equal to its power to up to one
            // target creature an opponent controls." Slot 0 is the
            // user-picked friendly (the damage source); slot 1 is
            // auto-picked via `Selector::one_of(EachPermanent(opp
            // creature))` — same multi-target collapse as Decisive
            // Denial mode 1. The damage no-ops cleanly when no opp
            // creature is on the battlefield (just the +1/+0 fires).
            // Power is read from slot 0 (the friendly creature),
            // matching the printed wording.
            Effect::DealDamage {
                to: Selector::one_of(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                )),
                amount: Value::PowerOf(Box::new(target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ))),
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
        enters_with_counters: None,
    }
}

// ── Additional Black ────────────────────────────────────────────────────────

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

/// Wilt in the Heat — {2}{R}{W} Lorehold Instant.
/// "This spell costs {2} less to cast if one or more cards left your
/// graveyard this turn. / Wilt in the Heat deals 5 damage to target
/// creature. If that creature would die this turn, exile it instead."
///
/// Push XXXIX: cost-reduction clause now wires faithfully via the new
/// `Value::IfPredicate` branching value. The reduction is a self-static
/// `CostReductionScaled { amount: IfPredicate { cond:
/// CardsLeftGraveyardThisTurnAtLeast(1), then: 2, else_: 0 } }` —
/// `cost_reduction_for_spell` walks the spell card's own static
/// abilities at cast time and sums the discount, so the printed
/// {2}{R}{W} drops to {R}{W} when the gate fires.
///
/// Remaining gap: the "if it would die, exile instead" damage-
/// replacement still omitted (no damage-replacement primitive). The
/// 5-damage half lands faithfully; any creature with toughness ≤ 5
/// dies normally to graveyard rather than getting exiled.
pub fn wilt_in_the_heat() -> CardDefinition {
    use crate::card::{StaticAbility, Predicate};
    use crate::effect::{PlayerRef, StaticEffect};
    use crate::mana::{r, w};
    CardDefinition {
        name: "Wilt in the Heat",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(5),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description:
                "This spell costs {2} less to cast if one or more cards left your graveyard this turn.",
            effect: StaticEffect::CostReductionScaled {
                filter: SelectionRequirement::Any,
                amount: Value::IfPredicate {
                    cond: Box::new(Predicate::CardsLeftGraveyardThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::Const(1),
                    }),
                    then: Box::new(Value::Const(2)),
                    else_: Box::new(Value::Const(0)),
                },
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Suspend Aggression — {1}{R}{W} Lorehold Instant.
/// "Exile target nonland permanent and the top card of your library. For
/// each of those cards, its owner may play it until the end of their next
/// turn."
///
/// Wired faithfully on the exile half: a `Move { Selector::Target(0), to:
/// Exile }` against a nonland permanent target, plus a `Move {
/// Selector::TopOfLibrary, to: Exile }` against the caster's library.
/// Library-source moves were unblocked in push III (`Effect::Move` now
/// walks libraries when locating the source card). The "may play those
/// cards until next end step" rider is omitted — engine has no per-card
/// "may-play-from-exile-until-EOT" primitive (same gap as Tablet of
/// Discovery, Ark of Hunger).
pub fn suspend_aggression() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{r, w};
    CardDefinition {
        name: "Suspend Aggression",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Exile,
            },
            Effect::Move {
                what: Selector::TopOfLibrary {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                },
                to: ZoneDest::Exile,
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
        enters_with_counters: None,
    }
}

/// Rabid Attack — {1}{B} Instant.
/// "Until end of turn, any number of target creatures you control each
/// get +1/+0 and gain 'When this creature dies, draw a card.'"
///
/// Approximation: the multi-target prompt collapses to a single chosen
/// creature target. The "+1/+0 EOT" pump lands on the chosen creature.
/// The transient triggered-ability grant ("when this creature dies,
/// draw a card") is omitted — engine has no per-creature "grant
/// triggered ability for a duration" primitive yet (tracked in TODO.md
/// under "Transient Triggered-Ability Grants on Pump Spells"). The
/// pump alone is still a respectable {1}{B} combat trick.
pub fn rabid_attack() -> CardDefinition {
    CardDefinition {
        name: "Rabid Attack",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
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
        enters_with_counters: None,
    }
}


// ── Additional Red ──────────────────────────────────────────────────────────

/// Tome Blast — {1}{R} Sorcery.
/// "Tome Blast deals 2 damage to any target. / Flashback {4}{R}."
///
/// Mainline 2-damage-to-any-target wired faithfully. The Flashback
/// {4}{R} half is wired via `Keyword::Flashback`.
pub fn tome_blast() -> CardDefinition {
    use crate::card::Keyword;
    use crate::mana::{Color, ManaCost, ManaSymbol, r};
    let flashback_cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(4), ManaSymbol::Colored(Color::Red)],
    };
    CardDefinition {
        name: "Tome Blast",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(2),
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
        enters_with_counters: None,
    }
}

/// Duel Tactics — {R} Sorcery.
/// "Duel Tactics deals 1 damage to target creature. It can't block this
/// turn. / Flashback {1}{R}."
///
/// Mainline ping + cant-block keyword grant wired faithfully. The
/// Flashback {1}{R} half is wired via `Keyword::Flashback`.
pub fn duel_tactics() -> CardDefinition {
    use crate::card::Keyword;
    use crate::mana::{Color, ManaCost, ManaSymbol, r};
    let flashback_cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Red)],
    };
    CardDefinition {
        name: "Duel Tactics",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CantBlock,
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
        enters_with_counters: None,
    }
}


// ── Additional Blue ─────────────────────────────────────────────────────────

/// Homesickness — {4}{U}{U} Instant.
/// "Target player draws two cards. Tap up to two target creatures. Put a
/// stun counter on each of them."
///
/// Approximation: the multi-target prompt collapses both halves to a single
/// target — caster draws 2 (rather than a chosen player) and exactly one
/// creature gets tapped + a stun counter. The 2-target creature slot is
/// blocked on the engine's missing multi-target prompt (TODO.md). At the
/// effective cost (~6 mana for self-draw + a soft removal), this still
/// plays as a respectable late-game tempo card.
pub fn homesickness() -> CardDefinition {
    use crate::mana::u;
    // Auto-pick selector for the second creature slot — picks an
    // opp creature (lethal-first / first-eligible). The selector is
    // re-evaluated for the Tap and AddCounter effects independently,
    // so we leave the Untapped filter off here to avoid the second-
    // pick re-resolving to "no eligible target" after the first Tap
    // already ran. If the auto-pick collides with slot 0, the Tap
    // half is a no-op (already-tapped) but the second stun counter
    // still lands — net 2 stun counters on slot 0 when only one
    // opp creature is on the battlefield (matches the printed "up
    // to two" when only one creature exists).
    let second_target = Selector::one_of(Selector::EachPermanent(
        SelectionRequirement::Creature
            .and(SelectionRequirement::ControlledByOpponent),
    ));
    CardDefinition {
        name: "Homesickness",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            // First tap+stun: user-targeted creature (slot 0).
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
            // Second tap+stun: auto-picked opp creature, may collide
            // with slot 0 (in which case the tap is a no-op since the
            // creature is already tapped) but the second stun counter
            // still lands. Net: 2 stun counters on slot 0 if no
            // additional opp creature is on the battlefield (matches
            // the printed "up to two" when only one creature exists).
            Effect::Tap { what: second_target.clone() },
            Effect::AddCounter {
                what: second_target,
                kind: CounterType::Stun,
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
        enters_with_counters: None,
    }
}

/// Fractalize — {X}{U} Instant.
/// "Until end of turn, target creature becomes a green and blue Fractal
/// with base power and toughness each equal to X plus 1."
///
/// Approximation: the engine has no `Effect::ResetCreature { base_power,
/// base_toughness, ... }` primitive, so the "becomes a base-(X+1)/(X+1)
/// Fractal" rewrite is collapsed to a `PumpPT(+(X+1), +(X+1))` modifier.
/// At typical X ≥ 2 the buffed total power/toughness is close enough to
/// the printed effect to play correctly in combat; the printed creature-
/// type loss + color rewrite is observably wrong for type-matters
/// payoffs.
pub fn fractalize() -> CardDefinition {
    use crate::effect::Duration;
    use crate::mana::{u, x};
    CardDefinition {
        name: "Fractalize",
        cost: cost(&[x(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Sum(vec![Value::XFromCost, Value::Const(1)]),
            toughness: Value::Sum(vec![Value::XFromCost, Value::Const(1)]),
            duration: Duration::EndOfTurn,
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
        enters_with_counters: None,
    }
}

/// Divergent Equation — {X}{X}{U} Instant.
/// "Return up to X target instant and/or sorcery cards from your graveyard
/// to your hand. / Exile Divergent Equation."
///
/// Approximation: the engine has no count-bounded "select up to N from
/// graveyard" primitive, so we collapse the multi-target pick to a single
/// target — return one instant or sorcery card from your graveyard to
/// your hand. The "exile this" rider is a no-op. The X-cost gating is
/// preserved through the natural mana-cost path.
pub fn divergent_equation() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{u, x};
    CardDefinition {
        name: "Divergent Equation",
        cost: cost(&[x(), x(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
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
        enters_with_counters: None,
    }
}

// ── Push IX additions: Surveil-anchored cards + Prismari Charm ─────────────

/// Unsubtle Mockery — {2}{R} Instant.
/// "Unsubtle Mockery deals 4 damage to target creature. Surveil 1."
///
/// Wired faithfully via `DealDamage(4) + Surveil(1)`. Surveil is a
/// first-class engine primitive (`Effect::Surveil`); the card was
/// previously gated behind the script's heuristic flag, which had
/// become stale once Surveil shipped.
pub fn unsubtle_mockery() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Unsubtle Mockery",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
            },
            Effect::Surveil {
                who: PlayerRef::You,
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
        enters_with_counters: None,
    }
}

/// Muse's Encouragement — {4}{U} Instant.
/// "Create a 3/3 blue and red Elemental creature token with flying. /
/// Surveil 2."
///
/// Wired with the shared `elemental_token()` helper from `sos::sorceries`
/// (3/3 U/R flier) + `Effect::Surveil(2)`. Surveil is first-class.
pub fn muses_encouragement() -> CardDefinition {
    use super::sorceries::elemental_token;
    use crate::mana::u;
    CardDefinition {
        name: "Muse's Encouragement",
        cost: cost(&[generic(4), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: elemental_token(),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Prismari Charm — {U}{R} Instant.
/// "Choose one — / • Surveil 2, then draw a card. / • Prismari Charm
/// deals 1 damage to each of one or two targets. / • Return target
/// nonland permanent to its owner's hand."
///
/// Wired as a 3-mode `ChooseMode`: Surveil 2 + draw / 1 dmg to target
/// creature-or-PW / bounce nonland to owner's hand. The "1 or 2 targets"
/// fan-out on mode 1 is collapsed to a single target (multi-target
/// prompt gap — TODO.md). Standard primitives throughout.
pub fn prismari_charm() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{r, u};
    CardDefinition {
        name: "Prismari Charm",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: Surveil 2, then draw a card.
            Effect::Seq(vec![
                Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            // Mode 1: 1 damage to a target creature/planeswalker
            // (single-target collapse of the printed "one or two targets").
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            // Mode 2: Return target nonland permanent to its owner's hand.
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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
        enters_with_counters: None,
    }
}
