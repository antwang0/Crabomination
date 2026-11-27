use serde::{Deserialize, Serialize};

use crate::card::{CardDefinition, CardId, CardInstance};
use crate::cow::CowBox;
use crate::mana::ManaPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub usize);

/// CR 402.2 — the default maximum hand size (seven cards). A player's
/// `max_hand_size` starts here; effects can raise/lower it or remove it.
pub const DEFAULT_MAX_HAND_SIZE: usize = 7;

/// Magnitude past which a board number a consumer *scales* (life, power,
/// toughness, sums of them) is no longer read literally.
///
/// Nothing bounds them: Beacon of Immortality saturates life at `i32::MAX`
/// in ~30 casts (`ENGINE_BACKLOG` OPEN, `cube` seed 1018), Exponential
/// Growth does it to power in one resolution. Unclamped, consumers wrap (the
/// evaluator scored unbounded life as losing) or feed the net 10^8 features.
///
/// Ten thousand keeps the evaluator's products inside `i32` for the shipped
/// profiles (`unit` 1 and 10), and no state inside `±10_000` reads
/// differently — the clamp is a guard, not a rescale.
///
/// Consumers only. Rules-facing values stay exact and merely saturate
/// ([`CardInstance::pump`](crate::card::CardInstance::pump),
/// `compute_permanent`), since clamping power would change combat damage.
pub const SCALE_CEILING: i32 = 10_000;

/// Serde default for `Player.max_hand_size` — the normal seven-card cap.
fn default_max_hand_size() -> Option<usize> {
    Some(DEFAULT_MAX_HAND_SIZE)
}

fn default_starting_life() -> i32 {
    20
}

/// CR 114 — an emblem owned by a player: only the triggered abilities it
/// grants, in the command zone for the rest of the game. Created by
/// `Effect::CreateEmblem`; a `this_turn` one carries an effect's "until end
/// of turn, whenever …" triggers.
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
    /// CR 603.7c — "until end of turn, whenever …": the delayed triggered
    /// abilities of `Effect::TriggersThisTurn`, dropped at cleanup.
    #[serde(default)]
    pub this_turn: bool,
    /// CR 611.2b — `Effect::TriggersUntilYourNextTurn`'s emblem, dropped as
    /// its owner's next turn begins.
    #[serde(default)]
    pub until_your_next_turn: bool,
    /// CR 603.7d — a delayed trigger's source is the spell or ability that
    /// made it: `TriggersThisTurn` / `TriggersUntilYourNextTurn` record the
    /// creating card, a real emblem (CR 114) has none (`CardId(0)`).
    #[serde(default)]
    pub source: Option<crate::card::CardId>,
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

impl LossCause {
    /// How a player reads the cause in a log line ("Bot 2 is out — ...").
    pub fn phrase(self) -> &'static str {
        match self {
            LossCause::LifeDepleted => "life total",
            LossCause::Poison => "poison",
            LossCause::Decked => "drew from an empty library",
            LossCause::CommanderDamage => "commander damage",
            LossCause::Conceded => "conceded",
            LossCause::Other => "a lose-the-game effect",
        }
    }
}

/// CR 800.4a — when a seat left the game: the turn, and how many seats had
/// left before it. Seats that leave in one state-based sweep share `before`
/// (they left at the same time, CR 104.4a), so they share a placing
/// (`GameState::placement`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Departure {
    pub turn: u32,
    pub before: u32,
}

/// One pending loyalty-ability copy grant (see
/// `PlayerCold::loyalty_copy_grants`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LoyaltyCopyGrant {
    /// Copies made per activation.
    pub copies: u32,
    /// Only planeswalkers of this type (Leori); `None` is any.
    pub subtype: Option<crate::card::PlaneswalkerSubtype>,
    /// Consumed by the first activation it copies ("the next").
    pub once: bool,
    /// The source whose spell rider shares this "when you next" (Repeated
    /// Reverberation): spending either spends both.
    #[serde(default)]
    pub linked: Option<CardId>,
    /// Copies the next activated ability with {X} in its activation cost
    /// instead of a loyalty ability (Magus Lucea Kane).
    #[serde(default)]
    pub x_ability: bool,
    /// Copies the next non-mana activated ability paid with at least this
    /// much mana instead (Dynaheir, Invoker Adept); 0 = not this kind.
    #[serde(default)]
    pub min_mana: u32,
}

