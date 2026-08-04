use serde::{Deserialize, Serialize};

use crate::card::{CardDefinition, CardId, CardInstance};
use crate::cow::CowBox;
use crate::mana::ManaPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub usize);

/// CR 402.2 — the default maximum hand size (seven cards). A player's
/// `max_hand_size` starts here; effects can raise/lower it or remove it.
pub const DEFAULT_MAX_HAND_SIZE: usize = 7;

/// Serde default for `Player.max_hand_size` — the normal seven-card cap.
fn default_max_hand_size() -> Option<usize> {
    Some(DEFAULT_MAX_HAND_SIZE)
}

fn default_starting_life() -> i32 {
    20
}

/// CR 114 — an emblem owned by a player. Has no characteristics other
/// than the triggered abilities it grants its owner, and sits in the
/// command zone for the rest of the game (emblems never leave). Created
/// by planeswalker ultimates via `Effect::CreateEmblem`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emblem {
    /// Source name, for display (e.g. "Professor Dellian Fel").
    pub name: String,
    /// Abilities the emblem grants its owner.
    pub triggered: Vec<crate::effect::TriggeredAbility>,
    /// Static abilities the emblem grants its owner (anthem-style emblems —
    /// Vivien Reid's −8 "Creatures you control get +2/+2 …"). Synthesized into
    /// continuous effects in `gather_continuous_effects`. Defaults to empty for
    /// snapshot back-compat.
    #[serde(default)]
    pub statics: Vec<crate::card::StaticAbility>,
}

/// CR 702.50 — a resolved Epic spell, snapshotted for the per-upkeep copy.
/// The copy re-resolves the named card's effect with these cast choices
/// (targets may be re-chosen per 702.50a; the AutoDecider keeps them).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpicSpell {
    pub name: String,
    pub target: Option<crate::game::Target>,
    pub additional_targets: Vec<crate::game::Target>,
    pub mode: Option<usize>,
    pub x_value: u32,
}

