//! Player-choice system for effect resolution.
//!
//! Many MTG cards require choices that can't be predetermined at cast time:
//! Scry needs to see the top N cards before ordering, `{T}: Add one mana of any
//! color` needs a color choice at resolution, tutors show legal candidates, etc.
//!
//! The engine pauses resolution by calling `Decider::decide`; the decider
//! (human UI, bot, scripted test) returns a `DecisionAnswer`.

use serde::{Deserialize, Serialize};

use crate::card::CardId;
use crate::game::Target;
use crate::mana::Color;

/// A choice the engine must resolve to continue. Carries enough context for
/// the decider to render / reason about the options.
///
/// Card-name and description fields are owned `String`s rather than the
/// `&'static str`s used elsewhere — `Decision` is part of the
/// authoritative `GameState`, which we serialize via serde, and a static
/// lifetime would force the parent `Deserialize<'de>` impl to require
/// `'de: 'static`. The catalog's name slices are copied in at decision-
/// construction time; the runtime overhead is negligible (decisions
/// surface a few times per turn at most).
/// What the "second bucket" of a `Decision::Scry` means to the UI. The engine
/// resolves all three through the same `ScryOrder` answer; this only changes
/// the modal's labels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScryMode {
    /// Scry — unkept cards go to the bottom of the library.
    #[default]
    Scry,
    /// Surveil — unkept cards go to the graveyard.
    Surveil,
    /// Rearrange (Index, Spire Owl) — every card stays on top; no second
    /// bucket, the player only reorders.
    Rearrange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    /// Pick a target satisfying the ability's selector.
    ///
    /// `source_name` is the printed name of the source card; `description`
    /// is a short human-readable rendering of the effect ("exile target
    /// card from a graveyard"). Both are populated at decision-construction
    /// time so the UI can render a "<name> — <description>" prompt without
    /// re-deriving them from the effect tree. Empty `description` is fine
    /// for effects whose `effect_short_text` doesn't recognise the shape.
    ChooseTarget {
        source: CardId,
        legal: Vec<Target>,
        #[serde(default)]
        source_name: String,
        #[serde(default)]
        description: String,
    },

    /// Pick a mode index from a modal spell or trigger (e.g. Command suite,
    /// Riot/Fabricate ETB choices). `mode_texts` carries a short rendering of
    /// each mode (`Effect::effect_short_text`) so a UI can label its buttons;
    /// empty for legacy senders.
    ChooseMode {
        source: CardId,
        num_modes: usize,
        #[serde(default)]
        mode_texts: Vec<String>,
    },

    /// Pick a color (Birds of Paradise, Gilded Lotus, Prismatic Lens).
    ChooseColor { source: CardId, legal: Vec<Color> },

    /// After looking at the top cards of a library, partition them into
    /// (kept on top in this order, sent to the bottom in this order). `mode`
    /// tells the UI what the second bucket means — bottom (Scry), graveyard
    /// (Surveil), or nothing at all (Rearrange, all cards stay on top).
    Scry {
        player: usize,
        cards: Vec<(CardId, String)>,
        #[serde(default)]
        mode: ScryMode,
    },

    /// Choose `count` cards from the given hand to discard.
    Discard {
        player: usize,
        count: u32,
        hand: Vec<(CardId, String)>,
    },

    /// Choose a card matching the tutor's selector, or decline (failed search).
    /// `eligible: Some(..)` marks the picks the engine will accept when the
    /// candidate list intentionally shows more (e.g. an Impulse reveal where
    /// only lands may be taken) — UIs grey the rest out.
    SearchLibrary {
        player: usize,
        candidates: Vec<(CardId, String)>,
        #[serde(default)]
        eligible: Option<Vec<CardId>>,
    },

    /// Answer a "may" trigger or optional cost.
    OptionalTrigger {
        source: CardId,
        description: String,
    },

    /// Choose `count` cards from hand to put on top of the library.
    /// The order of the returned IDs determines library order: index 0 ends up
    /// on top.
    PutOnLibrary {
        player: usize,
        count: usize,
        hand: Vec<(CardId, String)>,
    },

    /// Opening-hand keep-or-mulligan decision. The player sees their current
    /// hand and decides whether to keep, shuffle back and draw again, or use
    /// a Serum Powder to exile the hand and draw a fresh seven (the
    /// `serum_powders` field lists the mulligan-helper card IDs in hand —
    /// empty when none are present).
    Mulligan {
        player: usize,
        hand: Vec<(CardId, String)>,
        mulligans_taken: usize,
        serum_powders: Vec<CardId>,
    },

    /// "As this enters, choose a creature type." Cavern of Souls.
    ChooseCreatureType {
        /// Source permanent that's asking for the type.
        source: CardId,
        /// Heuristic candidates, most relevant first (types present across
        /// the game's zones, then common tribal staples). A UI renders these
        /// as pick buttons; an answer outside the list is still legal.
        #[serde(default)]
        suggestions: Vec<crate::card::CreatureType>,
    },

    /// CR 201.3 — "As this enters, choose a card name." Pithing Needle,
    /// Phyrexian Revoker. Answered with `DecisionAnswer::NamedCard(name)`
    /// (free text over the catalog; an empty string names nothing).
    /// `AutoDecider` picks the first suggestion (the engine ranks the
    /// relevant zone's names by frequency); `ScriptedDecider` overrides.
    NameCard {
        /// Source permanent that's asking for the name.
        source: CardId,
        #[serde(default)]
        source_name: String,
        /// Heuristic candidates, best first (e.g. the most common name in
        /// the targeted hand / library). Empty when nothing is known.
        #[serde(default)]
        suggestions: Vec<String>,
    },

    /// CR 705 — flip a coin. The decider answers with `Bool(true)` for
    /// heads or `Bool(false)` for tails. `AutoDecider` uses the engine's
    /// rng to pick randomly; `ScriptedDecider` can script the outcome.
    /// Used by `Effect::FlipCoin` (Karplusan Minotaur, Mana Clash, Ral
    /// Zarek's -7 ultimate).
    CoinFlip {
        /// Player flipping (typically `EffectContext.controller`).
        player: usize,
    },

    /// CR 706 — roll an N-sided die. The decider answers with
    /// `DieRoll(n)` where `1 <= n <= sides`. `AutoDecider` returns the
    /// die's middle value (deterministic, lets tests assert specific
    /// branches); `ScriptedDecider` can script any face.
    /// Used by `Effect::RollDie` (Goblin Goliath, Wand of the Elements,
    /// future Krark / Aether Sphere Harvester-style cards).
    DieRoll {
        /// Player rolling (typically `EffectContext.controller`).
        player: usize,
        /// Number of sides on the die (e.g. 6 for d6, 20 for d20).
        sides: u8,
    },

    /// CR 903.9b — the commander would land in `would_be` from
    /// somewhere; its owner *may* redirect to the command zone
    /// instead. Answered with `DecisionAnswer::Bool` (true = redirect,
    /// false = let it go to `would_be`). `AutoDecider` defaults to
    /// `true` since the safest play is almost always to save the
    /// commander; scripted scenarios (countering an opponent's
    /// graveyard recursion, "draw a card off Yargle's death", etc.)
    /// can answer `false`.
    CommanderRedirect {
        commander: CardId,
        would_be: crate::card::Zone,
    },

    /// CR 509.2 / 510.1c — the attacking player orders the creatures
    /// blocking one attacker; combat damage is then assigned in that order
    /// (each blocker must be dealt lethal before the next gets any). The
    /// decider answers `DamageOrder(ordered_blocker_ids)`; `AutoDecider`
    /// keeps the engine's default (declaration / CardId order). Any ids the
    /// answer omits are appended in their original order, so a partial or
    /// empty answer is always legal.
    CombatDamageOrder {
        attacker: CardId,
        blockers: Vec<(CardId, String)>,
    },

    /// CR 700.2d — choose `count` distinct modes (indices into `0..num_modes`)
    /// for a "choose N" modal spell/ability (Charms, the Strixhaven Command
    /// cycle). The decider answers `Modes(indices)`; `AutoDecider` returns the
    /// card's sensible `default` picks unchanged.
    ChooseModes {
        source: CardId,
        num_modes: usize,
        count: usize,
        default: Vec<u8>,
        /// Short rendering of each mode (`Effect::effect_short_text`) so a
        /// UI can label its toggles; empty for legacy senders.
        #[serde(default)]
        mode_texts: Vec<String>,
    },

    /// CR 701.45 — Learn. The player may reveal a Lesson from their
    /// sideboard (`lessons`) and put it into hand, discard a card from
    /// `hand` to draw a card, or decline. Only surfaced when `lessons` is
    /// non-empty (with no Lessons, the engine takes the legacy `Draw 1`
    /// path without a decision).
    Learn {
        player: usize,
        lessons: Vec<(CardId, String)>,
        hand: Vec<(CardId, String)>,
    },

    /// CR 603.3b — when several of one player's abilities trigger off the
    /// same event batch, that player puts them on the stack in any order
    /// they choose. `triggers` lists the simultaneous same-controller
    /// triggers (source id + label) in the engine's default order. The
    /// decider answers `TriggerOrder(ids)` giving the desired *stack-push*
    /// order (since the stack is LIFO, the last id listed resolves first).
    /// `AutoDecider` returns an empty answer = keep the default order.
    OrderTriggers {
        player: usize,
        triggers: Vec<(CardId, String)>,
    },

    /// CR 601.2d — divide `total` damage among the chosen `targets` (Forked
    /// Bolt, Pyrokinesis, Crackle with Power). The decider answers
    /// `DamageDivision(amounts)` parallel to `targets`, each ≥1 and summing
    /// to `total`. `AutoDecider` spreads as evenly as possible (front-loaded
    /// remainder).
    DivideDamage {
        source: CardId,
        total: u32,
        targets: Vec<Target>,
    },

    /// CR 510.1c-d — divide an attacker's combat damage among its multiple
    /// blockers. `blockers` lists `(id, name, lethal)` in the assignment
    /// order chosen earlier (see `CombatDamageOrder`); the attacker may
    /// assign more than `lethal` to any blocker (e.g. to deny trample) but
    /// must assign at least `lethal` to a blocker before any later blocker
    /// (or trample-over) receives damage. The decider answers
    /// `CombatDamageAssignment(amounts)`; `AutoDecider` returns an empty
    /// answer = the engine's default lethal-to-each-then-trample split.
    AssignCombatDamage {
        attacker: CardId,
        attacker_power: u32,
        blockers: Vec<(CardId, String, u32)>,
    },

    /// CR 704.5j — when a player controls two or more legendary permanents
    /// with the same name, that player chooses one to keep; the rest are put
    /// into their owners' graveyards. `duplicates` lists the tied permanents
    /// (id + name) sharing `name`. The decider answers `KeptLegend(id)` with
    /// the permanent to keep; an id not in `duplicates` (or `AutoDecider`'s
    /// default) keeps the newest (highest id).
    ChooseLegendToKeep {
        player: usize,
        name: String,
        duplicates: Vec<(CardId, String)>,
    },

    /// "Choose a number" — sacrifice *any number* of permanents, pay *any
    /// amount* of life, etc. The decider answers `Amount(n)` with `n` in
    /// `0..=max`. `AutoDecider` returns 0 (the conservative default — never
    /// pay life / sacrifice unprompted). Used by Plunge into Darkness
    /// (sacrifice any number of creatures; pay any amount of life).
    ChooseAmount {
        source: CardId,
        /// A short human-readable prompt ("Sacrifice how many creatures?").
        prompt: String,
        /// Inclusive upper bound (creatures you control, current life, …).
        max: u32,
    },

    /// "Choose any number of cards" — pick a subset of `candidates` (e.g.
    /// "exile any number of target cards from graveyards", Devious Cover-Up).
    /// The decider answers `Cards(ids)` with any subset (≤ `max`) of the
    /// offered ids; ids outside `candidates` are dropped and the list is
    /// truncated to `max`. `AutoDecider` returns an empty set (the
    /// conservative "up to" default — choose nothing unprompted).
    ChooseCards {
        source: CardId,
        /// A short human-readable prompt ("Exile which cards?").
        prompt: String,
        /// The selectable cards (id + display name), already filtered.
        candidates: Vec<(CardId, String)>,
        /// Inclusive lower bound on how many must be chosen. `0` for the
        /// "choose any number / up to" case (Devious Cover-Up); equal to `max`
        /// for a forced "choose exactly N" cost (sacrifice / exile-as-cost).
        #[serde(default)]
        min: u32,
        /// Inclusive upper bound on how many may be chosen.
        max: u32,
    },
}