/// The rarely-written, heap-owning tail of a seat (rare registries plus the
/// name), behind one CoW handle so a `PlayerData` unshare bumps a refcount
/// instead of cloning every collection. `Deref`/`DerefMut` keep
/// `player.field` access unchanged.
///
/// Membership rule, as for [`crate::game::ColdState`]: heap-owning *and*
/// rarely written. A per-turn `clear()` is a write, so reset loops guard
/// theirs on `is_empty()`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerCold {
    // ── Rare-format card zones ────────────────────────────────────────────
    // Ante, Archenemy, Planechase, Attractions: empty in every simulated
    // format, so kept off `PlayerData`'s unshare (PERF `(-152)`).
    /// CR 407 — cards this player owns in the ante zone. Non-empty only while
    /// `GameState.playing_for_ante`; the winner takes all of it (CR 407.2).
    #[serde(default)]
    pub ante: CowBox<Vec<CardInstance>>,
    /// CR 904.3 — the archenemy's face-down scheme deck. Like the command
    /// zone it never empties into another zone: a scheme is set in motion off
    /// the top (CR 904.9) into `command`, and abandoned back to the bottom
    /// here (CR 904.10). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub scheme_deck: CowBox<Vec<CardInstance>>,
    /// CR 901.3 — this player's face-down planar deck. Like the scheme deck it
    /// never empties into another zone: planeswalking moves the top card into
    /// `command` face up (CR 901.11) and puts the plane left behind on the
    /// bottom. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub planar_deck: CowBox<Vec<CardInstance>>,
    /// CR 718.2 — this player's face-down Attraction deck, in the command
    /// zone. Cards leave it only by being opened onto the battlefield
    /// (CR 701.51b). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub attraction_deck: CowBox<Vec<CardInstance>>,
    /// CR 718.6a — the face-up "junkyard" pile of Attraction cards that would
    /// have gone anywhere but the battlefield, exile, or the command zone.
    #[serde(default)]
    pub attraction_junkyard: CowBox<Vec<CardInstance>>,

    pub name: String,
    /// CR 309.7 — the dungeons this player has completed, by name, in order.
    #[serde(default)]
    pub completed_dungeons: Vec<String>,
    /// CardIds this player designated as commanders (set by
    /// `GameState::seat_commanders`, read by the 21-commander-damage SBA via
    /// `GameState::is_commander`). Independent of the command zone: a
    /// commander stays one in any zone.
    #[serde(default)]
    pub commanders: Vec<CardId>,
    /// Transient: Treasure mana the cast in flight spent, consumed by
    /// `finalize_cast` into `CardData::cast_with_treasure_mana`. Lives on the
    /// seat because `GameState` is size-capped; read before written so an
    /// ordinary cast never unshares the seat.
    #[serde(skip)]
    pub cast_paid_with_treasure: u32,
    /// Transient: the `MarksCast` marks whose mana funded the cast in flight
    /// (a bit per mark); consumed into `CardData::cast_marked_mana`.
    #[serde(skip)]
    pub cast_paid_marked: u8,
    /// Transient: how much artifact-produced mana this seat's cast in flight
    /// spent (the pool's `artifact` provenance fell). Consumed by
    /// `finalize_cast` into `CardData::cast_artifact_mana`, read-guarded the
    /// same way.
    #[serde(skip)]
    pub cast_paid_artifact_mana: u32,
    /// Transient: the creature-produced twin of `cast_paid_artifact_mana`,
    /// consumed into `CardData::cast_creature_mana`.
    #[serde(skip)]
    pub cast_paid_creature_mana: u32,
    /// Transient: the Desert-produced twin of `cast_paid_artifact_mana`,
    /// consumed into `CardData::cast_desert_mana`.
    #[serde(skip)]
    pub cast_paid_desert_mana: u32,
    /// Transient: the snow-produced twin, consumed into
    /// `CardData::cast_snow_mana`.
    #[serde(skip)]
    pub cast_paid_snow_mana: u32,
    /// Transient: the colours of that snow mana (a WUBRG bitmask), consumed
    /// into `CardData::cast_snow_colors`.
    #[serde(skip)]
    pub cast_paid_snow_colors: u8,
    /// CR 903.4a — this seat's combined commander colour identity, cached
    /// when commanders are seated (identity never changes) because the walk
    /// serializes a whole `CardDefinition` and every Command Tower / Arcane
    /// Signet tap asks. Empty means "not computed or colourless";
    /// `commander_identity_colors` then falls back to the walk, so old
    /// snapshots still answer right.
    #[serde(default)]
    pub commander_identity: crate::mana::ColorSet,
    /// When this seat left the game (CR 800.4a), stamped by the one leave
    /// pass; `None` while it is in. Written once a game, so it lives here.
    #[serde(default)]
    pub departure: Option<Departure>,
    /// Miss Highwater's contract counters, one entry per counter: (the
    /// beneficiary seat, the granting source). While any is held, losing the
    /// game copies this seat's artifacts and creatures for each beneficiary.
    #[serde(default)]
    pub contracts: Vec<(usize, CardId)>,
    /// CR 603.7e — pending "your next creature spell this turn enters with these
    /// keywords" riders (Summon: Brynhildr's "Gestalt Mode" haste). Applied to
    /// the next creature spell's permanent as it enters and cleared at cleanup.
    /// `#[serde(default)]`.
    #[serde(default)]
    pub pending_creature_etb_keywords: Vec<crate::card::Keyword>,
    /// Permanent types already cast from the graveyard this turn under a
    /// Muldrotha-style permission (one of each per turn). Reset at untap.
    #[serde(default)]
    pub graveyard_cast_types_this_turn: Vec<crate::card::CardType>,
    /// Coram, the Undertaker's per-turn budget: a spell / a land already
    /// played this turn from among cards milled into graveyards this turn.
    /// Reset with `graveyard_cast_types_this_turn`.
    #[serde(default)]
    pub milled_spell_cast_this_turn: bool,
    #[serde(default)]
    pub milled_land_played_this_turn: bool,
    /// Equip abilities this player activated this turn (Forge Anew's "the
    /// first equip ability you activate during each of your turns").
    #[serde(default)]
    pub equips_activated_this_turn: u32,
    /// Kagha, Shadow Archdruid's once-a-turn milled play, used this turn.
    #[serde(default)]
    pub milled_once_used_this_turn: bool,
    /// Sources whose once-per-turn graveyard-cast grant was used this turn:
    /// Exploration Broodship's {8+} band
    /// (`StaticEffect::GraveyardCastBySacrificingOncePerTurn`) and Kotis's
    /// (`GraveyardCardsHaveEscapeMatching { once_per_turn: true, .. }`). Reset with `graveyard_cast_types_this_turn`.
    #[serde(default)]
    pub graveyard_sac_cast_sources_this_turn: Vec<crate::card::CardId>,
    /// The last card this player cast or played off their library top under a
    /// once-per-turn `PlayFromLibraryTopOncePerTurn` grant, with the grant's
    /// source — "when you do" (The Fourth Doctor) reads it
    /// (`Predicate::UsedSourcesLibraryTopGrant`). Cleared at their untap.
    #[serde(default)]
    pub library_top_grant_used: Option<(crate::card::CardId, crate::card::CardId)>,
    /// "You can't cast [filter] spells this turn" (Cease-Fire). Cleared at the
    /// turn boundary alongside `cant_play_lands_this_turn`.
    #[serde(default)]
    pub cant_cast_matching_this_turn: Vec<crate::card::SelectionRequirement>,
    /// "The next [filter] spell you cast this turn can't be countered"
    /// (Insist, Overmaster). Consumed by the next matching cast; cleared at
    /// end of turn.
    #[serde(default)]
    pub next_spell_uncounterable: Vec<crate::card::SelectionRequirement>,
    /// CR 702.187 — ids of cards this player discarded this turn, so Mayhem
    /// can offer a graveyard cast only for cards actually discarded this turn.
    /// Populated in `discard_card`; cleared in `do_untap`.
    #[serde(default)]
    pub discarded_this_turn: crate::game::types::IdSet<crate::card::CardId>,
    /// "[Filter] spells you cast this turn cost {N} less" grants
    /// (`Effect::SpellsCostLessThisTurn` — Urza, Planeswalker's +2).
    /// Each entry applies to every matching spell for the rest of the
    /// turn; cleared in `finish_cleanup` (CR 514.2).
    #[serde(default)]
    pub turn_spell_discounts: Vec<(crate::card::SelectionRequirement, u32)>,
    /// CR 614 — queued "the next time you would draw a card this turn,
    /// [effect] instead" replacements (Onslaught Words), as `(source, effect,
    /// x)`. `draw_one` pops the front instead of drawing, replaying X
    /// (Aladdin's Lamp). Cleared at the turn boundary.
    #[serde(default)]
    pub next_draw_replacements: Vec<(crate::card::CardId, crate::effect::Effect, u32)>,
    /// CR 702.50 — resolved Epic spells this player controls. Each entry is
    /// copied onto the stack at the beginning of this player's upkeep for
    /// the rest of the game; while non-empty the player can't cast spells.
    #[serde(default)]
    pub epic_spells: Vec<EpicSpell>,
    /// CR 114 — emblems this player owns; they never leave. The dispatcher
    /// walks them alongside permanents (`dispatch_triggers_for_events`,
    /// `fire_step_triggers`).
    #[serde(default)]
    pub emblems: Vec<Emblem>,
    /// Jace's Machinations: the turn number its "Jace loyalty abilities at
    /// instant speed until end of turn" was granted (`None` = never). Valid
    /// only while it equals the game's turn number, so no cleanup clears it.
    #[serde(default)]
    pub jace_instant_loyalty_turn: Option<u32>,
    /// Urza Assembles the Titans III: the turn "you may activate the loyalty
    /// abilities of planeswalkers you control twice this turn" was granted.
    /// Valid only while it equals the turn number (`game/loyalty_twice_turn.rs`).
    #[serde(default)]
    pub loyalty_twice_turn: Option<u32>,
    /// Colours this player and their permanents have hexproof from this turn
    /// (Veil of Summer). Set by `Effect::GrantHexproofFromColorThisTurn`,
    /// reset for every player at `do_untap`; read by targeting legality.
    #[serde(default)]
    pub hexproof_from_colors_this_turn: Vec<crate::mana::Color>,
    /// Card names this player's opponents can't cast until this player's next
    /// turn (Academic Probation mode 0 — "Opponents can't cast spells with the
    /// chosen name until your next turn"). Reset for every player at the active
    /// player's `do_untap`; consulted by the cast-legality gate.
    #[serde(default)]
    pub opponents_cant_cast_named: Vec<String>,
    /// Mana values this player's opponents can't cast until this player's
    /// next turn (Living Breakthrough). Same lifetime as the name lock above.
    #[serde(default)]
    pub opponents_cant_cast_mv: Vec<u32>,
    /// Sources whose static effect this player has bought their way out of for
    /// the turn (`Effect::IgnoreStaticFromSourceThisTurn` — Damping Engine).
    /// Cleared at cleanup.
    #[serde(default)]
    pub statics_ignored_this_turn: Vec<crate::card::CardId>,
    /// CR 905.2b — the notes this seat took during the draft that produced
    /// their pool. Empty in every non-drafted game.
    #[serde(default)]
    pub draft_notes: crate::draft::DraftNotes,
    /// CR 508.1d — "during [this player]'s next turn, creatures they control
    /// attack [permanent] if able" (Gideon Jura's +2): the permanent and the
    /// turn number the lure was set on. Read by `GameState::attack_lure_of`,
    /// cleared at the end of this player's next turn.
    #[serde(default)]
    pub attack_lure: Option<(crate::card::CardId, u32)>,
    /// CR 508.1d — "during [this player]'s next turn, each creature they
    /// control attacks if able" (Rowan Kenrith's +2): the turn it was set on.
    /// Read by `GameState::side_attacks_if_able`, cleared with `attack_lure`.
    #[serde(default)]
    pub attacks_if_able_next_turn: Option<u32>,
    /// CR 508.1d — one-creature lures on this player's creatures (Gideon,
    /// Battle-Forged's +2): (creature, lured permanent, turn set on). Read
    /// by `GameState::creature_lure_of`, cleared with `attack_lure`.
    #[serde(default)]
    pub creature_attack_lures: Vec<(crate::card::CardId, crate::card::CardId, u32)>,
    /// "You may cast it from your graveyard as an Adventure until the end of
    /// your next turn" (Hildibrand Manderville): each card and the turn the
    /// grant was made on. Read by `GameState::adventure_grant_live`, lapsed
    /// at the end of this player's first turn after it.
    #[serde(default)]
    pub adventure_graveyard_grants: Vec<(crate::card::CardId, u32)>,
    /// "Can't cast spells until your next turn" (Innocuous Researcher):
    /// re-arms `silenced_this_turn` at every turn boundary until this
    /// player's own untap clears it.
    #[serde(default)]
    pub silenced_until_their_turn: bool,
    /// "Until your next turn, [this player] can't cast noncreature spells"
    /// (Hope of Ghirapur): a bit per locking seat, each cleared as that
    /// seat's turn begins; while any is set, every turn boundary re-arms
    /// `cant_cast_noncreature_this_turn`.
    #[serde(default)]
    pub noncreature_locked_by: u64,
    /// "Lands you control enter tapped this turn" (Nahiri's Lithoforming).
    /// Read by `apply_enters_tapped_replacement`; cleared at the turn's end.
    #[serde(default)]
    pub lands_enter_tapped_this_turn: bool,
    /// "Until end of turn, you may cast spells from your hand without paying
    /// their mana costs" (Chandra, Flame's Catalyst's −8): the turn-scoped
    /// `StaticEffect::CastHandSpellsFree`. Cleared with the flag above.
    #[serde(default)]
    pub cast_hand_free_this_turn: bool,
    /// "Until your next turn, creatures your opponents control enter tapped"
    /// (Cone of Cold's 20). Read by `apply_enters_tapped_replacement`;
    /// cleared as this player's next turn begins.
    #[serde(default)]
    pub opponents_creatures_enter_tapped: bool,
    /// Night Shift of the Living Dead's once-each-turn die shift is spent.
    #[serde(default)]
    pub die_shift_used_this_turn: bool,
    /// This player has cast a spell this game — set once, by the first cast
    /// (`Predicate::FirstSpellYouCastThisGame`: Once Upon a Time). Never
    /// cleared, so it costs one cold write a game.
    #[serde(default)]
    pub cast_a_spell_this_game: bool,
    /// This player ventured into the dungeon this turn (CR 701.49) — read by
    /// an opponent's `StaticEffect::OpponentsVentureOncePerTurn` (Keen-Eared
    /// Sentry). Set only while such a static is on the battlefield; cleared,
    /// guarded, at the turn boundary.
    #[serde(default)]
    pub ventured_this_turn: bool,
    /// "The next creature card you play this turn can be played as though it
    /// had flash" (Scout's Warning, `Effect::NextCreatureSpellHasFlashThisTurn`):
    /// spent by the next creature spell cast, cleared at the turn boundary.
    #[serde(default)]
    pub next_creature_spell_flash_this_turn: bool,
    /// "The next [filter] spell you cast this turn can be cast as though it had
    /// flash" (Quicken, `Effect::NextMatchingSpellHasFlashThisTurn`): one filter
    /// per grant, spent by the first matching cast, cleared at the turn boundary.
    #[serde(default)]
    pub next_matching_spell_flash: crate::oftenempty::OftenEmpty<crate::card::SelectionRequirement>,
    /// CR 702.131b — a permanent with ascend has entered under this player's
    /// control, so permanent entries check for the city's blessing.
    #[serde(default)]
    pub ascend_armed: bool,
    /// Storied: a storied permanent has entered under this player's control,
    /// so permanent entries check for the enduring story.
    #[serde(default)]
    pub storied_armed: bool,
    /// True once this player has an enduring story (storied); it lasts for
    /// the rest of the game.
    #[serde(default)]
    pub enduring_story: bool,
    /// Tezzeret, Betrayer of Flesh — this player has activated an artifact's
    /// ability this turn. Written once a turn; reset at the turn boundary.
    #[serde(default)]
    pub artifact_ability_activated_this_turn: bool,
    /// Professor Hojo — this player has activated an ability targeting a
    /// creature they control during their turn. Reset at the turn boundary.
    #[serde(default)]
    pub own_creature_ability_this_turn: bool,
    /// Nuka-Nuke Launcher — rad counters this player gets per spell cast,
    /// until the end of their next turn.
    #[serde(default)]
    pub rad_per_cast: u32,
    /// Set once this player's turn has begun under `rad_per_cast`; the next
    /// turn boundary clears the grant.
    #[serde(default)]
    pub rad_per_cast_their_turn: bool,
    /// The players whose planeswalkers this player attacked during their most
    /// recent turn — `attacked_players_this_turn`'s walker half, cleared with
    /// it (Sandswirl Wanderglyph's "attacked you or a planeswalker you
    /// control this turn").
    #[serde(default)]
    pub attacked_walkers_of_this_turn: Vec<usize>,
    /// CR 718.5 — `(turn_number, count)`: Attractions this player visited
    /// during that turn. Stamped, not cleared — a stale turn reads as zero.
    #[serde(default)]
    pub attractions_visited: (u32, u32),
}

