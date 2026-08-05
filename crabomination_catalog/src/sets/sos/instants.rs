//! Secrets of Strixhaven (SOS) — Instants.

use super::creatures::inkling_token;
use crate::card::{CardDefinition, CardType, CounterType, Effect, Keyword, SelectionRequirement};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, Value};
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

/// Harsh Annotation — {1}{W} Instant.
/// "Destroy target creature. Its controller creates a 1/1 white and black
/// Inkling creature token with flying." The token goes to the dead
/// creature's CONTROLLER via `PlayerRef::ControllerOf` — resolved off
/// the death-time LKI snapshot (CR 608.2h), so a stolen creature's
/// token correctly goes to the thief, not the owner.
pub fn harsh_annotation() -> CardDefinition {
    CardDefinition {
        name: "Harsh Annotation",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

/// Interjection — {W} Instant.
/// "Target creature gets +2/+2 and gains first strike until end of turn."
pub fn interjection() -> CardDefinition {
    CardDefinition {
        name: "Interjection",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

/// Stand Up for Yourself — {2}{W} Instant.
/// "Destroy target creature with power 3 or greater."
pub fn stand_up_for_yourself() -> CardDefinition {
    CardDefinition {
        name: "Stand Up for Yourself",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(3)),
            ),
        },
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

/// Brush Off — {2}{U}{U} Instant.
/// "This spell costs {1}{U} less to cast if it targets an instant or
/// sorcery spell. / Counter target spell."
///
/// Counter half wired via `Effect::CounterSpell`. The "{1}{U} less when
/// it targets an instant/sorcery" rider is a MANDATORY, colored-aware
/// cost reduction (`self_cost_reduction_cost_if_target` → CR 601.2f) —
/// aiming at an IS spell automatically pays {1}{U}; the caster cannot
/// choose to pay full price against an IS target.
pub fn brush_off() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Brush Off",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
        },
        self_cost_reduction_cost_if_target: Some((
            SelectionRequirement::IsSpellOnStack.and(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            ),
            cost(&[generic(1), u()]),
        )),
        ..Default::default()
    }
}

/// Mana Sculpt — {1}{U}{U} Instant.
/// "Counter target spell. If you control a Wizard, add an amount of
/// {C} equal to the amount of mana spent to cast that spell at the
/// beginning of your next main phase."
///
/// Fully wired: `Effect::CounterSpell` stamps the countered spell's
/// actually-PAID mana (`GameState.countered_spell_mana_spent` — real X
/// payments included, unlike a printed-CMC read), and
/// `Effect::AddManaAtNextMainPhase` evaluates that amount now and banks
/// the {C} on a `YourNextMainPhase` delayed trigger, so the mana lands
/// with main-phase windows open exactly as printed.
pub fn mana_sculpt() -> CardDefinition {
    use crate::card::Predicate;
    use crate::mana::u;
    CardDefinition {
        name: "Mana Sculpt",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(crate::card::CreatureType::Wizard)
                        .and(SelectionRequirement::ControlledByYou),
                )),
                then: Box::new(Effect::AddManaAtNextMainPhase {
                    amount: Value::CounteredSpellManaSpent,
                    any_color: false,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Run Behind — {3}{U} Instant. Target creature's owner puts it on their
/// choice of the top or bottom of their library (`LibraryPosition::
/// OwnerChoice`). The "{1} less if it targets an attacking creature" rider
/// is a MANDATORY automatic reduction via `self_cost_reduction_if_target`
/// (CR 601.2f).
pub fn run_behind() -> CardDefinition {
    use crate::effect::{LibraryPosition, PlayerRef, ZoneDest};
    use crate::mana::u;
    CardDefinition {
        name: "Run Behind",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::OwnerChoice,
            },
        },
        // "{1} less if it targets an attacking creature" — a MANDATORY
        // reduction (CR 601.2f) via `self_cost_reduction_if_target`, not
        // an opt-in alternative cost.
        self_cost_reduction_if_target: Some((
            SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
            1,
        )),
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // Slot 0: target opponent loses 1, you gain 1.
            Effect::LoseLife {
                who: target_filtered(SelectionRequirement::OpponentPlayer),
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
        ..Default::default()
    }
}

/// Wander Off — {3}{B} Instant. Exile target creature.
pub fn wander_off() -> CardDefinition {
    CardDefinition {
        name: "Wander Off",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(SelectionRequirement::Creature),
        },
        ..Default::default()
    }
}

