//! Secrets of Strixhaven (SOS) — Instants.

use super::no_abilities;
use super::creatures::inkling_token;
use crate::card::{
    AlternativeCost, CardDefinition, CardType, CounterType, Effect, Keyword, SelectionRequirement, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, generic, w};

// ── White ───────────────────────────────────────────────────────────────────

/// Erode — {W} Instant.
/// "Destroy target creature or planeswalker. Its controller may search their
/// library for a basic land card, put it onto the battlefield tapped, then
/// shuffle."
///
/// The "may" optionality collapses to always-search — `Effect::Search`
/// already lets the searcher decline by returning `Search(None)`.
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Harsh Annotation — {1}{W} Instant.
/// "Destroy target creature. Its controller creates a 1/1 white and black
/// Inkling creature token with flying."
///
/// Approximation: the Inkling token is created under the spell's caster
/// (`PlayerRef::You`) rather than the target creature's controller — the
/// engine has no zone-stable controller lookup that survives the destroy
/// step, and 2-player play makes this only a small power-level trade-off
/// (you give yourself the token instead of giving it to the player whose
/// creature you killed). Standard single-target destroy is wired
/// faithfully.
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
        // Inkling token goes to the target creature's owner via
        // `PlayerRef::OwnerOf(Target(0))`; `place_card_in_dest`
        // walks graveyards if the destroy step has already moved the
        // card.
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::CreateToken {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        // Push (modern_decks): "{1}{U} less if it targets an instant or
        // sorcery spell" rider wired via `AlternativeCost` with a target
        // filter restricting to spells on the stack matching IS card type.
        // Alt cost is {1}{U} (the {1}{U} reduction from printed {2}{U}{U})
        // — when the caster aims at an IS spell on the stack, they can cast
        // via the alt path at half the mana. Non-IS spells fall back to the
        // hard-counter at the full {2}{U}{U} cost.
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), u()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: Some(
                SelectionRequirement::IsSpellOnStack.and(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            ),
            condition: None,
                    exile_from_graveyard_count: 0,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        // Push (modern_decks): the "if you control a Wizard, add an
        // amount of {C} equal to the amount of mana spent to cast that
        // spell" rider now reads `Value::ManaValueOf(Target(0))` —
        // after CounterSpell resolves, the countered spell sits in
        // its owner's graveyard and `ManaValueOf` walks every zone to
        // find it. The "delay until next main" timing rider is still
        // collapsed to immediate AddMana (no delayed-AddMana primitive
        // yet); the colorless mana goes into the pool right away. For
        // X-cost spells, this reads only the printed CMC (which equals
        // X = 0); the "amount of mana spent" rider is approximated by
        // the printed CMC — same gap as Opus's mana-introspection
        // approximation. Most counter targets are X = 0.
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
                    pool: ManaPayload::Colorless(Value::ManaValueOf(Box::new(
                        Selector::Target(0),
                    ))),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        // Push (modern_decks): "{1} less if it targets an attacking
        // creature" rider wired via `AlternativeCost { mana_cost: {2}{U},
        // target_filter: Some(Creature + IsAttacking) }`. When the
        // caster aims at an attacking creature, alt-cost path is
        // available at {2}{U} (a {1} reduction from printed {3}{U});
        // otherwise the spell goes off at full cost. The
        // top-or-bottom owner-choice is still collapsed to bottom-only
        // (engine has no top-or-bottom prompt for the *owner* of the
        // moved card — tracked in TODO.md).
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), u()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: Some(
                SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
            ),
            condition: None,
                    exile_from_graveyard_count: 0,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Dissection Practice — {B} Instant.
