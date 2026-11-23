//! Player-choice system for effect resolution.
//!
//! Choices that can't be fixed at cast time (Scry's ordering, "add one mana
//! of any color", a tutor's pick) pause resolution in `Decider::decide`; the
//! decider (human UI, bot, scripted test) returns a `DecisionAnswer`.

use serde::{Deserialize, Serialize};

use crate::card::CardId;
use crate::game::Target;
use crate::mana::Color;

/// What the "second bucket" of a `Decision::Scry` means to the UI. The engine
/// resolves all variants through the same `ScryOrder` answer; this only
/// changes the modal's labels.
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
    /// "Put the rest on the bottom of your library in any order" — the cards
    /// are the library's bottom ones; the player only orders them (listed
    /// first = highest in the library).
    OrderBottom,
}

/// `Decision::ChooseTarget.description` for a cast-time *extra* target slot
/// (slot 1+, Knockout Maneuver's second target). The client folds that answer
/// into the cast it holds for manual mana payment so a `ManualTapRequired`
/// retry doesn't re-pose the slot on every land tap. Shared so the two sides
/// can't drift.
pub const EXTRA_CAST_TARGET_PROMPT: &str = "choose an additional target";

/// Tail of the `Decision::ChooseCards.prompt` for a slot-0 target in an
/// off-board zone (Sundering Archaic's graveyard target), which the in-scene
/// cursor can't click; full prompt `"<source name><this>"`. The client folds
/// the pick into its held action, or the server-side replay re-poses the
/// picker on every land tap. Shared so the two sides can't drift.
pub const OFFBOARD_TARGET_PROMPT_SUFFIX: &str = ": choose a card to target";

/// The same for a cast's further slot that lives off the board (Dance of the
/// Manse's "up to X target … cards from your graveyard"): the client appends
/// the pick to the held cast's extra targets.
pub const OFFBOARD_EXTRA_TARGET_PROMPT_SUFFIX: &str = ": choose another card to target";

/// Which way a [`Decision::ChooseCards`] pick cuts for the seat asked — does
/// picking *more* help the chooser or cost it?
///
/// Policies once guessed this from the prompt ("sacrifice"/"discard"), so a
/// reworded prompt flipped them and "exile"-worded costs read as upside. The
/// ask states it; no struct-literal default, so a new site can't skip it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PickValue {
    /// The picked cards are lost or harmed — sacrificed, discarded, exiled,
    /// tapped, destroyed, bounced, buried. Pick an opponent's when the list
    /// offers them; over our own, give up the least and only as many as
    /// `min` forces. The conservative default.
    #[default]
    Cost,
    /// Picking is the upside — the chooser gains or keeps what it names
    /// (drawn into hand, returned from a graveyard, cheated onto the
    /// battlefield, "choose N to keep"), or the pick *is* the mechanism and
    /// a wider one digs deeper (Scroll Rack). Take the best, up to `max`.
    Gain,
}

