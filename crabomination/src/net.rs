//! Wire protocol for client↔server communication.
//!
//! The server holds the authoritative [`crate::game::GameState`]; clients
//! interact via [`ClientMsg`] and receive [`ServerMsg`]. Each client sees a
//! per-seat [`ClientView`] projection that hides opponent hand contents,
//! library order, and other hidden information.
//!
//! [`DecisionWire`] and [`GameEventWire`] mirror the engine's `Decision` and
//! `GameEvent` types with owned strings in place of the engine's
//! `&'static str` card names, so the wire format round-trips through serde.

use serde::{Deserialize, Serialize};

use crate::card::{CardId, CardType, CounterType, Keyword};
use crate::decision::Decision;
use crate::game::{GameAction, GameEvent, Target, TurnStep};
use crate::mana::{Color, ManaCost, ManaPool};

// ── Client → server ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    /// Sent once on connect. The server replies with `YourSeat`.
    JoinMatch { name: String },
    /// A game action (including decision answers wrapped in `GameAction::SubmitDecision`).
    SubmitAction(GameAction),
    /// Debug-console cheat: bypasses turn order / priority and mutates the
    /// authoritative state directly. Applied as the *sender's* seat — the
    /// server routes it to whichever seat owns the originating channel.
    /// Intended for local single-player debugging only.
    Debug(DebugAction),
}

/// Direct-mutation cheats issued by the debug console. Each variant
/// targets the sending seat. Unlike `GameAction`, these do not flow
/// through `perform_action` — the server applies them as raw edits and
/// re-broadcasts the resulting `View`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugAction {
    /// Add `amount` mana of the given color to the sender's mana pool.
    /// `color == None` adds true colorless mana.
    AddMana { color: Option<Color>, amount: u32 },
    /// Look the named card up in the catalog and place a fresh instance
    /// into the sender's hand. Silently dropped if the name is unknown.
    AddCardToHand { name: String },
    /// Bump the sender's life total by `delta` (may be negative).
    AdjustLife { delta: i32 },
}

// ── Server → client ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    /// First message after a successful join. Identifies which seat this
    /// connection controls (needed so the client can filter for its own
    /// hand, decisions, etc.).
    YourSeat(usize),
    /// All seats are filled and the match is starting. Followed by the
    /// first `View`.
    MatchStarted,
    /// Authoritative snapshot of state, projected for this seat.
    View(ClientView),
    /// Events produced by the most recent action, in order. Clients animate
    /// off these; the accompanying `View` is the post-event state.
    Events(Vec<GameEventWire>),
    /// A submitted action was rejected.
    ActionError(String),
    /// The match has ended. `winner` follows `GameState::game_over` semantics.
    MatchOver { winner: Option<usize> },
}