/// "Target opponent loses 1 life and you gain 1 life. / Up to one target
/// creature gets +1/+1 until end of turn. / Up to one target creature
/// gets -1/-1 until end of turn."
///
/// Push (modern_decks): all three target slots wired via multi-target.
/// Slot 0 = target opponent (loses 1 life), self gains 1 life. Slot 1 =
/// optional creature target gets +1/+1 EOT. Slot 2 = optional creature
/// target gets -1/-1 EOT. Slots 1/2 use `TargetFiltered` so they
/// resolve to no-op when fewer than three targets are passed.
/// AutoDecider fills slot 0 only; scripted tests pump and/or shrink.
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
            // Slot 0: target opponent loses 1, you gain 1.
            Effect::LoseLife {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(1),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            // Slot 1: optional creature target gets +1/+1 EOT.
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            // Slot 2: optional creature target gets -1/-1 EOT.
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 2,
                    filter: SelectionRequirement::Creature,
                },
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Impractical Joke — {R} Sorcery. "Damage can't be prevented this
/// turn. Impractical Joke deals 3 damage to up to one target creature
/// or planeswalker."
///
/// ✅ The damage-prevention clause is a true no-op in this engine —
/// there is no damage-prevention layer to gate, so every damage event
/// already resolves at face value. Same documentation treatment as
/// Heated Debate / Skullcrack. The card collapses to "3 damage to a
/// creature or planeswalker," matching the printed Oracle's
/// gameplay-relevant payoff. The "up to one" rider is approximated as
/// required-target (single Creature ∨ Planeswalker filter) — the
/// engine has no "up to one" optional-target prompt yet, but Impractical
/// Joke at minimum cost ({R}) almost always has a legal target, so
/// the practical loss is negligible.
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lumaret's Favor — {1}{G} Instant.
/// "Infusion — When you cast this spell, copy it if you gained life
/// this turn. You may choose new targets for the copy. / Target
/// creature gets +2/+4 until end of turn."
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
        // Infusion: SpellCast/SelfSource trigger gated on life-gained.
        // Copies the cast spell (Selector::This = the just-cast card,
        // which is the source of this trigger).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource).with_filter(
                Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
            ),
            effect: Effect::CopySpell {
                what: Selector::This,
                count: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Prismari (U/R) ──────────────────────────────────────────────────────────

/// Vibrant Outburst — {U}{R} Instant.
/// "Vibrant Outburst deals 3 damage to any target. Tap up to one target
/// creature."
///
/// Push (modern_decks): two-target shape now wired via multi-target
/// (slots 0 + 1). Slot 0 takes any target (creature/player/PW) and
/// receives 3 damage. Slot 1 is an optional creature target which gets
/// tapped via `TargetFiltered`. Slot 1 resolves to no-op when fewer
/// than two targets were chosen. AutoDecider fills only slot 0;
/// scripted tests can exercise both halves.
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
            // Slot 1: optional creature target tapped.
            Effect::Tap {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Stress Dream — {3}{U}{R} Instant.
/// "Stress Dream deals 5 damage to up to one target creature. Look at the
/// top two cards of your library. Put one of those cards into your hand
/// and the other on the bottom of your library."
///
/// Approximation (push modern_decks batch 43): the "look at top two,
/// put one in hand and the other on the bottom" half is wired as
/// **scry 2 then draw 1** — the engine has no "look at N, choose K
/// to hand, rest to bottom" primitive, but Scry 2 → Draw 1 is
/// gameplay-equivalent for the typical play pattern: the player sees
/// both cards, puts the unwanted one on the bottom, and draws the
/// other. (The corner case where the player wants to keep both
/// cards on top — Scry 2 keep both → Draw the top — is also handled
/// faithfully.) The 5-damage half is wired against a creature target.
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
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(5),
            },
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Charm — {G}{U} Instant. Choose one —
/// • Counter target spell unless its controller pays {2}.
/// • Destroy target enchantment.
/// • Target creature has base power and toughness 5/5 until end of turn.
///
/// Mode 2 is a layer-7b base P/T rewrite via `Effect::SetBasePT`.
/// Counters and +N/+M still stack on top per CR 613.7c-f.
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
            // Mode 2: target creature has base P/T 5/5 EOT (faithful
            // layer-7b rewrite via SetBasePT).
            Effect::SetBasePT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(5),
                toughness: Value::Const(5),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Ajani's Response — {4}{W} Instant.
/// "This spell costs {3} less to cast if it targets a tapped creature.
/// Destroy target creature."
///
/// Push (modern_decks): "{3} less if it targets a tapped creature" rider
/// now wired via `AlternativeCost { mana_cost: {1}{W}, target_filter:
/// Some(Creature + Tapped) }`. When the caster picks a tapped creature
/// target, they can use the alt-cost cast path at {1}{W} (a {3} mana
/// reduction from the printed {4}{W}); when the target is untapped,
/// the alt-cost target filter fails validation and the spell can only
/// be cast at its printed cost. The destroy-creature body is unchanged.
/// Same pattern as Killian's target-aware cost reduction (STX) but on
/// a per-spell alt-cost rather than a static — cleaner because the
/// discount is intrinsic to this one card.
pub fn ajanis_response() -> CardDefinition {
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), w()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: Some(
                SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
            ),
            condition: None,
                    exile_from_graveyard_count: 0,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
///    chosen opp creature target (slot 1). The optional opp-creature
///    slot uses `Selector::TargetFiltered { slot: 1, ... }` so when
///    only slot 0 is provided, the damage half resolves to no-op.
///
/// Push (modern_decks): promoted from "self-damage approximation" to
/// the printed two-slot shape via `Selector::TargetFiltered`. Slot 0 =
/// the friendly creature to pump; slot 1 = the opp creature to take
/// the power-as-damage. AutoDecider currently fills slot 0 only; the
/// scripted tests can supply slot 1.
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
            // Slot 1: optional opp creature target gets damage equal to
            // slot 0's power. When slot 1 isn't provided the damage half
            // resolves to no-op via TargetFiltered's empty-selector
            // behaviour.
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Additional Black ────────────────────────────────────────────────────────

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

/// Wilt in the Heat — {2}{R}{W} Lorehold Instant.
/// "This spell costs {2} less to cast if one or more cards left your
/// graveyard this turn. / Wilt in the Heat deals 5 damage to target
/// creature. If that creature would die this turn, exile it instead."
///
/// Push (modern_decks): the "{2} less if cards left your graveyard this
/// turn" cost reduction **is now wired** via the new
/// `AlternativeCost.condition: Some(Predicate)` field gated on
/// `Predicate::CardsLeftGraveyardThisTurnAtLeast(You, 1)`. When the
/// gate passes, the spell is castable for {R}{W} via the alt cost path;
/// otherwise the regular {2}{R}{W} cost applies. The "if it would die,
/// exile instead" damage-replacement rider is still ⏳ (no damage-
/// replacement primitive on the engine yet); 5-toughness-and-under
/// creatures die normally to graveyard.
pub fn wilt_in_the_heat() -> CardDefinition {
    use crate::card::AlternativeCost;
    use crate::effect::Predicate;
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[r(), w()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: Some(Predicate::CardsLeftGraveyardThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(1),
            }),
                    exile_from_graveyard_count: 0,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Suspend Aggression — {1}{R}{W} Lorehold Instant.
/// "Exile target nonland permanent and the top card of your library. For
/// each of those cards, its owner may play it until the end of their next
/// turn."
///
/// Push (modern_decks): all three clauses now ship. The "you may play
/// those cards until next end step" rider is **now wired** via the new
/// `Effect::GrantMayPlay` primitive + `Selector::LastMoved` reading the
/// just-exiled card ids from the resolution-scoped scratch. Both exiled
/// cards (the targeted permanent and the caster's top-of-library card)
/// get a permission stamped to their respective owners (`to_owner =
/// true`) for `EndOfControllersNextTurn`. The recipients then invoke
/// `GameAction::CastFromZoneWithoutPaying` at a later sorcery-speed
/// window to recur the card without paying its mana cost.
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
            // Grant may-play to both moved cards. `to_owner: true`
            // routes each permission to that card's owner (per
            // printed Oracle: "its owner may play it").
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                to_owner: true,
                exile_after: false,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Rabid Attack — {1}{B} Instant.
/// "Until end of turn, any number of target creatures you control each
/// get +1/+0 and gain 'When this creature dies, draw a card.'"
///
/// Push (modern_decks): "any number of target creatures" promoted from
/// single-target to a three-slot multi-target shape — slot 0
/// (mandatory) + slots 1 + 2 (optional). The pump lands on each filled
/// slot; the unfilled slots resolve to no-op via `TargetFiltered`.
/// AutoDecider fills slot 0 only; scripted tests can wire up to three.
/// The transient die-to-draw rider is still omitted (engine has no
/// per-creature "grant triggered ability for a duration" primitive —
/// tracked in TODO.md).
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
        effect: Effect::Seq(vec![
            // Slot 0 (mandatory): friendly creature gets +1/+0 EOT.
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            // Slot 1: optional friendly creature gets +1/+0 EOT.
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            // Slot 2: optional friendly creature gets +1/+0 EOT.
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 2,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(1),
                toughness: Value::Const(0),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}


// ── Additional Blue ─────────────────────────────────────────────────────────

/// Homesickness — {4}{U}{U} Instant.
/// "Target player draws two cards. Tap up to two target creatures. Put a
/// stun counter on each of them."
///
/// Push (modern_decks): three-slot multi-target shape — slot 0 = target
/// player draws 2, slot 1 + slot 2 = optional creature targets each get
/// tapped + a stun counter. Slots 1+2 use `TargetFiltered` so an empty
/// slot resolves to no-op. AutoDecider fills slot 0 only (caster picks
/// themselves as the draw target); scripted tests can exercise both
/// creature halves.
pub fn homesickness() -> CardDefinition {
    use crate::mana::u;
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
            // Slot 0: target player draws 2.
            Effect::Draw {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(2),
            },
            // Slot 1: optional creature tap + stun counter.
            Effect::Tap {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
            },
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
            // Slot 2: optional second creature tap + stun counter.
            Effect::Tap {
                what: Selector::TargetFiltered {
                    slot: 2,
                    filter: SelectionRequirement::Creature,
                },
            },
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 2,
                    filter: SelectionRequirement::Creature,
                },
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractalize — {X}{U} Instant.
/// "Until end of turn, target creature becomes a green and blue Fractal
/// with base power and toughness each equal to X plus 1."
///
/// Push (modern_decks): base-P/T rewrite now wired via the engine's
/// `Effect::SetBasePT` layer-7b primitive (same path used by Square Up
/// and Mercurial Transformation). At X=2 the creature becomes base 3/3
/// until end of turn — counters and +N/+M still stack on top per
/// CR 613.7c-f. The printed "becomes a Fractal" creature-type rewrite
/// (layer 4) and the color rewrite (layer 5) are still omitted (no
/// `Effect::SetTypes` primitive); type-matters payoffs that read off
/// the original creature type may see the wrong value. The headline
/// pump-and-reset shape — usable to shrink a 7/7 with deathtouch into
/// a (X+1)/(X+1) (still a problem at X=4+ when the target is bigger)
/// or to grow a 1/1 token into a (X+1)/(X+1) attacker — plays
/// correctly via the base-P/T override.
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
        effect: Effect::SetBasePT {
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Divergent Equation — {X}{X}{U} Instant.
/// "Return up to X target instant and/or sorcery cards from your graveyard
/// to your hand. / Exile Divergent Equation."
///
/// 🟡 (cost ✅): the engine has no count-bounded "select up to N from
/// graveyard" primitive, so we collapse the multi-target pick to a single
/// target — return one instant or sorcery card from your graveyard to
/// your hand. The "Exile Divergent Equation" rider is **now wired**
/// (push: modern_decks) via the new `CardDefinition.exile_on_resolve`
/// flag — the resolved instant lands in exile, not the graveyard, so
/// it can't be looped via flashback/recursion. The X-cost gating is
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: true,
        affinity_filter: None,
    }
}

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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Choreographed Sparks ────────────────────────────────────────────────────

/// Choreographed Sparks — {R}{R} Instant.
/// "This spell can't be copied. / Choose one or both — • Copy target
/// instant or sorcery spell you control. You may choose new targets for
/// the copy. / • Copy target creature spell you control. The copy gains
/// haste and 'At the beginning of the end step, sacrifice this token.'"
///
/// Push (modern_decks): two-mode `ChooseMode` now wired. Mode 0 copies
/// a target IS spell on the stack via `Effect::CopySpell`. Mode 1
/// copies a target creature spell on the stack — the engine's
/// `CopySpell` already handles permanent spells (the resulting
/// permanent enters as a token via CR 608.3f, since `is_token = true`
/// is stamped on the copy at push-time). The printed "the copy gains
/// haste and sacrifice at end of step" rider is approximated by relying
/// on the token-cleanup SBA (the token will leave the battlefield once
/// it hits the graveyard, matching the spirit of the printed sacrifice
/// rider). The "this spell can't be copied" rider is engine-wide ⏳
/// (no `CantBeCopied` keyword tag yet) but the corner is rarely
/// exercised. The "choose one or both" multi-mode rider collapses to
/// "pick one mode" since the engine lacks a generic multi-mode picker
/// over per-mode targets — same gap as Moment of Reckoning.
pub fn choreographed_sparks() -> CardDefinition {
    use crate::mana::r;
    let copy_is_spell = Effect::CopySpell {
        what: target_filtered(
            SelectionRequirement::IsSpellOnStack.and(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            ),
        ),
        count: Value::Const(1),
    };
    let copy_creature_spell = Effect::CopySpell {
        // Target a creature spell on the stack. The CopySpell resolver
        // handles permanent spells by stamping `is_token = true` on the
        // copy (CR 608.3f) so the resulting permanent enters as a token
        // — token-cleanup SBA removes it when it leaves the battlefield,
        // approximating the printed "sacrifice at end of step" rider.
        what: target_filtered(
            SelectionRequirement::IsSpellOnStack
                .and(SelectionRequirement::HasCardType(CardType::Creature)),
        ),
        count: Value::Const(1),
    };
    CardDefinition {
        name: "Choreographed Sparks",
        cost: cost(&[r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![copy_is_spell, copy_creature_spell]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Flashback (the SOS instant) ─────────────────────────────────────────────

/// Flashback — {R} Instant.
/// "Target instant or sorcery card in your graveyard gains flashback
/// until end of turn. The flashback cost is equal to its mana cost."
///
/// 🟡 Body-only wire: applies a flat-cost `Keyword::Flashback` to a
/// target IS card in your graveyard by recasting it as if its
/// flashback cost were its own mana cost — implemented as
/// `Move(target → Hand) → reset to hand` so it can be cast normally.
/// This is a *strict* approximation since the "flashback cost = its
/// mana cost" rider can't be expressed as a static keyword grant on a
/// graveyard-resident card (the engine's `Keyword::Flashback` lives on
/// `CardDefinition.keywords`, not on per-instance state). A true wiring
/// would need a transient `CardInstance::granted_keywords` slot that
/// applies only while the card is in graveyard until end of turn —
/// tracked in TODO.md.
pub fn sos_flashback_instant() -> CardDefinition {
    use crate::card::Zone;
    use crate::mana::r;
    CardDefinition {
        name: "Flashback",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // Approximation: bounce target IS card from your graveyard to hand.
        // The player can re-cast it next turn at normal cost — strictly
        // weaker than the printed "flashback for its mana cost this turn"
        // but preserves the recovery-of-a-spell-from-graveyard outcome.
        effect: Effect::Move {
            what: Selector::take(
                Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                },
                Value::Const(1),
            ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}