/// The decider's answer to a `Decision::Learn`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearnChoice {
    /// Reveal this Lesson from the sideboard and put it into hand.
    FetchLesson(CardId),
    /// Discard this card from hand, then draw a card.
    Rummage { discard: CardId },
    /// Do neither.
    Decline,
}

/// The decider's answer to a `Decision`. Variants must match the decision kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionAnswer {
    Target(Target),
    Mode(usize),
    Color(Color),
    /// `kept_top` goes on top in listed order; `bottom` goes to the bottom in
    /// listed order. Cards appearing in neither list default to top.
    ScryOrder {
        kept_top: Vec<CardId>,
        bottom: Vec<CardId>,
    },
    Discard(Vec<CardId>),
    Search(Option<CardId>),
    Bool(bool),
    /// Ordered card IDs to put on top of library; index 0 = topmost.
    PutOnLibrary(Vec<CardId>),
    /// Keep the current opening hand.
    Keep,
    /// Shuffle the current hand back and draw a new one.
    TakeMulligan,
    /// Serum Powder: exile the current hand (including the named Serum
    /// Powder) and draw a fresh seven cards. Doesn't count as a mulligan
    /// (no London-mulligan card-bottoming on subsequent keep).
    SerumPowder(CardId),
    /// A named creature type (Cavern of Souls).
    CreatureType(crate::card::CreatureType),
    /// CR 201.3 — a named card (Pithing Needle / Phyrexian Revoker). An
    /// empty string names nothing.
    NamedCard(String),
    /// CR 706 — the natural result of a die roll. Must be in
    /// `1..=sides` where `sides` is the die's face count from the
    /// matching `Decision::DieRoll { sides, .. }`.
    DieRoll(u8),
    /// CR 510.1c — combat-damage assignment order: blocker ids in the order
    /// the attacker assigns damage. Ids omitted from a partial answer keep
    /// their original relative order at the end.
    DamageOrder(Vec<CardId>),
    /// CR 700.2d — the chosen distinct mode indices for a "choose N" spell.
    Modes(Vec<u8>),
    /// CR 701.45 — the chosen Learn action.
    Learn(LearnChoice),
    /// CR 603.3b — same-controller trigger stack-push order. Ids omitted
    /// from a partial answer keep their original relative order at the end;
    /// an empty answer keeps the engine default.
    TriggerOrder(Vec<CardId>),
    /// CR 601.2d — per-target damage amounts, parallel to the matching
    /// `Decision::DivideDamage { targets }`. A malformed answer (wrong
    /// length / wrong sum) is renormalized to an even split by the engine.
    DamageDivision(Vec<u32>),
    /// CR 510.1c-d — `(blocker_id, amount)` pairs answering
    /// `Decision::AssignCombatDamage`. An empty or invalid answer (one that
    /// violates the assign-lethal-before-next-blocker ordering rule, or
    /// over-assigns past the attacker's power) falls back to the engine's
    /// default lethal-to-each split.
    CombatDamageAssignment(Vec<(CardId, u32)>),
    /// CR 704.5j — the legendary permanent the controller keeps. An id not
    /// among the tied duplicates falls back to keeping the newest.
    KeptLegend(CardId),
    /// A chosen number answering `Decision::ChooseAmount`. Clamped to the
    /// decision's `max` by the engine.
    Amount(u32),
    /// A chosen subset of cards answering `Decision::ChooseCards`. Ids outside
    /// the offered candidates are dropped; the list is truncated to `max`.
    Cards(Vec<CardId>),
}