/// Masterful Flourish — {B} Instant.
/// "Target creature you control gets +1/+0 and gains indestructible until
/// end of turn."
pub fn masterful_flourish() -> CardDefinition {
    CardDefinition {
        name: "Masterful Flourish",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Impractical Joke — {R} Sorcery. "Damage can't be prevented this
/// turn. Impractical Joke deals 3 damage to up to one target creature
/// or planeswalker."
///
/// The "damage can't be prevented this turn" rider is now wired via
/// `Effect::DamageCantBePreventedThisTurn` (CR 615.12), which suppresses
/// the engine's prevention shields for the turn. "Up to one target" rides
/// the engine's optional-target model (a targeted spell resolves with no
/// target if none is chosen).
pub fn impractical_joke() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Impractical Joke",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // "Damage can't be prevented this turn." (CR 615.12)
            Effect::DamageCantBePreventedThisTurn,
            // "...deals 3 damage to up to one target creature or
            // planeswalker." Targets are optional in this engine, so the
            // "up to one" reading falls out for free.
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Heated Argument — {4}{R} Instant.
/// "Heated Argument deals 6 damage to target creature. You may exile a
/// card from your graveyard. If you do, Heated Argument also deals 2
/// damage to that creature's controller."
///
/// The optional rider: an `Effect::MayDo` gates a `Move` of exactly
/// one graveyard card to exile (via `Selector::take(CardsInZone, 1)`)
/// followed by 2 damage to the target creature's controller. Declining
/// the may skips both the exile and the bonus damage.
///
/// Approximation: only the yes/no of the `MayDo` is player-facing —
/// `Selector::take(..., 1)` auto-picks an arbitrary graveyard card
/// rather than letting the caster choose which card to exile.
pub fn heated_argument() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    use crate::mana::r;
    CardDefinition {
        name: "Heated Argument",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
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
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
                    what: target_filtered(SelectionRequirement::InGraveyard),
                    to: ZoneDest::Exile,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(4),
            duration: Duration::EndOfTurn,
        },
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
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::This,
                count: Value::Const(1),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // "Return UP TO ONE target nonland permanent" — the bounce may
            // be declined to just fetch (audit fix: was mandatory).
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
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
        ..Default::default()
    }
}

// ── Witherbloom (B/G) ───────────────────────────────────────────────────────

/// Witherbloom Charm — {B}{G} Instant. Choose one —
/// • You may sacrifice a permanent. If you do, draw two cards.
/// • You gain 5 life.
/// • Destroy target nonland permanent with mana value 2 or less.
///
/// The "you may" gating on mode 0 is honored via `Effect::MayDo` —
/// picking the mode still lets the controller decline the sacrifice
/// (and forgo the draw). AutoDecider falls through to mode 1 (gain 5)
/// when no permanent is sacrificable, so the card never bricks.
pub fn witherbloom_charm() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Witherbloom Charm",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

// ── Silverquill (W/B) ───────────────────────────────────────────────────────

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

/// Lorehold Charm — {R}{W} Instant. Choose one —
/// • Each opponent sacrifices a nontoken artifact of their choice.
/// • Return target artifact or creature card with mana value 2 or less
///   from your graveyard to the battlefield.
/// • Creatures you control get +1/+1 and gain trample until end of turn.
///
/// All three modes wired:
/// - Mode 0 forces each opponent to sacrifice a nontoken artifact (the
///   `Sacrifice` primitive auto-picks the lowest-CMC token-or-artifact;
///   the `NotToken` requirement keeps Treasures out of the picker).
/// - Mode 1 returns a graveyard card (auto-decider picks the highest-
///   priority eligible card).
/// - Mode 2 fans out the printed +1/+1 + trample EOT across creatures
///   the caster controls.
pub fn lorehold_charm() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::r;
    CardDefinition {
        name: "Lorehold Charm",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Instant],
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
                    .and(SelectionRequirement::ManaValueAtMost(2))
                    .and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            // Mode 2: creatures you control get +1/+1 and gain trample EOT.
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(1),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