// ── Projected view types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientView {
    pub your_seat: usize,
    pub active_player: usize,
    pub priority: usize,
    pub step: TurnStep,
    pub turn: u32,
    pub players: Vec<PlayerView>,
    pub battlefield: Vec<PermanentView>,
    pub stack: Vec<StackItemView>,
    pub pending_decision: Option<PendingDecisionView>,
    /// Cards currently in the shared exile zone. Always face-up — the
    /// engine has no face-down exile yet. Defaults to `Vec::new()` for
    /// snapshot compatibility with older serialized views.
    #[serde(default)]
    pub exile: Vec<ExileCardView>,
    /// `None` while the game is ongoing; `Some(None)` = draw; `Some(Some(i))` = seat `i` won.
    pub game_over: Option<Option<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerView {
    pub seat: usize,
    pub name: String,
    pub life: i32,
    pub poison_counters: u32,
    pub mana_pool: ManaPool,
    pub library: LibraryView,
    pub graveyard: Vec<GraveyardCardView>,
    /// One entry per card in hand. Each entry is either `Known` (the viewer is
    /// entitled to see the card — always for your own hand, sometimes for an
    /// opponent's via Peek/Thoughtseize/Telepathy-style reveals) or `Hidden`
    /// (id only). The vec length equals the player's hand size.
    pub hand: Vec<HandCardView>,
    pub lands_played_this_turn: u32,
    /// Pending Chancellor-of-the-Annex-style tax charges. Each charge taxes
    /// the player's next spell {1} more (consumed on the next successful
    /// cast). Surfaced to the client so the cast UI can preview the bumped
    /// cost before the player commits.
    #[serde(default)]
    pub first_spell_tax_charges: u32,
    /// Life this player has gained on the current turn (sum of every
    /// `Effect::GainLife`, `Effect::Drain` recipient side, and combat-
    /// lifelink resolution). Reset on the controller's untap. Surfaced
    /// so UIs can show "Infusion ready" hints on cards whose riders fire
    /// once you've gained life this turn (Foolish Fate, Old-Growth
    /// Educator, Tenured Concocter, etc.).
    #[serde(default)]
    pub life_gained_this_turn: u32,
    /// Cards this player has drawn on the current turn. Reset on the
    /// controller's untap. Surfaced so UIs can show "X +1/+1 counters"
    /// previews on Quandrix scaling cards (Fractal Anomaly, etc.).
    #[serde(default)]
    pub cards_drawn_this_turn: u32,
    /// Number of times a card has left this player's graveyard on the
    /// current turn. Reset on the controller's untap. Surfaced so UIs
    /// can show "Lorehold ready" hints on cards whose riders fire when
    /// any card has left your graveyard this turn (Living History,
    /// Primary Research, Wilt in the Heat).
    #[serde(default)]
    pub cards_left_graveyard_this_turn: u32,
    /// Number of creatures controlled by this player that died on the
    /// current turn. Reset on the controller's untap. Surfaced so UIs
    /// can show "Witherbloom end-step ready" hints on cards whose riders
    /// fire when a creature died under your control (Essenceknit Scholar).
    #[serde(default)]
    pub creatures_died_this_turn: u32,
    /// Number of cards this player has caused to be exiled on the current
    /// turn. Reset on the controller's untap. Surfaced so UIs can show
    /// "Ennis end-step counter ready" hints on Strixhaven cards whose
    /// riders fire when any card was put into exile this turn.
    #[serde(default)]
    pub cards_exiled_this_turn: u32,
    /// Number of instant or sorcery spells this player has cast on the
    /// current turn. Refines `spells_cast_this_turn` for cards that gate
    /// activations / triggers on the "instant or sorcery" subset
    /// specifically (Potioner's Trove, future Magecraft variants).
    #[serde(default)]
    pub instants_or_sorceries_cast_this_turn: u32,
    /// Number of creature spells this player has cast on the current
    /// turn. Reserved for future "if you've cast a creature spell this
    /// turn, …" payoffs.
    #[serde(default)]
    pub creatures_cast_this_turn: u32,
    /// True if this player has no maximum hand size for the rest of the
    /// game (Wisdom of Ages, Reliquary Tower-style effects). Surfaced
    /// so UIs can suppress the cleanup-step discard warning when the
    /// player is over 7 cards.
    #[serde(default)]
    pub no_maximum_hand_size: bool,
    /// Cards in this player's command zone (Commander commanders,
    /// Conspiracies, etc.). Always face-up — the command zone is a
    /// public zone, so every entry is `Known` regardless of viewer.
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub command: Vec<HandCardView>,
    /// CardIds the engine has flagged as this player's commanders
    /// (Phase J's `Player.commanders`). Surfaced so the UI can
    /// distinguish "your commander, currently on the battlefield"
    /// from a regular permanent (cast-tax preview, frame highlight).
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub commanders: Vec<CardId>,
}

/// A single hand-slot entry. `Hidden` for cards the viewer isn't entitled to
/// see (typical opponent cards); `Known` when a reveal, or ownership of the
/// hand, grants visibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandCardView {
    Known(KnownCard),
    Hidden { id: CardId },
}