/// Spread `total` damage across `n` targets as evenly as possible, with the
/// remainder front-loaded (target 0 gets the extra point first). Used by
/// `AutoDecider` and as the engine's fallback when a decider returns a
/// malformed `DamageDivision`.
pub fn even_damage_split(total: u32, n: usize) -> Vec<u32> {
    if n == 0 {
        return vec![];
    }
    let base = total / n as u32;
    let rem = (total % n as u32) as usize;
    (0..n)
        .map(|i| base + if i < rem { 1 } else { 0 })
        .collect()
}

pub trait Decider {
    fn decide(&mut self, decision: &Decision) -> DecisionAnswer;
    /// Project this decider to its serializable form. Default impl
    /// reports `DeciderKind::Auto`; the two stock impls (`AutoDecider`,
    /// `ScriptedDecider`) override to preserve their full state.
    fn kind(&self) -> DeciderKind {
        DeciderKind::Auto
    }
}

/// Tagged, serializable representation of every decider implementation
/// the engine ships with. `GameState.decider` is a `Box<dyn Decider>`,
/// which can't directly serialize; on snapshot it's projected to one of
/// these variants and reconstructed on load.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum DeciderKind {
    /// `AutoDecider` — picks the first legal answer. The default.
    #[default]
    Auto,
    /// `ScriptedDecider` with a queue of pre-canned answers and a log of
    /// every decision it has been asked. Round-tripped exactly.
    Scripted {
        answers: Vec<DecisionAnswer>,
        asked: Vec<Decision>,
    },
}