/// A seat's mutable state. Reached only through [`Player`], which owns it
/// behind an `Arc` — see that type for why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    // Fields whose clone calls something come first, plain ones after as one
    // run: declaration order is the derived `Clone`'s statement order and
    // `&self` is not `noalias`, so a scalar between collections costs its own
    // load/store pair. PERF `(-139)`, `(-141)`, `(-142)`.
    pub mana_pool: ManaPool,
    /// Top of library is `library[0]`. Card zones are [`CowBox`]-wrapped so a
    /// `GameState` clone shares them until written (see `crate::cow`).
    #[serde(deserialize_with = "crate::zone::deserialize_library")]
    pub library: crate::zone::CardPile,
    pub hand: crate::zone::CardPile,
    /// The graveyard is the one zone with a wrapper of its own: it also
    /// carries the answer two hot walkers ask of it. Reads and writes go
    /// through `Deref`/`DerefMut` unchanged — see [`crate::zone::Graveyard`].
    pub graveyard: crate::zone::Graveyard,
    /// The command zone (commanders, Conspiracies). Cards arrive at game
    /// start (`seat_commanders`), by replacement instead of hand/library
    /// (CR 903.9b), or by SBA from graveyard/exile (CR 903.9a).
    #[serde(default)]
    pub command: CowBox<Vec<CardInstance>>,
    /// CR 406 / 701.45 — the Lessons sideboard (cards from outside the game)
    /// a Learn ability can fetch. Filled by deck construction; when empty,
    /// Learn falls back to the legacy `Draw 1` approximation.
    #[serde(default)]
    pub sideboard: CowBox<Vec<CardInstance>>,
    /// Creatures that entered the battlefield under this player's control
    /// this turn (stamped at the shared ETB funnels). Rotated into
    /// `creatures_entered_last_turn` at end of turn — Ephara, God of the
    /// Polis reads the previous turn's entries at each upkeep.
    #[serde(default)]
    pub creatures_entered_this_turn: crate::oftenempty::OftenEmpty<crate::card::CardId>,
    /// Last turn's `creatures_entered_this_turn` (see above).
    #[serde(default)]
    pub creatures_entered_last_turn: crate::oftenempty::OftenEmpty<crate::card::CardId>,
    /// CR 309 — the dungeon this player is currently in: (name, room index).
    /// `None` between dungeons; venturing with `None` enters a new dungeon.
    #[serde(default)]
    pub dungeon: Option<(String, u8)>,
    /// CR 603.7e — pending "your next creature spell this turn enters with N
    /// extra counters" riders (FIN Summon sagas: Fenrir II, Brynhildr). All
    /// drain onto the next creature spell cast; cleared at cleanup.
    #[serde(default)]
    pub pending_creature_etb_counters: crate::oftenempty::OftenEmpty<(crate::card::CounterType, u32)>,
    /// Creatures that dealt damage to this player this turn (any damage),
    /// reset at `do_untap`. Backs
    /// `SelectionRequirement::DealtDamageToControllerThisTurn` (Spear of
    /// Heliod).
    #[serde(default)]
    pub creatures_that_damaged_me_this_turn: crate::oftenempty::OftenEmpty<crate::card::CardId>,
    /// The COMBAT half of `creatures_that_damaged_me_this_turn`: attackers
    /// and blockers that dealt this player combat damage this turn (Admiral
    /// Beckett Brass's three Pirates, Estinien Varlineau's Dragons).
    #[serde(default)]
    pub creatures_that_combat_damaged_me_this_turn: crate::oftenempty::OftenEmpty<crate::card::CardId>,
    /// Creature types among this player's creatures that dealt combat damage
    /// to a player this turn (CR 702.76 Prowl). Stamped at the combat-damage
    /// funnels, cleared at the turn boundary. `#[serde(default)]` for
    /// snapshot back-compat.
    #[serde(default)]
    pub prowl_types_this_turn: crate::oftenempty::OftenEmpty<crate::card::CreatureType>,
    /// Ids of the cards that reached this graveyard from anywhere this turn.
    /// Backs `SelectionRequirement::PutIntoGraveyardThisTurn` (Reenact the
    /// Crime).
    #[serde(default)]
    pub graveyard_ids_this_turn: crate::game::types::IdSet<CardId>,
    /// Ids of the cards milled into this graveyard this turn, recorded by
    /// `GameState::note_milled` beside every `GameEvent::CardMilled` push.
    /// Backs `PutIntoGraveyardFromLibraryThisTurn`.
    #[serde(default)]
    pub milled_ids_this_turn: crate::game::types::IdSet<CardId>,
    /// Cards surveiled into this graveyard this turn (CR 701.42a). With
    /// `milled_ids_this_turn` it backs
    /// `Value::CardsPutIntoGraveyardFromLibraryThisTurn` (Cruel Calculations,
    /// which drew nothing on FRA before 2026-11-16 counted surveil).
    #[serde(default)]
    pub surveiled_this_turn: u32,
    /// One-shot "next instant/sorcery you cast this turn costs {N} less"
    /// discounts (Thundertrap Trainer) as `(amount, granted_at)`, live only
    /// while `instants_or_sorceries_cast_this_turn` still equals `granted_at`.
    #[serde(default)]
    pub pending_is_discounts: crate::oftenempty::OftenEmpty<(u32, u32)>,
    /// Like `pending_is_discounts` but for *any* next spell this turn
    /// (Mutated Cultist): `(amount, spells_cast_this_turn at grant)`.
    #[serde(default)]
    pub pending_spell_discounts: crate::oftenempty::OftenEmpty<(u32, u32)>,
    /// "The next [filter] spell you cast this turn costs {N} less"
    /// (Invasion of the Giants): `(filter, N)` until a matching cast consumes
    /// it (`game/effects/next_spell_discount.rs`).
    #[serde(default)]
    pub next_matching_spell_discounts: crate::oftenempty::OftenEmpty<(crate::card::SelectionRequirement, u32)>,
    /// "The next spell you cast this turn has affinity for artifacts" (Saheeli,
    /// the Gifted): `spells_cast_this_turn` at grant. The discount is counted
    /// as the spell is cast, not as it's granted.
    #[serde(default)]
    pub pending_affinity_next_spell: crate::oftenempty::OftenEmpty<u32>,
    /// "The next [filter] spell you cast this turn can be cast without paying
    /// its mana cost" (Apex Observatory, World War Hulk): one filter per grant,
    /// spent by a matching cast, cleared at end of turn.
    #[serde(default)]
    pub free_next_spells: crate::oftenempty::OftenEmpty<crate::card::SelectionRequirement>,
    /// As `pending_affinity_next_spell`, for the next *noncreature* spell (Don
    /// & Raph, Hard Science): `noncreature_spells_cast_this_game_turn` at grant.
    #[serde(default)]
    pub pending_affinity_next_noncreature: crate::oftenempty::OftenEmpty<u32>,
    /// CR 500.4 exception — "you don't lose this mana as steps and phases
    /// end" mana (Savage Ventmaw). `empty_mana_pools` re-seeds it each empty;
    /// cleared at cleanup.
    #[serde(default)]
    pub kept_mana_this_turn: ManaPool,
    /// Colours + card types of each spell cast this turn, in order. Stamped
    /// in `finalize_cast`; read by `Predicate::CastSpellThisTurnWith` (Trap
    /// alternative costs, Ricochet Trap).
    #[serde(default)]
    pub spell_casts_this_turn:
        crate::copyvec::CopyVec<[crate::game::types::CastProfile; 4]>,
    /// The rarely-written tail — see [`PlayerCold`].
    #[serde(flatten)]
    pub cold: crate::cow::CowBox<PlayerCold>,
    pub id: PlayerId,
    pub life: i32,
    /// The life total this player began the game with (CR 103.4). Set by
    /// `apply_format`; drives "N more/less than your starting life total"
    /// thresholds (Righteous Valkyrie, Speaker of the Heavens). `#[serde(default
    /// = "default_starting_life")]` for snapshot back-compat.
    #[serde(default = "default_starting_life")]
    pub starting_life: i32,
    /// This player's life total as the current turn began (CR 119 — Sengir,
    /// the Dark Baron's "that player's life total as the turn began"). Set
    /// by `begin_turn_resets` for every seat.
    #[serde(default = "default_starting_life")]
    pub life_at_turn_start: i32,
    /// CR 609.4b — "you may spend mana as though it were mana of any type"
    /// for the rest of this turn (North Star). Cleared at cleanup.
    #[serde(default)]
    pub may_spend_any_color_this_turn: bool,
    /// CR 901.9 — how many times this player has taken the roll-the-planar-die
    /// special action this turn; the next roll costs that much generic mana.
    /// Reset at the turn boundary.
    #[serde(default)]
    pub planar_die_rolls_this_turn: u32,
    /// How many lands this player has played on their current turn.
    pub lands_played_this_turn: u32,
    /// CR 605 — this player tapped a land for mana this turn (Desolation).
    /// Cleared at cleanup.
    #[serde(default)]
    pub tapped_land_for_mana_this_turn: bool,
    /// Extra land plays granted this turn (Explore, Oracle of Mul Daya,
    /// Dryad of the Ilysian Grove, etc.). Defaults to 0. The player can
    /// play `1 + extra_land_plays` lands per turn total.
    #[serde(default)]
    pub extra_land_plays: u32,
    /// The Chain Veil — extra loyalty activations available this turn on top
    /// of the CR 606.3 one-per-planeswalker limit. Cleared each turn.
    #[serde(default)]
    pub extra_loyalty_activations: u32,
    /// "Copy the next loyalty ability you activate this turn" (Jaya's
    /// Phoenix, Repeated Reverberation) and Leori's type-scoped copy. Read as
    /// each loyalty ability goes on the stack; cleared at cleanup.
    #[serde(default)]
    pub loyalty_copy_grants: Vec<LoyaltyCopyGrant>,
    /// Whether this player has activated a loyalty ability this turn (The
    /// Chain Veil's end-step "if you didn't …" check). Cleared each turn.
    #[serde(default)]
    pub activated_loyalty_this_turn: bool,
    /// Spells cast since this player's **own** turn began (outlives other
    /// seats' turns). Rules text's "this turn" is
    /// `spells_cast_this_game_turn`; this one feeds `encode.rs`, day/night
    /// and the next-spell discount tallies.
    pub spells_cast_this_turn: u32,
    /// Sorcery spells cast this turn — Backdraft's "a player who cast one or
    /// more sorcery spells this turn". Reset at Cleanup.
    #[serde(default)]
    pub sorceries_cast_this_turn: u32,
    /// Per-name lifetime cast counter (Approach of the Second Sun), bumped in
    /// `finalize_cast`, never reset. Keyed by `&'static str` so the clone
    /// doesn't allocate per entry.
    ///
    /// ⚠ Only names that ask are recorded (`finalize_cast` gates on
    /// `CardDefinition::counts_own_name_casts`): a non-empty map in the hot
    /// group costs an allocation per `PlayerData` unshare, PERF `(-149)`. A
    /// second per-name tally by another route than
    /// `Predicate::CastOwnNameThisGameAtLeast` must widen the gate.
    #[serde(with = "crate::static_str_serde::map_u32", default)]
    pub spells_cast_by_name_this_game:
        crate::fxhash::HashMap<crate::static_str_serde::StaticStr, u32>,
    /// Like `spells_cast_this_turn` but reset for every player at each
    /// turn's Cleanup (not just the player's own untap) — the CR-correct
    /// scope for Rule of Law's "each player can't cast more than one spell
    /// each turn" (CR 611.2). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub spells_cast_this_game_turn: u32,
    /// Like `spells_cast_this_game_turn` but counting only noncreature spells
    /// (Deafening Silence's "each player can't cast more than one noncreature
    /// spell each turn"). Reset for every player at Cleanup. `#[serde(default)]`.
    #[serde(default)]
    pub noncreature_spells_cast_this_game_turn: u32,
    /// Like `spells_cast_this_game_turn` but counting only nonartifact spells
    /// (Ethersworn Canonist). Reset for every player at Cleanup. `#[serde(default)]`.
    #[serde(default)]
    pub nonartifact_spells_cast_this_game_turn: u32,
    /// Like `spells_cast_this_game_turn` but counting only non-Phyrexian
    /// spells (Phyrexian Censor). Reset at Cleanup.
    #[serde(default)]
    pub non_phyrexian_spells_cast_this_game_turn: u32,
    /// CR 706 — dice this player rolled this turn, planar die included
    /// (The Space Family Goblinson, Resolute Veggiesaur). Reset for every
    /// player at Cleanup.
    #[serde(default)]
    pub dice_rolled_this_turn: u32,
    /// Bamboozling Beeble — the seat that will choose which of this player's
    /// next roll's dice (one extra) to ignore, this turn. Cleared at Cleanup.
    #[serde(default)]
    pub next_roll_extra_die_by: Option<usize>,
    /// "You can cast only N more spells this turn" (Irencrag Feat): the
    /// `spells_cast_this_game_turn` count at which casting stops. Reset at
    /// Cleanup.
    #[serde(default)]
    pub spell_cap_this_turn: Option<u32>,
    /// Life gained this turn (`GainLife` and `Drain` to this player), reset
    /// in `do_untap`. Powers Infusion ("if you gained life this turn").
    #[serde(default)]
    pub life_gained_this_turn: u32,
    /// Set once this player's life-gain trigger batch finishes dispatching
    /// this turn: "first time each turn" filters (Leech Collector) read it
    /// before it flips, so only the first batch qualifies. Reset for every
    /// player at each untap ("each turn" keys on the turn, not the owner's).
    #[serde(default)]
    pub gained_life_earlier_this_turn: bool,
    /// Event batches this turn in which this player created tokens / surveiled
    /// (`Predicate::FirstBatchThisTurn`). Reset for every player each turn.
    #[serde(default)]
    pub token_batches_this_turn: u8,
    #[serde(default)]
    pub surveil_batches_this_turn: u8,
    /// Event batches this turn in which this player scried (CR 701.22);
    /// with `surveil_batches_this_turn`, "if you've scried or surveilled
    /// this turn" (`Predicate::ScriedOrSurveiledThisTurn`).
    #[serde(default)]
    pub scry_batches_this_turn: u8,
    /// Event batches this turn in which this player lost life
    /// (`FirstBatchKind::LifeLost`).
    #[serde(default)]
    pub life_loss_batches_this_turn: u8,
    /// Cards drawn this turn, reset in `do_untap`. Powers Quandrix scaling
    /// (Fractal Anomaly); surfaced in `PlayerView` for UI previews.
    #[serde(default)]
    pub cards_drawn_this_turn: u32,
    /// Cards drawn during the current step — reset for every player on each
    /// step change. Powers "except the first card they draw in their draw
    /// step" trigger gates (Orcish Bowmasters, CR 603.3).
    #[serde(default)]
    pub cards_drawn_this_step: u32,
    /// Times a card left this graveyard this turn, reset in `do_untap`
    /// (Lorehold payoffs: Living History, Primary Research, Wilt in the
    /// Heat). Bumped by `move_card_to`'s `CardLeftGraveyard` emission.
    #[serde(default)]
    pub cards_left_graveyard_this_turn: u32,
    /// The creature cards among `cards_left_graveyard_this_turn` ("if a
    /// creature card left your graveyard this turn" — Syrix, Carrier of the
    /// Flame). Bumped beside it, reset with it.
    #[serde(default)]
    pub creature_cards_left_graveyard_this_turn: u32,
    /// Creatures this player controlled that died this turn, reset in
    /// `do_untap` and bumped by the SBA dies handler (Witherbloom end-step
    /// payoffs, Essenceknit Scholar).
    #[serde(default)]
    pub creatures_died_this_turn: u32,
    /// Zubera that died under this player's control this turn. Reset in
    /// `do_untap`. Powers the Champions-of-Kamigawa Zubera cycle's "for each
    /// Zubera that died this turn" death triggers. `#[serde(default)]`.
    #[serde(default)]
    pub zuberas_died_this_turn: u32,
    /// CR 603 — count of artifacts that entered under this player's control
    /// this turn. Drives "if an artifact entered the battlefield under your
    /// control this turn" intervening-`if` gates (Akal Pakal). Reset at the
    /// active player's turn boundary.
    #[serde(default)]
    pub artifacts_entered_this_turn: u32,
    /// The Food subset of `artifacts_entered_this_turn` — "if a Food entered
    /// under your control this turn" (Motivated Pony).
    #[serde(default)]
    pub foods_entered_this_turn: u32,
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
    /// CR 701.49d — dungeons this player has completed this game.
    #[serde(default)]
    pub dungeons_completed: u32,
    /// CR 702.139 — true if a permanent left the battlefield under this
    /// player's control so far this turn (Revolt). Set from the battlefield-
    /// removal funnels keyed off the leaving permanent's controller; reset at
    /// the player's `do_untap`. Defaults to false for snapshot back-compat.
    #[serde(default)]
    pub permanent_left_battlefield_this_turn: bool,
    /// The creature subset of `permanent_left_battlefield_this_turn` ("if a
    /// creature left the battlefield under your control this turn" — Tale of
    /// Momo), read last-known before the creature left. Reset with it.
    #[serde(default)]
    pub creature_left_battlefield_this_turn: bool,
    /// How many creatures `creature_left_battlefield_this_turn` has seen
    /// (Kutzil's Flanker). Reset with it.
    #[serde(default)]
    pub creatures_left_battlefield_count: u32,
    /// The bending actions this player took this turn, one
    /// [`crate::effect::BendKind::bit`] each (Avatar Aang). Reset per turn.
    #[serde(default)]
    pub bends_this_turn: u8,
    /// Dealt any damage this turn (set in `deal_damage_to_from`, incl.
    /// infect); reset for all players at `do_untap`. The Bloodthirst window
    /// (CR 702.54).
    #[serde(default)]
    pub was_dealt_damage_this_turn: bool,
    /// How much damage this player has been dealt this turn, for
    /// "Bloodthirst X"-style riders that scale with the amount (Petrified
    /// Wood-Kin). Cleared with `was_dealt_damage_this_turn` at turn start.
    #[serde(default)]
    pub damage_taken_this_turn: u32,
    /// The largest total one source dealt one object or player this turn,
    /// as far as this seat's record goes (Impact Resonance): damage to this
    /// player per source, and the tallies of permanents this player owned
    /// that left the battlefield. Cleared at turn start.
    #[serde(default)]
    pub greatest_hit_this_turn: u32,
    /// Damage each source has dealt this player this turn, feeding
    /// `greatest_hit_this_turn`. Cleared at turn start.
    #[serde(default)]
    pub damage_by_source_this_turn: crate::copyvec::CopyVec<[(crate::card::CardId, u32); 2]>,
    /// The combat-damage-only slice of `damage_taken_this_turn` ("if a player
    /// was dealt 6 or more combat damage this turn" — Sidequest: Play
    /// Blitzball). Cleared alongside it at the turn boundary.
    #[serde(default)]
    pub combat_damage_taken_this_turn: u32,
    /// This player was dealt noncombat damage during the previous turn (any
    /// player's) — rolled from this turn's tallies at the turn boundary
    /// (Command the Stage's "if an opponent was dealt noncombat damage last turn").
    #[serde(default)]
    pub dealt_noncombat_damage_last_turn: bool,
    /// Moonlit Meditation — the once-per-turn "first token batch becomes
    /// copies" replacement has already fired this turn.
    #[serde(default)]
    pub token_copy_replacement_used_this_turn: bool,
    /// Tokens created under this player's control this turn (Thalisse,
    /// Reverent Medium). Counted at the mint funnel, cleared each turn.
    #[serde(default)]
    pub tokens_created_this_turn: u32,
    /// The Treasures among them (Vazi, Keen Negotiator).
    #[serde(default)]
    pub treasures_created_this_turn: u32,
    /// Cards this player cycled this turn (CR 702.29; Spellpyre Phoenix,
    /// Gavi's first-cycle-free). Cleared each turn.
    #[serde(default)]
    pub cards_cycled_this_turn: u32,
    /// The card being cycled right now, while its discard is placed — so
    /// Abandoned Sarcophagus's "if it wasn't cycled" redirect can tell.
    #[serde(default)]
    pub cycling_card: Option<crate::card::CardId>,
    /// Kaya, Geist Hunter's −2: this turn's extra token doublings (CR
    /// 614.13), added to the permanent doublers. Cleared each turn.
    #[serde(default)]
    pub token_doublings_this_turn: u8,
    /// This player declared a commander as an attacker this turn (CR 508.1;
    /// Neriv's "during any turn you attacked with a commander").
    #[serde(default)]
    pub attacked_with_commander_this_turn: bool,
    /// Crafty Cutpurse — tokens an opponent would create this turn are
    /// created under this player's control instead (CR 614).
    #[serde(default)]
    pub steals_opponent_tokens_this_turn: bool,
    /// True if this player has lost life this turn (damage or direct life
    /// loss). Set in `adjust_life` on a negative delta, reset at the active
    /// player's `do_untap`. Powers Spectacle (CR 702.137). Defaults to false
    /// for snapshot back-compat.
    #[serde(default)]
    pub lost_life_this_turn: bool,
    /// `lost_life_this_turn` as the previous turn ended — "if you lost life
    /// last turn" (Paladin of Atonement).
    #[serde(default)]
    pub lost_life_last_turn: bool,
    /// Total life lost this turn (CR 119.3). Bumped alongside
    /// `lost_life_this_turn`, reset with it. Powers `Value::LifeLostThisTurn`
    /// (Spinerock Knoll's "an opponent lost 7 or more life" gate).
    #[serde(default)]
    pub life_lost_this_turn: u32,
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
    /// player this turn. Set in `fire_combat_damage_to_player_triggers`,
    /// reset at the turn boundary.
    #[serde(default)]
    pub dealt_combat_damage_to_player_this_turn: bool,
    /// CR 702.173a — a player was dealt combat damage this turn by an Assassin
    /// creature or a commander this player controlled: their freerunning
    /// costs are open. Reset at the turn boundary.
    #[serde(default)]
    pub freerunning_unlocked_this_turn: bool,
    /// Seats (a bit each) whose Assassin attacked this player this turn
    /// (Ramses, Assassin Lord). Cleared as each turn begins.
    #[serde(default)]
    pub attacked_by_assassins_of: u64,
    /// CR 614.5 — "sources you control deal double damage this turn" (Quest
    /// for Pure Flame). Read in `scale_damage_to`, reset at the turn boundary.
    #[serde(default)]
    pub double_your_source_damage_this_turn: bool,
    /// The Flame of Keld's chapter III — this player's red sources deal this
    /// much extra damage to any permanent or player this turn (CR 614.1a).
    #[serde(default)]
    pub red_source_damage_bonus_this_turn: u32,
    /// Isengard Unleashed — this player's sources deal triple damage to their
    /// opponents and opponents' permanents this turn. Cleared at cleanup.
    #[serde(default)]
    pub triple_your_source_damage_to_opponents_this_turn: bool,
    /// Goblin Goliath — this player's sources deal double damage to their
    /// opponents (players only) this turn. Cleared at cleanup.
    #[serde(default)]
    pub double_your_source_damage_to_opponents_this_turn: bool,
    /// Hallowed Moonlight — until end of turn a creature that wasn't cast is
    /// exiled instead of entering, for every player. Cleared at cleanup.
    #[serde(default)]
    pub exiles_uncast_creatures_this_turn: bool,
    /// CR 305.1 — "target player can't play lands this turn" (Turf Wound).
    /// Consulted by `can_player_play_land`, reset at the turn boundary.
    #[serde(default)]
    pub cant_play_lands_this_turn: bool,
    /// Abeyance — "that player can't activate abilities that aren't mana
    /// abilities" this turn. Cleared at the turn boundary.
    #[serde(default)]
    pub cant_activate_nonmana_abilities_this_turn: bool,
    /// Winding Canyons — "you may cast creature spells this turn as though they
    /// had flash". Cleared at the turn boundary.
    #[serde(default)]
    pub creature_spells_as_flash_this_turn: bool,
    /// Complete the Circuit — this player's sorcery spells have flash this
    /// turn. Cleared at the turn boundary.
    #[serde(default)]
    pub sorcery_spells_as_flash_this_turn: bool,
    /// Veilstone Amulet — creatures this player controls can't be the targets
    /// of spells or abilities their opponents control this turn.
    #[serde(default)]
    pub creatures_untargetable_by_opponents_this_turn: bool,
    /// Bösium Strip — "until end of turn, you may cast instant and sorcery
    /// spells from the top of your graveyard". Cleared at the turn boundary.
    #[serde(default)]
    pub cast_from_graveyard_top_this_turn: bool,
    /// CR 700.13 — this player has committed a crime this turn (cast a spell or
    /// activated an ability targeting an opponent / their stuff). Set when a
    /// `CommittedCrime` event fires, reset at the turn boundary. Powers
    /// "if you've committed a crime this turn" gates (Nimble Brigand).
    #[serde(default)]
    pub committed_crime_this_turn: bool,
    /// CR 701.16a — this player has investigated this turn; the next
    /// `GameEvent::Investigated` is their first when it is false.
    #[serde(default)]
    pub investigated_this_turn: bool,
    /// A permanent was put into this player's hand from the battlefield this
    /// turn (Barrin, Tolarian Archmage's end-step draw).
    #[serde(default)]
    pub permanent_to_hand_this_turn: bool,
    /// CR 708 — a permanent entered the battlefield face down under this
    /// player's control, or they turned a permanent face up, this turn. Powers
    /// Oblivious Bookworm's "unless … entered face down / you turned a permanent
    /// face up this turn" discard-skip. Reset at the turn boundary.
    #[serde(default)]
    pub face_down_activity_this_turn: bool,
    /// Warped Space — this player has already used their once-per-turn
    /// "pay {0} rather than pay the mana cost for a spell you cast from
    /// exile". Reset at the turn boundary.
    #[serde(default)]
    pub free_exile_cast_used_this_turn: bool,
    /// Demon of Fate's Design — this player has used their once-per-turn
    /// "pay life equal to its mana value" cast. Reset at the turn boundary.
    #[serde(default)]
    pub life_alt_cast_used_this_turn: bool,
    /// Darksteel Monolith — this player has used their once-each-turn {0}
    /// cast. Reset at the turn boundary.
    #[serde(default)]
    pub zero_alt_cast_used_this_turn: bool,
    /// Aminatou's Augury — per exiled group (keyed by the `exiled_with`
    /// stamp), the card types this player may still cast a free spell of
    /// this turn, as a bitmask (`fateshift::TYPE_BITS`). Cleared at the turn
    /// boundary.
    #[serde(default)]
    pub free_type_cast_budgets: Vec<(crate::card::CardId, u8)>,
    /// Marshland Bloodcaster — the next spell this player casts this turn may
    /// be paid for with life equal to its mana value. Spent by that cast;
    /// reset at the turn boundary.
    #[serde(default)]
    pub life_alt_next_spell_this_turn: bool,
    /// The players this player attacked during their most recent turn (CR
    /// 508.1). Cleared as their next turn begins — O-Kagachi's "if that player
    /// attacked you during their last turn".
    #[serde(default)]
    pub attacked_players_this_turn: Vec<usize>,
    /// "The next spell you cast this turn has convoke" (Wand of the
    /// Worldsoul, Flockchaser Phantom). Spent by that cast; reset at the turn
    /// boundary.
    #[serde(default)]
    pub next_spell_convoke_this_turn: bool,
    /// "The next instant or sorcery spell you cast this turn has storm"
    /// (Storm, Force of Nature). Spent by that cast; reset at the turn boundary.
    #[serde(default)]
    pub next_is_spell_storm_this_turn: bool,
    /// Colors whose unspent mana this player keeps as steps end, until the
    /// cleanup step (The Last Agni Kai's red).
    #[serde(default)]
    pub kept_colors_this_turn: crate::mana::ColorSet,
    /// "The next spell you cast this turn has improvise" (Archway of
    /// Innovation). Spent by that cast; reset at the turn boundary.
    #[serde(default)]
    pub next_spell_improvise_this_turn: bool,
    /// "The next spell you cast this turn can be cast as though it had flash"
    /// (Ride the Avalanche). Spent by that cast; reset at the turn boundary.
    #[serde(default)]
    pub next_spell_flash_this_turn: bool,
    /// Progenitor's Icon — one entry per activation: the next spell of that
    /// creature type this turn has flash. Spent by that cast; cleared at the
    /// turn boundary.
    #[serde(default)]
    pub next_typed_spell_flash_this_turn: Vec<crate::card::CreatureType>,
    /// "You may cast spells this turn as though they had flash" (Alchemist's
    /// Refuge). Reset at the turn boundary.
    #[serde(default)]
    pub spells_as_flash_this_turn: bool,
    /// Creatures exiled from under this player's control this turn (Vren, the
    /// Relentless' end-step Rat count). Reset at the turn boundary.
    #[serde(default)]
    pub creatures_exiled_from_control_this_turn: u32,
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
    /// "Prevent all damage that would be dealt to you this turn" (Selfless
    /// Squire). Cleared at cleanup.
    #[serde(default)]
    pub all_damage_prevented_this_turn: bool,
    /// "Until your next turn, prevent all damage that would be dealt to you"
    /// (Morningtide's Light). Cleared as this player's turn begins.
    #[serde(default)]
    pub all_damage_prevented_until_next_turn: bool,
    /// "Until your next turn, you can't be attacked except by creatures with
    /// flying and/or islandwalk" (Island Sanctuary): only attackers matching
    /// the filter may attack this player. Cleared as this player's turn
    /// begins (`game::draw_skip_shield`).
    #[serde(default)]
    pub attackable_only_by_until_next_turn: Option<Box<crate::card::SelectionRequirement>>,
    /// CR 101.2 — "until your next turn, players can't play cards from their
    /// hand" (Memory Vessel): the seat whose next turn lifts the lock.
    #[serde(default)]
    pub hand_locked_until_turn_of: Option<usize>,
    /// Non-mana activated abilities this player activated this turn — the
    /// Commander bot's activation-loop brake (`server::activation_cap`).
    /// Reset for every seat as each turn begins.
    #[serde(default)]
    pub nonmana_activations_this_turn: u32,
    /// CR 700.2 — the dilemmas (mode choices for a spell or ability) this
    /// player faced this turn (The Caretaker, Captain Benjamin Sisko). Reset
    /// for every seat as each turn begins.
    #[serde(default)]
    pub dilemmas_faced_this_turn: u32,
    /// Permanents returned to this player's hand from the battlefield this
    /// turn (Miles O'Brien, Transporter Ops), tallied off
    /// `GameEvent::PermanentReturnedToHand` at dispatch. Reset each turn.
    #[serde(default)]
    pub returned_to_hand_this_turn: u32,
    /// CR 702.16 — "you gain protection from the color of your choice until
    /// end of turn" (Seht's Tiger): sources of these colors can't target this
    /// player and their damage to them is prevented. Cleared at cleanup.
    #[serde(default)]
    pub protection_colors_eot: crate::mana::ColorSet,
    /// CR 702.16 — "you gain protection from [player]": bit `q` set means
    /// nothing seat `q` controls can target this player or deal them damage.
    /// This mask ends at cleanup; the next one as this player's turn begins
    /// (Guardian Archon, Eon Frolicker).
    #[serde(default)]
    pub protected_from_seats_eot: u64,
    #[serde(default)]
    pub protected_from_seats_until_next_turn: u64,
    /// "Can't block with more than N creatures this combat" (Mirri,
    /// Weatherlight Duelist's attack trigger). Cleared as combat ends.
    #[serde(default)]
    pub block_cap_this_combat: Option<u8>,
    /// CR 401.6 — turn-scoped "look at the top card of your library any time;
    /// play lands and cast spells from it" (The Belligerent). A player flag
    /// set at resolution and cleared at end of turn, unlike
    /// `StaticEffect::PlayFromLibraryTop`. Read by `library_top_playable`.
    #[serde(default)]
    pub play_from_top_this_turn: bool,
    /// `play_from_top_this_turn`'s spells cost life, not mana (Gwenom).
    #[serde(default)]
    pub play_from_top_pay_life_this_turn: bool,
    /// Xanathar, Guild Kingpin — seats whose library top this player may
    /// look at and play this turn, spending mana as though it were mana of
    /// any color for those spells. Cleared at cleanup.
    #[serde(default)]
    pub may_play_library_top_of: Vec<usize>,
    /// Sen Triplets — seats from whose hand this player may play lands and
    /// cast spells this turn; those hands are revealed. Cleared at cleanup.
    #[serde(default)]
    pub may_play_hand_of: Vec<usize>,
    /// CR 603.3d-adjacent — set once this player casts/plays a card from the
    /// top of their library under a `PlayFromLibraryTopOncePerTurn` grant
    /// (Johann, Apprentice Sorcerer). Cleared at end of turn. Read by
    /// `library_top_playable` to enforce the once-per-turn cap.
    #[serde(default)]
    pub cast_from_library_top_this_turn: bool,
    /// Cards this player put into exile this turn, reset in `do_untap`
    /// (Ennis the Debate Moderator). Bumped by `place_card_in_dest`'s exile
    /// branch and `Effect::Exile`'s battlefield path.
    #[serde(default)]
    pub cards_exiled_this_turn: u32,
    /// Cards put into this player's graveyard from anywhere this turn,
    /// bumped at the graveyard-placement funnels and reset at the turn
    /// boundary (`Predicate::OpponentCardsToGraveyardThisTurnAtLeast`,
    /// Ravenous Trap).
    #[serde(default)]
    pub cards_to_graveyard_this_turn: u32,
    /// Creature cards put into this player's graveyard from anywhere this
    /// turn. Powers `Predicate::CreatureCardsToGraveyardThisTurnAtLeast`
    /// (Case of the Gorgon's Kiss).
    #[serde(default)]
    pub creature_cards_to_graveyard_this_turn: u32,
    /// Generic mana every spell this player casts costs more this turn —
    /// Academy Loremaster's "spells they cast this turn cost {2} more".
    /// Reset for every seat at the turn boundary.
    #[serde(default)]
    pub spell_tax_this_turn: u32,
    /// Kicked spells this player has cast this turn — Vine Gecko's "the first
    /// kicked spell you cast each turn". Reset at the turn boundary.
    #[serde(default)]
    pub kicked_spells_cast_this_turn: u32,
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
    /// Instant/sorcery spells cast this turn (bumped in `finalize_cast`,
    /// reset in `do_untap`): the precise gate for "if you've cast an instant
    /// or sorcery spell this turn" (Potioner's Trove).
    #[serde(default)]
    pub instants_or_sorceries_cast_this_turn: u32,
    /// The greatest mana value (X included, CR 202.3e) among the instant and
    /// sorcery spells this player cast this turn. Stamped in `finalize_cast`,
    /// cleared with `instants_or_sorceries_cast_this_turn`. Rootha.
    #[serde(default)]
    pub greatest_is_mana_value_this_turn: u32,
    /// Spells cast **from hand** this turn (CR 601); casts from exile,
    /// graveyard or command zone don't count. Gates "if you haven't cast a
    /// spell from your hand this turn" (Prairie Dog). Reset each turn.
    #[serde(default)]
    pub spells_cast_from_hand_this_turn: u32,
    /// Spells this player cast from exile this turn (Wild-Magic Sorcerer's
    /// "the first spell you cast from exile each turn"). Reset each turn.
    #[serde(default)]
    pub spells_cast_from_exile_this_turn: u32,
    /// This player played a land from exile this turn (Visions of
    /// Phyrexia's "if you didn't play a card from exile this turn", with
    /// `spells_cast_from_exile_this_turn`). Reset each turn.
    #[serde(default)]
    pub played_land_from_exile_this_turn: bool,
    /// This player played a land from anywhere other than their hand this
    /// turn (Spider-Man 2099). Reset each turn.
    #[serde(default)]
    pub played_land_not_from_hand_this_turn: bool,
    /// Instant and sorcery spells this player cast from a graveyard this turn
    /// (Sevinne, the Chronoclasm's "your first … each turn"). Reset each turn.
    #[serde(default)]
    pub instants_sorceries_cast_from_graveyard_this_turn: u32,
    /// Transient Hardened-Scales bonus granted "until end of turn" (Prairie
    /// Dog's {4}{W}). Adds to `plus_counter_adders_for`; cleared at cleanup.
    #[serde(default)]
    pub extra_plus_one_counters_this_turn: u32,
    /// "Until end of turn, if one or more counters would be put on a permanent
    /// you control, that many plus one of each kind" (Brad Boimler): adds to
    /// every placement on this player's permanents, whoever places; cleared at
    /// cleanup.
    #[serde(default)]
    pub extra_counters_this_turn: u32,
    /// Combine Guildmage — "this turn, each creature you control enters with an
    /// additional +1/+1 counter." Read at the ETB-counter site; cleared at
    /// cleanup. Distinct from `extra_plus_one_counters_this_turn` (which
    /// amplifies counters *placed*, Hardened-Scales style).
    #[serde(default)]
    pub extra_etb_p1p1_counters_this_turn: u32,
    /// The same grant "until your next turn" (Arlinn, the Pack's Hope's +1);
    /// cleared as this player's next turn begins.
    #[serde(default)]
    pub extra_etb_p1p1_until_your_turn: u32,
    /// "Until your next turn, you may cast creature spells as though they
    /// had flash" (Arlinn); cleared as this player's next turn begins.
    #[serde(default)]
    pub creature_flash_until_your_turn: bool,
    /// Cards this player discarded this turn (Hollow One's cost reduction).
    /// Bumped in `discard_card`; reset in `do_untap`.
    #[serde(default)]
    pub cards_discarded_this_turn: u32,
    /// Number of permanents this player has sacrificed so far this turn.
    /// Bumped in `dispatch_triggers_for_events` per `PermanentSacrificed`
    /// event; reset in `do_untap`. Powers "if you sacrificed a permanent
    /// this turn" payoffs (Sawblade Skinripper).
    #[serde(default)]
    pub permanents_sacrificed_this_turn: u32,
    /// Artifacts sacrificed this turn (subset of
    /// `permanents_sacrificed_this_turn`), bumped per artifact
    /// `PermanentSacrificed` event, reset in `do_untap` (Suspicious
    /// Detonation, Furtive Courier).
    #[serde(default)]
    pub artifacts_sacrificed_this_turn: u32,
    /// Nontoken permanents this player has sacrificed this turn (a subset of
    /// `permanents_sacrificed_this_turn`; Evendo Brushrazer). Same clock.
    #[serde(default)]
    pub nontoken_sacrificed_this_turn: u32,
    /// Foods this player has sacrificed this turn (a subset of
    /// `permanents_sacrificed_this_turn`; Elanor Gardner). Same clock.
    #[serde(default)]
    pub foods_sacrificed_this_turn: u32,
    /// Clues this player has sacrificed this turn (Briarbridge Patrol's
    /// "if you sacrificed three or more Clues this turn"). Same clock.
    #[serde(default)]
    pub clues_sacrificed_this_turn: u32,
    /// Artifacts and/or creatures this player has sacrificed this turn (a
    /// subset of `permanents_sacrificed_this_turn`; Dargo, the Shipwrecker).
    /// Same clock.
    #[serde(default)]
    pub artifacts_or_creatures_sacrificed_this_turn: u32,
    /// The permanent card types (one bit each: artifact, battle, creature,
    /// enchantment, kindred, land, planeswalker) among permanents this player
    /// sacrificed this turn (Korvold, Gleeful Glutton). Same clock.
    #[serde(default)]
    pub sacrificed_card_types_this_turn: u8,
    /// "Face-down spells you cast this turn cost {N} less to cast" (Goblin
    /// Maskmaker). Summed into `face_down_cast_cost`; cleared in
    /// `finish_cleanup` alongside `turn_spell_discounts`.
    #[serde(default)]
    pub face_down_discount_this_turn: u32,
    /// "The next face-down creature spell you cast this turn costs {N} less"
    /// (Panoptic Projektor). Summed into `face_down_cast_cost`, spent by the
    /// next face-down cast, cleared in `finish_cleanup`.
    #[serde(default)]
    pub next_face_down_discount_this_turn: u32,
    /// Face-down creature spells this player has cast this turn (Kadena,
    /// Slinking Sorcerer's "first … each turn" discount); cleared in
    /// `finish_cleanup` with the face-down discounts.
    #[serde(default)]
    pub face_down_spells_cast_this_turn: u32,
    /// Creature spells cast this turn, reset in `do_untap` (creature-cast
    /// magecraft payoffs).
    #[serde(default)]
    pub creatures_cast_this_turn: u32,
    /// CR 700.6 — historic spells (artifact, legendary or Saga) cast this
    /// turn; reset with `creatures_cast_this_turn`. Peri Brown's "the first
    /// historic spell you cast each turn".
    #[serde(default)]
    pub historic_spells_cast_this_turn: u32,
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
    /// CR 702.179 — speed (0–4). "Start your engines!" sets it to 1; it rises
    /// by 1, at most once per own turn, the first time an opponent loses
    /// life that turn. "Max speed —" abilities are active at 4.
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
    /// Experience counters (CR 122 / 721-era Commander mechanic). A per-player
    /// resource that only accumulates; payoffs read the count (Mizzix's cost
    /// reduction, Ezuri's +1/+1 distribution). Added by
    /// `Effect::AddExperience`. Default 0 for snapshot back-compat.
    #[serde(default)]
    pub experience: u32,
    /// CR 122.1i / 729 — rad counters on this player. At the start of
    /// their precombat main phase they mill that many cards; for each
    /// nonland milled, they lose 1 life and shed a rad counter (handled
    /// as a turn-based action in `do_rad_counters`). Default 0.
    #[serde(default)]
    pub rad_counters: u32,
    /// CR 123.2 — ticket counters ({TK}), from Command Performance.
    #[serde(default)]
    pub tickets: u32,
    /// CR 700.6 / 702.131 — true once this player has the city's blessing.
    /// Granted by an Ascend ability/permanent while they control ten or more
    /// permanents; once obtained it lasts for the rest of the game. Default
    /// false for snapshot back-compat.
    #[serde(default)]
    pub city_blessing: bool,
    /// CR 402.2 — maximum hand size: `Some(n)` (normally 7) or `None` for no
    /// maximum (Reliquary Tower). Read by `do_cleanup`'s CR 514.1 discard;
    /// set by `Effect::SetNoMaxHandSize` / `Effect::SetMaxHandSize` (Null
    /// Profusion). Old snapshots default to `Some(7)`.
    #[serde(default = "default_max_hand_size")]
    pub max_hand_size: Option<usize>,
    /// True once this player has lost the game (life ≤ 0, poison ≥ 10, or
    /// drew from an empty library). Eliminated players are skipped by turn
    /// and priority rotation; the game ends when ≤ 1 player remains.
    pub eliminated: bool,
    /// True once CR 800.4a has been applied for this seat (objects gone,
    /// stack items ceased, granted control effects ended).
    ///
    /// Separate from [`eliminated`](Self::eliminated): an effect that ends a
    /// game (unpaid Pact, `Effect::LoseGame`, win-the-game) sets only that,
    /// and the SBA sweep skips eliminated seats, so the leave pass never ran
    /// for them. The sweep drives the pass off this flag, so CR 800.4a
    /// happens in one place however a seat went out.
    #[serde(default)]
    pub left_game: bool,
    /// CR 104.3c — drew from an empty library; loses at the next SBA check,
    /// not inside the draw, so an "each opponent" later in the same
    /// resolution still sees them. `check_state_based_actions` promotes this
    /// to `eliminated` (and so through CR 800.4a).
    #[serde(default)]
    pub pending_deck_loss: bool,
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
    /// Why this player was eliminated, recorded when `eliminated` flips, so
    /// consumers (server win-kind stats) read the real cause instead of
    /// guessing from the board. `None` while still in the game.
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
    /// Forbidding Spirit's temporary Propaganda tax ({N} per attacker at this
    /// player or their planeswalkers), set on ETB, cleared at this player's
    /// untap. Summed with `AttackTaxToController` in `declare_attackers`.
    #[serde(default)]
    pub attack_tax_until_your_turn: u32,
    /// Sivitri, Dragon Master's +1 — the same window, paid in **life** per
    /// attacker (CR 119.4 gates it on the attacker's life total).
    #[serde(default)]
    pub attack_life_tax_until_your_turn: u32,
    /// Upcoming turns this player must skip (`Effect::SkipTurns`, Ral
    /// Zarek): `do_cleanup` decrements it and passes the turn on.
    #[serde(default)]
    pub skip_turns: u32,
    /// Names of the spells this player has cast this turn (Grim Reminder).
    /// Cleared at untap.
    #[serde(with = "crate::static_str_serde::vec", default)]
    pub spell_names_cast_this_turn: crate::copyvec::CopyVec<[crate::static_str_serde::StaticStr; 1]>,
    /// Card ids of the spells this player has cast this turn, in cast order —
    /// "the first instant spell … you've cast this turn" (Alania, Divergent
    /// Storm) resolves each id through `find_card_anywhere`. Cleared with
    /// `spell_names_cast_this_turn`.
    #[serde(default)]
    pub spell_ids_cast_this_turn: crate::copyvec::CopyVec<[crate::card::CardId; 4]>,
    /// Turn number through which this player may cast creature spells from
    /// their graveyard by foraging (Osteomancer Adept). Cleared at cleanup.
    #[serde(default)]
    pub forage_graveyard_casts_turn: Option<u32>,
    /// The last turn this player activated a power-up ability (CR 702.193;
    /// Advancing the Spirit's "first power-up ability … each of your turns").
    #[serde(default)]
    pub power_up_turn: Option<u32>,
    /// "You may cast a creature spell from your graveyard this turn"
    /// (Chainer, Nightmare Adept): the turn it was granted, cleared by the
    /// one cast it allows. Read by `graveyard_creature_cast_available`.
    #[serde(default)]
    pub graveyard_creature_cast_turn: Option<u32>,
    /// The Tomb of Aclazotz's rider on that cast: it enters with a finality
    /// counter and is this creature type in addition. Cleared with it.
    #[serde(default)]
    pub graveyard_creature_cast_rider: Option<crate::card::CreatureType>,
    /// CR 502.3 — number of this player's upcoming untap steps to skip
    /// (Yosei, the Morning Star; Frost Titan-style locks). Decremented when
    /// their untap step would run; while > 0 their permanents don't untap.
    #[serde(default)]
    pub skip_next_untap_step: u32,
    /// CR 504 — charges of "skip your next draw step" (Fatigue). Consumed one
    /// per draw step.
    #[serde(default)]
    pub skip_next_draw_step: u32,
    /// CR 506 — number of this player's upcoming combat phases to skip
    /// (Stonehorn Dignitary). Consumed when their turn reaches Begin Combat,
    /// jumping straight to the postcombat main. Defaults to 0 for snapshot
    /// back-compat.
    #[serde(default)]
    pub skip_next_combat: u32,
    /// "Can't attack [defender] or permanents they control during their next
    /// turn" (The Second Doctor): `(defender, turn registered)`, live on this
    /// player's first turn after the registration, dropped at its cleanup.
    #[serde(default)]
    pub next_turn_attack_bans: Vec<(usize, u32)>,
    /// Number of this player's upcoming untap steps in which the **lands** they
    /// control don't untap (Bontu's Last Reckoning). Decremented when their
    /// untap step runs; non-land permanents untap normally. `#[serde(default)]`.
    #[serde(default)]
    pub lands_dont_untap_next_untap: u32,
    /// Deep Water — while set, every land this player taps for mana produces
    /// this colour instead of its own. Cleared at end of turn.
    #[serde(default)]
    pub lands_produce_color_this_turn: Option<crate::mana::Color>,
    /// Dark Sphere — pending "prevent half the next damage dealt to you,
    /// rounded down" shields. Cleared at end of turn.
    #[serde(default)]
    pub half_damage_shields: u32,
    /// Eye for an Eye — pending mirrors: the next damage dealt to this player
    /// is also dealt to the damaging source's controller. Cleared at end of
    /// turn.
    #[serde(default)]
    pub damage_mirrors: u32,
    /// Blood of the Martyr — damage that would be dealt to any creature may
    /// be dealt to this player instead. Cleared at end of turn.
    #[serde(default)]
    pub creature_damage_to_you_this_turn: bool,
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
    /// CR 500.7 — extra turns this player will take. When `advance_turn`
    /// would pass the turn, an active player with `extra_turns > 0`
    /// decrements it and keeps the turn instead (Time Walk, Ral Zarek's
    /// -7 coin-flip emblem). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub extra_turns: u32,
    /// Bit `i` marks the extra turn at bank depth `i` as power-up-locked
    /// (Kang the Conqueror); the newest extra turn is taken first (CR 500.7).
    #[serde(default)]
    pub power_up_locked_extra: u32,
    /// The turn this player began a power-up-locked extra turn: no power-up
    /// ability can be activated during it, by anyone.
    #[serde(default)]
    pub power_up_locked_turn: Option<u32>,
    /// Turns this player has begun this game after the first, bumped by
    /// `advance_turn` as it hands them the turn (`Value::TurnsBegunBy`, which
    /// adds the game's first turn back for the starting seat).
    /// `#[serde(default)]`.
    #[serde(default)]
    pub turns_begun: u32,
    /// CR 119.7 — a battlefield effect stops this player gaining life.
    /// Recomputed by `compute_battlefield` from `StaticEffect::CannotGainLife`;
    /// `GameState::adjust_life` drops positive deltas while set.
    #[serde(default)]
    pub cannot_gain_life: bool,
    /// Sticky one-turn "can't gain life" lock (`Effect::LifeGainLockThisTurn`,
    /// Skullcrack), reset in `do_untap`. Same effect as `cannot_gain_life`
    /// but survives recomputes since no permanent backs it.
    #[serde(default)]
    pub cannot_gain_life_this_turn: bool,
    /// True while this player's life total can't change for the rest of the
    /// turn (Flare of Fortitude). Set by `Effect::LockLifeTotalThisTurn`,
    /// reset in `do_untap`. `adjust_life` treats it as both cannot-gain and
    /// cannot-lose, so any nonzero delta is dropped on the floor.
    #[serde(default)]
    pub life_locked_this_turn: bool,
    /// "Their life total can't change" until this player's next turn
    /// (Teferi's Reproach) — read beside `life_locked_this_turn`, cleared as
    /// that player's turn begins.
    #[serde(default)]
    pub life_locked_until_next_turn: bool,
    /// Spells this player controls can't be countered this turn (Veil of
    /// Summer). Set by `Effect::GrantSpellsUncounterableThisTurn`, reset at
    /// `do_untap`, read by `caster_grants_uncounterable_with_x`.
    #[serde(default)]
    pub spells_uncounterable_this_turn: bool,
    /// Like `spells_uncounterable_this_turn` but only for *creature* spells
    /// (Domri, Anarch of Bolas's +1). Reset alongside it at untap.
    #[serde(default)]
    pub creature_spells_uncounterable_this_turn: bool,
    /// Cast a blue or black spell this turn (set in `finalize_cast`, reset at
    /// `do_untap`) — Veil of Summer's draw gate.
    #[serde(default)]
    pub cast_blue_or_black_this_turn: bool,
    /// True while this player has hexproof until the start of their next
    /// turn (Blossoming Calm). Set by `Effect::GainHexproofUntilYourNextTurn`;
    /// cleared at this player's `do_untap`. `#[serde(default)]`.
    #[serde(default)]
    pub hexproof_until_next_turn: bool,
    /// Hexproof until end of turn (Everybody Lives!); cleared at the next
    /// untap.
    #[serde(default)]
    pub hexproof_this_turn: bool,
    /// "Can't lose life this turn" (Everybody Lives!) — life can still be
    /// gained; cleared at the next untap.
    #[serde(default)]
    pub cant_lose_life_this_turn: bool,
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
    /// This turn the player cast a spell from a graveyard or activated an
    /// ability of a card in a graveyard (Laboratory Drudge).
    #[serde(default)]
    pub used_graveyard_this_turn: bool,
    /// Gaea's Will — this player's graveyard-bound cards exile instead this
    /// turn (CR 614.6, own cards only).
    #[serde(default)]
    pub graveyard_bound_exiled_this_turn: bool,
    /// When true, decisions this player would make suspend via
    /// `pending_decision` so a UI can respond; when false, the engine calls
    /// the installed `Decider` synchronously (bot / tests).
    pub wants_ui: bool,
    /// CR 601.2g "proper tapping": this player picks their own mana sources,
    /// so auto-tap runs only when the payment is forced and otherwise the
    /// cast fails with `ManualTapRequired`.
    ///
    /// Separate from [`wants_ui`](Self::wants_ui), which once stood in for
    /// it: bots set `wants_ui` (their policies run off `pending_decision`)
    /// but must not hand-pick lands, and conflating the two broke every bot
    /// cast that needed a tap.
    #[serde(default)]
    pub manual_mana: bool,
    /// Auto-tap spends the most *replaceable* source for each pip instead of
    /// the first in battlefield order (see `ManaSourceInfo::redundancy`).
    ///
    /// Off: measured null (50.3 % [49.95, 50.68] over 28,800 paired games)
    /// on a field that does contain the target case, and it once carried a
    /// quadratic regression past the Commander bot test's timeout. Kept for
    /// re-measuring. Per-player, not a constant, because tapping happens in
    /// the engine — applied to both seats a mirror can't show anything.
    #[serde(default)]
    pub smart_tap: bool,
    /// Converge auto-tap (bot seats): a dual takes the fresh colour with the
    /// FEWEST other untapped sources, not the first in WUBRG order (which
    /// paid Paradox Gardens blue beside an Island and lost the green; 2026-09-06
    /// deck work). Pushed from `EvalWeights::converge_rarest`; off is the
    /// ladder control.
    pub converge_rarest: bool,
    /// Auto-target polarity for player slots (bot seats): a hostile "target
    /// player" / "any target" slot (`Effect::player_slot_is_hostile`) tries
    /// the opponent first. Off, the caster comes first (Arcane Omens
    /// discarded its own caster's hand; 2026-09-06 deck work). Pushed from
    /// `EvalWeights::hostile_player_targets`; off is the ladder control.
    pub hostile_player_targets: bool,
    /// Commander-era ability sinks in a duel too (bot seats): counter/level,
    /// scry/surveil/loot, the dry-run generic sink and basic landcycling.
    /// Evasion grants stay Commander-only (they lose duels). Pushed from
    /// `EvalWeights::duel_sinks`; off is the ladder control.
    pub duel_sinks: bool,
    /// A modal trigger's automatic mode pick skips a plain "tap target
    /// creature" mode when the tap can't matter (our turn after combat, or
    /// before it with nothing able to attack) and an "untap target creature"
    /// mode with nothing of ours tapped. Pushed from
    /// `EvalWeights::tap_timing`; off is the ladder control.
    pub tap_timing: bool,
    /// Colour fixing for a splash (bot seats): landcycling, scry / surveil,
    /// land recursion and a mill-then-take pick (Something Worth Saving's
    /// `Effect::MillThenToHandN`, answered by the engine's Auto rule) go
    /// for a land that makes a colour a spell in hand is stranded on. Pushed
    /// from `EvalWeights::color_fixing`; off is the ladder control.
    #[serde(default)]
    pub color_fixing: bool,
    /// CR 705.3 — Krark's Thumb coin-flip advantage: `Effect::FlipCoin` does
    /// `1 + N` flips and the flipper wins if any is heads (each Thumb lets
    /// you ignore one flip). Meant to stack additively across Thumbs once
    /// wired to a static; for now the Thumb's card body sets it directly.
    #[serde(default)]
    pub coin_flip_advantage: u32,
    /// This player flipped a coin this turn (Edgar, King of Figaro's
    /// Two-Headed Coin reads it). Cleared at each turn's start.
    #[serde(default)]
    pub flipped_coin_this_turn: bool,
    /// `graveyard_ids_this_turn.len()` as the last combat began: the ids at or
    /// past it reached the graveyard this combat (Storrev, Devkarin Lich).
    #[serde(default)]
    pub graveyard_ids_at_combat: u32,
}