/// What a [`Decision::ChooseAmount`]'s number buys — what a headless policy
/// can't read off the prompt. Prompt matching once answered `max` to
/// everything it missed, which for Devour is the whole board. Same fix as
/// [`PickValue`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmountKind {
    /// More is better for the chooser — extra counters, extra cards, a bigger
    /// X. Take `max`. The historical answer for everything the prose match
    /// did not recognise, and still the right one for most asks.
    #[default]
    Upside,
    /// A count of the chooser's OWN permanents given up (Devour, God-Eternal
    /// Bontu): `max` is the whole board, payoff linear in the loss.
    ///
    /// `free` is how many candidates, in the order the effect takes them,
    /// cost next to nothing (tokens). Only the effect knows that order, so
    /// the ask carries it; without it a bot answered 0 and Devour was a no-op.
    Cost { free: u32 },
    /// A life payment — pay only out of a healthy buffer.
    Life,
    /// A life payment whose `max` is the life left after this resolution's
    /// earlier payments, which are applied only once every ask is answered
    /// (Plague of Vermin's rounds): the buffer is kept against `max`, not the
    /// life total, which hasn't moved yet.
    LifeOfRemaining,
    /// A mana payment out of the floating pool (`MayPayX`), which is not the
    /// same as the ask's `max`: that is a bound over untapped sources too.
    Mana,
    /// "Choose a number; destroy all creatures with power N or greater" —
    /// the cutoff that kills the most of theirs and the least of ours.
    DestroyPowerCutoff,
    /// "Choose a number; each player sacrifices that many" (By Invitation
    /// Only) — the count that costs the opponents most and the chooser least.
    EachSacrificesCount,
    /// "Pay any amount; deal that much damage to each other creature"
    /// (Territorial Aetherkite) — `max` sweeps the chooser's own board too,
    /// so the amount that kills the most of theirs and the least of ours.
    SweepDamage,
    /// "As this enters, choose a number" keyed to what opponents cast
    /// (Sanctum Prelate, Talion): not more-is-better, so a headless seat
    /// answers 0 and the engine's own pick (the opponents' most common
    /// nonland mana value) stands.
    NamedNumber,
}

/// What a payment on an [`OptionalKind`] buys.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayFor {
    /// The ask's own payoff — a body, a free cast, a card put onto the
    /// battlefield. Paying is optional upside.
    #[default]
    Payoff,
    /// `source` stays on the battlefield instead of being sacrificed
    /// (echo, cumulative upkeep, "pay or sacrifice").
    KeepSource,
    /// A spell or ability the paying seat controls is not countered.
    SaveSpell,
    /// An effect the paying seat does not control does not happen.
    DenyEffect,
}

/// What a yes/no [`Decision::OptionalTrigger`] asks, in machine terms. A
/// headless policy answers off this, never off `description` (prose matching
/// once blanket-accepted ninety asks such as ante your library or pay any
/// life; same fix as [`PickValue`] and [`AmountKind`]).
///
/// [`MayBody`](OptionalKind::MayBody) is the default and the answer for every
/// catalog-authored `May*` node: the policy finds the node on the source's
/// definition by `description` and screens the body. Every other variant
/// names a trade the body can't show.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum OptionalKind {
    /// A catalog `May*` node. The policy screens the body the node names.
    #[default]
    MayBody,
    /// Yes pays mana. `cost` is `None` when only the engine can compute it at
    /// payment time (cumulative upkeep's N-fold cost, a tax summed over the
    /// board).
    PayMana { cost: Option<crate::mana::ManaCost>, purpose: PayFor },
    /// Yes pays `life` life.
    PayLife { life: u32, purpose: PayFor },
    /// Yes sacrifices permanents the seat controls for the ask's payoff
    /// (Exploit and the reflexive "you may sacrifice ~; if you do" family).
    SacrificeForPayoff,
    /// Yes discards `count` card(s) for the ask's payoff.
    DiscardForPayoff { count: u32 },
    /// Yes gives up another permanent — sacrifices one, returns a land — so
    /// that `source` is not sacrificed.
    KeepByGivingUp,
    /// Yes casts a card without paying its cost. Pure upside unless the card's
    /// own body drains the caster (Decorum Dissertation recurs every main
    /// phase and a blanket yes played it into the state-based loss).
    CastFree,
    /// Yes exiles `count` cards from the seat's OWN graveyard (Forage,
    /// collect evidence, Carrion Rats); accepted with spare graveyard depth,
    /// as the `AutoDecider` arms do. `count: 0` (paid another way, or the
    /// engine picks a minimal set): always accepted.
    ExileGraveyard { count: u32 },
    /// Yes costs the seat cards it cannot price here — ante the top of the
    /// library, exile its hand. Decline.
    SelfCost,
    /// Yes is free for the seat and the engine authored the ask (a card put
    /// onto the battlefield, a processed exile). Take it.
    FreeUpside,
    /// Neither answer costs anything the policy can price: an ordering, a
    /// which-pile or a which-guess choice. Historical yes.
    Neutral,
    /// A tempting offer (CR 701.x): every accepter pays the caster more than
    /// it gains. Decline.
    TemptingOffer,
    /// Orzhov Advokist's offer: yes puts +1/+1 counters on one of the seat's
    /// creatures, then none of its creatures can attack `offerer` until the
    /// offerer's next turn (CR 508.1a). Free while another opponent remains;
    /// with `offerer` the last one, yes forfeits every attack.
    PeaceOffer { offerer: usize },
    /// Ad Nauseam's per-reveal ask — the one stateful policy, answered on the
    /// bot struct across the series.
    RevealTopLoseLife,
    /// "As many times as you choose, exile the top card; if the total is
    /// `limit` or less, …" (Dance with Calamity): `total` is what is already
    /// exiled. Yes risks going over.
    PushYourLuck { total: u32, limit: u32 },
    /// Lim-Dûl's Vault: yes pays 1 life to bottom the five-card window at
    /// `offset` into the library and look at the next five.
    DigForLife { offset: u32 },
    /// Demonstrate (CR 702.144a): yes copies the spell (`source`) and an
    /// opponent of the seat's choice copies it too.
    Demonstrate,
}

