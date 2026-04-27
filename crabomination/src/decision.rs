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
#[derive(Debug, Clone)]
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
        cards: Vec<(CardId, &'static str)>,
    },

    /// Choose `count` cards from the given hand to discard.
    Discard {
        player: usize,
        count: u32,
        hand: Vec<(CardId, &'static str)>,
    },

    /// Choose a card matching the tutor's selector, or decline (failed search).
    SearchLibrary {
        player: usize,
        candidates: Vec<(CardId, &'static str)>,
    },

    /// Answer a "may" trigger or optional cost.
    OptionalTrigger {
        source: CardId,
        description: &'static str,
    },

    /// Choose `count` cards from hand to put on top of the library.
    /// The order of the returned IDs determines library order: index 0 ends up
    /// on top.
    PutOnLibrary {
        player: usize,
        count: usize,
        hand: Vec<(CardId, &'static str)>,
    },

    /// Opening-hand keep-or-mulligan decision. The player sees their current
    /// hand and decides whether to keep or shuffle back and draw again.
    Mulligan {
        player: usize,
        hand: Vec<(CardId, &'static str)>,
        mulligans_taken: usize,
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
    /// Serum Powder–style "exile some cards from hand, draw that many" — the
    /// engine resolves it by removing the listed cards from hand to exile,
    /// then drawing equal-count fresh cards. Triggered as a side-branch of
    /// the Mulligan decision; the player retains the same mulligan count
    /// after performing this swap.
    ExileFromHandAndRedraw(Vec<CardId>),
}

pub trait Decider {
    fn decide(&mut self, decision: &Decision) -> DecisionAnswer;
}

/// Default decider: picks the first/simplest legal answer. Good for tests
/// where the choice doesn't matter and for engine bootstrapping.
#[derive(Debug, Default, Clone, Copy)]
pub struct AutoDecider;

impl Decider for AutoDecider {
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
    fn decide(&mut self, decision: &Decision) -> DecisionAnswer {
        self.asked.push(decision.clone());
        self.answers
            .pop_front()
            .unwrap_or_else(|| AutoDecider.decide(decision))
    }
}