/// A seat: a copy-on-write handle around [`PlayerData`].
///
/// `PlayerData` is ~165 fields, and `GameState::clone` (every checkpoint and
/// bot probe) would copy them all; behind the `Arc` a clone is a refcount
/// bump and only a written seat deep-copies. `Deref`/`DerefMut` keep
/// `player.life -= 2` working (it unshares that seat first). Same shape as
/// [`crate::card::CardInstance`].
#[derive(Clone)]
pub struct Player {
    data: std::sync::Arc<PlayerData>,
    /// Count of `&mut` reaches into this seat — the per-object twin of
    /// [`crate::zone::Battlefield`]'s `writes` (PERF `(-306)`) and the
    /// players half of a state-level gather memo's key (PERF `(-303)`).
    ///
    /// Free to maintain: every `&mut` reach already goes through
    /// [`DerefMut`](std::ops::DerefMut) below, which is why `pub players`
    /// needs no `players_mut()` refactor. Clones carry it, so a probe
    /// diverges from its parent only once it writes; only a *change* is read.
    writes: u32,
}

impl Player {

    /// A card left this player's graveyard (CR 400.7): one more for
    /// `cards_left_graveyard_this_turn`, and for the creature-card count when
    /// it's a creature card — a card's printed type, as it's no permanent.
    pub fn note_card_left_graveyard(&mut self, card: &crate::card::CardInstance) {
        self.cards_left_graveyard_this_turn = self.cards_left_graveyard_this_turn.saturating_add(1);
        if card.definition.is_creature() {
            self.creature_cards_left_graveyard_this_turn += 1;
        }
    }
    /// CR 702.16 — this player has protection from seat `q` (either mask).
    pub fn protected_from_seat(&self, q: usize) -> bool {
        let bit = 1u64.checked_shl(q as u32).unwrap_or(0);
        (self.protected_from_seats_eot | self.protected_from_seats_until_next_turn) & bit != 0
    }