impl OptionalKind {
    /// CR 800.4f's test: is "yes" paying a cost, or a plain choice? A departed
    /// seat's cost is simply not paid; its other choices are re-seated under
    /// CR 800.4g (`GameState::route_ask`).
    ///
    /// Spending any resource of the asked seat (mana, life, a permanent, a
    /// card, the graveyard) counts. `CastFree`/`FreeUpside` cost nothing,
    /// `TemptingOffer` is CR 701's offer, the rest are plain choices.
    pub fn is_cost(&self) -> bool {
        match self {
            OptionalKind::PayMana { .. }
            | OptionalKind::PayLife { .. }
            | OptionalKind::SacrificeForPayoff
            | OptionalKind::DiscardForPayoff { .. }
            | OptionalKind::KeepByGivingUp
            | OptionalKind::ExileGraveyard { .. }
            | OptionalKind::SelfCost
            | OptionalKind::RevealTopLoseLife
            | OptionalKind::DigForLife { .. } => true,
            OptionalKind::MayBody
            | OptionalKind::CastFree
            | OptionalKind::FreeUpside
            | OptionalKind::Neutral
            | OptionalKind::TemptingOffer
            | OptionalKind::PeaceOffer { .. }
            | OptionalKind::PushYourLuck { .. }
            | OptionalKind::Demonstrate => false,
        }
    }
}