/// Authoritative cause of a player's elimination, recorded on the `Player`
/// when they lose. Mirrors the server's presentation-side `LossReason` but
/// lives in the engine so it can be stamped at the exact SBA/effect that
/// eliminated the seat (CR 104.3 / 704.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossCause {
    /// Life total 0 or less (CR 104.3a / 704.5a).
    LifeDepleted,
    /// Ten or more poison counters (CR 104.3c / 704.5c).
    Poison,
    /// Tried to draw from an empty library (CR 104.3c / 120.3).
    Decked,
    /// 21+ combat damage from a single commander (CR 903.10a).
    CommanderDamage,
    /// The player conceded (CR 104.3a — a player can concede at any time).
    Conceded,
    /// A "you lose the game" effect or other cause.
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub life: i32,
    /// The life total this player began the game with (CR 103.4). Set by
    /// `apply_format`; drives "N more/less than your starting life total"
    /// thresholds (Righteous Valkyrie, Speaker of the Heavens). `#[serde(default
    /// = "default_starting_life")]` for snapshot back-compat.
    #[serde(default = "default_starting_life")]
    pub starting_life: i32,
    pub mana_pool: ManaPool,
    /// Top of library is `library[0]`.
    ///
    /// The card zones are [`CowBox`]-wrapped so a `GameState` clone
    /// (affordance probes, the `perform_action` checkpoint) shares them
    /// until written — see `crate::cow`.
    pub library: CowBox<Vec<CardInstance>>,
    pub hand: CowBox<Vec<CardInstance>>,
    pub graveyard: CowBox<Vec<CardInstance>>,
    /// The command zone — Commander commanders, Conspiracies, etc.
    /// (Phase I.) Cards arrive here either at game start (initial
    /// commander seating via `seat_commanders`) or via a zone-change
    /// replacement effect when they would otherwise leave the
    /// battlefield (CR 903.9b).
    ///
    /// `#[serde(default)]` so snapshots written before the field
    /// existed deserialize cleanly as empty.
    #[serde(default)]
    pub command: CowBox<Vec<CardInstance>>,
    /// CR 609.4b — "you may spend mana as though it were mana of any type"
    /// for the rest of this turn (North Star). Cleared at cleanup.
    #[serde(default)]
    pub may_spend_any_color_this_turn: bool,
    /// CR 407 — cards this player owns in the ante zone. Only ever non-empty
    /// while `GameState.playing_for_ante`; the winner takes all of it
    /// (CR 407.2). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub ante: CowBox<Vec<CardInstance>>,
    /// CR 904.3 — the archenemy's face-down scheme deck. Like the command
    /// zone it never empties into another zone: a scheme is set in motion off
    /// the top (CR 904.9) into `command`, and abandoned back to the bottom
    /// here (CR 904.10). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub scheme_deck: CowBox<Vec<CardInstance>>,
    /// CR 406 / 701.45 — the Lessons "sideboard" (cards owned from outside
    /// the game). A Learn ability may reveal a Lesson card here and put it
    /// into hand. Populated by deck construction; empty by default (in
    /// which case Learn falls back to the legacy `Draw 1` approximation).
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub sideboard: CowBox<Vec<CardInstance>>,
    /// CardIds of cards this player has designated as Commanders
    /// (Phase J). Populated by `GameState::seat_commanders`. Read by
    /// the Phase M 21-commander-damage SBA via
    /// `GameState::is_commander`. Note this is *which cards are
    /// commanders for this player*, independent of the command zone
    /// — a commander on the battlefield (or any other zone) is
    /// still a commander, so the entry survives zone changes.
    #[serde(default)]
    pub commanders: Vec<CardId>,
    /// How many lands this player has played on their current turn.
    pub lands_played_this_turn: u32,
    /// Extra land plays granted this turn (Explore, Oracle of Mul Daya,
    /// Dryad of the Ilysian Grove, etc.). Defaults to 0. The player can
    /// play `1 + extra_land_plays` lands per turn total.
    #[serde(default)]
    pub extra_land_plays: u32,
    /// The Chain Veil — extra loyalty activations available this turn on top
    /// of the CR 606.3 one-per-planeswalker limit. Cleared each turn.
    #[serde(default)]
    pub extra_loyalty_activations: u32,
    /// Whether this player has activated a loyalty ability this turn (The
    /// Chain Veil's end-step "if you didn't …" check). Cleared each turn.
    #[serde(default)]
    pub activated_loyalty_this_turn: bool,
    /// How many spells this player has cast this turn. Reset on
    /// `TurnStarted`. Powers Damping Sphere's "second-and-onward spells
    /// cost {1} more" static.
    pub spells_cast_this_turn: u32,
    /// Sorcery spells cast this turn — Backdraft's "a player who cast one or
    /// more sorcery spells this turn". Reset at Cleanup.
    #[serde(default)]
    pub sorceries_cast_this_turn: u32,
    /// Per-name lifetime cast counter — "you've cast another spell named X
    /// this game" (Approach of the Second Sun). Bumped in `finalize_cast`
    /// with the cast card's printed name; never reset. Defaults empty for
    /// snapshot back-compat.
    #[serde(default)]
    pub spells_cast_by_name_this_game: std::collections::HashMap<String, u32>,
    /// Like `spells_cast_this_turn` but reset for every player at each
    /// turn's Cleanup (not just the player's own untap) — the CR-correct
    /// scope for Rule of Law's "each player can't cast more than one spell
    /// each turn" (CR 611.2). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub spells_cast_this_game_turn: u32,
    /// Has an opponent of this seat cast a spell since this seat's last turn
    /// ended? Set on every other seat when a spell is cast; cleared for the
    /// active player at cleanup. Backs
    /// `Predicate::OpponentCastSpellSinceYourTurn`.
    #[serde(default)]
    pub opponent_cast_spell_since_your_turn: bool,
    /// Like `spells_cast_this_game_turn` but counting only noncreature spells
    /// (Deafening Silence's "each player can't cast more than one noncreature
    /// spell each turn"). Reset for every player at Cleanup. `#[serde(default)]`.
    #[serde(default)]
    pub noncreature_spells_cast_this_game_turn: u32,
    /// Like `spells_cast_this_game_turn` but counting only nonartifact spells
    /// (Ethersworn Canonist). Reset for every player at Cleanup. `#[serde(default)]`.
    #[serde(default)]
    pub nonartifact_spells_cast_this_game_turn: u32,
    /// Total life gained by this player this turn (sum of every
    /// `Effect::GainLife` and `Effect::Drain`-to-this-player resolution).
    /// Reset to 0 in `do_untap`. Powers Strixhaven's **Infusion** rider —
    /// "If you gained life this turn, …" — and any future "you've gained
    /// life this turn" payoffs without needing a custom event log scan.
    /// Default-deserializes to 0 for snapshots predating the field.
    #[serde(default)]
    pub life_gained_this_turn: u32,
    /// True once a life-gain trigger batch for this player has finished
    /// dispatching on the current turn. Powers "whenever you gain life for
    /// the first time each turn" (Leech Collector): the trigger's filter
    /// reads the flag *before* it flips (set at the end of the dispatch
    /// batch), so only the turn's first gain batch qualifies — a gain that
    /// happened before the listener arrived still disqualifies later gains.
    /// Reset for every player at each untap (the printed "each turn" keys
    /// on the turn boundary, not the owner's turn). Defaults to false for
    /// snapshot back-compat.
    #[serde(default)]
    pub gained_life_earlier_this_turn: bool,
    /// Number of cards this player has drawn on the current turn. Reset
    /// to 0 in `do_untap`. Powers Strixhaven's Quandrix scaling — e.g.
    /// Fractal Anomaly creates a 0/0 with X +1/+1 counters where X is
    /// "cards drawn this turn." Surfaced through `PlayerView` so client
    /// UIs can preview the scaling. Defaults to 0 for snapshot
    /// backwards-compatibility.
    #[serde(default)]
    pub cards_drawn_this_turn: u32,
    /// Cards drawn during the current step — reset for every player on each
    /// step change. Powers "except the first card they draw in their draw
    /// step" trigger gates (Orcish Bowmasters, CR 603.3).
    #[serde(default)]
    pub cards_drawn_this_step: u32,
    /// Number of times a card has left this player's graveyard on the
    /// current turn. Reset to 0 in `do_untap`. Powers Strixhaven Lorehold
    /// "if a card left your graveyard this turn" payoffs (Living History,
    /// Primary Research's end-step draw rider, Wilt in the Heat's cost
    /// reduction). Backed by the `CardLeftGraveyard` event emission in
    /// `move_card_to`. Defaults to 0 for snapshot back-compat.
    #[serde(default)]
    pub cards_left_graveyard_this_turn: u32,
    /// Number of creatures controlled by this player that died this turn.
    /// Reset to 0 in `do_untap`. Powers Witherbloom "if a creature died
    /// under your control this turn, …" end-step payoffs (Essenceknit
    /// Scholar). Bumped from `apply_state_based_actions`'s SBA dies
    /// handler keyed off the dying creature's controller. Defaults to 0
    /// for snapshot back-compat.
    #[serde(default)]
    pub creatures_died_this_turn: u32,
    /// Zubera that died under this player's control this turn. Reset in
    /// `do_untap`. Powers the Champions-of-Kamigawa Zubera cycle's "for each
    /// Zubera that died this turn" death triggers. `#[serde(default)]`.
    #[serde(default)]
    pub zuberas_died_this_turn: u32,
    /// Creatures that entered the battlefield under this player's control
    /// this turn (stamped at the shared ETB funnels). Rotated into
    /// `creatures_entered_last_turn` at end of turn — Ephara, God of the
    /// Polis reads the previous turn's entries at each upkeep.
    #[serde(default)]
    pub creatures_entered_this_turn: Vec<crate::card::CardId>,
    /// Last turn's `creatures_entered_this_turn` (see above).
    #[serde(default)]
    pub creatures_entered_last_turn: Vec<crate::card::CardId>,
    /// CR 603 — count of artifacts that entered under this player's control
    /// this turn. Drives "if an artifact entered the battlefield under your
    /// control this turn" intervening-`if` gates (Akal Pakal). Reset at the
    /// active player's turn boundary.
    #[serde(default)]
    pub artifacts_entered_this_turn: u32,
    /// Count of planeswalkers that entered under this player's control this
    /// turn — the `Predicate::PlaneswalkerEnteredThisTurn` gate (Oath of
    /// Chandra). Reset at the active player's turn boundary.
    #[serde(default)]
    pub planeswalkers_entered_this_turn: u32,
    /// CR 702.176-era **Celebration** (WOE) — count of nonland permanents that
    /// entered under this player's control this turn. Gates "if two or more
    /// nonland permanents entered … this turn" (`Predicate::CelebrationActive`
    /// — Armory Mice, Belligerent of the Ball). Reset at cleanup.
    #[serde(default)]
    pub nonland_permanents_entered_this_turn: u32,
    /// DFT — count of Mounts and/or Vehicles that entered under this player's
    /// control this turn (`Value::MountsVehiclesEnteredThisTurn` — Cloudspire
    /// Coordinator's token count). Reset at the active player's turn boundary.
    #[serde(default)]
    pub mounts_vehicles_entered_this_turn: u32,
    /// Multicolored spells this player has cast this turn (Zenith
    /// Chronicler's "first multicolored spell each turn"). Reset at cleanup.
    #[serde(default)]
    pub multicolored_spells_cast_this_turn: u32,
    /// Churning Reservoir's gate: an oil counter left one of this player's
    /// permanents this turn, or an oil-countered permanent of theirs hit a
    /// graveyard. Reset at cleanup.
    #[serde(default)]
    pub oil_activity_this_turn: bool,
    /// Channel — until end of turn this player may pay 1 life per missing
    /// generic/colorless mana ("pay 1 life: add {C}"). The payment funnel
    /// converts life into the shortfall on demand. Reset at cleanup.
    #[serde(default)]
    pub channel_life_for_mana: bool,
    /// CR 309 — the dungeon this player is currently in: (name, room index).
    /// `None` between dungeons; venturing with `None` enters a new dungeon.
    #[serde(default)]
    pub dungeon: Option<(String, u8)>,
    /// CR 701.49d — dungeons this player has completed this game.
    #[serde(default)]
    pub dungeons_completed: u32,
    /// Number of times an "Nth time this turn" landfall ability this player
    /// controls has resolved this turn (Omnath, Locus of Creation). Bumped by
    /// `Effect::NthResolutionThisTurn`, reset at the player's `do_untap`.
    /// Defaults to 0 for snapshot back-compat.
    #[serde(default)]
    pub escalating_resolutions_this_turn: u32,
    /// CR 603.7e — pending "your next creature spell this turn enters with N
    /// extra counters of this kind" riders (the FIN "Summon" saga chapters —
    /// Fenrir II "Heavenward Howl", Brynhildr). Each entry is drained onto the
    /// *next* creature spell this player casts (all pending riders stack onto
    /// the same creature); the list clears at cleanup so an unused rider
    /// expires with the turn. `#[serde(default)]`.
    #[serde(default)]
    pub pending_creature_etb_counters: Vec<(crate::card::CounterType, u32)>,
    /// CR 603.7e — pending "your next creature spell this turn enters with these
    /// keywords" riders (Summon: Brynhildr's "Gestalt Mode" haste). Applied to
    /// the next creature spell's permanent as it enters and cleared at cleanup.
    /// `#[serde(default)]`.
    #[serde(default)]
    pub pending_creature_etb_keywords: Vec<crate::card::Keyword>,
    /// CR 702.139 — true if a permanent left the battlefield under this
    /// player's control so far this turn (Revolt). Set from the battlefield-
    /// removal funnels keyed off the leaving permanent's controller; reset at
    /// the player's `do_untap`. Defaults to false for snapshot back-compat.
    #[serde(default)]
    pub permanent_left_battlefield_this_turn: bool,
    /// True if this player has been dealt damage so far this turn. Set in
    /// `deal_damage_to_from`'s player branch (combat or non-combat, incl.
    /// infect/poison), reset for *all* players at the active player's
    /// `do_untap` so it reflects "damaged since this turn began" — the
    /// Bloodthirst (CR 702.54) window. Defaults to false for snapshot
    /// back-compat.
    #[serde(default)]
    pub was_dealt_damage_this_turn: bool,
    /// How much damage this player has been dealt this turn, for
    /// "Bloodthirst X"-style riders that scale with the amount (Petrified
    /// Wood-Kin). Cleared with `was_dealt_damage_this_turn` at turn start.
    #[serde(default)]
    pub damage_taken_this_turn: u32,
    /// True if this player has lost life this turn (damage or direct life
    /// loss). Set in `adjust_life` on a negative delta, reset at the active
    /// player's `do_untap`. Powers Spectacle (CR 702.111). Defaults to false
    /// for snapshot back-compat.
    #[serde(default)]
    pub lost_life_this_turn: bool,
    /// Permanent types already cast from the graveyard this turn under a
    /// Muldrotha-style permission (one of each per turn). Reset at untap.
    #[serde(default)]
    pub graveyard_cast_types_this_turn: Vec<crate::card::CardType>,
    /// Total life lost this turn (CR 119.3). Bumped alongside
    /// `lost_life_this_turn`, reset with it. Powers `Value::LifeLostThisTurn`
    /// (Spinerock Knoll's "an opponent lost 7 or more life" gate).
    #[serde(default)]
    pub life_lost_this_turn: u32,
    /// Card ids of creatures that have dealt damage to this player so far
    /// this turn (combat or non-combat). Reset for all players at the
    /// active player's `do_untap`. Powers "destroy target creature that
    /// dealt damage to you this turn" (Spear of Heliod, CR uses
    /// `SelectionRequirement::DealtDamageToControllerThisTurn`). Defaults
    /// empty for snapshot back-compat.
    #[serde(default)]
    pub creatures_that_damaged_me_this_turn: Vec<crate::card::CardId>,
    /// Lands that entered the battlefield under this player's control this
    /// turn, played or otherwise. Cleared at the turn boundary; read by
    /// `Predicate::LandsEnteredThisTurnAtLeast` (Lavaball Trap).
    #[serde(default)]
    pub lands_entered_this_turn: u32,
    /// A creature spell this player cast this turn was countered by a spell or
    /// ability an opponent controlled (Summoning Trap). Stamped at the counter
    /// funnel, cleared at the turn boundary.
    #[serde(default)]
    pub creature_spell_countered_by_opponent_this_turn: bool,
    /// A noncreature permanent this player controlled was destroyed this turn
    /// by a spell or ability an opponent controlled (Cobra Trap). Stamped in
    /// the destroy funnel, cleared at the turn boundary.
    #[serde(default)]
    pub noncreature_destroyed_by_opponent_this_turn: bool,
    /// Creature types among this player's creatures that dealt combat damage
    /// to a player this turn (CR 702.76 Prowl). Stamped at the combat-damage
    /// funnels, cleared at the turn boundary. `#[serde(default)]` for
    /// snapshot back-compat.
    #[serde(default)]
    pub prowl_types_this_turn: Vec<crate::card::CreatureType>,
    /// A Changeling of this player's dealt combat damage to a player this
    /// turn — it counts as every creature type for the prowl window.
    #[serde(default)]
    pub prowl_any_type_this_turn: bool,
    /// True once this player has declared an attacker this turn (Raid, CR
    /// 702.108 ability word). Set in `declare_attackers`, reset at the turn
    /// boundary in `do_untap`. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub attacked_this_turn: bool,
    /// Number of creatures this player declared as attackers this turn.
    /// Powers `Value::CreaturesAttackedWithThisTurn` (Windbrisk Heights).
    #[serde(default)]
    pub creatures_attacked_this_turn: u32,
    /// True once a creature this player controlled dealt combat damage to a
    /// player this turn (CR 702.179 — Freerunning). Set in
    /// `fire_combat_damage_to_player_triggers`, reset at the turn boundary.
    #[serde(default)]
    pub dealt_combat_damage_to_player_this_turn: bool,
    /// CR 614.5 — "sources you control deal double damage this turn" (Quest
    /// for Pure Flame). Read in `scale_damage_to`, reset at the turn boundary.
    #[serde(default)]
    pub double_your_source_damage_this_turn: bool,
    /// CR 305.1 — "target player can't play lands this turn" (Turf Wound).
    /// Consulted by `can_player_play_land`, reset at the turn boundary.
    #[serde(default)]
    pub cant_play_lands_this_turn: bool,
    /// "You can't cast [filter] spells this turn" (Cease-Fire). Cleared at the
    /// turn boundary alongside `cant_play_lands_this_turn`.
    #[serde(default)]
    pub cant_cast_matching_this_turn: Vec<crate::card::SelectionRequirement>,
    /// "The next [filter] spell you cast this turn can't be countered"
    /// (Insist, Overmaster). Consumed by the next matching cast; cleared at
    /// end of turn.
    #[serde(default)]
    pub next_spell_uncounterable: Vec<crate::card::SelectionRequirement>,
    /// CR 700.13 — this player has committed a crime this turn (cast a spell or
    /// activated an ability targeting an opponent / their stuff). Set when a
    /// `CommittedCrime` event fires, reset at the turn boundary. Powers
    /// "if you've committed a crime this turn" gates (Nimble Brigand).
    #[serde(default)]
    pub committed_crime_this_turn: bool,
    /// CR 708 — a permanent entered the battlefield face down under this
    /// player's control, or they turned a permanent face up, this turn. Powers
    /// Oblivious Bookworm's "unless … entered face down / you turned a permanent
    /// face up this turn" discard-skip. Reset at the turn boundary.
    #[serde(default)]
    pub face_down_activity_this_turn: bool,
    /// Revel in Silence: this player can't cast spells or activate loyalty
    /// abilities for the rest of the turn. Reset at the turn boundary.
    #[serde(default)]
    pub silenced_this_turn: bool,
    /// True once this player has cast a spell for its Warp cost this turn (EOE).
    /// Reset at the turn boundary; the right half of `Predicate::VoidActive`.
    #[serde(default)]
    pub warped_spell_this_turn: bool,
    /// True once this player has searched their own library this turn
    /// (CR 701.19). Reset for every player at the turn boundary. Powers
    /// `Predicate::SearchedLibraryThisTurn` (Archive Trap's free alt cost).
    #[serde(default)]
    pub searched_library_this_turn: bool,
    /// CR 702.18 — "You gain shroud until end of turn" (Gilded Light). While
    /// set, no player may target this one. Cleared at cleanup.
    #[serde(default)]
    pub shroud_this_turn: bool,
    /// "You gain protection from everything until your next turn" (The One
    /// Ring). While set: this player can't be targeted and all damage to
    /// them is prevented. Cleared when their turn begins.
    #[serde(default)]
    pub protected_from_everything: bool,
    /// CR 401.6 — turn-scoped "you may look at the top card of your library
    /// any time, and you may play lands and cast spells from the top of your
    /// library" grant (The Belligerent, Bonehoard's Dracosaur-style player
    /// permissions). Unlike `StaticEffect::PlayFromLibraryTop` this is a
    /// player flag set at resolution and cleared at end of turn. Read by
    /// `library_top_playable` and the top-revealed view.
    #[serde(default)]
    pub play_from_top_this_turn: bool,
    /// CR 603.3d-adjacent — set once this player casts/plays a card from the
    /// top of their library under a `PlayFromLibraryTopOncePerTurn` grant
    /// (Johann, Apprentice Sorcerer). Cleared at end of turn. Read by
    /// `library_top_playable` to enforce the once-per-turn cap.
    #[serde(default)]
    pub cast_from_library_top_this_turn: bool,
    /// Number of cards this player has caused to be put into exile on
    /// the current turn. Reset to 0 in `do_untap`. Powers Strixhaven
    /// "if one or more cards were put into exile this turn" payoffs
    /// (Ennis the Debate Moderator). Bumped from `place_card_in_dest`'s
    /// exile branch and the battlefield-to-exile path in
    /// `Effect::Exile`. Defaults to 0 for snapshot back-compat.
    #[serde(default)]
    pub cards_exiled_this_turn: u32,
    /// Cards put into THIS player's graveyard from anywhere this turn
    /// (CR 700.4-adjacent tally). Bumped at the shared graveyard-placement
    /// funnels, reset for every player at the turn boundary. Powers
    /// `Predicate::OpponentCardsToGraveyardThisTurnAtLeast` (Ravenous
    /// Trap's free alt cost).
    #[serde(default)]
    pub cards_to_graveyard_this_turn: u32,
    /// CR 700.11 — true if a *permanent* card was put into this player's
    /// graveyard from anywhere this turn ("you descended this turn"). Set in
    /// `send_to_graveyard`, reset at untap. Gates "if you descended this turn"
    /// riders (Deep Goblin Skulltaker, Child of the Volcano).
    #[serde(default)]
    pub descended_this_turn: bool,
    /// CR 700.11 — how many times this player descended this turn (a permanent
    /// card hitting the graveyard). Reset at untap. Powers "X = the number of
    /// times you descended this turn" (The Mycotyrant).
    #[serde(default)]
    pub descend_count_this_turn: u32,
    /// Number of instant or sorcery spells this player has cast on the
    /// current turn. Reset to 0 in `do_untap`. Refines
    /// `spells_cast_this_turn` (which counts every spell type) so cards
    /// like Potioner's Trove can gate "activate only if you've cast an
    /// instant or sorcery spell this turn" precisely. Bumped in
    /// `finalize_cast` whenever the resolving spell card carries the
    /// Instant or Sorcery card type. Defaults to 0 for snapshot
    /// back-compat.
    #[serde(default)]
    pub instants_or_sorceries_cast_this_turn: u32,
    /// Count of spells this player cast **from their hand** this turn (CR
    /// 601). Unlike `spells_cast_this_turn`, casts from exile (Plot, impulse),
    /// graveyard (flashback/escape/disturb/retrace/aftermath), or the command
    /// zone don't bump it. Gates "if you haven't cast a spell from your hand
    /// this turn" payoffs (Prairie Dog, Emergent Haunting). Reset each turn.
    #[serde(default)]
    pub spells_cast_from_hand_this_turn: u32,
    /// Transient Hardened-Scales bonus granted "until end of turn" (Prairie
    /// Dog's {4}{W}). Adds to `plus_counter_adders_for`; cleared at cleanup.
    #[serde(default)]
    pub extra_plus_one_counters_this_turn: u32,
    /// Combine Guildmage — "this turn, each creature you control enters with an
    /// additional +1/+1 counter." Read at the ETB-counter site; cleared at
    /// cleanup. Distinct from `extra_plus_one_counters_this_turn` (which
    /// amplifies counters *placed*, Hardened-Scales style).
    #[serde(default)]
    pub extra_etb_p1p1_counters_this_turn: u32,
    /// One-shot "next instant/sorcery you cast this turn costs {N} less"
    /// discounts (Thundertrap Trainer). Each entry is `(amount, granted_at)`
    /// where `granted_at` is `instants_or_sorceries_cast_this_turn` at grant
    /// time; the discount applies only while the tally still equals it.
    /// Cleared each turn alongside the tally.
    #[serde(default)]
    pub pending_is_discounts: Vec<(u32, u32)>,
    /// Like `pending_is_discounts` but for *any* next spell this turn
    /// (Mutated Cultist): `(amount, spells_cast_this_turn at grant)`.
    #[serde(default)]
    pub pending_spell_discounts: Vec<(u32, u32)>,
    /// Cards this player discarded this turn (Hollow One's cost reduction).
    /// Bumped in `discard_card`; reset in `do_untap`.
    #[serde(default)]
    pub cards_discarded_this_turn: u32,
    /// CR 702.187 — ids of cards this player discarded this turn, so Mayhem
    /// can offer a graveyard cast only for cards actually discarded this turn.
    /// Populated in `discard_card`; cleared in `do_untap`.
    #[serde(default)]
    pub discarded_this_turn: std::collections::HashSet<crate::card::CardId>,
    /// Number of permanents this player has sacrificed so far this turn.
    /// Bumped in `dispatch_triggers_for_events` per `PermanentSacrificed`
    /// event; reset in `do_untap`. Powers "if you sacrificed a permanent
    /// this turn" payoffs (Sawblade Skinripper).
    #[serde(default)]
    pub permanents_sacrificed_this_turn: u32,
    /// Number of *artifacts* this player has sacrificed so far this turn (a
    /// subset of `permanents_sacrificed_this_turn`). Bumped in
    /// `dispatch_triggers_for_events` per `PermanentSacrificed` event whose
    /// subject snapshot is an artifact; reset in `do_untap`. Powers "if you've
    /// sacrificed an artifact this turn" riders (Suspicious Detonation,
    /// Furtive Courier).
    #[serde(default)]
    pub artifacts_sacrificed_this_turn: u32,
    /// "[Filter] spells you cast this turn cost {N} less" grants
    /// (`Effect::SpellsCostLessThisTurn` — Urza, Planeswalker's +2).
    /// Each entry applies to every matching spell for the rest of the
    /// turn; cleared in `finish_cleanup` (CR 514.2).
    #[serde(default)]
    pub turn_spell_discounts: Vec<(crate::card::SelectionRequirement, u32)>,
    /// Number of creature spells this player has cast on the current
    /// turn. Reset to 0 in `do_untap`. Powers creature-cast magecraft
    /// payoffs ("if you've cast a creature spell this turn, …") and
    /// future creature-spell-matters cards. Defaults to 0 for snapshot
    /// back-compat.
    #[serde(default)]
    pub creatures_cast_this_turn: u32,
    /// Pending "first spell costs {1} more" taxes against this player.
    /// Each spell cast consumes one charge, charging the caster {1} extra
    /// generic in `extra_cost_for_spell`. Set by Chancellor of the Annex's
    /// opening-hand reveal (one charge per Annex revealed by an opponent).
    pub first_spell_tax_charges: u32,
    /// True if this player can cast sorceries at instant speed until their
    /// next turn. Set by Teferi, Time Raveler's +1; cleared in `do_untap`
    /// when this player's own turn begins.
    pub sorceries_as_flash: bool,
    /// Poison counters (player loses at 10).
    pub poison_counters: u32,
    /// CR 614 — Melira, the Living Cure's cap already replaced a poison
    /// placement this turn; further poison this turn is dropped. Reset at
    /// the turn boundary. Default false for snapshot back-compat.
    #[serde(default)]
    pub poison_capped_this_turn: bool,
    /// CR 701.54 — how many times the Ring has tempted this player (0–4; the
    /// printed abilities are "two/three/four or more", so we cap the stored
    /// value at 4). Each step up activates another of The Ring's emblem
    /// abilities. Default 0 for snapshot back-compat.
    #[serde(default)]
    pub ring_temptations: u32,
    /// CR 701.54a/b — this player's designated Ring-bearer, if any. A
    /// non-copiable permanent designation; cleared lazily (the
    /// `effective_ring_bearer` helper re-checks battlefield presence and
    /// control). Default `None`.
    #[serde(default)]
    pub ring_bearer: Option<crate::card::CardId>,
    /// CR 702.179 — this player's speed (0–4). Starts at 0; a "Start your
    /// engines!" object sets it to 1, and it increases by 1 (once per the
    /// player's own turn, capped at 4) the first time an opponent loses life
    /// during their turn. "Max speed —" abilities are active at 4. Default 0
    /// for snapshot back-compat.
    #[serde(default)]
    pub speed: u32,
    /// CR 702.179 — set once this player's speed has already increased on the
    /// current turn (the "increases once on each of your turns" clause). Reset
    /// at the start of each of this player's turns.
    #[serde(default)]
    pub speed_increased_this_turn: bool,
    /// CR 122 / 107.16 — energy counters ({E}) this player has. A
    /// generic resource pool added by `Effect::AddEnergy` and spent by
    /// `Effect::PayEnergy`. Defaults to 0 for snapshot back-compat.
    #[serde(default)]
    pub energy: u32,
    /// Energy spent (paid or lost) so far this turn — the tally behind
    /// "activate only if you've paid or lost N+ {E} this turn" gates (Izzet
    /// Generatorium). Reset each untap; routed through `spend_energy`.
    #[serde(default)]
    pub energy_spent_this_turn: u32,
    /// Experience counters (CR 122 / 720-era Commander mechanic). A per-player
    /// resource that only accumulates; payoffs read the count (Mizzix's cost
    /// reduction, Ezuri's +1/+1 distribution). Added by
    /// `Effect::AddExperience`. Default 0 for snapshot back-compat.
    #[serde(default)]
    pub experience: u32,
    /// CR 122.1i / 728 — rad counters on this player. At the start of
    /// their precombat main phase they mill that many cards; for each
    /// nonland milled, they lose 1 life and shed a rad counter (handled
    /// as a turn-based action in `do_rad_counters`). Default 0.
    #[serde(default)]
    pub rad_counters: u32,
    /// CR 700.6 / 702.131 — true once this player has the city's blessing.
    /// Granted by an Ascend ability/permanent while they control ten or more
    /// permanents; once obtained it lasts for the rest of the game. Default
    /// false for snapshot back-compat.
    #[serde(default)]
    pub city_blessing: bool,
    /// CR 402.2 — this player's maximum hand size. `Some(n)` caps the hand at
    /// `n` cards (normally `Some(7)`); `None` means no maximum (Wisdom of
    /// Ages, Reliquary Tower-style effects). The cleanup-step CR 514.1
    /// enforcement in `do_cleanup` reads this: `None` skips the discard-down
    /// step, `Some(n)` discards down to `n`. Set by
    /// `Effect::SetNoMaxHandSize` (→ `None`) and `Effect::SetMaxHandSize`
    /// (→ `Some(n)`, e.g. Null Profusion's "maximum hand size is zero").
    /// `#[serde(default)]` yields the normal `Some(7)` for snapshot
    /// back-compat.
    #[serde(default = "default_max_hand_size")]
    pub max_hand_size: Option<usize>,
    /// True once this player has lost the game (life ≤ 0, poison ≥ 10, or
    /// drew from an empty library). Eliminated players are skipped by turn
    /// and priority rotation; the game ends when ≤ 1 player remains.
    pub eliminated: bool,
    /// CR 801.2a — this seat's own range of influence, overriding the table
    /// default (`GameState.range_of_influence`). Emperor games give the
    /// emperor 2 and each general 1 (CR 809.3a).
    #[serde(default)]
    pub range_of_influence: Option<u32>,
    /// The last card this player drew this turn (CR 121 — "the last card you
    /// drew this turn"). Cleared as their turn begins. Sindbad, Jandor's Ring.
    #[serde(default)]
    pub last_drawn_card: Option<crate::card::CardId>,
    /// CR 809.2 — this seat is their team's emperor. Their team loses when
    /// they lose (CR 809.5b).
    #[serde(default)]
    pub is_emperor: bool,
    /// The authoritative reason this player was eliminated, recorded at the
    /// moment `eliminated` flips to `true`. Lets consumers (the server's
    /// win-kind stats) read the true cause instead of guessing from the
    /// final board state — a "you lose the game" effect on an empty-library
    /// seat is `Concession`/`Other`, not a deck-out. `None` while still in
    /// the game. Defaults to `None` via `#[serde(default)]`.
    #[serde(default)]
    pub loss_cause: Option<LossCause>,
    /// CR 104.3d — Angel's Grace: this player can't lose the game this turn
    /// (and their opponents can't win it). Cleared at the turn boundary.
    #[serde(default)]
    pub cant_lose_this_turn: bool,
    /// Angel's Grace's second rider — damage that would reduce this player's
    /// life below 1 reduces it to 1 instead, this turn. Cleared at the turn
    /// boundary.
    #[serde(default)]
    pub damage_floor_this_turn: bool,
    /// Forbidding Spirit — a temporary Propaganda tax: creatures can't attack
    /// this player or their planeswalkers unless the attacker's controller
    /// pays {N} for each. Set on ETB, cleared at this player's own untap step
    /// ("until your next turn"). Summed alongside `AttackTaxToController`
    /// statics in `declare_attackers`.
    #[serde(default)]
    pub attack_tax_until_your_turn: u32,
    /// Number of upcoming turns this player must skip. Read by the
    /// turn-advance logic in `do_cleanup` — when the engine would hand
    /// the next turn to this player, the counter is decremented and the
    /// turn is bypassed (advancing to the player after). Set by
    /// `Effect::SkipTurns` (Ral Zarek, Guest Lecturer's -7 ult). Defaults
    /// to 0 for snapshot back-compat.
    #[serde(default)]
    pub skip_turns: u32,
    /// Names of the spells this player has cast this turn (Grim Reminder).
    /// Cleared at untap.
    #[serde(default)]
    pub spell_names_cast_this_turn: Vec<String>,
    /// CR 502.3 — number of this player's upcoming untap steps to skip
    /// (Yosei, the Morning Star; Frost Titan-style locks). Decremented when
    /// their untap step would run; while > 0 their permanents don't untap.
    #[serde(default)]
    pub skip_next_untap_step: u32,
    /// CR 504 — charges of "skip your next draw step" (Fatigue). Consumed one
    /// per draw step.
    #[serde(default)]
    pub skip_next_draw_step: u32,
    /// CR 614 — queued one-shot draw replacements: "the next time you would
    /// draw a card this turn, [effect] instead" (the Onslaught Words cycle).
    /// Each entry is `(source, effect)`; `draw_one` pops the front instead of
    /// drawing. Cleared at the turn boundary.
    #[serde(default)]
    pub next_draw_replacements: Vec<(crate::card::CardId, crate::effect::Effect)>,
    /// CR 506 — number of this player's upcoming combat phases to skip
    /// (Stonehorn Dignitary). Consumed when their turn reaches Begin Combat,
    /// jumping straight to the postcombat main. Defaults to 0 for snapshot
    /// back-compat.
    #[serde(default)]
    pub skip_next_combat: u32,
    /// Number of this player's upcoming untap steps in which the **lands** they
    /// control don't untap (Bontu's Last Reckoning). Decremented when their
    /// untap step runs; non-land permanents untap normally. `#[serde(default)]`.
    #[serde(default)]
    pub lands_dont_untap_next_untap: u32,
    /// Deep Water — while set, every land this player taps for mana produces
    /// this colour instead of its own. Cleared at end of turn.
    #[serde(default)]
    pub lands_produce_color_this_turn: Option<crate::mana::Color>,
    /// CR 502.3 sibling of `lands_dont_untap_next_untap` for creatures
    /// (Blinding Beam's "creatures don't untap during target player's next
    /// untap step"). Decremented and applied in `do_untap`.
    #[serde(default)]
    pub creatures_dont_untap_next_untap: u32,
    /// CR 702.189 — red mana added by Firebending this combat that survives
    /// step/phase mana emptying ("you don't lose this mana as steps and phases
    /// end"). Re-seeded into the pool by `empty_mana_pools`; cleared at end of
    /// combat. `#[serde(default)]`.
    #[serde(default)]
    pub firebending_kept_red: u32,
    /// CR 500.4 exception — mana added by an effect that says "you don't lose
    /// this mana as steps and phases end" (Savage Ventmaw's attack trigger).
    /// Re-seeded into the pool by `empty_mana_pools` on every step/phase empty
    /// and cleared at cleanup, so the mana survives the turn but not past it.
    /// `#[serde(default)]`.
    #[serde(default)]
    pub kept_mana_this_turn: ManaPool,
    /// CR 500.7 — extra turns this player will take. When `advance_turn`
    /// would pass the turn, an active player with `extra_turns > 0`
    /// decrements it and keeps the turn instead (Time Walk, Ral Zarek's
    /// -7 coin-flip emblem). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub extra_turns: u32,
    /// CR 702.50 — resolved Epic spells this player controls. Each entry is
    /// copied onto the stack at the beginning of this player's upkeep for
    /// the rest of the game; while non-empty the player can't cast spells.
    #[serde(default)]
    pub epic_spells: Vec<EpicSpell>,
    /// CR 114 — emblems this player owns. Each carries a name (for
    /// display) and a set of triggered abilities that fire from the
    /// command zone; emblems never leave once created. The trigger
    /// dispatcher walks every player's emblems alongside battlefield
    /// permanents (event-keyed kinds in `dispatch_triggers_for_events`,
    /// step-keyed kinds in `fire_step_triggers`). Created by
    /// `Effect::CreateEmblem` (planeswalker ultimates). `#[serde(default)]`
    /// for snapshot back-compat.
    #[serde(default)]
    pub emblems: Vec<Emblem>,
    /// True while a continuous effect on the battlefield prevents this
    /// player from gaining life (CR 119.7). Set by
    /// `StaticEffect::CannotGainLife` in `compute_battlefield`'s player-
    /// static pass, reset there each recompute. Honored by
    /// `GameState::adjust_life` — a positive delta is dropped while the
    /// flag is set. Powers Tainted Remedy / Erebos / Sulfuric Vortex
    /// style effects.
    #[serde(default)]
    pub cannot_gain_life: bool,
    /// Sticky one-turn "you can't gain life" lock — separate from the
    /// recomputed `cannot_gain_life` static. Set by `Effect::LifeGainLockThisTurn`
    /// (Skullcrack, Rampaging Ferocidon's one-shot version), reset in
    /// `do_untap`. Honored by `GameState::adjust_life` (treated identically
    /// to `cannot_gain_life`, but persists across `compute_battlefield`
    /// recomputes since no permanent backs it).
    #[serde(default)]
    pub cannot_gain_life_this_turn: bool,
    /// True while this player's life total can't change for the rest of the
    /// turn (Flare of Fortitude). Set by `Effect::LockLifeTotalThisTurn`,
    /// reset in `do_untap`. `adjust_life` treats it as both cannot-gain and
    /// cannot-lose, so any nonzero delta is dropped on the floor.
    #[serde(default)]
    pub life_locked_this_turn: bool,
    /// True while spells this player controls can't be countered for the
    /// rest of the turn (Veil of Summer's "spells your opponents control
    /// can't counter spells you control this turn"). Set by
    /// `Effect::GrantSpellsUncounterableThisTurn`; reset for every player at
    /// the active player's `do_untap`. Consulted by
    /// `caster_grants_uncounterable_with_x`. `#[serde(default)]` for
    /// snapshot back-compat.
    #[serde(default)]
    pub spells_uncounterable_this_turn: bool,
    /// Like `spells_uncounterable_this_turn` but only for *creature* spells
    /// (Domri, Anarch of Bolas's +1). Reset alongside it at untap.
    #[serde(default)]
    pub creature_spells_uncounterable_this_turn: bool,
    /// Colors this player (and their permanents) have hexproof from for the
    /// rest of the turn (Veil of Summer's "you and permanents you control
    /// gain hexproof from blue and from black until end of turn"). Set by
    /// `Effect::GrantHexproofFromColorThisTurn`; reset for every player at
    /// the active player's `do_untap`. Consulted by the targeting-legality
    /// checks. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub hexproof_from_colors_this_turn: Vec<crate::mana::Color>,
    /// True once this player has cast a blue or black spell this turn. Set
    /// in `finalize_cast`; reset for every player at the active player's
    /// `do_untap`. Powers Veil of Summer's "draw a card if an opponent has
    /// cast a blue or black spell this turn" gate. `#[serde(default)]` for
    /// snapshot back-compat.
    #[serde(default)]
    pub cast_blue_or_black_this_turn: bool,
    /// Colors + card types of each spell this player cast this turn, in cast
    /// order. Stamped in `finalize_cast` and cleared alongside
    /// `cast_blue_or_black_this_turn`; read by
    /// `Predicate::CastSpellThisTurnWith` — the Trap alternative costs
    /// ("if an opponent cast a blue spell this turn", Ricochet Trap).
    #[serde(default)]
    pub spell_casts_this_turn: Vec<crate::game::types::CastProfile>,
    /// True while this player has hexproof until the start of their next
    /// turn (Blossoming Calm). Set by `Effect::GainHexproofUntilYourNextTurn`;
    /// cleared at this player's `do_untap`. `#[serde(default)]`.
    #[serde(default)]
    pub hexproof_until_next_turn: bool,
    /// True while this player can't cast noncreature spells for the rest of
    /// the turn (Ranger-Captain of Eos's sacrifice ability). Set by
    /// `Effect::CantCastNoncreatureThisTurn`; reset for every player at the
    /// active player's `do_untap`. Consulted by the cast-legality gate.
    #[serde(default)]
    pub cant_cast_noncreature_this_turn: bool,
    /// "You may cast spells from your hand this turn without paying their
    /// mana costs" (Yusri's five-win jackpot). Cleared at end-of-turn.
    #[serde(default)]
    pub free_spells_from_hand_this_turn: bool,
    /// Gaea's Will — lands playable from this player's graveyard this turn.
    #[serde(default)]
    pub play_from_graveyard_this_turn: bool,
    /// Gaea's Will — this player's graveyard-bound cards exile instead this
    /// turn (CR 614.6, own cards only).
    #[serde(default)]
    pub graveyard_bound_exiled_this_turn: bool,
    /// Card names this player's opponents can't cast until this player's next
    /// turn (Academic Probation mode 0 — "Opponents can't cast spells with the
    /// chosen name until your next turn"). Reset for every player at the active
    /// player's `do_untap`; consulted by the cast-legality gate.
    #[serde(default)]
    pub opponents_cant_cast_named: Vec<String>,
    /// Sources whose static effect this player has bought their way out of for
    /// the turn (`Effect::IgnoreStaticFromSourceThisTurn` — Damping Engine).
    /// Cleared at cleanup.
    #[serde(default)]
    pub statics_ignored_this_turn: Vec<crate::card::CardId>,
    /// When true, decisions this player would make suspend via
    /// `pending_decision` so a UI can respond; when false, the engine calls
    /// the installed `Decider` synchronously (bot / tests).
    pub wants_ui: bool,
    /// CR 601.2g "proper tapping": when true this player chooses their own
    /// mana sources, so the engine auto-taps only when the payment is
    /// forced and otherwise rejects the cast with `ManualTapRequired` for
    /// them to tap manually.
    ///
    /// Split out from [`wants_ui`](Self::wants_ui), which used to stand in
    /// for it. The two are different questions — a bot wants its decisions
    /// surfaced as `pending_decision` (that is how its policies run) but
    /// emphatically does *not* want to hand-pick lands — and conflating
    /// them silently broke every bot cast that needed a tap, because bot
    /// seats set `wants_ui`. It went unnoticed only because the bot used to
    /// pre-tap its whole board, so its pool always covered the cost and the
    /// auto-tap path was never reached.
    #[serde(default)]
    pub manual_mana: bool,
    /// CR 705.3 — Krark's Thumb-style coin-flip advantage. When non-zero,
    /// every coin flip this player makes is replayed `coin_flip_advantage`
    /// extra times and they get to keep the result they prefer. Practically
    /// modelled in `Effect::FlipCoin` as "do `1 + N` flips and treat the
    /// flipper as winning if any of them came up heads" — the standard
    /// rules interpretation of stacking Krark's Thumbs (each Thumb lets
    /// you "ignore one and choose the other," so two Thumbs = three flips,
    /// pick the best).
    ///
    /// `#[serde(default)]` keeps snapshots from before this field forward-
    /// compatible. Stacks additively when multiple Krark's Thumbs are on
    /// the battlefield (compute_battlefield sums the contributing
    /// static-ability counts when this primitive is eventually wired to
    /// a permanent — for now only one Krark's Thumb is needed and we set
    /// the value directly via the Thumb card body).
    #[serde(default)]
    pub coin_flip_advantage: u32,
}