    /// This seat's write counter — see [`Player::writes`].
    #[inline]
    pub fn writes(&self) -> u32 {
        self.writes
    }
}

/// Field access on the cold group reads like a `PlayerData` field.
impl std::ops::Deref for PlayerData {
    type Target = PlayerCold;
    #[inline]
    fn deref(&self) -> &PlayerCold {
        &self.cold
    }
}

/// Writing any cold field unshares the whole group once — see [`PlayerCold`].
impl std::ops::DerefMut for PlayerData {
    #[inline]
    fn deref_mut(&mut self) -> &mut PlayerCold {
        &mut self.cold
    }
}

impl std::ops::Deref for Player {
    type Target = PlayerData;
    #[inline]
    fn deref(&self) -> &PlayerData {
        &self.data
    }
}

/// The unshare point: every `&mut` reach into a seat goes through here, so a
/// shared seat is cloned exactly once and then written in place.
impl std::ops::DerefMut for Player {
    #[inline]
    fn deref_mut(&mut self) -> &mut PlayerData {
        // PERF `(-178)`: the same Weak-free uniqueness check as the other two
        // CoW handles — this `Arc` is a private field nothing downgrades.
        self.writes = self.writes.wrapping_add(1);
        crate::cow::make_mut(&mut self.data)
    }
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&*self.data, f)
    }
}