/// A choice the engine must resolve to continue, with enough context for the
/// decider to render or reason about the options.
///
/// Names and descriptions are owned `String`s, not `&'static str`:
/// `Decision` is serialized inside `GameState`, and a static lifetime would
/// force `'de: 'static` on the parent `Deserialize`. Decisions are rare, so
/// the copy is negligible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    /// Pick a target satisfying the ability's selector. `source_name` and
    /// `description` ("exile target card from a graveyard") are filled at
    /// construction so the UI renders "<name> — <description>" without the
    /// effect tree; an empty `description` is fine.
    ChooseTarget {
        source: CardId,
        legal: Vec<Target>,
        #[serde(default)]
        source_name: String,
        #[serde(default)]
        description: String,
        /// True when the pick may be declined ("up to N targets" — CR 601.2c
        /// lets the chooser take fewer). A decliner answers
        /// `DecisionAnswer::DeclineTarget`; declining ends target selection
        /// for the effect (targets fill left-to-right).
        #[serde(default)]
        optional: bool,
        /// The cast-time extra-slot suspend (`CastExtraTargetPick`) of a
        /// multi-target spell; the client folds the answer into its held cast
        /// or a `ManualTapRequired` bounce re-poses it on every tap. A flag,
        /// not a match on `description`, which broke once the prompt named
        /// its slot.
        #[serde(default)]
        extra_cast_slot: bool,
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
    ///
    /// `description` renders the question for a human; `kind` states it for a
    /// headless policy — see [`OptionalKind`].
    OptionalTrigger {
        source: CardId,
        description: String,
        #[serde(default)]
        kind: OptionalKind,
    },

    /// Choose `count` cards from hand to put on top of the library.
    /// The order of the returned IDs determines library order: index 0 ends up
    /// on top.
    PutOnLibrary {
        player: usize,
        count: usize,
        hand: Vec<(CardId, String)>,
    },

    /// Opening-hand keep or mulligan, or exile the hand with a Serum Powder
    /// (`serum_powders`: those card ids in hand) and draw a fresh seven.
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
        /// CR 205.3m — types the printed effect forbids ("a creature type
        /// other than Wall"). Never offered, and rejected if named anyway.
        #[serde(default)]
        excluded: Vec<crate::card::CreatureType>,
    },

    /// CR 612.1 — replace one creature type with another (Artificial
    /// Evolution), answered `CreatureTypePair(from, to)`. `excluded` applies
    /// to the replacement only (CR 205.3m: not Wall). `AutoDecider` swaps the
    /// target's first printed type for the first legal suggestion.
    ChooseCreatureTypePair {
        source: CardId,
        #[serde(default)]
        suggestions: Vec<crate::card::CreatureType>,
        #[serde(default)]
        excluded: Vec<crate::card::CreatureType>,
    },

    /// CR 201.3 — "As this enters, choose a card name" (Pithing Needle).
    /// Answered `NamedCard(name)` (empty names nothing). `AutoDecider` takes
    /// the first suggestion (names ranked by frequency in the relevant zone).
    NameCard {
        /// Source permanent that's asking for the name.
        source: CardId,
        #[serde(default)]
        source_name: String,
        /// Heuristic candidates, best first (e.g. the most common name in
        /// the targeted hand / library). Empty when nothing is known.
        #[serde(default)]
        suggestions: Vec<String>,
        /// CR 201.4a — a noun naming the restricted namespace ("land"), when
        /// the card only allows names with certain characteristics.
        #[serde(default)]
        restriction: Option<String>,
    },

    /// CR 705 — flip a coin (`Effect::FlipCoin`), answered `Bool` (true =
    /// heads). `AutoDecider` returns the engine's roll; `ScriptedDecider` can
    /// override.
    CoinFlip {
        /// Player flipping (typically `EffectContext.controller`).
        player: usize,
        /// The engine's roll, from the `GameState`'s `GameRng` so a seeded game
        /// reflips identically. A flip isn't a choice (it's a decision only so
        /// tests can script it), so the randomness lives here, not in
        /// `AutoDecider`.
        #[serde(default)]
        heads: bool,
    },

    /// CR 706 — roll an N-sided die. The decider answers with
    /// `DieRoll(n)` where `1 <= n <= sides`. `AutoDecider` takes `rolled`;
    /// `ScriptedDecider` can script any face. Used by `Effect::RollDie`
    /// (Goblin Goliath, Wand of the Elements) and the planar die.
    DieRoll {
        /// Player rolling (typically `EffectContext.controller`).
        player: usize,
        /// Number of sides on the die (e.g. 6 for d6, 20 for d20).
        sides: u8,
        /// CR 706.1 — the face the engine rolled off the game's seeded
        /// stream, like `CoinFlip::heads`. 0 (an old save) reads as the
        /// midpoint.
        #[serde(default)]
        rolled: u8,
    },

    /// CR 903.9a / 903.9b — the commander would go to (hand, library) or has
    /// gone to (graveyard, exile; asked by the SBA) `would_be`; its owner may
    /// move it to the command zone instead. `Bool(true)` redirects;
    /// `AutoDecider` answers true.
    CommanderRedirect {
        commander: CardId,
        would_be: crate::card::Zone,
    },

    /// CR 509.2 / 510.1c — the attacker orders one attacker's blockers for
    /// damage assignment. Answered `DamageOrder(ids)`; omitted ids are
    /// appended in original order, so any answer is legal. `AutoDecider`
    /// keeps the default (declaration / CardId order).
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

    /// CR 701.48 — Learn: reveal a Lesson from `lessons` into hand, discard
    /// from `hand` to draw, or decline. Only posed when `lessons` is
    /// non-empty (otherwise the legacy `Draw 1` path).
    Learn {
        player: usize,
        lessons: Vec<(CardId, String)>,
        hand: Vec<(CardId, String)>,
    },

    /// CR 603.3b — order one player's simultaneous triggers. `triggers` is in
    /// the default order; the answer `TriggerOrder(ids)` is the *push* order
    /// (LIFO: the last listed resolves first). `AutoDecider`'s empty answer
    /// keeps the default.
    OrderTriggers {
        player: usize,
        triggers: Vec<(CardId, String)>,
    },

    /// CR 601.2d — divide `total` damage among `targets` (Forked Bolt).
    /// Answer `DamageDivision(amounts)`, parallel, each ≥1, summing to
    /// `total`. `AutoDecider` spreads evenly, remainder front-loaded.
    DivideDamage {
        source: CardId,
        total: u32,
        targets: Vec<Target>,
        /// What's being divided, for UI labelling — "damage" for divided-damage
        /// spells, "+1/+1 counter" for `Effect::DistributeCounters` (which
        /// reuses this decision). The division math is identical either way.
        noun: String,
    },

    /// CR 510.1c-d — divide an attacker's combat damage among its blockers,
    /// `(id, name, lethal)` in the `CombatDamageOrder` order. Each blocker
    /// must get `lethal` before a later one (or trample) gets any; more is
    /// allowed. Answer `CombatDamageAssignment(amounts)`; empty = the default
    /// lethal-then-trample split.
    AssignCombatDamage {
        attacker: CardId,
        attacker_power: u32,
        blockers: Vec<(CardId, String, u32)>,
    },

    /// CR 704.5j — the legend rule: keep one of `duplicates` sharing `name`,
    /// the rest go to the graveyard. Answer `KeptLegend(id)`; an id outside
    /// `duplicates` (or the `AutoDecider` default) keeps the newest.
    ChooseLegendToKeep {
        player: usize,
        name: String,
        duplicates: Vec<(CardId, String)>,
    },

    /// "Choose a number" — sacrifice any number, pay any amount of life
    /// (Plunge into Darkness). Answer `Amount(n)`, `n` in `0..=max`;
    /// `AutoDecider` returns 0 (never pay unprompted).
    ChooseAmount {
        source: CardId,
        /// A short human-readable prompt ("Sacrifice how many creatures?").
        prompt: String,
        /// Inclusive upper bound (creatures you control, current life, …).
        max: u32,
        /// What the number buys or costs — see [`AmountKind`]. Read by
        /// headless policies in place of matching `prompt`.
        #[serde(default)]
        kind: AmountKind,
    },
    /// CR 701.38 — pick one option from a named ballot. Answered as
    /// `Amount(i)`, the index into `options`, so it shares
    /// `ChooseAmount`'s suspend/replay plumbing; the client renders the
    /// labels as buttons rather than a number stepper.
    ChooseOption {
        source: CardId,
        prompt: String,
        options: Vec<String>,
    },

    /// "Choose any number of cards" from `candidates` (Devious Cover-Up).
    /// Answer `Cards(ids)`; outside ids are dropped and the list truncated to
    /// `max`. `AutoDecider` chooses nothing.
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
        /// When `Some`, only these candidates may actually be chosen —
        /// the rest are shown (so the player sees the whole reveal) but
        /// are not legal picks. Zimone's Experiment reveals five cards
        /// and takes only creatures and lands; without this the client
        /// had no way to know which was which, and an illegal pick was
        /// silently dropped by the resolver. `None` means every candidate
        /// is legal.
        #[serde(default)]
        eligible: Option<Vec<CardId>>,
        /// Whether picking costs the asked seat something or gains it
        /// something — see [`PickValue`]. Read by headless policies in
        /// place of pattern-matching `prompt`.
        #[serde(default)]
        value: PickValue,
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
///
/// `PartialEq` is here because `GameAction::SubmitDecision` carries one and the
/// bot dedupes its candidate actions by value — see PERF `(-290)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecisionAnswer {
    Target(Target),
    /// Abandon the action whose *replay* this decision gathers input for — a
    /// cast or activation suspended before anything was paid (see
    /// `ResumeContext::is_action_replay`), e.g. backing out of an {X} picker
    /// with no mana. The client's Cancel. Rejected for any other decision: a
    /// resolution already on the stack must finish.
    CancelAction,
    /// Decline an optional target pick (`ChooseTarget { optional: true }` —
    /// "up to N targets"). Ends target selection for the effect.
    DeclineTarget,
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
    /// CR 612.1 — `(from, to)` answering `Decision::ChooseCreatureTypePair`.
    CreatureTypePair(crate::card::CreatureType, crate::card::CreatureType),
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
    /// CR 701.48 — the chosen Learn action.
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
    /// `Decision::AssignCombatDamage`. An invalid answer (breaks the
    /// lethal-first order or exceeds power) or empty one falls back to the
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
    /// Re-point a scripted decider at a single answer, keeping its buffers
    /// (no-op by default), so a hot caller holds one boxed decider across a
    /// loop (`auto_tap_for_cost_inner`, PERF `(-159)`).
    fn rearm_script(&mut self, _answer: DecisionAnswer) {}
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
            DeciderKind::Scripted { answers, asked } => {
                Box::new(ScriptedDecider { answers: answers.into_iter().collect(), asked })
            }
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
            // Empty `legal` shouldn't be raised, but the server rope path
            // calls this for stalled seats — decline (consumers fall back on
            // a non-Target answer) instead of panicking.
            Decision::ChooseTarget { legal, .. } => match legal.first() {
                Some(t) => DecisionAnswer::Target(t.clone()),
                None => DecisionAnswer::Bool(false),
            },
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
            // A search names what it's looking for, so taking the first
            // eligible hit is the play pattern; declining (the old default)
            // silently blanked every tutor a headless seat resolved.
            Decision::SearchLibrary { candidates, eligible, .. } => DecisionAnswer::Search(
                match eligible {
                    Some(ok) => ok.first().copied(),
                    None => candidates.first().map(|(id, _)| *id),
                },
            ),
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
            // CR 205.3m — never name a forbidden type; fall back to the top
            // suggestion when the default (Demon) is excluded.
            Decision::ChooseCreatureType { suggestions, excluded, .. } => {
                let demon = crate::card::CreatureType::Demon;
                let pick = if excluded.contains(&demon) {
                    suggestions.iter().find(|t| !excluded.contains(t)).copied().unwrap_or(demon)
                } else {
                    demon
                };
                DecisionAnswer::CreatureType(pick)
            }
            // CR 612.1 — replace the target's first printed type with the
            // first legal suggestion that isn't already it.
            Decision::ChooseCreatureTypePair { suggestions, excluded, .. } => {
                let from = suggestions.first().copied().unwrap_or(crate::card::CreatureType::Human);
                let to = suggestions
                    .iter()
                    .find(|t| **t != from && !excluded.contains(t))
                    .copied()
                    .unwrap_or(crate::card::CreatureType::Human);
                DecisionAnswer::CreatureTypePair(from, to)
            }
            // CR 201.3 — AutoDecider takes the engine's best suggestion
            // (most-common name in the relevant zone); names nothing when
            // there's no context. ScriptedDecider supplies a name.
            Decision::NameCard { suggestions, .. } => {
                DecisionAnswer::NamedCard(suggestions.first().cloned().unwrap_or_default())
            }
            // CR 705 — take the roll the engine made off the game's seeded
            // stream. Drawing from the thread RNG here once made every fixed
            // seed (Mana Crypt's upkeep flip) unreproducible across processes.
            Decision::CoinFlip { heads, .. } => DecisionAnswer::Bool(*heads),
            // CR 706.1 — the engine rolled off the game's seeded stream, as
            // for a coin. Answering the midpoint here made every d20 a 10 and
            // every planar die blank in self-play.
            Decision::DieRoll { sides, rolled, .. } => {
                let midpoint = (*sides as u32).max(1).div_ceil(2) as u8;
                DecisionAnswer::DieRoll(if (1..=*sides).contains(rolled) { *rolled } else { midpoint })
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
            // A ballot has no "safe" abstention — take the first option.
            Decision::ChooseOption { .. } => DecisionAnswer::Amount(0),
            // Forced "choose exactly N" (min > 0) takes the first N so a
            // non-interactive decider still pays; "up to" (min == 0, Devious
            // Cover-Up) chooses nothing.
            Decision::ChooseCards { candidates, min, .. } => DecisionAnswer::Cards(
                candidates.iter().take(*min as usize).map(|(id, _)| *id).collect(),
            ),
        }
    }
}

