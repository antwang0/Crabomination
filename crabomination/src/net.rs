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
    View(Box<ClientView>),
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
    /// CR 615.12 — true while "damage can't be prevented this turn" is in
    /// effect. Surfaced so UIs can warn that prevention shields are off.
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub damage_cant_be_prevented_this_turn: bool,
    /// Projected combat outcome for the current attacker/blocker
    /// assignment (Tier-7 #3 "combat math preview"). `None` outside of
    /// combat or when no attackers are declared. `#[serde(default)]` for
    /// snapshot back-compat.
    #[serde(default)]
    pub combat_preview: Option<CombatPreview>,
    /// CardIds in the viewer's own hand they could begin casting (or play,
    /// for lands) right now — computed server-side via the engine's
    /// `would_accept` dry-run so it already accounts for timing,
    /// auto-tappable mana, cost taxes, and target availability. Drives the
    /// client's "castable" hand-card highlight. Empty when the viewer
    /// doesn't hold priority. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub castable_hand: Vec<CardId>,
    /// CardIds in the viewer's hand with an activatable "from hand" ability
    /// right now (Spirit-Guide pitch: "Exile this from your hand: Add mana").
    /// Lets the client show a pitch affordance separate from the castable
    /// highlight. Empty off-priority. `#[serde(default)]` for back-compat.
    #[serde(default)]
    pub pitchable_hand: Vec<CardId>,
    /// CardIds in the viewer's hand they could cast with their Kicker paid
    /// right now (CR 702.32). Lets the client offer a "pay kicker?" toggle
    /// distinct from the plain castable highlight. Empty off-priority.
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub kickable_hand: Vec<CardId>,
    /// CardIds in the viewer's hand they could cast paying their Buyback
    /// cost right now (CR 702.27). Lets the client offer a "pay buyback?"
    /// toggle distinct from the plain castable highlight. Empty
    /// off-priority. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub buyback_hand: Vec<CardId>,
    /// CardIds in the viewer's hand they could cast via their Dash
    /// alternative cost right now (CR 702.110). Lets the client offer a
    /// "dash?" affordance distinct from the plain castable highlight.
    /// Empty off-priority. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub dashable_hand: Vec<CardId>,
    /// CardIds of permanents the viewer controls with an activated ability
    /// they could use right now (timing/mana/tap/target all checked). Lets
    /// the client highlight "this permanent can do something." Empty
    /// off-priority. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub activatable_permanents: Vec<CardId>,
    /// Creatures the viewer controls that may be declared as attackers right
    /// now (only during the viewer's Declare Attackers step). Drives the
    /// client's legal-attacker highlight. Empty otherwise. `#[serde(default)]`
    /// for snapshot back-compat.
    #[serde(default)]
    pub legal_attackers: Vec<CardId>,
    /// Creatures the viewer controls that could legally block one of the
    /// declared attackers right now (only during Declare Blockers). Drives
    /// the client's legal-blocker highlight. Empty otherwise.
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub legal_blockers: Vec<CardId>,
}