impl From<PlayerData> for Player {
    fn from(data: PlayerData) -> Self {
        Self { data: std::sync::Arc::new(data), writes: 0 }
    }
}

// Transparent serde: the wire format is `PlayerData`'s, so snapshots written
// before the handle existed round-trip unchanged.
impl Serialize for Player {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (*self.data).serialize(s)
    }
}

impl<'de> Deserialize<'de> for Player {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        PlayerData::deserialize(d).map(Self::from)
    }
}

impl Player {
    /// CR 111.7 / 704.5d — the cards in this player's graveyard: a token that
    /// died this resolution sits there until the next state-based check, and
    /// a token is never a card.
    pub fn graveyard_card_count(&self) -> usize {
        self.graveyard.iter().filter(|c| !c.is_token).count()
    }

    pub fn new(idx: usize, name: impl Into<String>) -> Self {
        Self::from(PlayerData {
            id: PlayerId(idx),
            cold: crate::cow::CowBox::new(PlayerCold {
                ante: CowBox::default(),
                scheme_deck: CowBox::default(),
                planar_deck: CowBox::default(),
                attraction_deck: CowBox::default(),
                attraction_junkyard: CowBox::default(),
                name: name.into(),
                ..Default::default()
            }),
            life: 20,
            life_at_turn_start: 20,
            starting_life: 20,
            mana_pool: ManaPool::new(),
            kept_mana_this_turn: ManaPool::new(),
            library: crate::zone::CardPile::library(),
            hand: crate::zone::CardPile::default(),
            graveyard: crate::zone::Graveyard::default(),
            command: CowBox::default(),
            may_spend_any_color_this_turn: false,
            planar_die_rolls_this_turn: 0,
            sideboard: CowBox::default(),
            lands_played_this_turn: 0,
            tapped_land_for_mana_this_turn: false,
            extra_land_plays: 0,
            cant_play_lands_this_turn: false,
            cant_activate_nonmana_abilities_this_turn: false,
            creature_spells_as_flash_this_turn: false,
            sorcery_spells_as_flash_this_turn: false,
            creatures_untargetable_by_opponents_this_turn: false,
            spells_as_flash_this_turn: false,
            cast_from_graveyard_top_this_turn: false,
            extra_loyalty_activations: 0,
            loyalty_copy_grants: Vec::new(),
            activated_loyalty_this_turn: false,
            spells_cast_this_turn: 0,
            sorceries_cast_this_turn: 0,
            spells_cast_by_name_this_game: crate::fxhash::HashMap::default(),
            spells_cast_this_game_turn: 0,
            noncreature_spells_cast_this_game_turn: 0,
            nonartifact_spells_cast_this_game_turn: 0,
            non_phyrexian_spells_cast_this_game_turn: 0,
            dice_rolled_this_turn: 0,
            next_roll_extra_die_by: None,
            spell_cap_this_turn: None,
            life_gained_this_turn: 0,
            gained_life_earlier_this_turn: false,
            token_batches_this_turn: 0,
            life_loss_batches_this_turn: 0,
            surveil_batches_this_turn: 0,
            scry_batches_this_turn: 0,
            cards_drawn_this_turn: 0,
            cards_drawn_this_step: 0,
            cards_left_graveyard_this_turn: 0,
            creature_cards_left_graveyard_this_turn: 0,
            creatures_died_this_turn: 0,
            zuberas_died_this_turn: 0,
            creatures_entered_this_turn: Default::default(),
            creatures_entered_last_turn: Default::default(),
            artifacts_entered_this_turn: 0,
            foods_entered_this_turn: 0,
            planeswalkers_entered_this_turn: 0,
            nonland_permanents_entered_this_turn: 0,
            mounts_vehicles_entered_this_turn: 0,
            multicolored_spells_cast_this_turn: 0,
            oil_activity_this_turn: false,
            channel_life_for_mana: false,
            dungeon: None,
            dungeons_completed: 0,
            pending_creature_etb_counters: Default::default(),
            permanent_left_battlefield_this_turn: false,
            creature_left_battlefield_this_turn: false,
            creatures_left_battlefield_count: 0,
            bends_this_turn: 0,
            was_dealt_damage_this_turn: false,
            damage_taken_this_turn: 0,
            greatest_hit_this_turn: 0,
            damage_by_source_this_turn: crate::copyvec::CopyVec::new(),
            combat_damage_taken_this_turn: 0,
            dealt_noncombat_damage_last_turn: false,
            token_copy_replacement_used_this_turn: false,
            tokens_created_this_turn: 0,
            treasures_created_this_turn: 0,
            cards_cycled_this_turn: 0,
            cycling_card: None,
            token_doublings_this_turn: 0,
            attacked_with_commander_this_turn: false,
            steals_opponent_tokens_this_turn: false,
            lost_life_this_turn: false,
            play_from_top_this_turn: false,
            may_play_library_top_of: Vec::new(),
            may_play_hand_of: Vec::new(),
            play_from_top_pay_life_this_turn: false,
            cast_from_library_top_this_turn: false,
            life_lost_this_turn: 0,
            lost_life_last_turn: false,
            creatures_that_damaged_me_this_turn: Default::default(),
            creatures_that_combat_damaged_me_this_turn: Default::default(),
            lands_entered_this_turn: 0,
            creature_spell_countered_by_opponent_this_turn: false,
            noncreature_destroyed_by_opponent_this_turn: false,
            prowl_types_this_turn: Default::default(),
            prowl_any_type_this_turn: false,
            attacked_this_turn: false,
            creatures_attacked_this_turn: 0,
            dealt_combat_damage_to_player_this_turn: false,
            freerunning_unlocked_this_turn: false,
            attacked_by_assassins_of: 0,
            double_your_source_damage_this_turn: false,
            red_source_damage_bonus_this_turn: 0,
            triple_your_source_damage_to_opponents_this_turn: false,
            double_your_source_damage_to_opponents_this_turn: false,
            exiles_uncast_creatures_this_turn: false,
            committed_crime_this_turn: false,
            investigated_this_turn: false,
            permanent_to_hand_this_turn: false,
            face_down_activity_this_turn: false,
            free_exile_cast_used_this_turn: false,
            life_alt_cast_used_this_turn: false,
            zero_alt_cast_used_this_turn: false,
            free_type_cast_budgets: Vec::new(),
            life_alt_next_spell_this_turn: false,
            attacked_players_this_turn: Vec::new(),
            next_spell_convoke_this_turn: false,
            next_is_spell_storm_this_turn: false,
            kept_colors_this_turn: crate::mana::ColorSet::empty(),
            next_spell_improvise_this_turn: false,
            next_spell_flash_this_turn: false,
            next_typed_spell_flash_this_turn: Vec::new(),
            creatures_exiled_from_control_this_turn: 0,
            descended_this_turn: false,
            descend_count_this_turn: 0,
            silenced_this_turn: false,
            warped_spell_this_turn: false,
            searched_library_this_turn: false,
            shroud_this_turn: false,
            protected_from_everything: false,
            all_damage_prevented_this_turn: false,
            all_damage_prevented_until_next_turn: false,
            attackable_only_by_until_next_turn: None,
            hand_locked_until_turn_of: None,
            nonmana_activations_this_turn: 0,
            dilemmas_faced_this_turn: 0,
            returned_to_hand_this_turn: 0,
            protection_colors_eot: crate::mana::ColorSet::empty(),
            protected_from_seats_eot: 0,
            protected_from_seats_until_next_turn: 0,
            block_cap_this_combat: None,
            cards_exiled_this_turn: 0,
            cards_to_graveyard_this_turn: 0,
            creature_cards_to_graveyard_this_turn: 0,
            spell_tax_this_turn: 0,
            kicked_spells_cast_this_turn: 0,
            graveyard_ids_this_turn: Default::default(),
            milled_ids_this_turn: Default::default(),
            surveiled_this_turn: 0,
            instants_or_sorceries_cast_this_turn: 0,
            greatest_is_mana_value_this_turn: 0,
            spells_cast_from_hand_this_turn: 0,
            spells_cast_from_exile_this_turn: 0,
            played_land_from_exile_this_turn: false,
            played_land_not_from_hand_this_turn: false,
            instants_sorceries_cast_from_graveyard_this_turn: 0,
            extra_plus_one_counters_this_turn: 0,
            extra_counters_this_turn: 0,
            extra_etb_p1p1_counters_this_turn: 0,
            extra_etb_p1p1_until_your_turn: 0,
            creature_flash_until_your_turn: false,
            pending_is_discounts: Default::default(),
            pending_spell_discounts: Default::default(),
            next_matching_spell_discounts: Default::default(),
            pending_affinity_next_spell: Default::default(),
            free_next_spells: Default::default(),
            pending_affinity_next_noncreature: Default::default(),
            face_down_discount_this_turn: 0,
            next_face_down_discount_this_turn: 0,
            face_down_spells_cast_this_turn: 0,
            cards_discarded_this_turn: 0,
            permanents_sacrificed_this_turn: 0,
            artifacts_sacrificed_this_turn: 0,
            nontoken_sacrificed_this_turn: 0,
            foods_sacrificed_this_turn: 0,
            clues_sacrificed_this_turn: 0,
            artifacts_or_creatures_sacrificed_this_turn: 0,
            sacrificed_card_types_this_turn: 0,
            creatures_cast_this_turn: 0,
            historic_spells_cast_this_turn: 0,
            cannot_gain_life_this_turn: false,
            life_locked_this_turn: false,
            life_locked_until_next_turn: false,
            spells_uncounterable_this_turn: false,
            creature_spells_uncounterable_this_turn: false,
            hexproof_until_next_turn: false,
            hexproof_this_turn: false,
            cant_lose_life_this_turn: false,
            cast_blue_or_black_this_turn: false,
            spell_casts_this_turn: crate::copyvec::CopyVec::new(),
            cant_cast_noncreature_this_turn: false,
            free_spells_from_hand_this_turn: false,
            play_from_graveyard_this_turn: false,
            used_graveyard_this_turn: false,
            graveyard_bound_exiled_this_turn: false,
            first_spell_tax_charges: 0,
            sorceries_as_flash: false,
            poison_counters: 0,
            tickets: 0,
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
            left_game: false,
            pending_deck_loss: false,
            last_drawn_card: None,
            range_of_influence: None,
            is_emperor: false,
            loss_cause: None,
            cant_lose_this_turn: false,
            damage_floor_this_turn: false,
            attack_tax_until_your_turn: 0,
            attack_life_tax_until_your_turn: 0,
            spell_names_cast_this_turn: crate::copyvec::CopyVec::new(),
            spell_ids_cast_this_turn: crate::copyvec::CopyVec::new(),
            forage_graveyard_casts_turn: None,
            power_up_turn: None,
            graveyard_creature_cast_turn: None,
            graveyard_creature_cast_rider: None,
            skip_turns: 0,
            skip_next_untap_step: 0,
            skip_next_draw_step: 0,
            skip_next_combat: 0,
            next_turn_attack_bans: Vec::new(),
            lands_dont_untap_next_untap: 0,
            lands_produce_color_this_turn: None,
            half_damage_shields: 0,
            damage_mirrors: 0,
            creature_damage_to_you_this_turn: false,
            creatures_dont_untap_next_untap: 0,
            firebending_kept_red: 0,
            extra_turns: 0,
            power_up_locked_extra: 0,
            power_up_locked_turn: None,
            turns_begun: 0,
            cannot_gain_life: false,
            wants_ui: false,
            smart_tap: false,
            converge_rarest: false,
            hostile_player_targets: false,
            duel_sinks: false,
            tap_timing: false,
            color_fixing: false,
            manual_mana: false,
            coin_flip_advantage: 0,
            flipped_coin_this_turn: false,
            graveyard_ids_at_combat: 0,
        })
    }
}