/// Pre-scripted decider for tests: pops answers FIFO, falls back to
/// `AutoDecider` when exhausted, records every decision it saw.
///
/// `answers` is inline (a script is one or two answers). Test-only since
/// PERF `(-164)` — the engine's scripted path recycles an
/// [`OneColorDecider`].
#[derive(Debug)]
pub struct ScriptedDecider {
    answers: smallvec::SmallVec<[DecisionAnswer; 2]>,
    pub asked: Vec<Decision>,
}

impl Default for ScriptedDecider {
    fn default() -> Self {
        Self::new([])
    }
}

impl ScriptedDecider {
    pub fn new(answers: impl IntoIterator<Item = DecisionAnswer>) -> Self {
        Self { answers: answers.into_iter().collect(), asked: Vec::new() }
    }
}

/// Answers one `Color` decision, then defers to [`AutoDecider`] — all
/// `auto_tap_for_cost_inner` ever used [`ScriptedDecider`] for, and the
/// decider its `scripted_slot` recycles. An `Option<Color>`, so re-arming is
/// a two-byte store and the type is `Copy`. PERF `(-159)`, `(-164)`.
///
/// `kind()` reports `Scripted` like `ScriptedDecider`; engine sites branch on
/// `matches!(decider.kind(), Auto)` without reading the payload, so the two
/// are interchangeable there (`one_color_decider_matches_the_scripted_one`).
///
/// ⚠ Unlike a one-answer script, every colour ask gets the same colour: a
/// filter land asks once per pip, and a fall-through second pip meant Graven
/// Cairns couldn't make {R}{R}.
#[derive(Debug, Default, Clone, Copy)]
pub struct OneColorDecider {
    answer: Option<Color>,
    spent: bool,
}