/// A projected combat-damage summary, computed from the currently
/// declared attackers and blocks *before* damage is actually dealt.
/// Drives the client's "life swing / who dies" combat math display.
///
/// The projection is a single regular-damage step (first/double strike
/// is treated as a single hit) and assumes blockers deal their full
/// power back to their attacker. It mirrors `resolve_combat`'s outcome
/// for the common cases (unblocked swings, 1-for-1 trades, trample
/// overflow, deathtouch, lifelink) without mutating state or firing
/// triggers.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CombatPreview {
    /// `(seat, damage)` — combat damage each player would take (unblocked
    /// attackers + trample overflow).
    pub damage_to_players: Vec<(usize, i32)>,
    /// `(seat, life)` — lifelink life each player would gain from their
    /// creatures' projected combat damage.
    pub lifegain_to_players: Vec<(usize, i32)>,
    /// CardIds of creatures (attackers and blockers) projected to die from
    /// this combat's damage.
    pub dying_creatures: Vec<CardId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerView {
    pub seat: usize,
    pub name: String,
    pub life: i32,
    pub poison_counters: u32,
    /// CR 122 energy counters ({E}) this player has. Surfaced so the
    /// client HUD can show an energy chip alongside life/poison.
    #[serde(default)]
    pub energy: u32,
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
    /// CR 121.2b — the per-turn draw cap currently imposed on this player by
    /// any `StaticEffect::CapDrawsPerTurn` (Spirit of the Labyrinth / Notion
    /// Thief-class locks), or `None` if they may draw freely. UIs show
    /// "draws: X / cap" and grey out draw payoffs once the cap is reached.
    #[serde(default)]
    pub draw_cap: Option<u32>,
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
    /// Total spells cast this turn (all types). Enables storm-count
    /// display and cards with "if you've cast N spells this turn" gates.
    #[serde(default)]
    pub spells_cast_this_turn: u32,
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
    /// True when this player has lost the game (life ≤ 0, drew from empty
    /// library, Pact fail, etc.). Surfaced so UIs can grey out eliminated
    /// players' portraits and skip them in turn order display.
    #[serde(default)]
    pub eliminated: bool,
    /// Names of CR 114 emblems this player owns (command-zone, never
    /// leave). Surfaced so the UI can show active planeswalker-ultimate
    /// emblems. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub emblems: Vec<String>,
    /// True when an active prevention shield (CR 615) protects this player
    /// from some/all damage this turn. Surfaced so UIs can flag a shielded
    /// player. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub has_prevention_shield: bool,
    /// CR 700.5 — this player's devotion to each color, ordered W, U, B, R,
    /// G (the count of mana symbols of that color among the mana costs of
    /// permanents they control). Surfaced so UIs can show a devotion readout
    /// for Theros decks (Nyx gods, Nykthos, Gray Merchant). `#[serde(default)]`
    /// for snapshot back-compat (defaults to all-zero).
    #[serde(default)]
    pub devotion: [u32; 5],
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
    /// True if this card has `Keyword::Cycling(cost)`. Drives the
    /// client's "Cycle" hand action — when true, the client can submit
    /// `GameAction::Cycle` to discard-and-draw at the cycling cost
    /// (rendered as `cycling_cost_label` for the UI).
    /// Defaults to `false` for older clients.
    #[serde(default)]
    pub has_cycling: bool,
    /// Pre-rendered cycling cost label (e.g. "{1}{U}"). Empty string
    /// when `has_cycling == false`. Used by the client to render the
    /// cycle activation hint. Defaults to "" for older clients.
    #[serde(default)]
    pub cycling_cost_label: String,
    /// One short description per mode for a "Choose one —" modal spell
    /// (Artistic Process, Charms, the Command cycle, etc.). Drawn from
    /// `Effect::ChooseMode(modes).iter().map(effect_short_text)`. Empty
    /// when the card is non-modal. Drives the client's mode-pick modal:
    /// when non-empty, clicking to cast surfaces the mode list and the
    /// chosen index is threaded into `GameAction::CastSpell.mode`.
    #[serde(default)]
    pub modal_descriptions: Vec<String>,
    /// Parallel to `modal_descriptions`: `true` at index `i` when mode
    /// `i` carries a targeted slot. The client uses this to decide
    /// whether to drop into the existing targeting cursor after the
    /// mode pick or to fire the cast immediately.
    #[serde(default)]
    pub modal_needs_target: Vec<bool>,
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
    #[serde(default)]
    pub card_types: Vec<CardType>,
    #[serde(default)]
    pub mana_cost: crate::mana::ManaCost,
    #[serde(default)]
    pub power: i32,
    #[serde(default)]
    pub toughness: i32,
    /// Flashback cost (CR 702.34) if this card can be cast from the
    /// graveyard via `GameAction::CastFlashback`. `None` otherwise.
    #[serde(default)]
    pub flashback_cost: Option<crate::mana::ManaCost>,
    /// True if this card has Retrace (CR 702.81) and can be recast from
    /// the graveyard via `GameAction::CastRetrace` (cost + discard a land).
    #[serde(default)]
    pub retrace: bool,
    /// Escape (CR 702.139) cost + count of other graveyard cards to exile,
    /// if this card can be cast from the graveyard via
    /// `GameAction::CastEscape`. `None` otherwise.
    #[serde(default)]
    pub escape: Option<(crate::mana::ManaCost, u32)>,
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
    /// If `Some`, the seat that has a `may_play_until` permission on
    /// this exiled card. Lets the client surface a "may play X" badge
    /// on Conspiracy Theorist / Suspend Aggression / Elemental Mascot /
    /// The Dawning Archaic-style exiled-with-permission cards.
    /// `None` for plain exile (no may-play grant).
    #[serde(default)]
    pub may_play_recipient: Option<usize>,
    /// Card's mana value (CMC). Surfaced so the client can render the
    /// cost badge on exile-browser entries without needing the
    /// full CardDefinition. 0 for cards with no cost (lands).
    #[serde(default)]
    pub mana_value: u32,
    /// Whether this card is a token (which would normally be cleaned
    /// up by SBA but may exist transiently). Lets the client style
    /// token entries distinctly.
    #[serde(default)]
    pub is_token: bool,
    /// CR 603.6e — if this card is exiled "until ~ leaves the
    /// battlefield" (Banisher Priest / Oblivion Ring / Brain Maggot),
    /// the `CardId` of the permanent it's linked to. Lets the client draw
    /// a "returns when X leaves" tether. `None` for plain exile.
    #[serde(default)]
    pub exiled_by: Option<CardId>,
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
    /// If this permanent is declared as a blocker, the attacker it is
    /// blocking. `None` when the permanent isn't a blocker. Exposed so
    /// the client can animate the blocker toward its attacker.
    #[serde(default)]
    pub blocking_attacker: Option<CardId>,
    /// Activated abilities visible to the client.
    pub abilities: Vec<AbilityView>,
    /// Loyalty abilities (only populated for planeswalkers).
    #[serde(default)]
    pub loyalty_abilities: Vec<LoyaltyAbilityView>,
    /// Compact one-line summary per triggered ability ("ETB: Draw a
    /// card", "Magecraft: Drain 1", "Dies: Mill 2"). Lets the client
    /// surface the printed trigger text in tooltips without round-
    /// tripping the full `Effect` tree. Defaults to empty for older
    /// clients. Populated by `project_permanent` via
    /// `triggered_ability_label`.
    #[serde(default)]
    pub triggered_ability_labels: Vec<String>,
    /// Compact descriptions of the printed static abilities ("Other
    /// Inkling creatures you control get +2/+2.", "Each opponent
    /// can't gain life.", "Spells you cast that target a creature
    /// cost {2} less to cast."). Pulled straight from
    /// `StaticAbility.description` so the client tooltip can render
    /// the printed Oracle wording without the engine threading
    /// `Effect`/`StaticEffect` trees over the wire. Defaults to
    /// empty for older clients without this field.
    #[serde(default)]
    pub static_ability_labels: Vec<String>,
    /// True when the permanent has one or more stun counters — a UI
    /// hint so the client can badge stunned permanents without scanning
    /// the full `counters` vec. Populated by `project_permanent`.
    #[serde(default)]
    pub has_stun_counters: bool,
    /// True when the permanent has one or more finality counters
    /// (CR 122.1h). Clients can badge with a "→ exile on death" icon
    /// so the player knows the permanent will exile instead of going
    /// to the graveyard. Populated by `project_permanent`.
    #[serde(default)]
    pub has_finality_counters: bool,
    /// True when the permanent has one or more shield counters
    /// (CR 122.1c). Clients can badge with a "🛡" icon — the shield
    /// counter creates a damage-prevention + destroy-replacement that
    /// pops a counter on each trigger. Populated by `project_permanent`.
    #[serde(default)]
    pub has_shield_counters: bool,
    /// True when an active prevention shield (CR 615, distinct from a
    /// shield *counter*) protects this permanent from some/all damage
    /// this turn. Populated by `project_permanent`.
    #[serde(default)]
    pub has_prevention_shield: bool,
    /// True when this creature is goaded (CR 701.38) — a UI hint so the
    /// client can badge it as "must attack." Populated by
    /// `project_permanent`.
    #[serde(default)]
    pub goaded: bool,
    /// True when this permanent is monstrous (CR 701.31). Populated by
    /// `project_permanent`.
    #[serde(default)]
    pub monstrous: bool,
    /// True when the permanent's computed power or toughness differs
    /// from its base (printed) values — a UI hint for rendering
    /// modified P/T in a distinct color. Always false for non-creatures.
    #[serde(default)]
    pub pt_modified: bool,
    /// Human-readable mana cost string (e.g. "{2}{W}{B}"). Empty for
    /// tokens and lands. Lets the client render the CMC badge in
    /// tooltips and draft-pick overlays.
    #[serde(default)]
    pub mana_cost_display: String,
    /// Creature subtypes (e.g. ["Human", "Wizard"]). Empty for
    /// non-creatures. Enables client tooltip type-line rendering and
    /// tribal-filter UIs without decoding the full card definition.
    #[serde(default)]
    pub creature_types: Vec<String>,
    /// Ward cost (generic mana) on this permanent, if any. 0 means no Ward.
    #[serde(default)]
    pub ward_cost: u32,
    /// Mana value (converted mana cost) of the card. Useful for UI display
    /// and for client-side filtering/sorting.
    #[serde(default)]
    pub mana_value: u32,
    /// Whether this permanent is legendary. Surfaced for UI display
    /// (crown icon, gold name border).
    #[serde(default)]
    pub is_legendary: bool,
    /// True when the permanent has one or more +1/+1 counters. Surfaced
    /// so clients can badge boosted creatures with a "+1/+1" overlay
    /// without scanning the full `counters` vec. Populated by
    /// `project_permanent`. Push (modern_decks): added in batch 174.
    #[serde(default)]
    pub has_plus_one_counters: bool,
    /// True when the permanent has one or more -1/-1 counters. Surfaced
    /// so clients can badge damaged creatures with a "-1/-1" overlay
    /// without scanning the full `counters` vec. Populated by
    /// `project_permanent`. Push (modern_decks): added in batch 174.
    #[serde(default)]
    pub has_minus_one_counters: bool,
    /// Total number of counters on this permanent (sum across all
    /// counter types). Surfaced for client UIs that want a single
    /// counter-count badge (e.g. on planeswalkers and Sagas). Populated
    /// by `project_permanent`. Push (modern_decks): added in batch 174.
    #[serde(default)]
    pub total_counter_count: u32,
    /// Per-keyword counter map for CR 122.1b keyword counters
    /// (flying, first strike, deathtouch, trample, lifelink, haste,
    /// vigilance, reach, …). Surfaced so a client tooltip can render
    /// "+1 flying counter", "+2 first strike counters" etc. alongside
    /// the existing +1/+1 / shield / finality / stun highlights.
    /// Populated by `project_permanent`. Push (modern_decks, batch 187).
    #[serde(default)]
    pub keyword_counters: Vec<(Keyword, u32)>,
    /// Number of shield counters on this permanent. Each shield counter
    /// absorbs one damage event or one destroy event (CR 122.1c) and is
    /// then consumed. Surfaced so the client tooltip can render
    /// "🛡 ×N (absorbs N events)" instead of the binary "shielded". 0
    /// when no shields. Push (claude/modern_decks, batches 192-193).
    #[serde(default)]
    pub shield_counter_count: u32,
    /// Number of stun counters on this permanent. Each stun counter
    /// causes the next untap to be skipped (CR 122.1d) and is then
    /// consumed. Surfaced so the client tooltip can render "stunned ×N"
    /// (skips N untap steps). 0 when not stunned. Push
    /// (claude/modern_decks, batches 192-193).
    #[serde(default)]
    pub stun_counter_count: u32,
    /// Number of finality counters on this permanent. One or more
    /// finality counters redirect graveyard moves to exile (CR 122.1h);
    /// the redirect is single-event-per-counter so multiple finality
    /// counters chain. Surfaced for symmetry with shield/stun counts.
    /// Push (claude/modern_decks, batches 192-193).
    #[serde(default)]
    pub finality_counter_count: u32,
    /// Number of regeneration shields on this permanent (CR 701.15). Each
    /// shield replaces the next destruction this turn with a tap + heal +
    /// remove-from-combat. Transient (cleared at cleanup); surfaced so the
    /// client can badge a "regen-shielded" creature.
    #[serde(default)]
    pub regeneration_shields: u32,
    /// True when this permanent is an Equipment that carries an equip ability
    /// (`Keyword::Equip`). Lets the client offer the "equip" action (CR
    /// 702.6) on the permanent without decoding its keyword list. Populated
    /// by `project_permanent`.
    #[serde(default)]
    pub equippable: bool,
    /// Crew cost (required total power, CR 702.122) when this permanent is a
    /// Vehicle with `Keyword::Crew(N)`. 0 means not crewable. Lets the client
    /// offer the "crew" action. Populated by `project_permanent`.
    #[serde(default)]
    pub crew_value: u32,
    /// True when this creature's marked damage is already lethal (≥ its
    /// current toughness) and it isn't indestructible — i.e. it will die
    /// at the next state-based-action check. Lets the client grey out /
    /// flag doomed creatures during combat-damage preview without
    /// recomputing toughness. Populated by `project_permanent`.
    #[serde(default)]
    pub marked_lethal: bool,
    /// The card name this permanent has chosen (CR 201.3 — Pithing Needle,
    /// Phyrexian Revoker). Lets the client badge "naming: <X>" and grey out
    /// the suppressed source's activated abilities. `None` for the common
    /// case. Populated by `project_permanent`.
    #[serde(default)]
    pub named_card: Option<String>,
}