impl Player {
    pub fn new(idx: usize, name: impl Into<String>) -> Self {
        Self {
            id: PlayerId(idx),
            name: name.into(),
            life: 20,
            starting_life: 20,
            mana_pool: ManaPool::new(),
            kept_mana_this_turn: ManaPool::new(),
            library: CowBox::default(),
            hand: CowBox::default(),
            graveyard: CowBox::default(),
            command: CowBox::default(),
            may_spend_any_color_this_turn: false,
            ante: CowBox::default(),
            scheme_deck: CowBox::default(),
            opponent_cast_spell_since_your_turn: false,
            sideboard: CowBox::default(),
            commanders: Vec::new(),
            lands_played_this_turn: 0,
            extra_land_plays: 0,
            cant_play_lands_this_turn: false,
            cant_cast_matching_this_turn: Vec::new(),
            next_spell_uncounterable: Vec::new(),
            extra_loyalty_activations: 0,
            activated_loyalty_this_turn: false,
            spells_cast_this_turn: 0,
            sorceries_cast_this_turn: 0,
            spells_cast_by_name_this_game: std::collections::HashMap::new(),
            spells_cast_this_game_turn: 0,
            noncreature_spells_cast_this_game_turn: 0,
            nonartifact_spells_cast_this_game_turn: 0,
            life_gained_this_turn: 0,
            gained_life_earlier_this_turn: false,
            cards_drawn_this_turn: 0,
            cards_drawn_this_step: 0,
            cards_left_graveyard_this_turn: 0,
            creatures_died_this_turn: 0,
            zuberas_died_this_turn: 0,
            creatures_entered_this_turn: Vec::new(),
            creatures_entered_last_turn: Vec::new(),
            artifacts_entered_this_turn: 0,
            planeswalkers_entered_this_turn: 0,
            nonland_permanents_entered_this_turn: 0,
            mounts_vehicles_entered_this_turn: 0,
            multicolored_spells_cast_this_turn: 0,
            oil_activity_this_turn: false,
            channel_life_for_mana: false,
            dungeon: None,
            dungeons_completed: 0,
            escalating_resolutions_this_turn: 0,
            pending_creature_etb_counters: Vec::new(),
            pending_creature_etb_keywords: Vec::new(),
            permanent_left_battlefield_this_turn: false,
            was_dealt_damage_this_turn: false,
            damage_taken_this_turn: 0,
            lost_life_this_turn: false,
            graveyard_cast_types_this_turn: Vec::new(),
            play_from_top_this_turn: false,
            cast_from_library_top_this_turn: false,
            life_lost_this_turn: 0,
            creatures_that_damaged_me_this_turn: Vec::new(),
            lands_entered_this_turn: 0,
            creature_spell_countered_by_opponent_this_turn: false,
            noncreature_destroyed_by_opponent_this_turn: false,
            prowl_types_this_turn: Vec::new(),
            prowl_any_type_this_turn: false,
            attacked_this_turn: false,
            creatures_attacked_this_turn: 0,
            dealt_combat_damage_to_player_this_turn: false,
            double_your_source_damage_this_turn: false,
            committed_crime_this_turn: false,
            face_down_activity_this_turn: false,
            descended_this_turn: false,
            descend_count_this_turn: 0,
            silenced_this_turn: false,
            warped_spell_this_turn: false,
            searched_library_this_turn: false,
            shroud_this_turn: false,
            protected_from_everything: false,
            cards_exiled_this_turn: 0,
            cards_to_graveyard_this_turn: 0,
            instants_or_sorceries_cast_this_turn: 0,
            spells_cast_from_hand_this_turn: 0,
            extra_plus_one_counters_this_turn: 0,
            extra_etb_p1p1_counters_this_turn: 0,
            pending_is_discounts: Vec::new(),
            pending_spell_discounts: Vec::new(),
            turn_spell_discounts: Vec::new(),
            cards_discarded_this_turn: 0,
            discarded_this_turn: std::collections::HashSet::new(),
            permanents_sacrificed_this_turn: 0,
            artifacts_sacrificed_this_turn: 0,
            creatures_cast_this_turn: 0,
            cannot_gain_life_this_turn: false,
            life_locked_this_turn: false,
            spells_uncounterable_this_turn: false,
            creature_spells_uncounterable_this_turn: false,
            hexproof_from_colors_this_turn: Vec::new(),
            hexproof_until_next_turn: false,
            cast_blue_or_black_this_turn: false,
            spell_casts_this_turn: Vec::new(),
            cant_cast_noncreature_this_turn: false,
            free_spells_from_hand_this_turn: false,
            play_from_graveyard_this_turn: false,
            graveyard_bound_exiled_this_turn: false,
            opponents_cant_cast_named: Vec::new(),
            statics_ignored_this_turn: Vec::new(),
            first_spell_tax_charges: 0,
            sorceries_as_flash: false,
            poison_counters: 0,
            poison_capped_this_turn: false,
            ring_temptations: 0,
            ring_bearer: None,
            speed: 0,
            speed_increased_this_turn: false,
            energy: 0,
            energy_spent_this_turn: 0,
            experience: 0,
            rad_counters: 0,
            city_blessing: false,
            max_hand_size: default_max_hand_size(),
            eliminated: false,
            last_drawn_card: None,
            range_of_influence: None,
            is_emperor: false,
            loss_cause: None,
            cant_lose_this_turn: false,
            damage_floor_this_turn: false,
            attack_tax_until_your_turn: 0,
            spell_names_cast_this_turn: Vec::new(),
            skip_turns: 0,
            skip_next_untap_step: 0,
            skip_next_draw_step: 0,
            next_draw_replacements: Vec::new(),
            skip_next_combat: 0,
            lands_dont_untap_next_untap: 0,
            lands_produce_color_this_turn: None,
            creatures_dont_untap_next_untap: 0,
            firebending_kept_red: 0,
            extra_turns: 0,
            epic_spells: Vec::new(),
            emblems: Vec::new(),
            cannot_gain_life: false,
            wants_ui: false,
            manual_mana: false,
            coin_flip_advantage: 0,
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.eliminated
    }

    /// Baseline per-turn land-play check — `true` iff this player has
    /// not yet played any land this turn. NOTE: this is a vanilla CR
    /// 305.2 default and **does not** consult
    /// `StaticEffect::ExtraLandPerTurn` (Exploration, Azusa). For the
    /// CR-correct check that honors continuous-effect grants, use
    /// `GameState::can_player_play_land(seat)` which sums
    /// `extra_land_plays_per_turn(seat)` into the cap.
    pub fn can_play_land(&self) -> bool {
        self.lands_played_this_turn < 1 + self.extra_land_plays
    }

    /// Draw the top card into hand.  Returns `None` if the library is empty.
    /// Increments `cards_drawn_this_turn` so per-turn draw payoffs (e.g.
    /// Strixhaven's Quandrix scaling) see a fresh count.
    pub fn draw_top(&mut self) -> Option<CardId> {
        if self.library.is_empty() {
            return None;
        }
        let card = self.library.remove(0);
        let id = card.id;
        self.hand.push(card);
        self.cards_drawn_this_turn = self.cards_drawn_this_turn.saturating_add(1);
        self.cards_drawn_this_step = self.cards_drawn_this_step.saturating_add(1);
        Some(id)
    }

    /// Return all hand cards to the bottom of the library.
    /// Call `library.shuffle(&mut rng)` afterwards to randomize.
    pub fn return_hand_to_library(&mut self) {
        while let Some(card) = self.hand.pop() {
            self.library.push(card);
        }
    }

    pub fn has_in_hand(&self, id: CardId) -> bool {
        self.hand.iter().any(|c| c.id == id)
    }

    pub fn remove_from_hand(&mut self, id: CardId) -> Option<CardInstance> {
        self.hand
            .iter()
            .position(|c| c.id == id)
            .map(|i| self.hand.remove(i))
    }

    pub fn send_to_graveyard(&mut self, mut card: CardInstance) {
        // CR 122.2 — counters cease to exist on zone change; the graveyard
        // object carries none (dies-with-counters triggers read LKI caches).
        card.counters.clear();
        card.keyword_counters.clear();
        self.cards_to_graveyard_this_turn += 1;
        // CR 700.11 — descending requires a *permanent* card hitting the gy.
        if card.definition.is_permanent() {
            self.descended_this_turn = true;
            self.descend_count_this_turn += 1;
        }
        self.graveyard.push(card);
    }

    /// Push a card definition directly into the library (top of deck = index 0).
    pub fn add_to_library_top(&mut self, id: CardId, definition: CardDefinition) {
        self.library.insert(0, CardInstance::new(id, definition, self.id.0));
    }

    /// Push a card definition to the bottom of the library.
    pub fn add_to_library_bottom(&mut self, id: CardId, definition: CardDefinition) {
        self.library.push(CardInstance::new(id, definition, self.id.0));
    }
}