impl OneColorDecider {
    pub fn new(color: Color) -> Self {
        Self { answer: Some(color), spent: false }
    }
}

impl Decider for OneColorDecider {
    fn kind(&self) -> DeciderKind {
        DeciderKind::Scripted {
            answers: self.answer.filter(|_| !self.spent).map(DecisionAnswer::Color).into_iter().collect(),
            asked: Vec::new(),
        }
    }
    fn decide(&mut self, decision: &Decision) -> DecisionAnswer {
        match self.answer {
            Some(c) if !self.spent || matches!(decision, Decision::ChooseColor { .. }) => {
                self.spent = true;
                DecisionAnswer::Color(c)
            }
            _ => AutoDecider.decide(decision),
        }
    }
    /// Only a `Color` can be scripted here — the two auto-tap sites pass
    /// nothing else, and a non-colour answer would be a caller bug rather than
    /// a state to carry. Debug-asserted; in release it leaves the slot empty,
    /// which falls through to `AutoDecider` exactly as an exhausted script does.
    fn rearm_script(&mut self, answer: DecisionAnswer) {
        self.spent = false;
        self.answer = match answer {
            DecisionAnswer::Color(c) => Some(c),
            other => {
                debug_assert!(false, "OneColorDecider re-armed with {other:?}");
                None
            }
        };
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
        if self.answers.is_empty() {
            AutoDecider.decide(decision)
        } else {
            self.answers.remove(0)
        }
    }
    fn rearm_script(&mut self, answer: DecisionAnswer) {
        self.answers.clear();
        self.answers.push(answer);
        self.asked.clear();
    }
}