impl PermanentView {
    pub fn is_land(&self) -> bool {
        self.card_types.contains(&CardType::Land)
    }

    pub fn is_creature(&self) -> bool {
        self.card_types.contains(&CardType::Creature)
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
    /// Additional targets (slots 1+) for multi-target spells/abilities —
    /// divided-damage burn (Forked Bolt, Crackle with Power), Snow Day,
    /// "choose one or both" modes. Lets the client draw a targeting arrow
    /// to *every* target, not just slot 0. Empty for single-target items;
    /// `#[serde(default)]` for view back-compat.
    #[serde(default)]
    pub additional_targets: Vec<Target>,
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
        /// Printed name of the source card (e.g. "Ascendant Dustspeaker").
        /// Copied from the engine's `Decision::ChooseTarget.source_name`
        /// at projection time so the client can show "<name> — <desc>"
        /// in its trigger prompt without re-looking up the card. Empty
        /// when the source can't be resolved (rare; pre-#TriggerTargetPick
        /// snapshots).
        #[serde(default)]
        source_name: String,
        /// Short human-readable rendering of the trigger's effect
        /// (e.g. "exile target card from a graveyard"). May be empty
        /// when the effect's shape isn't covered by `effect_short_text`.
        #[serde(default)]
        description: String,
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
    /// CR 201.3 — "As [card] enters, choose a card name." Pithing Needle.
    NameCard {
        source: CardId,
        source_name: String,
    },
    /// CR 903.9b commander redirect — owner may send the commander to
    /// the command zone instead of `would_be`.
    CommanderRedirect {
        commander: CardId,
        would_be: crate::card::Zone,
    },
    /// CR 705 — flip a coin. Decider answers `Bool(true)` for heads,
    /// `Bool(false)` for tails.
    CoinFlip {
        player: usize,
    },
    /// CR 706 — roll an N-sided die. Decider answers `DieRoll(n)`
    /// where `1 <= n <= sides`.
    DieRoll {
        player: usize,
        sides: u8,
    },
    /// CR 510.1c — order the blockers of one attacker for combat-damage
    /// assignment. Decider answers `DamageOrder(ordered_ids)`.
    CombatDamageOrder {
        attacker: CardId,
        blockers: Vec<(CardId, String)>,
    },
    /// CR 700.2d — choose `count` distinct modes for a "choose N" spell.
    ChooseModes {
        source: CardId,
        num_modes: usize,
        count: usize,
        default: Vec<u8>,
    },
    /// CR 701.45 — Learn: reveal a Lesson from `lessons` into hand, or
    /// discard a card from `hand` to draw.
    Learn {
        player: usize,
        lessons: Vec<(CardId, String)>,
        hand: Vec<(CardId, String)>,
    },
    /// CR 603.3b — order your own simultaneous triggers. `triggers` lists
    /// the simultaneous same-controller triggers in the engine's default
    /// order; the client answers with the desired stack-push order.
    OrderTriggers {
        player: usize,
        triggers: Vec<(CardId, String)>,
    },
    /// CR 601.2d — divide `total` damage among `targets`. Decider answers
    /// `DamageDivision(amounts)` parallel to `targets`.
    DivideDamage {
        source: CardId,
        total: u32,
        targets: Vec<Target>,
    },
    /// CR 510.1c-d — divide an attacker's combat damage among its blockers.
    /// Decider answers `CombatDamageAssignment((id, amount) pairs)`.
    AssignCombatDamage {
        attacker: CardId,
        attacker_power: u32,
        blockers: Vec<(CardId, String, u32)>,
    },
    /// CR 704.5j — choose which of several same-named legends to keep.
    /// Decider answers `KeptLegend(id)`.
    ChooseLegendToKeep {
        player: usize,
        name: String,
        duplicates: Vec<(CardId, String)>,
    },
    /// Choose a number (sacrifice any number / pay any amount of life).
    /// Decider answers `Amount(n)`.
    ChooseAmount {
        source: CardId,
        prompt: String,
        max: u32,
    },
}

impl From<&Decision> for DecisionWire {
    fn from(d: &Decision) -> Self {
        match d {
            Decision::ChooseTarget { source, legal, source_name, description } => {
                DecisionWire::ChooseTarget {
                    source: *source,
                    legal: legal.clone(),
                    source_name: source_name.clone(),
                    description: description.clone(),
                }
            }
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
            Decision::NameCard { source, source_name } => DecisionWire::NameCard {
                source: *source,
                source_name: source_name.clone(),
            },
            Decision::CommanderRedirect { commander, would_be } => {
                DecisionWire::CommanderRedirect {
                    commander: *commander,
                    would_be: *would_be,
                }
            }
            Decision::CoinFlip { player } => DecisionWire::CoinFlip { player: *player },
            Decision::DieRoll { player, sides } => DecisionWire::DieRoll {
                player: *player,
                sides: *sides,
            },
            Decision::CombatDamageOrder { attacker, blockers } => {
                DecisionWire::CombatDamageOrder {
                    attacker: *attacker,
                    blockers: blockers.clone(),
                }
            }
            Decision::ChooseModes { source, num_modes, count, default } => {
                DecisionWire::ChooseModes {
                    source: *source,
                    num_modes: *num_modes,
                    count: *count,
                    default: default.clone(),
                }
            }
            Decision::Learn { player, lessons, hand } => DecisionWire::Learn {
                player: *player,
                lessons: lessons.clone(),
                hand: hand.clone(),
            },
            Decision::OrderTriggers { player, triggers } => DecisionWire::OrderTriggers {
                player: *player,
                triggers: triggers.clone(),
            },
            Decision::DivideDamage { source, total, targets } => DecisionWire::DivideDamage {
                source: *source,
                total: *total,
                targets: targets.clone(),
            },
            Decision::AssignCombatDamage { attacker, attacker_power, blockers } => {
                DecisionWire::AssignCombatDamage {
                    attacker: *attacker,
                    attacker_power: *attacker_power,
                    blockers: blockers.clone(),
                }
            }
            Decision::ChooseLegendToKeep { player, name, duplicates } => {
                DecisionWire::ChooseLegendToKeep {
                    player: *player,
                    name: name.clone(),
                    duplicates: duplicates.clone(),
                }
            }
            Decision::ChooseAmount { source, prompt, max } => DecisionWire::ChooseAmount {
                source: *source,
                prompt: prompt.clone(),
                max: *max,
            },
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
    /// Wire mirror of `GameEvent::DamagePrevented` (CR 615.13).
    DamagePrevented { amount: u32, to_player: Option<usize>, to_card: Option<CardId> },
    LifeLost { player: usize, amount: u32 },
    LifeGained { player: usize, amount: u32 },
    /// Wire mirror of `GameEvent::EnergyGained`.
    EnergyGained { player: usize, amount: u32 },
    CreatureDied { card_id: CardId },
    /// Wire mirror of `GameEvent::CreatureSacrificed`. Surfaced so client
    /// UIs can highlight a sacrifice (CR 701.16) distinctly from a
    /// natural death — useful for replay rewinds and aristocrats payoffs.
    CreatureSacrificed { card_id: CardId, who: usize },
    /// Wire mirror of `GameEvent::PermanentSacrificed`. Broader-scope
    /// sibling of `CreatureSacrificed` — fires for non-creature
    /// sacrifices too (Treasure / Clue / Food / land / planeswalker).
    /// Surfaced so client UIs can highlight every sacrifice
    /// regardless of type for replay rewinds and Korvold/Mayhem-Devil
    /// payoffs.
    PermanentSacrificed { card_id: CardId, who: usize },
    PumpApplied { card_id: CardId, power: i32, toughness: i32 },
    CounterAdded { card_id: CardId, counter_type: CounterType, count: u32 },
    CounterRemoved { card_id: CardId, counter_type: CounterType, count: u32 },
    PermanentTapped { card_id: CardId },
    PermanentUntapped { card_id: CardId },
    Explored { card_id: CardId, controller: usize },
    BecameMonstrous { card_id: CardId },
    TokenCreated { card_id: CardId },
    CardMilled { player: usize, card_id: CardId },
    ScryPerformed { player: usize, looked_at: usize, bottomed: usize },
    AttackerDeclared(CardId),
    BlockerDeclared { blocker: CardId, attacker: CardId },
    AttackerWentUnblocked { attacker: CardId },
    CombatResolved,
    FirstStrikeDamageResolved,
    TopCardRevealed { player: usize, card_name: String, is_land: bool },
    AttachmentMoved { attachment: CardId, attached_to: Option<CardId> },
    VehicleCrewed { vehicle: CardId },
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
    /// Wire mirror of `GameEvent::CardCycled`. Surfaced so client UIs
    /// can animate cycle activations distinctly from regular
    /// hand-discards. Per CR 702.29.
    CardCycled { player: usize, card_id: CardId },
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
            GameEvent::DamagePrevented { amount, to_player, to_card } => {
                GameEventWire::DamagePrevented {
                    amount: *amount,
                    to_player: *to_player,
                    to_card: *to_card,
                }
            }
            GameEvent::LifeLost { player, amount } => GameEventWire::LifeLost {
                player: *player,
                amount: *amount,
            },
            GameEvent::LifeGained { player, amount } => GameEventWire::LifeGained {
                player: *player,
                amount: *amount,
            },
            GameEvent::EnergyGained { player, amount } => GameEventWire::EnergyGained {
                player: *player,
                amount: *amount,
            },
            GameEvent::CreatureDied { card_id } => {
                GameEventWire::CreatureDied { card_id: *card_id }
            }
            GameEvent::CreatureSacrificed { card_id, who } => {
                GameEventWire::CreatureSacrificed { card_id: *card_id, who: *who }
            }
            GameEvent::PermanentSacrificed { card_id, who } => {
                GameEventWire::PermanentSacrificed { card_id: *card_id, who: *who }
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
            GameEvent::Explored { card_id, controller } => {
                GameEventWire::Explored { card_id: *card_id, controller: *controller }
            }
            GameEvent::BecameMonstrous { card_id } => {
                GameEventWire::BecameMonstrous { card_id: *card_id }
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
            GameEvent::AttackerWentUnblocked { attacker } => {
                GameEventWire::AttackerWentUnblocked { attacker: *attacker }
            }
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
            GameEvent::VehicleCrewed { vehicle } => {
                GameEventWire::VehicleCrewed { vehicle: *vehicle }
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
            GameEvent::CardCycled { player, card_id } => GameEventWire::CardCycled {
                player: *player,
                card_id: *card_id,
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
            E::DamagePrevented {
                amount,
                to_player,
                to_card,
            } => match (to_player, to_card) {
                (Some(p), _) => format!("prevented {amount} damage → P{p}"),
                (_, Some(cid)) => format!("prevented {amount} damage → {}", name(*cid)),
                _ => format!("prevented {amount} damage"),
            },
            E::LifeLost { player, amount } => format!("P{player} loses {amount} life"),
            E::LifeGained { player, amount } => format!("P{player} gains {amount} life"),
            E::EnergyGained { player, amount } => format!("P{player} gets {amount} energy"),
            E::CreatureDied { card_id } => format!("{} died", name(*card_id)),
            E::CreatureSacrificed { card_id, who } => {
                format!("P{who} sacrificed creature {}", name(*card_id))
            }
            E::PermanentSacrificed { card_id, who } => {
                format!("P{who} sacrificed permanent {}", name(*card_id))
            }
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
            E::Explored { card_id, .. } => format!("{} explored", name(*card_id)),
            E::BecameMonstrous { card_id } => format!("{} became monstrous", name(*card_id)),
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
            E::AttackerWentUnblocked { attacker } => {
                format!("{} attacks and is unblocked", name(*attacker))
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
            E::VehicleCrewed { vehicle } => format!("{} crewed", name(*vehicle)),
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
            E::CardCycled { player, card_id } => {
                format!("P{player} cycled {}", name(*card_id))
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