impl DeciderKind {
    /// Materialize back into a boxed decider trait object.
    pub fn into_boxed(self) -> Box<dyn Decider + Send + Sync> {
        match self {
            DeciderKind::Auto => Box::new(AutoDecider),
            DeciderKind::Scripted { answers, asked } => Box::new(ScriptedDecider {
                answers: answers.into_iter().collect(),
                asked,
            }),
        }
    }
}

/// Default decider: picks the first/simplest legal answer. Good for tests
/// where the choice doesn't matter and for engine bootstrapping.
#[derive(Debug, Default, Clone, Copy)]
pub struct AutoDecider;

impl Decider for AutoDecider {
    fn kind(&self) -> DeciderKind {
        DeciderKind::Auto
    }
    fn decide(&mut self, decision: &Decision) -> DecisionAnswer {
        match decision {
            Decision::ChooseTarget { legal, .. } => {
                DecisionAnswer::Target(legal.first().cloned().expect("no legal target"))
            }
            Decision::ChooseMode { .. } => DecisionAnswer::Mode(0),
            Decision::ChooseColor { legal, .. } => {
                DecisionAnswer::Color(*legal.first().unwrap_or(&Color::Green))
            }
            Decision::Scry { cards, .. } => DecisionAnswer::ScryOrder {
                kept_top: cards.iter().map(|(id, _)| *id).collect(),
                bottom: vec![],
            },
            Decision::Discard { hand, count, .. } => DecisionAnswer::Discard(
                hand.iter().take(*count as usize).map(|(id, _)| *id).collect(),
            ),
            Decision::SearchLibrary { .. } => DecisionAnswer::Search(None),
            Decision::OptionalTrigger { .. } => DecisionAnswer::Bool(false),
            // Default to redirecting — saving the commander matches
            // the typical printed-play pattern. Tests that need the
            // opposite (let it land in the graveyard / exile) script
            // a `Bool(false)` via `ScriptedDecider`.
            Decision::CommanderRedirect { .. } => DecisionAnswer::Bool(true),
            Decision::PutOnLibrary { hand, count, .. } => DecisionAnswer::PutOnLibrary(
                hand.iter().take(*count).map(|(id, _)| *id).collect(),
            ),
            Decision::Mulligan { .. } => DecisionAnswer::Keep,
            // AutoDecider picks Demon — the demo Goryo's deck includes
            // Griselbrand (Demon), so this is the gameplay-optimal default.
            Decision::ChooseCreatureType { .. } => {
                DecisionAnswer::CreatureType(crate::card::CreatureType::Demon)
            }
            // CR 201.3 — AutoDecider takes the engine's best suggestion
            // (most-common name in the relevant zone); names nothing when
            // there's no context. ScriptedDecider supplies a name.
            Decision::NameCard { suggestions, .. } => {
                DecisionAnswer::NamedCard(suggestions.first().cloned().unwrap_or_default())
            }
            // CR 705 — AutoDecider always picks heads. For deterministic
            // tests; a real client would use an rng. ScriptedDecider can
            // override with `DecisionAnswer::Bool(false)` for tails.
            Decision::CoinFlip { .. } => DecisionAnswer::Bool(true),
            // CR 706 — AutoDecider returns the die's midpoint (rounded
            // up) so the result is deterministic AND lands on a typical
            // "middle" result-table band. For a d6 that's 3; for a d20
            // that's 10. ScriptedDecider can script any specific face
            // for testing branch coverage of result tables.
            Decision::DieRoll { sides, .. } => {
                let midpoint = (*sides as u32).max(1).div_ceil(2);
                DecisionAnswer::DieRoll(midpoint as u8)
            }
            // CR 510.1c — keep the engine's default order (empty answer is
            // treated as "all blockers in their original order").
            Decision::CombatDamageOrder { .. } => DecisionAnswer::DamageOrder(vec![]),
            // CR 700.2d — keep the card's sensible default mode picks.
            Decision::ChooseModes { default, .. } => DecisionAnswer::Modes(default.clone()),
            // CR 701.45 — prefer fetching a Lesson (card advantage) over
            // rummaging; the decision is only surfaced when a Lesson exists.
            Decision::Learn { lessons, .. } => DecisionAnswer::Learn(
                lessons
                    .first()
                    .map(|(id, _)| LearnChoice::FetchLesson(*id))
                    .unwrap_or(LearnChoice::Decline),
            ),
            // CR 603.3b — keep the engine's default order (empty answer).
            Decision::OrderTriggers { .. } => DecisionAnswer::TriggerOrder(vec![]),
            // CR 601.2d — spread the damage as evenly as possible.
            Decision::DivideDamage { total, targets, .. } => {
                DecisionAnswer::DamageDivision(even_damage_split(*total, targets.len()))
            }
            // CR 510.1c-d — keep the engine's default lethal-to-each split.
            Decision::AssignCombatDamage { .. } => {
                DecisionAnswer::CombatDamageAssignment(vec![])
            }
            // CR 704.5j — keep the newest (highest id); the engine treats an
            // out-of-set id this way, so any sentinel works.
            Decision::ChooseLegendToKeep { duplicates, .. } => DecisionAnswer::KeptLegend(
                duplicates.iter().map(|(id, _)| *id).max().unwrap_or(CardId(0)),
            ),
            // Conservative default — never pay life / sacrifice unprompted.
            // ScriptedDecider supplies a positive amount.
            Decision::ChooseAmount { .. } => DecisionAnswer::Amount(0),
            // Conservative "up to" default — choose nothing unprompted.
            // ScriptedDecider / the bot supply a positive subset.
            // Forced "choose exactly N" (min > 0) auto-picks the first N
            // candidates so a non-interactive decider still pays the cost;
            // the "up to" case (min == 0, Devious Cover-Up) chooses nothing.
            Decision::ChooseCards { candidates, min, .. } => DecisionAnswer::Cards(
                candidates.iter().take(*min as usize).map(|(id, _)| *id).collect(),
            ),
        }
    }
}

/// Pre-scripted decider for tests. Pops answers in FIFO order; falls back to
/// `AutoDecider` when exhausted. Records every decision it saw for assertions.
#[derive(Debug, Default)]
pub struct ScriptedDecider {
    answers: std::collections::VecDeque<DecisionAnswer>,
    pub asked: Vec<Decision>,
}

impl ScriptedDecider {
    pub fn new(answers: impl IntoIterator<Item = DecisionAnswer>) -> Self {
        Self {
            answers: answers.into_iter().collect(),
            asked: Vec::new(),
        }
    }
}

impl Decider for ScriptedDecider {
    fn kind(&self) -> DeciderKind {
        DeciderKind::Scripted {
            answers: self.answers.iter().cloned().collect(),
            asked: self.asked.clone(),
        }
    }
    fn decide(&mut self, decision: &Decision) -> DecisionAnswer {
        self.asked.push(decision.clone());
        self.answers
            .pop_front()
            .unwrap_or_else(|| AutoDecider.decide(decision))
    }
}
