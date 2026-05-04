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
    /// Number of distinct card types across cards in this player's
    /// graveyard. Surfaced so UIs can show "Delirium active" hints on
    /// MH2 Delirium payoffs (Dragon's Rage Channeler's body buff,
    /// Unholy Heat's 3 → 6 damage scale-up, future cards). The
    /// threshold is 4; clients can render the full count or
    /// `min(count, 4)` as preferred. Push XLVII addition; defaulted
    /// via `#[serde(default)]` for back-compat with older serialized
    /// views.
    #[serde(default)]
    pub distinct_card_types_in_graveyard: u32,
    /// True if this player can't gain life this turn (Skullcrack-
    /// style lock). Cleared at the player's next untap. Surfaced so
    /// UIs can show a "no lifegain" badge on the player frame and
    /// suppress lifegain-payoff hints (Soul Warden, Bayou Groff,
    /// Sheoldred lifelink) for the rest of the turn. Push XLVII
    /// addition; defaulted via `#[serde(default)]` for back-compat.
    #[serde(default)]
    pub lifegain_prevented_this_turn: bool,
    /// Convenience derived flag: true iff
    /// `distinct_card_types_in_graveyard >= 4` (the printed Delirium
    /// threshold for MH2's Delirium cycle: Unholy Heat, Dragon's Rage
    /// Channeler, Tourach's Canticle, etc.). Pre-computed by the
    /// server so clients can render a "Delirium active" badge without
    /// needing to recompute the threshold themselves. Same shape as
    /// `instants_or_sorceries_cast_this_turn` — derived field, but
    /// surfaced for one-glance UI rendering. Defaulted via
    /// `#[serde(default)]` for back-compat.
    #[serde(default)]
    pub delirium_active: bool,
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
    /// Short label for an additional cast cost beyond mana — currently
    /// "sacrifice a creature" / similar. `None` when the spell has no
    /// extra cost. Lets the client warn before wasting mana on a spell
    /// the controller can't currently afford (Daemogoth Woe-Eater /
    /// Eyeblight Cullers without a creature to sacrifice). Defaulted
    /// to `None` for older serialized views.
    #[serde(default)]
    pub additional_cost_label: Option<String>,
    /// Short label for the printed "this enters with N counters"
    /// replacement (`CardDefinition.enters_with_counters`). Lets the
    /// client tooltip show "Enters with 2 +1/+1 counters" or
    /// "Enters with X +1/+1 counters" so players can see the printed
    /// body modification before casting (Star Pupil's "0/0 enters
    /// with two +1/+1 counters", Pterafractyl's "1/0 enters with X
    /// +1/+1 counters"). Defaulted to `None` for older serialized
    /// views.
    #[serde(default)]
    pub enters_with_counters_label: Option<String>,
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
    /// Current loyalty (= count of `CounterType::Loyalty` counters on
    /// the permanent). `Some(N)` only for planeswalkers, `None` for
    /// non-planeswalkers. Lets clients render "Liliana 3" without
    /// scanning `counters` for the loyalty entry. Per CR 306.5c. Push
    /// XLII addition; defaulted via `#[serde(default)]` so older
    /// serialized views continue to deserialize.
    #[serde(default)]
    pub loyalty: Option<i32>,
    /// Short descriptions of static abilities (anthem effects, cost
    /// reductions, taxes, etc.) — one entry per `StaticAbility` on the
    /// underlying card definition. Populated from
    /// `StaticAbility.description` so the UI can render the printed
    /// rules text without rebuilding it from the static-effect
    /// structure. Defaulted to empty for back-compat with older
    /// serialized views.
    #[serde(default)]
    pub static_abilities: Vec<String>,
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
    /// Multi-modal mode pick (Strixhaven Commands, Moment of Reckoning).
    /// Client renders a `pick_count`-of-`modes_count` checkbox group with
    /// the printed mode labels (the labels are surfaced separately via the
    /// existing `view::ability_effect_label`).
    ChooseModes {
        source: CardId,
        modes_count: usize,
        pick_count: usize,
        up_to: bool,
        allow_duplicates: bool,
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
            Decision::ChooseModes {
                source,
                modes_count,
                pick_count,
                up_to,
                allow_duplicates,
            } => DecisionWire::ChooseModes {
                source: *source,
                modes_count: *modes_count,
                pick_count: *pick_count,
                up_to: *up_to,
                allow_duplicates: *allow_duplicates,
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
    /// Wire mirror of `GameEvent::CombatDamagePreventedThisTurn`.
    /// Surfaced so spectator UIs can render the prevention shield
    /// (Owlin Shieldmage / Holy Day-style fog).
    CombatDamagePreventedThisTurn,
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
            GameEvent::CombatDamagePreventedThisTurn => GameEventWire::CombatDamagePreventedThisTurn,
            GameEvent::GameOver { winner } => GameEventWire::GameOver { winner: *winner },
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
