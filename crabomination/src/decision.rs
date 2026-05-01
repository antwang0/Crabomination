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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    /// Pick a target satisfying the ability's selector.
    ChooseTarget { source: CardId, legal: Vec<Target> },

    /// Pick a mode index from a modal spell (e.g. Command suite).
    ChooseMode { source: CardId, num_modes: usize },

    /// Pick a color (Birds of Paradise, Gilded Lotus, Prismatic Lens).
    ChooseColor { source: CardId, legal: Vec<Color> },

    /// After looking at the top cards of a library, partition them into
    /// (kept on top in this order, sent to the bottom in this order).
    Scry {
        player: usize,
        cards: Vec<(CardId, String)>,
    },

    /// Choose `count` cards from the given hand to discard.
    Discard {
        player: usize,
        count: u32,
        hand: Vec<(CardId, String)>,
    },

    /// Choose a card matching the tutor's selector, or decline (failed search).
    SearchLibrary {
        player: usize,
        candidates: Vec<(CardId, String)>,
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
    },
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
            Decision::PutOnLibrary { hand, count, .. } => DecisionAnswer::PutOnLibrary(
                hand.iter().take(*count).map(|(id, _)| *id).collect(),
            ),
            Decision::Mulligan { .. } => DecisionAnswer::Keep,
            // AutoDecider picks Demon — the demo Goryo's deck includes
            // Griselbrand (Demon), so this is the gameplay-optimal default.
            Decision::ChooseCreatureType { .. } => {
                DecisionAnswer::CreatureType(crate::card::CreatureType::Demon)
            }
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