impl PlayerData {
    /// Fold `amount` damage from `src` into this player's per-source tally
    /// and `greatest_hit_this_turn` (2014-11-07 Impact Resonance ruling: one
    /// source's total to one victim). A sourceless hit counts on its own.
    pub fn record_damage_from(&mut self, src: Option<crate::card::CardId>, amount: u32) {
        let total = match src {
            None => amount,
            Some(src) => match self.damage_by_source_this_turn.iter_mut().find(|(s, _)| *s == src) {
                Some((_, n)) => {
                    *n = n.saturating_add(amount);
                    *n
                }
                None => {
                    self.damage_by_source_this_turn.push((src, amount));
                    amount
                }
            },
        };
        self.greatest_hit_this_turn = self.greatest_hit_this_turn.max(total);
    }

    pub fn is_alive(&self) -> bool {
        !self.eliminated
    }

    /// Vanilla CR 305.2 check: no land played this turn. Does **not** consult
    /// `StaticEffect::ExtraLandPerTurn`; the CR-correct check is
    /// `GameState::can_player_play_land(seat)`.
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
        let mut card = self.library.remove(0);
        // CR 400.7 — a "play the top card" permission stamped on it ends as
        // it is drawn (Reforge the Soul sat in hand with it).
        card.forget_spell_choices();
        let id = card.id;
        self.hand.push(card);
        self.cards_drawn_this_turn = self.cards_drawn_this_turn.saturating_add(1);
        self.cards_drawn_this_step = self.cards_drawn_this_step.saturating_add(1);
        Some(id)
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
        // CR 122.2 — counters cease to exist on zone change (dies-with-counters
        // triggers read LKI). Guarded: a `clear()` of an empty map still
        // deep-copies a shared card through `Arc::make_mut`.
        card.drop_counters_for_zone_change(crate::card::Zone::Graveyard);
        // CR 400.7 — nor a resolved spell's kicker (Into the Roil).
        card.forget_spell_choices();
        // CR 400.7 / 607 — a link to the exile it once left is not this
        // object's (a card cast from exile kept it). Read first: the write
        // is a `make_mut`.
        if card.exiled_with.is_some() || card.exiled_by.is_some() {
            card.exiled_with = None;
            card.exiled_by = None;
        }
        // CR 111.8 — a token that dies is no card: it feeds none of the
        // "cards put into your graveyard" tallies, and doesn't descend
        // (CR 700.11 "a permanent card"). It sits here until 704.5d.
        if !card.is_token {
            self.cards_to_graveyard_this_turn += 1;
            if card.definition.is_creature() {
                self.creature_cards_to_graveyard_this_turn += 1;
            }
            self.graveyard_ids_this_turn.insert(card.id);
            // CR 700.11 — descending requires a *permanent* card hitting the gy.
            if card.definition.is_permanent() {
                self.descended_this_turn = true;
                self.descend_count_this_turn += 1;
            }
        }
        self.graveyard.push(card);
    }

    /// Push a card definition directly into the library (top of deck = index 0).
    pub fn add_to_library_top(
        &mut self,
        id: CardId,
        definition: impl Into<std::sync::Arc<CardDefinition>>,
    ) {
        self.library.insert(0, Self::placed_in_library(id, definition.into(), self.id.0));
    }

    /// Push a card definition to the bottom of the library. Takes
    /// `impl Into<Arc<_>>` so a memoized definition
    /// ([`crate::cube::card_arc`]) is a refcount, not a fresh 8 KB `Arc`
    /// copy (deck construction's largest `__memcpy`).
    pub fn add_to_library_bottom(
        &mut self,
        id: CardId,
        definition: impl Into<std::sync::Arc<CardDefinition>>,
    ) {
        self.library.push(Self::placed_in_library(id, definition.into(), self.id.0));
    }

    /// A card built straight into a library (deck construction, fixtures):
    /// stamped as already there, so it isn't "put into a library".
    fn placed_in_library(id: CardId, definition: std::sync::Arc<CardDefinition>, owner: usize) -> CardInstance {
        let mut card = CardInstance::new(id, definition, owner);
        card.in_library = true;
        card
    }
}