/// Stress Dream — {3}{U}{R} Instant.
/// "Stress Dream deals 5 damage to up to one target creature. Look at the
/// top two cards of your library. Put one of those cards into your hand
/// and the other on the bottom of your library."
///
/// Faithful (audit fixes): the 5 damage is "UP TO one target creature"
/// (`ApplyToTargets min 0`), and the card-selection half is a real
/// `LookPickToHand` over the top two — one to hand, the other bottomed.
pub fn stress_dream() -> CardDefinition {
    use crate::mana::{r, u};
    CardDefinition {
        name: "Stress Dream",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(5),
                }),
            },
            Effect::LookPickToHand {
                then_if_picked: None,
                who: PlayerRef::You,
                count: Value::Const(2),
                rest_to_graveyard: false,
                pick_filter: None,
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: true,
                rest_to_exile: false,
            },
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
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
    use crate::mana::{ManaCost, g, generic as gen_pip, u};
    let counter_cost = ManaCost {
        symbols: vec![gen_pip(2)],
    };
    CardDefinition {
        name: "Quandrix Charm",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            // Mode 0: counter target spell unless controller pays {2}.
            Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: counter_cost,
                exile: false,
                extra_generic: None,
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
        ..Default::default()
    }
}

/// Ajani's Response — {4}{W} Instant.
/// "This spell costs {3} less to cast if it targets a tapped creature.
/// Destroy target creature."
///
/// The "{3} less if it targets a tapped creature" rider is a MANDATORY
/// automatic reduction via `self_cost_reduction_if_target` (CR 601.2f):
/// against a tapped target a normal cast simply costs {1}{W}; against
/// an untapped target the printed {4}{W} applies. The destroy-creature
/// body is unchanged.
pub fn ajanis_response() -> CardDefinition {
    CardDefinition {
        name: "Ajani's Response",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Creature),
        },
        // "{3} less if it targets a tapped creature" — a MANDATORY
        // reduction (CR 601.2f) via `self_cost_reduction_if_target`, not
        // an opt-in alternative cost.
        self_cost_reduction_if_target: Some((
            SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
            3,
        )),
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
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
///    `Predicate::InstantsOrSorceriesCastThisTurnAtLeast { who: You, at_least: 2 }`
///    (≥2 because Burrog Barrage itself counts as one cast — we want
///    "another instant or sorcery"; the I/S-restricted predicate keeps a
///    creature spell cast earlier in the turn from enabling the pump).
///    Pumps the chosen target friendly creature +1/+0 EOT.
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
///
/// The damage half is `Effect::DealDamageEqualToPower` (one-sided
/// fight), so the pumped creature itself is the damage source — its
/// deathtouch / lifelink / wither apply, matching the printed "it deals
/// damage equal to its power". (An earlier revision used a plain
/// `DealDamage` attributed to the spell, losing that.)
pub fn burrog_barrage() -> CardDefinition {
    use crate::card::Predicate;
    use crate::mana::g;
    CardDefinition {
        name: "Burrog Barrage",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::InstantsOrSorceriesCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
                then: Box::new(Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            // Slot 1: optional opp creature target takes damage FROM the
            // pumped creature (one-sided fight — source attribution keeps
            // its deathtouch/lifelink live). When either slot is missing
            // the damage half no-ops via the empty-selector behaviour.
            Effect::DealDamageEqualToPower {
                source: Selector::Target(0),
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

// ── Additional Black ────────────────────────────────────────────────────────

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

/// Wilt in the Heat — {2}{R}{W} Lorehold Instant.
/// "This spell costs {2} less to cast if one or more cards left your
/// graveyard this turn. / Wilt in the Heat deals 5 damage to target
/// creature. If that creature would die this turn, exile it instead."
///
/// The "{2} less if cards left your graveyard this turn" cost reduction
/// is a MANDATORY automatic reduction via `StaticEffect::SelfCostReducedIf`
/// gated on `Predicate::CardsLeftGraveyardThisTurnAtLeast(You, 1)` — when
/// the gate passes, a normal cast simply costs {R}{W} (CR 601.2f). The "if that creature
/// would die this turn, exile it instead" rider **is now wired** via the
/// `Effect::ExileIfWouldDieThisTurn` death-replacement primitive: the
/// replacement is installed first, then the 5 damage is dealt, so the
/// resulting SBA death is redirected to exile. Because the replacement
/// lasts the whole turn, a creature that survives the 5 (e.g. toughness
/// > 5) but dies later this turn is exiled too, and an indestructible
/// creature — which never dies — is correctly left alone.
pub fn wilt_in_the_heat() -> CardDefinition {
    use crate::effect::Predicate;
    use crate::mana::{r, w};
    CardDefinition {
        name: "Wilt in the Heat",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Instant],
        // Install the "if it would die this turn, exile instead" death
        // replacement on the target, then deal 5. The SBA that kills it
        // routes through `remove_from_battlefield_to_graveyard`, which
        // honors the replacement and sends it to exile.
        // Slot 0 carries the printed "target creature" filter — a bare
        // `Target(0)` had no filter, so the auto-targeter fell back to
        // "any target" and burned the opponent's face.
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(5),
            },
        ]),
        // "{2} less if one or more cards left your graveyard this turn" —
        // a MANDATORY reduction (CR 601.2f) via `SelfCostReducedIf`, not
        // an opt-in alternative cost.
        static_abilities: vec![crate::effect::StaticAbility {
            description: "This spell costs {2} less to cast if one or more cards \
                          left your graveyard this turn.",
            effect: crate::effect::StaticEffect::SelfCostReducedIf {
                condition: Predicate::CardsLeftGraveyardThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                amount: 2,
            },
        }],
        ..Default::default()
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
/// true`) for `EndOfControllersNextTurn`. Printed text says "may play
/// it" — a normal play paying the card's own cost — so the permission
/// carries `pay_own_cost: true` (not a free cast).
pub fn suspend_aggression() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{r, w};
    CardDefinition {
        name: "Suspend Aggression",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Instant],
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
                pay_own_cost: true,
                any_color: false,
            },
        ]),
        ..Default::default()
    }
}