impl HandCardView {
    pub fn id(&self) -> CardId {
        match self {
            HandCardView::Known(k) => k.id,
            HandCardView::Hidden { id } => *id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownCard {
    pub id: CardId,
    pub name: String,
    pub cost: ManaCost,
    pub card_types: Vec<CardType>,
    pub needs_target: bool,
    /// True if this card has an `alternative_cost` (Force of Will / Force of
    /// Negation pitch, Solitude evoke). Drives the client's right-click
    /// "Cast for alt cost" menu entry.
    #[serde(default)]
    pub has_alternative_cost: bool,
    /// MDFC back-face name, if any (e.g. Blightstep Pathway → "Searstep
    /// Pathway"). Drives the client's right-click flip on hand cards: when
    /// `Some`, right-click toggles the card's hand visual to the back face
    /// and a subsequent left-click submits `PlayLandBack` instead of
    /// `PlayLand`.
    #[serde(default)]
    pub back_face_name: Option<String>,
}

/// One activated ability as projected for the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityView {
    pub index: usize,
    pub cost_label: String,
    pub effect_label: String,
    pub needs_target: bool,
    pub is_mana: bool,
    /// True if this ability is flagged "Activate only once each turn" and
    /// has already been used on the current turn — the client can grey
    /// out the button. Defaults to `false` for older clients.
    #[serde(default)]
    pub once_per_turn_used: bool,
    /// True if this ability carries an `ActivatedAbility.condition` gate
    /// (printed "Activate only if …" rider). Clients can show a hint
    /// icon next to the activator. The string is a short human-readable
    /// description of the gate ("≥7 in hand", "after IS-cast this turn",
    /// etc.); empty when no gate is set. Defaults to `("", false)` for
    /// older clients without this field.
    #[serde(default)]
    pub gate_label: String,
    /// True if the ability has a printed gate AND the gate currently
    /// rejects activation. `false` means either no gate (always
    /// activatable on cost grounds) or gate currently passes. The
    /// client can grey out the button when this is `true`. Note: this
    /// is a snapshot at view-projection time; the gate can flip between
    /// snapshots (Resonating Lute flips when hand-size crosses 7).
    #[serde(default)]
    pub gate_blocked: bool,
}

/// A planeswalker's loyalty ability as visible to the client. Mirrors
/// `LoyaltyAbility` but with pre-rendered labels — the client doesn't
/// need access to the engine's `Effect` enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyAbilityView {
    pub index: usize,
    /// Raw loyalty cost: positive (+1, +2) or negative (-1, -3, …).
    pub loyalty_cost: i32,
    /// Pre-rendered effect label ("Draw cards", "Destroy permanent", …).
    pub effect_label: String,
    pub needs_target: bool,
}

/// A library as visible to a specific seat. `size` is always the full count;
/// `known_top` holds any cards the viewer is entitled to see at the top,
/// ordered top-first (populated e.g. after Scry peeks or "look at the top N"
/// effects). Empty means "no visibility beyond the size".
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryView {
    pub size: usize,
    pub known_top: Vec<KnownCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraveyardCardView {
    pub id: CardId,
    pub name: String,
}

/// A single card sitting in the shared exile zone. Owners are surfaced so
/// the client can render "exiled by X" / "exiled from Y's library"
/// distinctions; the engine keeps `CardInstance.owner` in sync across
/// zone moves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExileCardView {
    pub id: CardId,
    pub name: String,
    /// The card's owner (the player to whom the card returns if it
    /// later moves to its owner's hand/library/graveyard).
    pub owner: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentView {
    pub id: CardId,
    pub name: String,
    pub controller: usize,
    pub owner: usize,
    pub card_types: Vec<CardType>,
    pub tapped: bool,
    pub damage: u32,
    pub summoning_sick: bool,
    /// Computed power after layer effects (0 for non-creatures).
    pub power: i32,
    /// Computed toughness after layer effects (0 for non-creatures).
    pub toughness: i32,
    /// Base (printed) power from the card definition. Lets clients
    /// flag a card whose computed P/T differs from the card text.
    pub base_power: i32,
    /// Base (printed) toughness from the card definition.
    pub base_toughness: i32,
    /// Effective keywords (after layer effects).
    pub keywords: Vec<Keyword>,
    pub counters: Vec<(CounterType, u32)>,
    pub attached_to: Option<CardId>,
    pub is_token: bool,
    /// Whether this permanent is currently declared as an attacker.
    pub attacking: bool,
    /// Activated abilities visible to the client.
    pub abilities: Vec<AbilityView>,
    /// Loyalty abilities (only populated for planeswalkers).
    #[serde(default)]
    pub loyalty_abilities: Vec<LoyaltyAbilityView>,
}

impl PermanentView {
    pub fn is_land(&self) -> bool {
        self.card_types.contains(&CardType::Land)
    }
}

/// One item on the stack, as visible to a specific seat. `Hidden` covers
/// face-down spells (e.g. Morph) that reveal only to their caster; `Known`
/// otherwise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StackItemView {
    Known(KnownStackItem),
    Hidden { source: CardId, controller: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StackItemKind {
    /// A spell cast by `controller`. The viewer typically wants to
    /// hold priority on these so they can respond with a counter.
    Spell,
    /// A triggered or activated ability waiting to resolve. Triggers
    /// from your own permanents are mostly bookkeeping (Tireless
    /// Tracker investigate, Goldspan Dragon attack draw, etc.) — UIs
    /// can auto-pass priority on them to keep the flow snappy.
    Trigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownStackItem {
    pub source: CardId,
    pub controller: usize,
    pub name: String,
    pub target: Option<Target>,
    /// Whether this stack item is a `Spell` (caster cast a card) or a
    /// `Trigger` (an ability fired). Defaulted to `Trigger` for
    /// backwards compat with older serialized views.
    #[serde(default = "default_stack_item_kind")]
    pub kind: StackItemKind,
}

fn default_stack_item_kind() -> StackItemKind {
    StackItemKind::Trigger
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDecisionView {
    pub acting_player: usize,
    /// `Some` when the viewer is entitled to see the decision specifics
    /// (typically: the viewer is the acting player). `None` when the viewer
    /// is a spectator who should only know that some other seat is deciding.
    pub decision: Option<DecisionWire>,
}

// ── Wire-side mirrors of engine types with static-string fields ─────────────

/// Mirror of [`Decision`] using owned strings. Engine `&'static str` card
/// names are copied to `String` at projection time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionWire {
    ChooseTarget {
        source: CardId,
        legal: Vec<Target>,
    },
    ChooseMode {
        source: CardId,
        num_modes: usize,
    },
    ChooseColor {
        source: CardId,
        legal: Vec<Color>,
    },
    Scry {
        player: usize,
        cards: Vec<(CardId, String)>,
    },
    Discard {
        player: usize,
        count: u32,
        hand: Vec<(CardId, String)>,
    },
    SearchLibrary {
        player: usize,
        candidates: Vec<(CardId, String)>,
    },
    OptionalTrigger {
        source: CardId,
        description: String,
    },
    PutOnLibrary {
        player: usize,
        count: usize,
        hand: Vec<(CardId, String)>,
    },
    Mulligan {
        player: usize,
        hand: Vec<(CardId, String)>,
        mulligans_taken: usize,
        /// IDs of in-hand Serum-Powder-style mulligan helpers. The client
        /// renders one button per ID alongside the standard Keep/Mulligan
        /// pair; clicking sends `DecisionAnswer::SerumPowder(id)`.
        serum_powders: Vec<CardId>,
    },
    /// "As [card] enters, choose a creature type." Cavern of Souls.
    ChooseCreatureType {
        source: CardId,
    },
    /// CR 903.9b commander redirect — owner may send the commander to
    /// the command zone instead of `would_be`.
    CommanderRedirect {
        commander: CardId,
        would_be: crate::card::Zone,
    },
}

impl From<&Decision> for DecisionWire {
    fn from(d: &Decision) -> Self {
        match d {
            Decision::ChooseTarget { source, legal } => DecisionWire::ChooseTarget {
                source: *source,
                legal: legal.clone(),
            },
            Decision::ChooseMode { source, num_modes } => DecisionWire::ChooseMode {
                source: *source,
                num_modes: *num_modes,
            },
            Decision::ChooseColor { source, legal } => DecisionWire::ChooseColor {
                source: *source,
                legal: legal.clone(),
            },
            Decision::Scry { player, cards } => DecisionWire::Scry {
                player: *player,
                cards: cards.iter().map(|(id, n)| (*id, (*n).to_string())).collect(),
            },
            Decision::Discard { player, count, hand } => DecisionWire::Discard {
                player: *player,
                count: *count,
                hand: hand.iter().map(|(id, n)| (*id, (*n).to_string())).collect(),
            },
            Decision::SearchLibrary { player, candidates } => DecisionWire::SearchLibrary {
                player: *player,
                candidates: candidates
                    .iter()
                    .map(|(id, n)| (*id, (*n).to_string()))
                    .collect(),
            },
            Decision::OptionalTrigger { source, description } => DecisionWire::OptionalTrigger {
                source: *source,
                description: (*description).to_string(),
            },
            Decision::PutOnLibrary { player, count, hand } => DecisionWire::PutOnLibrary {
                player: *player,
                count: *count,
                hand: hand.iter().map(|(id, n)| (*id, (*n).to_string())).collect(),
            },
            Decision::Mulligan { player, hand, mulligans_taken, serum_powders } => {
                DecisionWire::Mulligan {
                    player: *player,
                    hand: hand.iter().map(|(id, n)| (*id, (*n).to_string())).collect(),
                    mulligans_taken: *mulligans_taken,
                    serum_powders: serum_powders.clone(),
                }
            }
            Decision::ChooseCreatureType { source } => {
                DecisionWire::ChooseCreatureType { source: *source }
            }
            Decision::CommanderRedirect { commander, would_be } => {
                DecisionWire::CommanderRedirect {
                    commander: *commander,
                    would_be: *would_be,
                }
            }
        }
    }
}

/// Mirror of [`GameEvent`] using owned strings in the one variant that carries
/// a card name (`TopCardRevealed`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEventWire {
    StepChanged(TurnStep),
    TurnStarted { player: usize, turn: u32 },
    CardDrawn { player: usize, card_id: CardId },
    CardDiscarded { player: usize, card_id: CardId },
    LandPlayed { player: usize, card_id: CardId },
    /// `face` lets replays / spectator UIs distinguish a back-face MDFC
    /// cast (Back) from a normal hand cast (Front) and a flashback
    /// graveyard replay (Flashback). Defaults to `Front` on snapshots
    /// predating the field.
    SpellCast {
        player: usize,
        card_id: CardId,
        #[serde(default)]
        face: crate::game::CastFace,
    },
    AbilityActivated { source: CardId },
    ManaAdded { player: usize, color: Color },
    ColorlessManaAdded { player: usize },
    PermanentEntered { card_id: CardId },
    PermanentExiled { card_id: CardId },
    DamageDealt { amount: u32, to_player: Option<usize>, to_card: Option<CardId> },
    LifeLost { player: usize, amount: u32 },
    LifeGained { player: usize, amount: u32 },
    CreatureDied { card_id: CardId },
    /// Wire mirror of `GameEvent::CreatureSacrificed`. Surfaced so client
    /// UIs can highlight a sacrifice (CR 701.16) distinctly from a
    /// natural death — useful for replay rewinds and aristocrats payoffs.
    CreatureSacrificed { card_id: CardId, who: usize },
    PumpApplied { card_id: CardId, power: i32, toughness: i32 },
    CounterAdded { card_id: CardId, counter_type: CounterType, count: u32 },
    CounterRemoved { card_id: CardId, counter_type: CounterType, count: u32 },
    PermanentTapped { card_id: CardId },
    PermanentUntapped { card_id: CardId },
    TokenCreated { card_id: CardId },
    CardMilled { player: usize, card_id: CardId },
    ScryPerformed { player: usize, looked_at: usize, bottomed: usize },
    AttackerDeclared(CardId),
    BlockerDeclared { blocker: CardId, attacker: CardId },
    CombatResolved,
    FirstStrikeDamageResolved,
    TopCardRevealed { player: usize, card_name: String, is_land: bool },
    AttachmentMoved { attachment: CardId, attached_to: Option<CardId> },
    PoisonAdded { player: usize, amount: u32 },
    LoyaltyAbilityActivated { planeswalker: CardId, loyalty_change: i32 },
    LoyaltyChanged { card_id: CardId, new_loyalty: i32 },
    PlaneswalkerDied { card_id: CardId },
    SpellsCopied { original: CardId, count: u32 },
    SurveilPerformed { player: usize, looked_at: usize, graveyarded: usize },
    /// Wire mirror of `GameEvent::CardLeftGraveyard`. Surfaced so client UIs
    /// can animate "card returned from graveyard" or highlight Lorehold
    /// "cards left graveyard this turn" payoffs.
    CardLeftGraveyard { player: usize, card_id: CardId },
    /// Wire mirror of `GameEvent::BecameTarget`. Surfaced so client UIs
    /// can highlight a permanent that just got targeted by a spell or
    /// ability (Tenured Concocter's "you may draw" trigger, future
    /// targeting-payoff cards).
    BecameTarget { target: CardId, caster: usize },
    GameOver { winner: Option<usize> },
}

impl From<&GameEvent> for GameEventWire {
    fn from(e: &GameEvent) -> Self {
        match e {
            GameEvent::StepChanged(s) => GameEventWire::StepChanged(*s),
            GameEvent::TurnStarted { player, turn } => GameEventWire::TurnStarted {
                player: *player,
                turn: *turn,
            },
            GameEvent::CardDrawn { player, card_id } => GameEventWire::CardDrawn {
                player: *player,
                card_id: *card_id,
            },
            GameEvent::CardDiscarded { player, card_id } => GameEventWire::CardDiscarded {
                player: *player,
                card_id: *card_id,
            },
            GameEvent::LandPlayed { player, card_id } => GameEventWire::LandPlayed {
                player: *player,
                card_id: *card_id,
            },
            GameEvent::SpellCast { player, card_id, face } => GameEventWire::SpellCast {
                player: *player,
                card_id: *card_id,
                face: *face,
            },
            GameEvent::AbilityActivated { source } => {
                GameEventWire::AbilityActivated { source: *source }
            }
            GameEvent::ManaAdded { player, color } => GameEventWire::ManaAdded {
                player: *player,
                color: *color,
            },
            GameEvent::ColorlessManaAdded { player } => {
                GameEventWire::ColorlessManaAdded { player: *player }
            }
            GameEvent::PermanentEntered { card_id } => {
                GameEventWire::PermanentEntered { card_id: *card_id }
            }
            GameEvent::PermanentExiled { card_id } => {
                GameEventWire::PermanentExiled { card_id: *card_id }
            }
            GameEvent::DamageDealt { amount, to_player, to_card } => GameEventWire::DamageDealt {
                amount: *amount,
                to_player: *to_player,
                to_card: *to_card,
            },
            GameEvent::LifeLost { player, amount } => GameEventWire::LifeLost {
                player: *player,
                amount: *amount,
            },
            GameEvent::LifeGained { player, amount } => GameEventWire::LifeGained {
                player: *player,
                amount: *amount,
            },
            GameEvent::CreatureDied { card_id } => {
                GameEventWire::CreatureDied { card_id: *card_id }
            }
            GameEvent::CreatureSacrificed { card_id, who } => {
                GameEventWire::CreatureSacrificed { card_id: *card_id, who: *who }
            }
            GameEvent::PumpApplied { card_id, power, toughness } => GameEventWire::PumpApplied {
                card_id: *card_id,
                power: *power,
                toughness: *toughness,
            },
            GameEvent::CounterAdded { card_id, counter_type, count } => {
                GameEventWire::CounterAdded {
                    card_id: *card_id,
                    counter_type: *counter_type,
                    count: *count,
                }
            }
            GameEvent::CounterRemoved { card_id, counter_type, count } => {
                GameEventWire::CounterRemoved {
                    card_id: *card_id,
                    counter_type: *counter_type,
                    count: *count,
                }
            }
            GameEvent::PermanentTapped { card_id } => {
                GameEventWire::PermanentTapped { card_id: *card_id }
            }
            GameEvent::PermanentUntapped { card_id } => {
                GameEventWire::PermanentUntapped { card_id: *card_id }
            }
            GameEvent::TokenCreated { card_id } => {
                GameEventWire::TokenCreated { card_id: *card_id }
            }
            GameEvent::CardMilled { player, card_id } => GameEventWire::CardMilled {
                player: *player,
                card_id: *card_id,
            },
            GameEvent::ScryPerformed { player, looked_at, bottomed } => {
                GameEventWire::ScryPerformed {
                    player: *player,
                    looked_at: *looked_at,
                    bottomed: *bottomed,
                }
            }
            GameEvent::AttackerDeclared(id) => GameEventWire::AttackerDeclared(*id),
            GameEvent::BlockerDeclared { blocker, attacker } => GameEventWire::BlockerDeclared {
                blocker: *blocker,
                attacker: *attacker,
            },
            GameEvent::CombatResolved => GameEventWire::CombatResolved,
            GameEvent::FirstStrikeDamageResolved => GameEventWire::FirstStrikeDamageResolved,
            GameEvent::TopCardRevealed { player, card_name, is_land } => {
                GameEventWire::TopCardRevealed {
                    player: *player,
                    card_name: (*card_name).to_string(),
                    is_land: *is_land,
                }
            }
            GameEvent::AttachmentMoved { attachment, attached_to } => {
                GameEventWire::AttachmentMoved {
                    attachment: *attachment,
                    attached_to: *attached_to,
                }
            }
            GameEvent::PoisonAdded { player, amount } => GameEventWire::PoisonAdded {
                player: *player,
                amount: *amount,
            },
            GameEvent::LoyaltyAbilityActivated { planeswalker, loyalty_change } => {
                GameEventWire::LoyaltyAbilityActivated {
                    planeswalker: *planeswalker,
                    loyalty_change: *loyalty_change,
                }
            }
            GameEvent::LoyaltyChanged { card_id, new_loyalty } => {
                GameEventWire::LoyaltyChanged {
                    card_id: *card_id,
                    new_loyalty: *new_loyalty,
                }
            }
            GameEvent::PlaneswalkerDied { card_id } => {
                GameEventWire::PlaneswalkerDied { card_id: *card_id }
            }
            GameEvent::SpellsCopied { original, count } => GameEventWire::SpellsCopied {
                original: *original,
                count: *count,
            },
            GameEvent::SurveilPerformed { player, looked_at, graveyarded } => {
                GameEventWire::SurveilPerformed {
                    player: *player,
                    looked_at: *looked_at,
                    graveyarded: *graveyarded,
                }
            }
            GameEvent::CardLeftGraveyard { player, card_id } => {
                GameEventWire::CardLeftGraveyard {
                    player: *player,
                    card_id: *card_id,
                }
            }
            GameEvent::BecameTarget { target, caster } => GameEventWire::BecameTarget {
                target: *target,
                caster: *caster,
            },
            GameEvent::GameOver { winner } => GameEventWire::GameOver { winner: *winner },
        }
    }
}

impl GameEventWire {
    /// Render this event as a one-line human-readable log entry. `name`
    /// resolves card ids to display names (typically the client's
    /// `CardNames` table); unknown ids should fall back to a stable
    /// placeholder like `#N`.
    pub fn fmt_for_log(&self, name: &dyn Fn(CardId) -> String) -> String {
        use GameEventWire as E;
        match self {
            E::StepChanged(s) => format!("Step → {s:?}"),
            E::TurnStarted { player, turn } => format!("Turn {turn} — P{player}"),
            E::CardDrawn { player, card_id } => format!("P{player} drew {}", name(*card_id)),
            E::CardDiscarded { player, card_id } => {
                format!("P{player} discarded {}", name(*card_id))
            }
            E::LandPlayed { player, card_id } => format!("P{player} played {}", name(*card_id)),
            E::SpellCast { player, card_id, .. } => format!("P{player} cast {}", name(*card_id)),
            E::AbilityActivated { source } => format!("{} ability activated", name(*source)),
            E::ManaAdded { player, color } => format!("P{player} adds {color:?}"),
            E::ColorlessManaAdded { player } => format!("P{player} adds colorless"),
            E::PermanentEntered { card_id } => {
                format!("{} entered the battlefield", name(*card_id))
            }
            E::PermanentExiled { card_id } => format!("{} was exiled", name(*card_id)),
            E::DamageDealt {
                amount,
                to_player,
                to_card,
            } => match (to_player, to_card) {
                (Some(p), _) => format!("{amount} damage → P{p}"),
                (_, Some(cid)) => format!("{amount} damage → {}", name(*cid)),
                _ => format!("{amount} damage"),
            },
            E::LifeLost { player, amount } => format!("P{player} loses {amount} life"),
            E::LifeGained { player, amount } => format!("P{player} gains {amount} life"),
            E::CreatureDied { card_id } => format!("{} died", name(*card_id)),
            E::PumpApplied {
                card_id,
                power,
                toughness,
            } => format!("{} +{power}/+{toughness}", name(*card_id)),
            E::CounterAdded {
                card_id,
                counter_type,
                count,
            } => format!("+{count} {counter_type:?} on {}", name(*card_id)),
            E::CounterRemoved {
                card_id,
                counter_type,
                count,
            } => format!("−{count} {counter_type:?} on {}", name(*card_id)),
            E::PermanentTapped { card_id } => format!("{} tapped", name(*card_id)),
            E::PermanentUntapped { card_id } => format!("{} untapped", name(*card_id)),
            E::TokenCreated { card_id } => format!("token {} created", name(*card_id)),
            E::CardMilled { player, card_id } => {
                format!("P{player} milled {}", name(*card_id))
            }
            E::ScryPerformed {
                player,
                looked_at,
                bottomed,
            } => format!("P{player} scry {looked_at} ({bottomed} to bottom)"),
            E::AttackerDeclared(cid) => format!("{} attacks", name(*cid)),
            E::BlockerDeclared { blocker, attacker } => {
                format!("{} blocks {}", name(*blocker), name(*attacker))
            }
            E::CombatResolved => "Combat resolved".into(),
            E::FirstStrikeDamageResolved => "First-strike damage resolved".into(),
            E::TopCardRevealed {
                player, card_name, ..
            } => format!("P{player} revealed {card_name}"),
            E::AttachmentMoved {
                attachment,
                attached_to,
            } => match attached_to {
                Some(target) => format!("{} attached to {}", name(*attachment), name(*target)),
                None => format!("{} unattached", name(*attachment)),
            },
            E::PoisonAdded { player, amount } => format!("P{player} +{amount} poison"),
            E::LoyaltyAbilityActivated {
                planeswalker,
                loyalty_change,
            } => format!("{} loyalty {loyalty_change:+}", name(*planeswalker)),
            E::LoyaltyChanged {
                card_id,
                new_loyalty,
            } => format!("{} loyalty = {new_loyalty}", name(*card_id)),
            E::PlaneswalkerDied { card_id } => {
                format!("{} died (planeswalker)", name(*card_id))
            }
            E::SpellsCopied { original, count } => {
                format!("{} copied ×{count}", name(*original))
            }
            E::SurveilPerformed {
                player,
                looked_at,
                graveyarded,
            } => format!("P{player} surveil {looked_at} ({graveyarded} to graveyard)"),
            E::CardLeftGraveyard { player, card_id } => {
                format!("P{player} {} left graveyard", name(*card_id))
            }
            E::BecameTarget { target, caster } => {
                format!("{} targeted by P{caster}", name(*target))
            }
            E::GameOver { winner } => match winner {
                Some(p) => format!("Game over — P{p} wins"),
                None => "Game over — draw".into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_msg_roundtrips() {
        let msg = ClientMsg::SubmitAction(GameAction::PlayLand(CardId(7)));
        let json = serde_json::to_string(&msg).unwrap();
        let back: ClientMsg = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            back,
            ClientMsg::SubmitAction(GameAction::PlayLand(CardId(7)))
        ));
    }

    #[test]
    fn decision_wire_converts() {
        let d = Decision::Scry {
            player: 0,
            cards: vec![(CardId(1), "Island".into()), (CardId(2), "Forest".into())],
        };
        let w: DecisionWire = (&d).into();
        let json = serde_json::to_string(&w).unwrap();
        let back: DecisionWire = serde_json::from_str(&json).unwrap();
        match back {
            DecisionWire::Scry { player, cards } => {
                assert_eq!(player, 0);
                assert_eq!(cards[0].1, "Island");
            }
            _ => panic!("wrong variant"),
        }
    }
}