#[cfg(test)]
mod one_color_decider_tests {
    use super::{Decider, DeciderKind, Decision, DecisionAnswer, OneColorDecider, ScriptedDecider};
    use crate::card::CardId;
    use crate::mana::Color;

    /// `OneColorDecider` replaced `ScriptedDecider::new([Color(c)])` for
    /// allocations only, so both run the same sequence and must agree. The
    /// `kind()` leg is load-bearing: a `OneColorDecider` reporting `Auto` would
    /// silently take the other arm at every `DeciderKind::Auto` branch. PERF
    /// `(-164)`.
    #[test]
    fn one_color_decider_matches_the_scripted_one() {
        let ask = Decision::ChooseColor {
            source: CardId(1),
            legal: vec![Color::White, Color::Green],
        };
        let later = Decision::OptionalTrigger {
            source: CardId(1),
            description: String::new(),
            kind: super::OptionalKind::default(),
        };
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            let mut scripted = ScriptedDecider::new([DecisionAnswer::Color(c)]);
            let mut one = OneColorDecider::new(c);
            // A non-`Auto` kind is what the thirty-eight gates read.
            assert!(matches!(scripted.kind(), DeciderKind::Scripted { .. }));
            assert!(matches!(one.kind(), DeciderKind::Scripted { .. }));
            // `DecisionAnswer` carries no `PartialEq` (a derive on it is a
            // monomorphization the engine never asks for), so compare renderings.
            let same = |x: DecisionAnswer, y: DecisionAnswer| {
                assert_eq!(format!("{x:?}"), format!("{y:?}"));
                x
            };
            // First ask: the scripted answer, whatever the decision is.
            let first = same(scripted.decide(&ask), one.decide(&ask));
            assert!(matches!(first, DecisionAnswer::Color(got) if got == c));
            // Spent, so `kind()` reports an empty queue.
            assert!(
                matches!(one.kind(), DeciderKind::Scripted { ref answers, .. } if answers.is_empty())
            );
            // Every ask after the first falls through to `AutoDecider`.
            same(scripted.decide(&later), one.decide(&later));
            // A second colour ask repeats the colour (a filter land's second
            // pip); the one-answer script would fall through.
            assert!(matches!(one.decide(&ask), DecisionAnswer::Color(got) if got == c));
            // And the recycled path: `auto_tap_for_cost_inner` holds one box
            // for the whole payment and re-arms it per pip, so a spent decider
            // must answer the next pip's colour.
            scripted.rearm_script(DecisionAnswer::Color(c));
            one.rearm_script(DecisionAnswer::Color(c));
            let again = same(scripted.decide(&ask), one.decide(&ask));
            assert!(matches!(again, DecisionAnswer::Color(got) if got == c));
        }
    }
}