/// Rabid Attack — {1}{B} Instant.
/// "Until end of turn, any number of target creatures you control each
/// get +1/+0 and gain 'When this creature dies, draw a card.'"
///
/// "Any number of target creatures" is wired via
/// `Effect::ApplyToTargets { max_targets: 16, min_targets: 0 }` — every
/// slot is an optional pick (a `wants_ui` caster declines to stop; the
/// bot max-fills), and the body runs once per supplied target with
/// `Selector::Target(0)` rebound. 16 matches the engine-wide slot cap.
/// The transient die-to-draw rider is wired via
/// `Effect::GrantTriggeredAbility` — each pumped target gains a
/// CreatureDied/SelfSource draw-a-card trigger until end of turn (the
/// grant lives on `granted_triggers_eot`; the SBA dies handler walks
/// printed + granted Dies triggers, so the draw fires even though the
/// creature has left the battlefield by trigger-resolution time).
pub fn rabid_attack() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    let die_draw_trigger = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
    };
    CardDefinition {
        name: "Rabid Attack",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 16,
            min_targets: 0,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            effect: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    trigger: Box::new(die_draw_trigger),
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(2),
        },
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

/// Fractalize — {X}{U} Instant. "Until end of turn, target creature becomes a
/// green and blue Fractal with base power and toughness each equal to X plus 1."
pub fn fractalize() -> CardDefinition {
    use crate::card::CreatureType;
    use crate::mana::{Color, u, x};
    CardDefinition {
        name: "Fractalize",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::BecomeColor {
                what: Selector::Target(0),
                colors: vec![Color::Green, Color::Blue],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            Effect::BecomeCreatureType {
                what: Selector::Target(0),
                creature_types: vec![CreatureType::Fractal],
                duration: Duration::EndOfTurn,
            },
            Effect::SetBasePT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Sum(vec![Value::XFromCost, Value::Const(1)]),
                toughness: Value::Sum(vec![Value::XFromCost, Value::Const(1)]),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Divergent Equation — {X}{X}{U} Instant.
/// "Return up to X target instant and/or sorcery cards from your graveyard
/// to your hand. / Exile Divergent Equation."
///
/// DELIBERATE APPROXIMATION — the "up to X" pick is player-chosen at
/// RESOLUTION via `Effect::MoveChosen` (`Decision::ChooseCards`; auto
/// decider maximizes), not declared as cast-time targets. Printed
/// wording targets at cast, which would need X-scaled target-slot
/// counts in the cast pipeline (see TODO "ManaValueAtMostV /
/// Value-keyed targets" — same gap as Mind into Matter). Behaviorally
/// near-equivalent for graveyard cards (no protection/hexproof in
/// yards); the observable difference is opponents responding without
/// knowing the picks. "Exile Divergent Equation" rides
/// `exile_on_resolve`.
pub fn divergent_equation() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    use crate::mana::{u, x};
    CardDefinition {
        name: "Divergent Equation",
        cost: cost(&[x(), x(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::MoveChosen {
            from: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            },
            filter: None,
            count: Value::XFromCost,
            up_to: true,
            to: ZoneDest::Hand(PlayerRef::You),
        },
        exile_on_resolve: true,
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

/// Prismari Charm — {U}{R} Instant.
/// "Choose one — / • Surveil 2, then draw a card. / • Prismari Charm
/// deals 1 damage to each of one or two targets. / • Return target
/// nonland permanent to its owner's hand."
///
/// Wired as a 3-mode `ChooseMode`: Surveil 2 + draw / 1 dmg to each of
/// one or two any-targets (creature/player/planeswalker) / bounce
/// nonland to owner's hand. Mode 1's "each of one or two targets" is
/// the printed shape via `ApplyToTargets { max_targets: 2,
/// min_targets: 1 }` — slot 0 required, slot 1 an optional pick.
pub fn prismari_charm() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{r, u};
    CardDefinition {
        name: "Prismari Charm",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Instant],
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
            // Mode 1: 1 damage to each of one or two targets — no type
            // restriction, so players are legal targets too.
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
                effect: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(1),
                }),
            },
            // Mode 2: Return target nonland permanent to its owner's hand.
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
/// haste and 'At the beginning of the end step, sacrifice this token'"
/// rider is **now wired** via `Effect::CopySpellWithRiders` — the copy
/// carries `CardInstance.resolve_riders`, and the permanent-spell
/// resolution path grants haste EOT + schedules a next-end-step
/// sacrifice on the resulting token.
/// The "this spell can't be copied" rider is now wired via the
/// `CardDefinition.cant_be_copied` flag, which `Effect::CopySpell` honors
/// by skipping it as a copy target. The "choose one or both" multi-mode
/// rider is a true cast-time selection via `Effect::ChooseModesCast`
/// (min 1, max 2): `CastSpellSpree` carries the chosen modes, each with
/// its own target slot. A plain `CastSpell { mode }` still works as a
/// single-mode cast.
pub fn choreographed_sparks() -> CardDefinition {
    use crate::card::Keyword;
    use crate::mana::r;
    let copy_is_spell = Effect::CopySpellMayChooseTargets {
        what: target_filtered(
            SelectionRequirement::IsSpellOnStack.and(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            ),
        ),
        count: Value::Const(1),
    };
    let copy_creature_spell = Effect::CopySpellWithRiders {
        // Target a creature spell on the stack. The copy enters as a
        // token (CR 608.3f) carrying the printed riders: haste until
        // end of turn + sacrificed at the beginning of the next end
        // step (`CardInstance.resolve_riders`).
        what: target_filtered(
            SelectionRequirement::IsSpellOnStack
                .and(SelectionRequirement::HasCardType(CardType::Creature)),
        ),
        count: Value::Const(1),
        grant_haste: true,
        sacrifice_eot: true,
    };
    CardDefinition {
        name: "Choreographed Sparks",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        // "This spell can't be copied." (CR 707) — an Effect::CopySpell
        // targeting Choreographed Sparks is skipped by the resolver.
        keywords: vec![Keyword::CantBeCopied],
        effect: Effect::ChooseModesCast {
            modes: vec![copy_is_spell, copy_creature_spell],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

// ── Flashback (the SOS instant) ─────────────────────────────────────────────

/// Flashback — {R} Instant.
/// "Target instant or sorcery card in your graveyard gains flashback
/// until end of turn. The flashback cost is equal to its mana cost."
///
/// Now wired faithfully via `Effect::GrantFlashbackThisTurn`: the target
/// instant/sorcery card in your graveyard gains an until-end-of-turn
/// flashback grant whose cost equals its own mana cost (stored in
/// `CardInstance::granted_flashback_eot`). The card is a real target
/// (slot 0, I/S-in-your-graveyard filter) so the caster chooses which
/// card gains flashback. The controller recasts it this turn through the
/// regular `GameAction::CastFlashback` path — paying the printed mana
/// cost and exiling on resolve (CR 702.34d) — rather than the earlier
/// free-cast approximation.
pub fn sos_flashback_instant() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Flashback",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantFlashbackThisTurn {
            what: target_filtered(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery))
                    .and(SelectionRequirement::InYourGraveyard),
            ),
        },
        ..Default::default()
    }
}
