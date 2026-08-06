//! Primitive, data-driven effect system.
//!
//! Replaces the earlier variant-per-effect `SpellEffect`/`TriggerCondition`/
//! `EffectCondition`/`StaticAbilityTemplate` quartet with a small set of
//! composable algebras:
//!
//! * [`Selector`] — lazy reference to game objects or players, resolved at
//!   effect-time.
//! * [`Value`]    — numeric expression (counts, life totals, X).
//! * [`Predicate`]— game-state boolean (for conditional effects).
//! * [`Effect`]   — the unified instruction tree executed by the resolver.
//! * [`EventSpec`]— structural trigger filter over the [`GameEvent`] stream.
//! * [`StaticEffect`] — description of a static ability's continuous effect.
//!
//! Everything that was previously a one-off enum variant lives as a tree of
//! these primitives; a new card rarely needs engine changes.

use serde::{Deserialize, Serialize};

use crate::card::{CounterType, Keyword, LandType, SelectionRequirement, TokenDefinition, Zone};
use crate::mana::{Color, SpendRestriction};

// ── PlayerRef / ZoneRef ───────────────────────────────────────────────────────

/// Lightweight reference to one or more players.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerRef {
    /// The controller of the spell this resolution just countered (Fold into
    /// Aether's "its controller may put a creature card …"). Resolves to
    /// `GameState.countered_spell_controller`, stamped by the counter path.
    CounteredSpellController,
    /// The seat that said yes to the innermost [`Effect::AnyPlayerMayAccept`]
    /// offer (the punisher family's "have this deal damage to *them*").
    /// Resolves to `GameState.accepting_player`.
    AcceptingPlayer,
    /// CR 701.38 — the player who cast the vote a `VoteTally::PerVote` body is
    /// running for ("choose a permanent owned by the voter" — Expropriate).
    /// Falls back to the controller outside a ballot.
    CurrentVoter,
    /// The controller of the ability/spell.
    You,
    /// A specific chosen target slot (must resolve to a player).
    Target(u8),
    /// Each opponent of the controller.
    EachOpponent,
    /// Every player in turn order.
    EachPlayer,
    /// CR 804.2 — every *other* seat on the controller's team ("target
    /// teammate"). Empty in a free-for-all, where every team is a singleton.
    EachTeammate,
    /// Every player whose CR 702.179 speed is below max (< 4) — "each player
    /// who doesn't have max speed" (Outpace Oblivion). A player with no speed
    /// (never started their engines) counts as below max.
    EachPlayerWithoutMaxSpeed,
    /// The active player (whose turn it is).
    ActivePlayer,
    /// The single player with more life than every other (Ghazbán Ogre).
    /// Resolves to nothing when two or more players are tied for the lead.
    PlayerWithMostLife,
    /// The owner of a selected entity.
    OwnerOf(Box<Selector>),
    /// The owner of the card currently being moved (resolved per-card by
    /// `place_card_in_dest`). Lets one `Move`/`Search` route *each* card to
    /// its own owner — "return all attacking creatures to their owners'
    /// hands" (Aetherize, Evacuation). Has no meaning outside a placement
    /// context, where it resolves to `None`.
    OwnerOfMoved,
    /// The controller of a selected entity.
    ControllerOf(Box<Selector>),
    /// Every player except the controller of the selected entity, in APNAP
    /// order ("each player other than its controller" — Fractured Identity).
    EachPlayerExceptControllerOf(Box<Selector>),
    /// CR 303.4a — the player the source Aura enchants ("enchanted player":
    /// the Curse cycle, Psychic Possession). Resolves to nothing when the
    /// source isn't attached to a player.
    EnchantedPlayer,
    /// An opponent of the referenced player. The singular resolver takes the
    /// first alive one; `resolve_players` returns all of them. Lets a
    /// per-player body name "one of *their* opponents" (Bend or Break).
    OpponentOf(Box<PlayerRef>),
    /// The player who triggered the event (for triggered abilities).
    /// Every opponent other than the trigger's own player — "each other
    /// opponent" on a trigger whose player is one of them (Grenzo's Ruffians).
    EachOpponentExceptTriggerer,
    Triggerer,
    /// The seat that *caused* the firing event — the caster for a
    /// `BecameTarget`, the damaged seat for a combat-damage trigger. Reads
    /// the trigger's stamped actor (`GameState::trigger_event_player_scratch`),
    /// so it resolves "that spell or ability's controller" (Lava Runner)
    /// where `Triggerer` would give the targeted permanent's controller.
    TriggerEventPlayer,
    /// A specific seat index. Used internally to flatten selector-based
    /// player refs (e.g. `OwnerOf(Selector)`) into a concrete seat before
    /// passing them across context boundaries — the original-card lookup
    /// can become stale once the card has been moved out of its source zone.
    Seat(usize),
    /// The player or planeswalker controller being attacked by the source
    /// creature. Resolves to `None` when the source isn't currently
    /// attacking. Used for "defending player" triggers (Goblin Guide,
    /// Hypnotic Specter).
    DefendingPlayer,
    /// The controller of the source that most recently dealt combat damage to
    /// the permanent the inner selector resolves to (reads
    /// `CardInstance.combat_damager_controller`). Survives the combat teardown
    /// that clears `block_map`, so a "whenever this is dealt combat damage"
    /// trigger can name the attacking player (Souls of the Faultless).
    CombatDamagerController(Box<Selector>),
    /// The controller of the most recent source that dealt damage to the
    /// permanent the inner selector resolves to this turn (reads the last entry
    /// of `CardInstance.damaged_by_this_turn`). Unlike `CombatDamagerController`
    /// this covers noncombat damage too — Belltower Sphinx's "that source's
    /// controller mills that many."
    LastDamagerControllerOf(Box<Selector>),
    /// The player with the lowest life total, ties broken by the resolving
    /// controller's choice (auto-picks the earliest seat). Loxodon Peacekeeper.
    LowestLife,
    /// The player with the highest life total, ties broken toward the earliest
    /// seat. Wild Dogs' "the player with the most life gains control".
    HighestLife,
    /// The player with the most cards in hand, ties broken toward the earliest
    /// seat. Sokenzan Renegade's "the player who has the most cards in hand
    /// gains control of this."
    MostCardsInHand,
    /// The player controlling the most creatures; ties go to the earliest seat
    /// (Wild Mammoth's upkeep defection).
    MostCreatures,
    /// The single player controlling strictly more permanents matching the
    /// filter than every other player (Thoughtbound Primoc). Unresolved on a
    /// tie, so the effect does nothing.
    MostControlledMatching(Box<SelectionRequirement>),
    /// The player the source remembered (`CardInstance.chosen_player`, stamped
    /// by [`Effect::RememberPlayerOnSource`]) — "that player" on a later
    /// trigger. Soul Scourge, Laquatus's Champion.
    ChosenPlayerOfSource,
    /// CR 701.38 — each opponent whose vote in the most recent ballot differed
    /// from the effect's controller's (Grudge Keeper).
    OpponentsWhoVotedDifferently,
}

/// Which players a player-targeted static effect affects. The static
/// is anchored on a permanent (the "source") and reads off that
/// source's controller seat at recompute time. Used by
/// `StaticEffect::PlayerCannotGainLife` and any future player-static
/// (lose-life redirection, hand-size caps, draw caps, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerStaticTarget {
    /// The source's controller — Sulfuric Vortex's "no player can gain
    /// life" applied to the controller-only side. Rare.
    Controller,
    /// Each opponent of the source's controller — the default for the
    /// printed "your opponents can't gain life" wording (Erebos,
    /// Rampaging Ferocidon, Tainted Remedy approximation).
    EachOpponent,
    /// Every player on the table — Sulfuric Vortex (each player can't
    /// gain life), Stigma Lasher's "permanents you control share the
    /// no-lifegain rider" template.
    EachPlayer,
    /// The player the source Aura enchants (`CardInstance.attached_to_player`
    /// — Grievous Wound's "enchanted player can't gain life").
    EnchantedPlayer,
}

/// A zone plus optional owner (for zones like Hand/Library/Graveyard that
/// are per-player). Battlefield, Stack, Command are global.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneRef {
    Library(PlayerRef),
    Hand(PlayerRef),
    Graveyard(PlayerRef),
    Exile,
    Battlefield,
    Stack,
    Command,
}

// ── Selector ─────────────────────────────────────────────────────────────────

/// A lazy reference to a (possibly empty, possibly multi-) set of game
/// objects — permanents, cards in other zones, or players.
///
/// Resolved by the effect engine at execution time against the current game
/// state; used as the operand of most [`Effect`] mutations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selector {
    /// The source permanent/spell/ability itself.
    This,
    /// The ability/spell's controller as a "player object" (for damage, etc).
    You,
    /// A chosen target slot from the cast-time target list.
    Target(u8),
    /// A chosen target slot with a restriction that must be validated at cast time.
    TargetFiltered { slot: u8, filter: SelectionRequirement },
    /// Every declared target slot (permanents/players still legal). Powers
    /// "then [do X] to each of those [targets]" riders that operate on the
    /// whole cast-time target list rather than a single slot — Biogenic
    /// Upgrade ("distribute counters among 1–3 creatures, then double the
    /// counters on each of those creatures").
    AllTargets,
    /// "…a card at random from among them" — resolve the inner selector and
    /// keep one entity chosen uniformly at random (Volatile Chimera).
    RandomOf(Box<Selector>),
    /// The object that caused this trigger (attacker, dying creature, etc).
    TriggerSource,
    /// The permanent the source chose and remembered as it entered — the
    /// `CardInstance.chosen_permanent` stamp (Dauntless Bodyguard's
    /// "the chosen creature"). Resolves to nothing if the remembered
    /// permanent has left the battlefield.
    ChosenPermanentOfSource,
    /// CR 509.1 — every attacker the source permanent is currently blocking
    /// (all of them under a multi-block). Resolves via
    /// `GameState.block_map[source]`. Used by "whenever this blocks a creature,
    /// [affect that creature]" triggers (Wall of Frost).
    BlockedAttacker,
    /// The mirror of `BlockedAttacker`: every creature currently blocking the
    /// source attacker (reverse-lookup of `GameState.block_map`). Used by
    /// "whenever this becomes blocked by a creature, [affect that creature]"
    /// triggers (Grasping Giant).
    BlockingCreatures,
    /// Every creature blocking or blocked by the (resolved) subject creature —
    /// the symmetric combat-partner set. Trial // Error's "return all creatures
    /// blocking or blocked by target creature". Excludes the subject itself.
    CreaturesInCombatWith(Box<Selector>),
    /// The permanent whose ability created the source token
    /// (`CardInstance.created_by`) — empty once it has left the battlefield.
    /// Backs a token CDA that reads counters on its creator (Saproling Burst).
    CreatorOfSource,
    /// Every battlefield token the source's abilities created
    /// (`CardInstance.created_by`) — "all tokens created with this
    /// enchantment" (Saproling Burst).
    TokensCreatedBySource,
    /// Every creature the source blocked *this turn*, read from
    /// `CardInstance.blocked_attackers_this_turn` — so it still resolves after
    /// combat has been torn down (Defiant Vanguard's end-of-combat sweep).
    CreaturesBlockedBySourceThisTurn,
    /// CR 702.171 — the creatures that saddled the source this turn
    /// (`CardInstance.saddled_by`; Calamity, The Gitrog).
    CreaturesThatSaddledSource,
    /// The same read, but scoped to whatever `Selector` resolves to rather
    /// than the effect's source — "creatures that were blocked by target Wall
    /// this turn" (the Legends Glyph cycle).
    CreaturesBlockedByThisTurn(Box<Selector>),
    /// CR 702.76 — the card hidden (exiled) by the source via Hideaway: the
    /// exile-zone card stamped `exiled_with == ctx.source`. Resolves to that
    /// single card so the activated ability can play it from exile.
    CardExiledWithSource,

    /// The last card the effect's controller drew this turn, if it's still in
    /// their hand (`Player.last_drawn_card`). Sindbad's "discard it if it isn't
    /// a land", Jandor's Ring's discard cost.
    LastCardYouDrew,

    /// The most-recently-created token from `Effect::CreateToken` in
    /// the current resolution. Used by Quandrix-style "create a token,
    /// then put X +1/+1 counters on it" cards (Fractal Anomaly,
    /// Applied Geometry). Resets between resolution roots — within a
    /// single `Effect::Seq`, the latest CreateToken's id is visible.
    LastCreatedToken,
    /// The card `Effect::RevealRandomFromHand` last revealed this resolution
    /// (Planeswalker's Mischief exiles it). Empty when nothing was revealed.
    LastRevealedCard,

    /// All tokens created by `Effect::CreateToken` in the current
    /// resolution (the multi-token variant of `LastCreatedToken`). Used
    /// by Fractal Spawning ("create two 0/0 Fractals, put a +1/+1
    /// counter on each of them") and any multi-mint-then-counter
    /// printed Oracle. Resets between resolution roots; within an
    /// `Effect::Seq`, every CreateToken from the current resolution
    /// is included. Push: modern_decks batch 28.
    LastCreatedTokens,

    /// All cards moved by `Effect::Move` (and Mill / Exile shortcuts)
    /// in the current resolution. Used by Practiced Scrollsmith,
    /// Suspend Aggression, Tablet of Discovery, Ark of Hunger, etc.
    /// to chain a `GrantMayPlay` immediately after the Move targets
    /// the same card(s). Cleared between resolution roots.
    LastMoved,
    /// The most recent source that dealt damage to the permanent the inner
    /// selector resolves to this turn (the last entry of
    /// `CardInstance.damaged_by_this_turn`) — Vraska the Unseen's "destroy
    /// that creature".
    LastDamagerOf(Box<Selector>),
    /// Every creature in the sector picked by the enclosing
    /// `Effect::ChooseSector`.
    CreaturesInChosenSector,
    /// The permanent sacrificed to pay the current spell's / ability's cost
    /// (`GameState.sacrificed_card`, stamped by `Effect::WithSacrificedPt`).
    /// Now a card in its owner's graveyard — Rescue from the Underworld
    /// reanimates it alongside its target.
    SacrificedCard,
    /// The card(s) exiled from a graveyard to pay this activation's cost
    /// (Necropolis). Empty outside such an activation.
    CostExiledCards,
    /// Every player and planeswalker the source has dealt damage to this
    /// game (The Fallen).
    DamagedBySourceThisGame,

    /// The chosen target slot (0-indexed) of the spell whose cast
    /// triggered this ability. Resolves against the topmost matching
    /// `StackItem::Spell` (the just-cast spell whose `SpellCast` event
    /// produced this trigger). Empty when the trigger source isn't a
    /// spell or the slot is unfilled. Used by Strixhaven Repartee
    /// payoff effects whose body operates on the spell's target rather
    /// than choosing a fresh one — e.g. Conciliator's Duelist's "exile
    /// up to one *target* creature".
    CastSpellTarget(u8),
    /// Every permanent the just-cast spell targets, across all its slots
    /// ("gain control of those permanents" — Dack Fayden's emblem).
    AllCastSpellTargets,

    /// CR 702 Radiance (Ravnica) — the `subject` permanent plus every other
    /// permanent on the battlefield that shares a card type with it (creatures
    /// for the usual cards, enchantments for Leave No Trace) and shares a
    /// computed color with it. A colorless subject resolves to just itself.
    /// Lets a non-damage body (untap + pump for Rally the Righteous, destroy
    /// for Leave No Trace) fan out the same way `Effect::RadianceDamage` does.
    /// `subject` is usually a target slot.
    RadianceGroup { subject: Box<Selector> },

    /// All game objects matching `filter` in `zone`.
    EachMatching { zone: ZoneRef, filter: SelectionRequirement },
    /// All permanents on the battlefield matching `filter`.
    EachPermanent(SelectionRequirement),
    /// All battlefield permanents matching `filter` EXCEPT the cast-time
    /// target list — the "choose one, [destroy] the rest" shape (Deadly
    /// Vanity: "Choose a creature or planeswalker. Destroy the rest.").
    EachPermanentExceptTargets(SelectionRequirement),
    /// All battlefield permanents controlled by `who` matching `filter`.
    /// The player-relative sibling of `EachPermanent` — lets one effect
    /// touch every permanent a *targeted* player controls (Sleep: tap +
    /// stun all creatures target player controls).
    ControlledBy { who: PlayerRef, filter: SelectionRequirement },
    /// All battlefield permanents *owned* by `who` matching `filter` — the
    /// ownership twin of `ControlledBy` ("a permanent owned by the voter" —
    /// Expropriate), so a stolen permanent still answers to its owner.
    OwnedBy { who: PlayerRef, filter: SelectionRequirement },
    /// Every creature controlled by the controller of `subject`, **except**
    /// `subject` itself ("other creatures that player controls" — Mark for
    /// Death). Empty if `subject` resolves to nothing.
    OtherCreaturesControlledByControllerOf(Box<Selector>),
    /// The single creature `ctx.controller` controls with the least
    /// toughness (first in battlefield order on a tie). Resolves to an
    /// empty set when the controller has no creatures. Powers Bolster
    /// (CR 701.21 — "choose a creature with the least toughness among
    /// creatures you control").
    LeastToughnessYouControl,
    /// The single creature `ctx.controller` controls with the greatest power
    /// (first in battlefield order on a tie). Empty when the controller has
    /// no creatures. Powers Triumph of Gerrard's "target creature you control
    /// with the greatest power" chapters (modeled non-targeted).
    GreatestPowerYouControl,
    /// The single permanent `who` controls with the greatest mana value among
    /// those matching `filter` (Juxtapose). Ties go to the first in
    /// battlefield order, standing in for the printed controller's choice.
    GreatestManaValueControlledMatching { who: PlayerRef, filter: SelectionRequirement },
    /// The single creature `ctx.controller` controls with the greatest power
    /// among those matching `filter`. Empty when none match — read through
    /// `Value::PowerOf` this yields 0, so `Max(Const(n), PowerOf(..))` floors at
    /// n (Triumphant Chomp — "2 or the greatest power among Dinosaurs you
    /// control, whichever is greater").
    GreatestPowerControlledMatching(SelectionRequirement),
    /// The single *other* creature `ctx.controller` controls with the greatest
    /// toughness (first in battlefield order on a tie), excluding the effect's
    /// own source. Empty when the controller has no other creatures. Powers
    /// "greatest toughness among other creatures you control" (Flourishing
    /// Hunter, read through `Value::ToughnessOf`).
    GreatestToughnessYouControl,
    /// The single creature with the least power among ALL creatures on the
    /// battlefield, any controller (first in battlefield order on a tie —
    /// stands in for "you choose one"). Porphyry Nodes' upkeep destroy.
    LeastPowerAmongAll,
    /// The creature with the least toughness on the battlefield, any
    /// controller (first in battlefield order on a tie — stands in for "you
    /// choose one"). Purging Scythe's upkeep ping.
    LeastToughnessAmongAll,
    /// The permanent this one is attached to (for Auras/Equipment).
    AttachedTo(Box<Selector>),
    /// All permanents attached to `anchor`.
    AttachedToMe(Box<Selector>),
    /// CR 702.6e — the Aura/Equipment attached to the source that granted the
    /// ability now resolving, for granted lines that name their granter
    /// ("{T}: Put an aim counter on Hankyu" — Hankyu, Rakdos Riteknife).
    /// Resolves to the attachments on the source that grant abilities; two
    /// granters on one host are indistinguishable, so all of them match.
    AttachmentGranting,
    /// One card the effect's controller chooses from their own hand matching
    /// `filter` — the "reveal a [card] in your hand" shape (Assembly Hall).
    /// Resolves to nothing when nothing matches. The pick is the first match
    /// in hand order (selector resolution has no decision hook).
    ChosenCardInHand(SelectionRequirement),
    /// One legal object chosen uniformly at random from the battlefield
    /// permanents and players matching `filter` — "any target chosen at
    /// random" (Goblin Test Pilot). Resolves to nothing when the pool is
    /// empty; never asks for a target slot.
    RandomAmong(SelectionRequirement),

    /// Top `count` cards of `who`'s library.
    TopOfLibrary { who: PlayerRef, count: Value },
    /// Greedy walk of the top of `who`'s library, including each card
    /// (in order) until the running mana-value sum reaches `threshold`
    /// inclusive (i.e. the final card pushes the sum past the gate, and
    /// is included). Used by Improvisation Capstone's "exile cards from
    /// the top of your library until you exile cards with total mana
    /// value 4 or greater" rider — the engine previously hard-coded
    /// `Const(4)` cards, which under-counts when the top is land-heavy
    /// (top three MV-0 lands + one MV-4 spell = 4 cards, sum 4; the
    /// printed Oracle would walk past lands and stop when sum hits the
    /// threshold). Resolution: walk top→down summing each card's
    /// computed MV; stop after including the card that raises sum to
    /// ≥ threshold. Empty library returns nothing; library smaller than
    /// the running cap returns the whole library.
    TopOfLibraryUntilMvAtLeast { who: PlayerRef, threshold: Value },
    /// Bottom `count` cards of `who`'s library.
    BottomOfLibrary { who: PlayerRef, count: Value },
    /// Every card in `who`'s zone matching `filter`.
    CardsInZone { who: PlayerRef, zone: Zone, filter: SelectionRequirement },

    /// Cards discarded earlier in this same resolution (across all players)
    /// matching `filter`. Backed by
    /// `GameState.discarded_card_ids_this_resolution`. Used by Mind Roots's
    /// "Put up to one land card discarded this way onto the battlefield
    /// tapped under your control" rider — at resolution time the discarded
    /// cards have already moved into their owner's graveyard, and this
    /// selector walks `discarded_card_ids_this_resolution` then filters in
    /// the gy zone.
    DiscardedThisResolution { filter: SelectionRequirement },

    /// Cards put into exile earlier in this same resolution matching `filter`.
    /// Backed by `GameState.exiled_card_ids_this_resolution`; looked up in the
    /// exile zone. "If you exiled a land/nonland card this way" (Bonehoard
    /// Dracosaur).
    ExiledThisResolution { filter: SelectionRequirement },

    /// Entities damaged earlier in this same resolution: players (always) plus
    /// permanents matching `filter`. Backed by
    /// `GameState.damaged_this_resolution`. "Tap each creature damaged this
    /// way / those players can't cast noncreature spells" (Aurelia's Fury) —
    /// `Effect::Tap` reads the creatures, `CantCastNoncreatureThisTurn` reads
    /// the players.
    DamagedThisResolution { filter: SelectionRequirement },

    /// Cards destroyed earlier in this same resolution that match `filter`,
    /// read from `GameState.destroyed_this_resolution` (wherever they ended
    /// up). "Destroy all enchantments, then return all enchantment cards put
    /// into graveyards this way to the battlefield" — Cleansing Meditation's
    /// Threshold half.
    DestroyedThisResolution { filter: SelectionRequirement },

    /// A single player, lifted to selector form.
    Player(PlayerRef),

    /// Narrow another selector's result set to the entities matching `filter`
    /// ("the *nonblack* creature blocking or blocked by this" — Deathgazer).
    /// Player entities never match a permanent filter and are dropped.
    MatchingAmong { inner: Box<Selector>, filter: SelectionRequirement },
    /// Take at most `count` entities from `inner` (in resolution order).
    /// Wraps another selector to clamp how many entities flow through —
    /// used by SOS Heated Argument's "you may exile *a card* from your
    /// graveyard", Practiced Scrollsmith's "exile *target* noncreature/
    /// nonland card from your graveyard", and Pull from the Grave's
    /// "up to two creature cards from your graveyard". The cap is
    /// evaluated against the controller's resolution context, so values
    /// like `Value::CountersOn(...)` work as expected.
    Take { inner: Box<Selector>, count: Box<Value> },
    /// Like `Take` but picks `count` entities uniformly at random instead of
    /// the first in resolution order (Capricious Hellraiser's random exile).
    TakeRandom { inner: Box<Selector>, count: Box<Value> },

    /// Walk `inner` in iteration order, accumulating `value_of_each`
    /// per entity, and take entities greedily while the running sum
    /// stays ≤ `cap`. Entities whose value would push the sum over
    /// `cap` are skipped; iteration continues so smaller items can
    /// still fit. Used by Spell Satchel's "Choose any number of
    /// target IS cards in your graveyard with total mana value 4 or
    /// less. Return them to your hand." The greedy walk gives the
    /// AutoDecider a deterministic pick; a real UI player would
    /// surface a per-card pick prompt with the same running cap.
    TakeWithSumCap {
        inner: Box<Selector>,
        cap: Box<Value>,
        value_of_each: Box<Value>,
    },

    /// All battlefield permanents (including the anchor itself) whose
    /// printed name matches the entity resolved by `inner`. Powers the
    /// printed "and each other permanent with the same name" / "all
    /// permanents with that name" riders — Maelstrom Pulse, Echoing Truth,
    /// Bile Blight-style sweepers. `inner` is typically `Target(0)`; if it
    /// resolves to nothing (or a non-permanent), this yields nothing.
    SharingNameWith(Box<Selector>),

    /// Every *other* battlefield permanent sharing at least one colour with
    /// the entity `inner` resolves to (Spreading Plague). Colourless
    /// subjects match nothing.
    SharingColorWith(Box<Selector>),

    /// Every battlefield creature sharing a creature type with the entity
    /// `inner` resolves to (Faces of the Past). Changelings match everything
    /// (CR 702.73a); a dead anchor is read from its death LKI.
    SharingCreatureTypeWith(Box<Selector>),

    /// The union of two selectors, de-duplicated in left-then-right order.
    /// Lets one effect name two target slots at once (Barrin's Spite).
    Both(Box<Selector>, Box<Selector>),

    /// One half of the split made by the enclosing `Effect::SeparateIntoPiles`
    /// — the pile the chooser picked (`chosen: true`) or the leftover.
    /// Backed by `GameState.separated_piles`; ids on the battlefield resolve
    /// as permanents, the rest as cards (Death or Glory splits a graveyard).
    SeparatedPile { chosen: bool },

    /// No entities (placeholder/default).
    None,
}

impl Selector {
    pub fn attached_to(inner: Selector) -> Self {
        Selector::AttachedTo(Box::new(inner))
    }

    /// Wrap `inner` so it returns at most `count` entities in resolution
    /// order. Sugar for `Selector::Take { inner, count }`.
    pub fn take(inner: Selector, count: Value) -> Self {
        Selector::Take {
            inner: Box::new(inner),
            count: Box::new(count),
        }
    }

    /// Wrap `inner` so it returns at most one entity. Sugar for
    /// `Selector::Take { inner, count: 1 }`.
    pub fn one_of(inner: Selector) -> Self {
        Selector::take(inner, Value::Const(1))
    }
}

// ── Value ────────────────────────────────────────────────────────────────────

/// How `Value::DraftNoteNumber` folds a card name's draft notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraftNoteAgg {
    /// Highest note (Lurking Automaton).
    Max,
    /// Sum of every note (Cogwork Grinder).
    Sum,
}

/// A numeric expression evaluated at effect-time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    /// Half of a player's library size, rounded up (Ulamog, the Defiler's
    /// cast trigger).
    HalfLibrarySizeRoundedUp(PlayerRef),
    /// Half the player's life total, rounded up (Garza's Assassin's
    /// "Recover—Pay half your life, rounded up").
    HalfLifeRoundedUp(PlayerRef),
    /// The greatest mana value among cards in exile (Ulamog, the Defiler's
    /// enters-with-counters count).
    GreatestManaValueInExile,
    /// "The number of artifacts that were put into graveyards from the
    /// battlefield this turn" (Anzrag's Rampage).
    ArtifactsToGraveyardFromBattlefieldThisTurn,
    /// "The number of different color pairs among permanents you control that
    /// are exactly two colors" (Niv-Mizzet, Guildpact).
    DistinctTwoColorPairsControlled(PlayerRef),
    /// The greatest mana value among cards in `who`'s graveyard (Wick's Patrol's
    /// `-X/-X` where X is the greatest MV in your graveyard). 0 if empty.
    GreatestManaValueInGraveyard(PlayerRef),
    /// The greatest mana value among permanents `who` controls (Ugin's Insight's
    /// "scry X"). 0 with no permanents.
    GreatestManaValueAmongPermanents(PlayerRef),
    Const(i32),
    /// Number of entities the selector resolves to.
    CountOf(Box<Selector>),
    /// Number of entities the selector resolves to that also match `filter`.
    /// Lets a count read a filtered slice of a non-zone selector (e.g. count
    /// the land/blue/red cards among `Selector::LastMoved`). Culmination of
    /// Studies.
    CountMatching { sel: Box<Selector>, filter: SelectionRequirement },
    /// Sum of the mana values of the entities the selector resolves to
    /// (Summon: Bahamut's IV — "total mana value of other permanents you
    /// control"). Off-battlefield objects contribute their printed CMC.
    TotalManaValueOf(Box<Selector>),
    PowerOf(Box<Selector>),
    /// CR 700.18 — the size of the controller's party: the number of distinct
    /// roles (Cleric, Rogue, Warrior, Wizard) among creatures they control,
    /// capped at 4. A single creature counts for at most one role. Powers
    /// Zendikar Rising party payoffs (Squad Commander, Tajuru Paragon).
    PartyCount,
    /// Number of creatures currently blocking the resolved permanent
    /// (Spined Sliver's "+1/+1 for each creature blocking it").
    BlockersOf(Box<Selector>),
    /// CR 509.1b — number of attackers the resolved permanent is blocking
    /// (Guardian of the Gateless's "+1/+1 for each creature it's blocking").
    CreaturesBlockedBy(Box<Selector>),
    ToughnessOf(Box<Selector>),
    /// Marked damage on the first entity the selector resolves to (CR 120.3).
    /// Read from the live permanent, or from CR 603.10 leaves-battlefield LKI
    /// when the source has already died — so a dies-trigger can scale on "the
    /// amount of damage dealt to it this turn" (Tangled Colony's Rat count).
    /// Approximates "dealt this turn" as the marked damage at death (exact
    /// unless the damage was removed mid-turn).
    MarkedDamageOn(Box<Selector>),
    LifeOf(PlayerRef),
    /// CR 702.179 — `who`'s current speed (0–4). Momentum Breaker's "gain
    /// life equal to your speed". Backed by `Player.speed`.
    PlayerSpeed(PlayerRef),
    /// Lowest life total among all players in the game (CR 119.7 — Repay
    /// in Kind: "each player's life total becomes the lowest life total
    /// among all players").
    LowestLifeTotal,
    /// The highest life total among all (alive) players. Sorin, Grim
    /// Nemesis −9.
    HighestLifeTotal,
    HandSizeOf(PlayerRef),
    /// The number of the source controller's opponents who have `n` or fewer
    /// cards in hand. Powers "draw an additional card for each opponent who
    /// has one or fewer cards in hand" (Bandit's Talent, level 3).
    OpponentsWithHandSizeAtMost(u32),
    /// CR 700.2 — how many modes were chosen for the resolved spell (Riku of
    /// Many Paths reads the triggering spell's mode count).
    ModesChosenOf(Box<Selector>),
    /// The number of the source controller's opponents still in the game —
    /// "for each opponent, …" (Mardu Siegebreaker's token copies).
    OpponentCount,
    /// CR 709.5 — how many doors are unlocked among the Rooms `who` controls
    /// (Misty Salon's X/X Spirit).
    UnlockedDoorsControlled(PlayerRef),
    /// Creatures exiled from under `who`'s control this turn — the **sum** over
    /// the players `who` resolves to, so `EachOpponent` reads Vren, the
    /// Relentless' "creatures that were exiled under your opponents' control".
    CreaturesExiledFromControlThisTurn(PlayerRef),
    /// Life `who` has gained so far this turn (CR 119.3). Backed by
    /// `Player.life_gained_this_turn`. Used by Accomplished Alchemist's
    /// "{T}: Add X mana of any one color, where X is the amount of life
    /// you gained this turn."
    LifeGainedThisTurn(PlayerRef),
    /// Life lost so far this turn (CR 119.3) — the **maximum** over the
    /// players `who` resolves to, so `EachOpponent` reads "the most life any
    /// opponent has lost this turn" (Spinerock Knoll's hideaway gate).
    /// Backed by `Player.life_lost_this_turn`.
    LifeLostThisTurn(PlayerRef),
    /// Damage dealt to a player this turn (`Player.damage_taken_this_turn`) —
    /// "Bloodthirst X, where X is the damage dealt to your opponents this
    /// turn" (Petrified Wood-Kin). Sums across the resolved players.
    DamageTakenThisTurn(PlayerRef),
    /// The combat-damage-only slice of `DamageTakenThisTurn`
    /// (`Player.combat_damage_taken_this_turn`). Takes the MAX across the
    /// resolved players, so `EachPlayer` reads "the most combat damage any one
    /// player was dealt this turn" (Sidequest: Play Blitzball).
    CombatDamageTakenThisTurn(PlayerRef),
    /// Total damage dealt to the ability's source permanent this turn
    /// (`CardInstance.damage_dealt_to_this_turn`) — "if 4 or more damage was
    /// dealt to it this turn" (Rushing-Tide Zubera). Reads death-time LKI so a
    /// dies-trigger sees the damage that killed it.
    DamageDealtToSourceThisTurn,
    /// Damage dealt to the source this turn by *other* sources sharing its
    /// name (Blazing Effigy). Reads the per-name tally on the death LKI
    /// snapshot, so a dies-trigger sees the whole turn.
    DamageToSourceThisTurnFromOthersNamedSame,
    /// Creatures that died during the resolution currently underway — a
    /// sweeper billing its caster for its own kills (Hellfire). The turn-wide
    /// `CreaturesDiedThisTurn` over-counts on a busy turn.
    CreaturesDiedThisResolution,
    /// Number of cards `who` owns in the exile zone (Kaya, Orzhov Usurper's −5).
    CardsInExileOwnedBy(PlayerRef),
    /// Distinct card types among cards in `who`'s graveyard (the delirium
    /// count as a number — Lucid Dreams' "draw X").
    CardTypesInGraveyard(PlayerRef),
    /// Cards `who` has discarded this turn (max over resolved players).
    /// Backed by `Player.cards_discarded_this_turn` (Dihada's Ploy).
    CardsDiscardedThisTurn(PlayerRef),
    /// Number of card types on the most recently discarded card (Mount
    /// Velus Manticore's "X = the number of card types the discarded card
    /// has"). Backed by `GameState.last_discarded_card_types`.
    LastDiscardedCardTypes,
    /// Mana value of the most recently discarded card, falling back to the one
    /// discarded to pay the resolving ability's cost (Slumbering Tora's X).
    /// 0 when nothing has been discarded.
    LastDiscardedManaValue,
    /// Mana value of the card most recently revealed at random from a hand by
    /// `Effect::RevealRandomFromHand` (the Planeswalker's cycle's X). 0 when
    /// nothing was revealed this resolution.
    LastRevealedManaValue,
    /// CR 706.8c — the greatest number of `Selector::This`'s stored die
    /// results that share a value (Centaur of Attention's +X/+X). 0 with no
    /// stored results.
    GreatestSameStoredResult,
    /// Distinct card types across every graveyard (Altar of the Goyf,
    /// Lhurgoyf-style counts as a spell value).
    CardTypesInAllGraveyards,
    /// Noncreature spells `who` has cast so far this turn — the **maximum**
    /// over the resolved players. Backed by
    /// `Player.noncreature_spells_cast_this_game_turn`. Magebane Lizard's
    /// "damage equal to the number of noncreature spells they've cast this
    /// turn" (the count includes the spell that triggered it).
    NoncreatureSpellsCastThisTurn(PlayerRef),
    /// Total spells `who` has cast so far this turn (max over resolved
    /// players). Backed by `Player.spells_cast_this_turn` — Narset, Jeskai
    /// Waymaster draws this many after discarding her hand.
    SpellsCastThisTurn(PlayerRef),
    /// Spells `who` has cast this turn **other than** the source spell/ability
    /// itself — `spells_cast_this_turn` minus one, clamped at 0. The current
    /// spell is already counted at cast time, so subtracting one yields "the
    /// number of other spells you've cast this turn" (Thunder Salvo).
    /// Total spells cast this turn across every player (Erayo's "the fourth
    /// spell of a turn"). The summed sibling of `SpellsCastThisTurn`, which
    /// takes the max over the resolved seats.
    SpellsCastThisTurnTotal,
    OtherSpellsCastThisTurn(PlayerRef),
    /// Creatures `who` declared as attackers this turn (max over resolved
    /// players). Creatures *put onto the battlefield attacking* don't count,
    /// matching the Windbrisk Heights ruling on "attacked with N creatures".
    /// Backed by `Player.creatures_attacked_this_turn`.
    CreaturesAttackedWithThisTurn(PlayerRef),
    /// CR 702.122 (Melee) — the number of distinct opponents the active
    /// player attacked this combat, read from the live `GameState.attacking`
    /// declarations (Player / Planeswalker-controller / Battle-protector map
    /// to a defending player). One in a normal 1v1 combat; more in multiplayer
    /// when a batch spreads attackers across seats.
    OpponentsAttackedThisCombat,
    /// The game's current turn number (CR 500 — the first turn is 1). Powers
    /// "the first upkeep" gates (Sentinel Dispatch).
    TurnNumber,
    /// CR 905.2b — a number the controller noted as they drafted cards with
    /// this source's name. `Max` takes the highest note (Lurking Automaton),
    /// `Sum` totals them (Cogwork Grinder). Zero outside a drafted game.
    DraftNoteNumber { agg: DraftNoteAgg },
    GraveyardSizeOf(PlayerRef),
    /// Number of cards in `who`'s graveyard matching `filter`. Powers
    /// "equal to the number of Arcane cards in your graveyard" (Ire of
    /// Kaminari) and similar graveyard-count payoffs.
    CardsInGraveyardMatching { who: PlayerRef, filter: SelectionRequirement },
    /// The same count across *every* player's graveyard (Invigorating Falls,
    /// Mortal Combat).
    CardsInAllGraveyardsMatching { filter: SelectionRequirement },
    /// The same count across every *opponent's* graveyard (Nameless Race's
    /// "white cards in their graveyards"). Team-aware.
    CardsInOpponentsGraveyardsMatching { filter: SelectionRequirement },
    /// Number of cards in `who`'s hand matching `filter`. Powers Amplify
    /// (CR 702.38 — "+N/+N counters for each [type] card you reveal in your
    /// hand"; all matching cards are auto-revealed) and other reveal-from-hand
    /// counts.
    CardsInHandMatching { who: PlayerRef, filter: SelectionRequirement },
    /// Maximum graveyard size across **every alive player** in the game.
    /// Reads `players[*].graveyard.len()` and returns the max. Backs
    /// "if a graveyard has 20 or more cards" payoffs (Visions of Beyond,
    /// future Tombstalker / Mercurial Chemister-style scaling). Distinct
    /// from `GraveyardSizeOf(p)` which only inspects a single player.
    MaxGraveyardSize,
    /// Number of cards in `who`'s library. Used by Body of Research's
    /// "for each card in your library" Fractal-token scaling.
    LibrarySizeOf(PlayerRef),
    /// The X value paid in the spell's cost.
    XFromCost,
    /// Number of spells cast this turn by controller (Storm).
    StormCount,
    /// Dungeons the effect's controller has completed this game
    /// (CR 701.49d — "you've completed a dungeon" gates read ≥ 1).
    DungeonsCompleted,
    /// The controller's current experience-counter count (Ezuri's "X is the
    /// number of experience counters you have").
    ControllerExperience,
    /// CR 702.140 — the number of times the trigger source has mutated (the
    /// count of mutate cards merged onto it). Archipelagore / Insatiable
    /// Hemophage's "X is the number of times this creature has mutated".
    MutateCount,
    /// CR 700.5 — controller's devotion to the given color(s): the number
    /// of mana symbols matching any listed color among the mana costs of
    /// permanents they control. Hybrid / Phyrexian pips count for each
    /// color half they contain. Drives Gray Merchant of Asphodel, the Nyx
    /// gods, Nykthos.
    DevotionTo(Vec<crate::mana::Color>),
    /// The greatest number of the controller's creatures that share a
    /// creature type (Skemfar Shadowsage — "creatures you control that have
    /// a creature type in common"). Changelings count for every type.
    LargestCreatureTypeCount,
    /// Counters of the given type on `what`.
    CountersOn { what: Box<Selector>, kind: CounterType },
    /// All counters (every kind) on `what` — "for each counter on it"
    /// (Twitching Doll). Sums every counter the permanent carries.
    TotalCountersOn { what: Box<Selector> },
    /// CR 120.10 — the amount of excess damage dealt during the current
    /// resolution ("gain life / add mana equal to the excess damage" — The
    /// Last Agni Kai, Razor Rings). Backed by
    /// `GameState.excess_damage_this_resolution`.
    ExcessDamageDealtThisResolution,
    /// Face-down creatures on the battlefield, any controller (Ixidron's
    /// characteristic-defining power/toughness).
    FaceDownCreatures,
    /// Total damage that actually landed during this resolution — "gain life
    /// equal to the damage dealt this way" (Brightflame).
    DamageDealtThisResolution,
    /// Total mana spent to cast the spell most recently countered during
    /// the CURRENT resolution (`GameState.countered_spell_mana_spent`,
    /// stamped by `Effect::CounterSpell`). Mana Sculpt's "add an amount of
    /// {C} equal to the amount of mana spent to cast that spell".
    CounteredSpellManaSpent,
    /// The mana *value* of the spell countered during this resolution — the
    /// printed-cost sibling of `CounteredSpellManaSpent` (which reads what was
    /// actually paid). Plasm Capture. Backed by
    /// `GameState.countered_spell_mana_value`, cleared per resolution.
    CounteredSpellManaValue,
    /// The controller's starting life total (Resolute Archangel).
    StartingLifeTotal,
    /// The number just picked by [`Effect::PlayerChoosesNumber`] (0 outside
    /// one). Choice of Damnations' "that much" / "all but that many".
    ChosenNumber,
    /// The number stored on the source permanent by
    /// `Effect::ChooseNumberForSource` (Phyrexian Processor's "the life paid
    /// as this artifact entered"). Zero when nothing was chosen.
    ChosenNumberOfSource,
    /// Nonland cards exiled by the enclosing
    /// [`Effect::ExileTopBatchesUntilLandLast`] (Rally the Horde).
    NonlandCardsExiledThisEffect,
    Sum(Vec<Value>),
    Diff(Box<Value>, Box<Value>),
    Times(Box<Value>, Box<Value>),
    Min(Box<Value>, Box<Value>),
    Max(Box<Value>, Box<Value>),
    /// Clamp the inner value to ≥0.
    NonNeg(Box<Value>),
    /// The inner value divided by `by`, rounded down ("for each five counters
    /// removed this way" — Sage of Hours). `by == 0` evaluates to 0.
    DivDown(Box<Value>, u32),
    /// Half the inner value, rounded up (Tamiyo -7's "half the number of
    /// cards in your library").
    HalvedRoundUp(Box<Value>),
    /// Half the inner value, rounded down (Imskir Iron-Eater's "half the
    /// number of artifacts you control, rounded down").
    HalvedRoundDown(Box<Value>),
    /// Conditional: if `value` ≥ `threshold`, evaluate `then`, else `else_`.
    /// Powers "if X is 4 or more, …" scaling (Mossborn Hydra's doubled
    /// counters at X≥4).
    IfAtLeast { value: Box<Value>, threshold: i32, then: Box<Value>, else_: Box<Value> },
    /// Power of the most recently sacrificed creature this resolution
    /// (set by `Effect::SacrificeAndRemember`). Used by Thud / Greater
    /// Gargadon-style sacrifice + damage spells.
    /// The power of the card revealed to pay this cast's
    /// `AdditionalCastCost::RevealFromHand` (Titan's Presence). 0 with no reveal.
    RevealedForCostPower,
    SacrificedPower,
    /// How many counters the current resolution removed via
    /// `Effect::RemoveCountersUpTo` — "for each counter removed this way"
    /// (Hex Parasite). Reset between independent resolutions.
    CountersRemovedThisEffect,
    /// How many counters the current activation's `remove_all_counters_cost`
    /// paid — "for each counter removed this way" where the removal was a COST
    /// rather than part of the resolution (Essence Bottle).
    CountersRemovedAsCost,
    /// The arithmetic negation of `inner` — "all creatures get -X/-X, where X
    /// is …" (Ichor Explosion) without a bespoke variant per source value.
    Negate(Box<Value>),
    /// Summed power of every permanent sacrificed this resolution — the
    /// "total power of the sacrificed creatures" wording (Soulblast).
    SacrificedTotalPower,
    /// How many permanents the cast's additional cost (or the current
    /// resolution) sacrificed — "for each creature sacrificed this way"
    /// (Vicious Betrayal). Reads `GameState.sacrificed_count`.
    SacrificedCount,
    /// CR 702.184a — power of the creature tapped to pay a Station ability's
    /// cost, carried to resolution by `Effect::WithTappedPower`.
    TappedForCostPower,
    /// Toughness of the most recently sacrificed creature this
    /// resolution (set by `Effect::SacrificeAndRemember`). Used by
    /// Tribute to Hunger (gain life equal to sacrificed creature's
    /// toughness) and similar sacrifice + lifegain spells.
    SacrificedToughness,
    /// Mana value of the most recently sacrificed permanent this
    /// resolution (set by `Effect::SacrificeAndRemember`). Reckoner's
    /// Bargain ("gain life equal to the sacrificed permanent's mana value").
    SacrificedManaValue,
    /// Mana value of the permanent exiled to pay this activation's
    /// `ActivatedAbility::exile_permanent_cost` (Food Chain).
    ExiledForCostManaValue,
    /// Number of cards in *every* graveyard sharing the resolving source's
    /// name (Accumulated Knowledge).
    CardsNamedLikeSourceInAllGraveyards,
    /// Number of cards in *every* graveyard sharing the name of the spell that
    /// fired this trigger (the Odyssey Shrine cycle).
    CardsNamedLikeTriggerSpellInAllGraveyards,
    /// Number of cards discarded so far within the current effect
    /// resolution. Bumped by every `GameEvent::CardDiscarded` emission
    /// in `Effect::Discard` / `Effect::DiscardChosen`. Used by Borrowed
    /// Knowledge mode 1 ("draw cards equal to the number of cards
    /// discarded this way"), Colossus of the Blood Age's die trigger,
    /// and similar "draw what you discarded" payoffs. Reset to 0
    /// between independent resolutions, so a `Seq([Discard, Draw])`
    /// reads exactly the discards from this resolution.
    CardsDiscardedThisEffect,
    /// Number of cards drawn so far within the current effect resolution — the
    /// draw-side twin of `CardsDiscardedThisEffect` ("draws that many cards,
    /// then discards that many cards" — Laquatus's Creativity).
    CardsDrawnThisEffect,
    /// Mana value of the last card exiled within the current resolution
    /// (Undying Flames' "damage equal to that card's mana value"). 0 when
    /// nothing was exiled. Reset between independent resolutions.
    LastExiledManaValue,
    /// Number of permanents returned to hand by an
    /// `Effect::ReturnAnyNumberToHand` earlier in the current resolution — the
    /// Sweep count. Reset between independent resolutions.
    PermanentsReturnedThisEffect,
    /// Number of permanents actually tapped by an `Effect::Tap` earlier in the
    /// current resolution (Angel's Trumpet's "damage equal to the number of
    /// creatures tapped this way"). Reset between independent resolutions.
    PermanentsTappedThisEffect,
    /// Cards revealed from hand by an `Effect::RevealAnyNumberFromHand` earlier
    /// in the current resolution — X for the Scent / Seer cycle. Reset between
    /// independent resolutions.
    CardsRevealedThisEffect,
    /// Amount of {E} paid by an `Effect::PayAnyEnergy` earlier in the current
    /// resolution. Reset between independent resolutions. Aether Spike's
    /// "counter that spell unless its controller pays {1} for each {E} paid
    /// this way" reads it as the `extra_generic` of a `CounterUnlessPaid`.
    EnergyPaidThisEffect,
    /// Maximum, across all players, of cards discarded so far within
    /// the current effect resolution. Reads from
    /// `state.cards_discarded_per_player_this_resolution`. Used by
    /// Windfall's printed "draws cards equal to the greatest number of
    /// cards a player discarded this way" — a `Seq([Discard(EachPlayer,
    /// 100), Draw(EachPlayer, MaxCardsDiscardedThisEffectByAnyPlayer)])`
    /// produces the correct dynamic yield instead of the prior flat 7.
    /// Reset to 0 between independent resolutions.
    MaxCardsDiscardedThisEffectByAnyPlayer,
    /// Number of *creature* cards discarded so far within the current
    /// effect resolution. Bumped alongside `CardsDiscardedThisEffect`
    /// whenever the discarded card has `CardType::Creature`. Used by
    /// Plargg, Dean of Chaos's "if a creature card was discarded this
    /// way, this creature deals 2 damage to any target" conditional
    /// rider — gates an `Effect::If { ValueAtLeast(this, 1), ... }`.
    /// Reset to 0 between independent resolutions.
    CreatureCardsDiscardedThisEffect,
    /// Greatest mana value among cards discarded within the current effect
    /// resolution (`GameState.greatest_discarded_mv_this_resolution`). Used by
    /// Ill-Timed Explosion's "deals X damage to each creature, where X is the
    /// greatest mana value among cards discarded this way." 0 if none discarded.
    GreatestDiscardedManaValueThisEffect,
    /// Number of creature cards among the cards moved so far within the current
    /// effect resolution (`last_moved_cards`) that are now in a graveyard —
    /// i.e. creature cards milled this way (Dread Summons' "for each creature
    /// card put into a graveyard this way, create a 2/2 Zombie"). Reset between
    /// resolutions.
    CreatureCardsMilledThisEffect,
    /// The filtered sibling: cards moved to a graveyard earlier in this
    /// resolution matching `filter` (Saprazzan Breaker's "if a land card was
    /// milled this way").
    CardsMilledThisEffectMatching { filter: SelectionRequirement },
    /// Number of *distinct* mana values among nonland cards the controller owns
    /// in exile that carry one or more `counter` counters. Kianne's Fractal
    /// (`CounterType::Study`).
    DistinctManaValuesInExileWithCounter { counter: crate::card::CounterType },
    /// Number of *distinct* mana values among nonland permanents the controller
    /// controls. Lunar Insight ("draw a card for each different mana value among
    /// nonland permanents you control").
    DistinctManaValuesAmongControlledNonland,
    /// Number of *distinct* mana values among cards in `who`'s graveyard.
    /// Aven Heartstabber ("five or more mana values among cards in your
    /// graveyard").
    DistinctManaValuesInGraveyard(PlayerRef),
    /// Greatest power among creatures the controller controls *and* creature
    /// cards in the controller's graveyard (0 if none). Ambitious Dragonborn
    /// enters with X +1/+1 counters equal to this.
    GreatestPowerControlledAndGraveyard,
    /// The greatest power among creatures `who` controls; 0 with no creatures
    /// (Season of Gathering's "draw cards equal to the greatest power among
    /// creatures you control").
    GreatestPowerControlled { who: PlayerRef },
    /// Mana value (CMC) of the first card the selector resolves to.
    /// Looks the card up across the battlefield, graveyards, exile, and
    /// hands. Used by Wrath of the Skies (destroy each nonland with mana
    /// value X) and similar "filter by mana value" effects.
    ManaValueOf(Box<Selector>),
    /// Highest mana value among the (battlefield) permanents the selector
    /// resolves to; 0 if none. Rush of Knowledge ("draw cards equal to the
    /// highest mana value among permanents you control").
    HighestManaValueAmong(Box<Selector>),
    /// Number of distinct colors of the resolved permanent/card (CR 105.2 —
    /// "for each of its colors"). Reads printed colors; a colorless/devoid
    /// object counts 0. Breathe Your Last.
    ColorCountOf(Box<Selector>),
    /// Number of distinct colors among the entities the selector resolves to,
    /// unioned across all of them ("there are five colors among permanents you
    /// control" — Case of the Shattered Pact). Contrast `ColorCountOf`, which
    /// reads a single object's colors.
    DistinctColorsAmong(Box<Selector>),
    /// Converge value: the number of distinct colors of mana spent on the
    /// spell's cost. Stashed on `StackItem::Spell` at cast time and read
    /// from `EffectContext.converged_value` here. Used by Prismatic
    /// Ending and Pest Control.
    ConvergedValue,
    /// CR 702.157 — the number of times the source permanent's Squad cost was
    /// paid (`CardInstance.squad_count`). Reads `ctx.source`. Zero off-source.
    SquadCount,
    /// CR 702.33c — the number of times the source permanent's Multikicker
    /// cost was paid (`CardInstance.kick_count`). Reads `ctx.source`. Zero
    /// off-source (Everflowing Chalice).
    TimesKicked,
    /// CR 702.33c — how many times the *triggering* spell was kicked, read off
    /// the trigger's subject on the stack (Rumbling Aftershocks). Zero when the
    /// subject isn't a spell.
    CastSpellTimesKicked,
    /// Total mana spent paying the originating spell's cost. Stashed on
    /// `StackItem::Spell.mana_spent` at cast time, propagated onto
    /// spell-cast `StackItem::Trigger.mana_spent`, and read from
    /// `EffectContext.mana_spent` here. Powers SOS's Increment /
    /// Opus payoffs: Cuboid Colony / Berta / Fractal Tender's
    /// "Whenever you cast a spell, if the amount of mana you spent is
    /// greater than this creature's power or toughness, put a +1/+1
    /// counter on this creature", and Opus's "this creature gets +N/+N
    /// for the rest of the turn (and an extra +N/+N if five or more
    /// mana was spent)".
    CastSpellManaSpent,
    /// Number of distinct card types in the top `count` cards of `who`'s
    /// library. Used by Atraxa, Grand Unifier's reveal-and-sort ETB —
    /// "reveal the top 10, take up to one of each card type" is
    /// approximated as "draw N where N = distinct types in those 10".
    DistinctTypesInTopOfLibrary { who: PlayerRef, count: Box<Value> },
    /// Number of distinct card types among the cards in `who`'s graveyard.
    /// Backs Broodspinner's "create that many Insects equal to the number of
    /// card types among cards in your graveyard" payoff.
    DistinctTypesInGraveyard { who: PlayerRef },
    /// Number of distinct card types among the cards in exile stamped
    /// `exiled_with = source` (the resolving source). Backs Keen-Eyed
    /// Curator's "four or more card types among cards exiled with this
    /// creature" threshold.
    DistinctCardTypesExiledWith,
    /// Number of cards in exile stamped `exiled_with = source` (the resolving
    /// source). Backs "as long as three or more cards are exiled with this
    /// creature" static thresholds (Veteran Survivor).
    CardsExiledWithSourceCount,
    /// Damage dealt to `who` this turn by artifact sources (Reverse Polarity).
    ArtifactDamageToPlayerThisTurn { who: PlayerRef },
    /// Half (rounded down) the damage dealt this turn by the highest-dealing
    /// sorcery spell `who` cast this turn. Backdraft; 0 if they cast none.
    HalfGreatestSorceryDamageThisTurn { who: PlayerRef },
    /// Number of distinct power values among creatures the controller
    /// controls. Backs Golden Ratio's "draw a card for each different
    /// power among creatures you control."
    DistinctPowerYouControl,
    /// Total (computed) toughness of all creatures the controller controls.
    /// Betor, Kin to All's tiered end-step check (10/20/40).
    TotalToughnessControlled,
    /// Total (computed) power of all creatures the controller controls
    /// (Case of the Trampled Garden's "total power 8 or greater" solve).
    TotalPowerControlled,
    /// CR 702.9 — number of creatures that crewed the source this turn
    /// (`CardInstance.crewed_by`; Luxurious Locomotive).
    SourceCrewerCount,
    /// How many *differently named* permanents matching the filter the
    /// controller controls — All-Fates Scroll's and Emil's "differently named
    /// lands you control", and any future distinct-name count.
    DistinctNamesControlledMatching(crate::card::SelectionRequirement),
    /// Number of Gates you control with different names (Maze's End's
    /// ten-different-Gates win condition, CR 704).
    DistinctlyNamedGatesControlled,
    /// Number of differently-named creature *tokens* the controller controls
    /// (Audience with Trostani's "draw cards equal to the number of differently
    /// named creature tokens you control").
    DifferentlyNamedCreatureTokensControlled,
    /// Cards the controller owns in exile and in their graveyard that are Oozes
    /// or are named "Slime Against Humanity" (Slime Against Humanity's counter
    /// count).
    OozesInExileAndGraveyard,
    /// Number of cards `who` has drawn on the current turn. Powers
    /// Strixhaven's Quandrix scaling — Fractal Anomaly's "X +1/+1
    /// counters where X is the number of cards you've drawn this turn"
    /// and similar payoffs. Backed by `Player.cards_drawn_this_turn`,
    /// reset on the player's untap.
    CardsDrawnThisTurn(PlayerRef),
    /// Cards `who` has drawn during the current step. Backed by
    /// `Player.cards_drawn_this_step`, reset on every step change.
    /// Powers Orcish Bowmasters' "except the first one they draw in
    /// each of their draw steps" exemption.
    CardsDrawnThisStep(PlayerRef),
    /// Number of lands the player has played this turn (CR 305 / landfall).
    /// Backed by `Player.lands_played_this_turn`; a `> 0` value powers
    /// "if a land entered under your control this turn" landfall riders
    /// (Groundswell, Searing Blaze). Reset on the player's untap.
    LandsPlayedThisTurn(PlayerRef),
    /// Artifacts that entered the battlefield under the player's control this
    /// turn (`Player.artifacts_entered_this_turn`) — Malcator's end-step gate.
    ArtifactsEnteredThisTurn(PlayerRef),
    /// Mounts and/or Vehicles that entered under the player's control this turn
    /// (`Player.mounts_vehicles_entered_this_turn`) — Cloudspire Coordinator's
    /// X token count.
    MountsVehiclesEnteredThisTurn(PlayerRef),
    /// Creatures of `1` (other than the resolution's trigger source) that
    /// entered under the controller's control this turn — Geralf, the
    /// Fleshwright's "+1/+1 counter for each other Zombie that entered the
    /// battlefield under your control this turn".
    OtherCreaturesOfTypeEnteredThisTurn(crate::card::CreatureType),
    /// The number of distinct power values among creatures `0` controls
    /// ("one mana of that color for each different power among creatures you
    /// control" — Selvala, Eager Trailblazer).
    DistinctPowersAmongCreaturesControlled(PlayerRef),
    /// The player's poison counters (Vraska's −9 "counters equal to the
    /// difference" top-up).
    PoisonCountersOf(PlayerRef),
    /// How many of the controller's opponents lost life this turn (Kaito,
    /// Bane of Nightmares' 0).
    OpponentsWhoLostLifeThisTurn,
    /// Multicolored spells the player has cast this turn (Zenith Chronicler).
    MulticoloredSpellsCastThisTurn(PlayerRef),
    /// The greatest total toxic value among creatures the player controls
    /// (Goliath Hatchery's Corrupted draw; CR 702.180b sums instances).
    GreatestToxicAmongControlled(PlayerRef),
    /// Two raised to the inner value, clamped to a sane upper bound (≤30).
    /// Used by SOS Mathemagics — "target player draws 2ˣ cards" — so the
    /// X-cost bombshell scales correctly at the small/medium values
    /// typical of casting it. The clamp avoids deck-out / overflow when
    /// X is ≥31.
    Pow2(Box<Value>),
    /// Half of the inner value, rounded down. Used by SOS Pox Plague's
    /// "loses half their life", "discards half", "sacrifices half"
    /// clauses.
    HalfDown(Box<Value>),
    /// Number of permanents controlled by the resolved player. Useful for
    /// per-player effects like Pox Plague's "sacrifices half the
    /// permanents they control" clause inside a `ForEach` over each
    /// player, where `Selector::EachPermanent(ControlledByYou)` would
    /// always read the spell's controller instead of the iterated
    /// player.
    PermanentCountControlledBy(PlayerRef),
    /// The filtered sibling of `PermanentCountControlledBy` — how many
    /// permanents the resolved player controls that match the filter
    /// (Mana Cache's "for each untapped land that player controls").
    PermanentCountControlledByMatching(PlayerRef, crate::card::SelectionRequirement),
    /// Number of players still in the game (not eliminated). "For each player"
    /// riders — Benediction of Moons's "gain 1 life for each player".
    PlayerCount,
    /// CR 700.11 — how many times the controller descended this turn (permanent
    /// cards put into their graveyard). The Mycotyrant's end-step token count.
    TimesDescendedThisTurn,
    /// Number of creatures controlled by the resolved player. Sibling of
    /// `PermanentCountControlledBy` filtered to creatures. Powers
    /// Biorhythm's "each player's life total becomes the number of
    /// creatures they control" inside a `ForEach` over each player.
    CreatureCountControlledBy(PlayerRef),
    /// The size of the largest group of creatures the controller controls that
    /// share a creature type (CR — "the greatest number of creatures you
    /// control that have a creature type in common"). Changelings count toward
    /// every type. 0 if you control no creatures. White Lotus Tile's mana
    /// ability.
    GreatestSharedCreatureTypeCount,
    /// Creatures the evaluating player controls that share a creature type
    /// with `.0` (Mana Echoes). Changelings share with everything.
    CreaturesSharingTypeWith(Box<Selector>),
    /// Number of nonbasic lands controlled by the resolved player. Read
    /// per-recipient inside a `ForEach` over each player so a single effect
    /// scales independently for each player — Sunspine Lynx's "deals damage
    /// to each player equal to the number of nonbasic lands that player
    /// controls."
    NonbasicLandCountControlledBy(PlayerRef),
    /// Number of loyalty counters on the first permanent the selector
    /// resolves to. Used by Strixhaven's **Confront the Past** mode 2
    /// ("Confront the Past deals damage to target planeswalker equal to
    /// the number of loyalty counters on it") and any future
    /// "loyalty-counter-X" payoff. Returns 0 for non-permanents and
    /// non-planeswalkers (the field is just the `CounterType::Loyalty`
    /// count, which is 0 for cards without loyalty).
    LoyaltyOf(Box<Selector>),
    /// The amount carried by the event that fired the current trigger
    /// (life gained, life lost, damage dealt, cards drawn, …). Read
    /// from `EffectContext.event_amount`, which is set by the
    /// `dispatch_triggers_for_events` dispatcher from the event's
    /// `amount` field. Used by Light of Promise's "Whenever you gain
    /// life, put that many +1/+1 counters on target creature you
    /// control." — the trigger body reads `Value::TriggerEventAmount`
    /// for the count of counters to drop. Returns 0 in non-trigger
    /// resolution contexts (spells, activated abilities, delayed
    /// triggers that have moved past the original event).
    TriggerEventAmount,
    /// The number of Auras the trigger's controller controlled that were
    /// attached to the dying creature (the trigger subject) when it left the
    /// battlefield. Read from `GameState.auras_at_death`. Powers Hateful
    /// Eidolon's "draw a card for each Aura you controlled that was attached
    /// to it".
    AurasYouControlledOnDyingSubject,
    /// CR 706.4 — the result of the most recent die roll in this resolution.
    /// Set by the `Effect::RollDie` resolver just before it runs each result-
    /// table arm, so an inner effect can reference the rolled face ("create
    /// that many Treasure tokens" — Ancient Copper Dragon). Returns 0 outside
    /// a die-roll context.
    LastDieRoll,
    /// Number of creatures that died under `who`'s control so far this
    /// turn. Backed by `Player.creatures_died_this_turn` (bumped from the
    /// SBA death loop). Powers Witherbloom "harvest" payoffs that scale
    /// off the turn's carnage (e.g. "draw a card for each creature that
    /// died under your control this turn"). The companion predicate is
    /// `Predicate::CreaturesDiedThisTurnAtLeast`.
    CreaturesDiedThisTurn(PlayerRef),
    /// Number of permanents `who` has sacrificed so far this turn. Backed by
    /// `Player.permanents_sacrificed_this_turn`.
    PermanentsSacrificedThisTurn(PlayerRef),
    /// Number of creatures that died this turn across **every** player.
    /// Sums `Player.creatures_died_this_turn` over all seats. Powers
    /// table-wide aristocrat scaling, mirroring
    /// `Predicate::CreaturesDiedThisTurnTotalAtLeast`.
    CreaturesDiedThisTurnTotal,
    /// Number of creatures that died **under the controller's control** this
    /// turn (`Player.creatures_died_this_turn` for `ctx.controller`). Liliana's
    /// Standard Bearer.
    ControllerCreaturesDiedThisTurn,
    /// Number of Zubera that died this turn across **every** player. Sums
    /// `Player.zuberas_died_this_turn`. Powers the Champions-of-Kamigawa
    /// Zubera death-trigger cycle ("for each Zubera that died this turn").
    ZuberasDiedThisTurnTotal,
    /// Number of permanents destroyed by `Effect::Destroy` earlier in this
    /// same resolution. Backed by `GameState.permanents_destroyed_this_resolution`.
    /// Powers Culling Ritual's "Add {B} or {G} for each permanent destroyed
    /// this way" — evaluate it in a later `Seq` step after the destruction.
    PermanentsDestroyedThisResolution,
    /// Number of snow permanents (CR 205.4g — supertype Snow) controlled by
    /// the resolved player. Powers Skred ("deals damage to target creature
    /// equal to the number of snow permanents you control") and other
    /// snow-matters scaling.
    SnowPermanentCountControlledBy(PlayerRef),
    /// CR 702.43 — Domain: the number of distinct basic land types among lands
    /// the resolved player controls (0–5). Powers Tribal Flames / Territorial
    /// Kavu, and (as a generic cost reduction) Leyline Binding.
    DomainCount(PlayerRef),
    /// Cards in *every* player's graveyard whose name matches the resolving
    /// spell's name (`EffectContext.source_name`). Rune Snag's counter tax.
    SameNamedInAllGraveyards,
    /// Conditional value: `then` when `pred` holds, else `else_`.
    /// Polukranos, Unchained's "enters with six… escapes with twelve instead".
    IfPred { pred: Box<Predicate>, then: Box<Value>, else_: Box<Value> },
    /// The number stamped on the source by `Effect::LoseAllButLifeRemembered`
    /// — "the life you lost when it entered" (Soulgorger Orgg). Reads the
    /// source's death LKI too, so a leave trigger still sees it.
    RememberedAmountOfSource,
}

impl Value {
    pub const ZERO: Value = Value::Const(0);
    pub const ONE: Value = Value::Const(1);
    pub fn count(sel: Selector) -> Self { Value::CountOf(Box::new(sel)) }
}

// ── Predicate ────────────────────────────────────────────────────────────────

/// A boolean game-state condition (for `Effect::If` / cast-time checks).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Predicate {
    #[default]
    True,
    False,
    Not(Box<Predicate>),
    All(Vec<Predicate>),
    Any(Vec<Predicate>),
    /// At least one entity matches the selector.
    SelectorExists(Selector),
    /// Selector resolves to at least `n` entities.
    SelectorCountAtLeast { sel: Selector, n: Value },
    /// Every battlefield permanent matching `filter` shares at least one
    /// colour with all the others (Common Cause). Vacuously true when nothing
    /// matches; false as soon as one match is colourless.
    AllMatchingShareAColor(crate::card::SelectionRequirement),
    /// `who` controls a land of every basic land type (Coalition Victory).
    ControlsLandOfEachBasicType(PlayerRef),
    /// `who` controls at least one creature of each of the five colours
    /// (Coalition Victory). A multicoloured creature covers every colour it is.
    ControlsCreatureOfEachColor(PlayerRef),
    /// "…as long as [colour] is the most common colour among all permanents or
    /// is tied for most common" (the Invasion Djinn cycle). Each permanent
    /// counts once per colour it is; false when no permanent is coloured.
    ColorIsMostCommonAmongPermanents(crate::mana::Color),
    /// lhs ≥ rhs.
    ValueAtLeast(Value, Value),
    /// lhs ≤ rhs.
    ValueAtMost(Value, Value),
    /// lhs = rhs. Compresses the previous `All([≥, ≤])` idiom used by
    /// MV-equals filters (Postmortem Lunge "creature card with mana
    /// value X", Fix What's Broken "each card with mana value X").
    ValueEquals(Value, Value),
    /// The value is odd. Powers "if it has an odd number of counters on it"
    /// riders (Sab-Sunen, Luxa Embodied). `ValueIsOdd(0)` is false (zero is
    /// even, CR-flavor).
    ValueIsOdd(Value),
    /// The value is a prime number (2, 3, 5, 7, …). Powers Zimone,
    /// All-Questioning's "if you control a prime number of lands". 0 and 1
    /// are not prime.
    ValueIsPrime(Value),
    /// CR 105 — the two named target slots have exactly the same colour set
    /// ("unless either one is a color the other isn't" — Dead Ringers).
    TargetsHaveIdenticalColors(u8, u8),
    /// The object in target slot `0` shares a colour with a permanent the
    /// controller controls matching `filter` (Jaded Response's "if it shares a
    /// color with a creature you control"). Reads the target wherever it is —
    /// stack spell or permanent.
    TargetSharesColorWithControlled { slot: u8, filter: crate::card::SelectionRequirement },
    /// True if `who` sacrificed at least one permanent during the current
    /// resolution. Backed by `GameState::players_sacrificed_this_resolution`.
    /// Gates "if you sacrificed a permanent this way, …" (Deadly Brew).
    PlayerSacrificedThisResolution(PlayerRef),
    /// CR 120.10 — true if excess damage was dealt to a creature / planeswalker
    /// / battle during the current resolution (Orbital Plunge's "if excess
    /// damage was dealt this way"). Backed by
    /// `GameState.excess_damage_this_resolution`.
    ExcessDamageDealtThisResolution,
    /// The controller has cast a spell with the SOURCE's printed name at
    /// least `n` times this game (per-name lifetime tally, self-inclusive:
    /// at resolution the resolving cast already counts, so "you've cast
    /// ANOTHER spell named X this game" is `n = 2`). Approach of the
    /// Second Sun.
    CastOwnNameThisGameAtLeast(u32),
    /// At least `n` mode instances were chosen for this cast
    /// (`ctx.spree_modes`, stamped by `CastSpellSpree` /
    /// `ChooseModesCast`). With distinct modes and `max = n`, this is
    /// exactly "if you chose all of the above" (Multiple Choice's fourth
    /// bullet). A plain single-mode `CastSpell { mode }` fallback carries
    /// one mode, so the gate correctly fails.
    ChoseModesAtLeast(u8),
    /// It's `who`'s turn.
    IsTurnOf(PlayerRef),
    /// `who` resolves to a player who is an opponent of the source's
    /// controller ("…deals combat damage to one of your opponents" —
    /// Gonti, Night Minister). Teammates and the controller are false.
    PlayerIsOpponent { who: PlayerRef },
    /// True during the controller's main phase (CR — RNA Addendum). Read at an
    /// instant's resolution, which for these cards is the same step it was cast
    /// in (the stack can't advance a step mid-resolution), so it faithfully
    /// captures "if you cast this spell during your main phase."
    YourMainPhase,
    /// The active player (whose turn it is) controls at least one permanent the
    /// selector resolves to. Gates "at the beginning of the upkeep of enchanted
    /// [permanent]'s controller, …" Aura triggers (Warp Artifact, Cursed Land,
    /// Wanderlust): pair with an `AnyPlayer` upkeep trigger so it fires only on
    /// the enchanted permanent's controller's upkeep.
    ActivePlayerControls(Box<Selector>),
    /// The game is currently in the given turn step (CR 500). Gates
    /// "activate only during [your] upkeep / end step" abilities; pair with
    /// `IsTurnOf(You)` for the "your" qualifier.
    CurrentStepIs(crate::turn_step::TurnStep),
    /// The given entity's properties match the filter.
    EntityMatches { what: Selector, filter: SelectionRequirement },
    /// At least one entity the selector resolves to matches the filter
    /// (false when the selector is empty — unlike `EntityMatches`, which is
    /// vacuously true). Gates "if a [creature] card was exiled this way, …"
    /// over an optional ("up to one") target — Diregraf Scavenger.
    EntityMatchesAny { what: Selector, filter: SelectionRequirement },
    /// `who` has gained at least `at_least` total life this turn.
    /// Backed by `Player.life_gained_this_turn`. Used by Strixhaven's
    /// **Infusion** rider — "If you gained life this turn, …".
    LifeGainedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// True while `who` has NOT yet completed a life-gain trigger batch
    /// this turn. Used as the event filter on `LifeGained` triggers that
    /// read "whenever you gain life for the first time each turn" (Leech
    /// Collector): a gain that happened before the listener arrived
    /// already flipped the flag, so later gains that turn don't qualify.
    /// Pair with `once_per_turn()` so a multi-event first batch still
    /// fires only once. Backed by `Player.gained_life_earlier_this_turn`.
    FirstLifeGainThisTurn { who: PlayerRef },
    /// CR 700.14 — true exactly on the spell-cast whose payment first
    /// pushes the active player's running mana-spent-on-spells total to
    /// `n` (i.e. prior total `< n` and new total `>= n`). Used as the
    /// resolve-time filter on `EventKind::Expend` triggers (Roughshod Duo
    /// "Whenever you expend 4").
    ExpendReached(u32),
    /// CR 700.6 — `who` has the city's blessing. "As long as you have the
    /// city's blessing, …" (Ascend payoffs).
    HasCityBlessing { who: PlayerRef },
    /// `who` has discarded a card this turn (Asmoranomardicadaistinaculdacar's
    /// alternative-cost gate). Reads `Player.discarded_this_turn`.
    DiscardedThisTurn { who: PlayerRef },
    /// CR 731 — it's currently day.
    IsDay,
    /// CR 731 — it's currently night.
    IsNight,
    /// CR 706.4 — true when the firing roll's greatest result is at least `n`.
    /// Gates "whenever you roll a 5 or higher" triggers off a `RolledDice`
    /// event via `EffectContext.event_amount` (Ground Pounder's trample rider).
    DieResultAtLeast(u8),
    /// CR 725 — `who` is the monarch ("as long as you're the monarch, …").
    IsMonarch { who: PlayerRef },
    /// CR 725 — `who` was the monarch as the current turn began. Reads the
    /// turn-start snapshot, so a mid-turn monarch change doesn't rewrite it
    /// (Knights of the Black Rose's "if you were the monarch as the turn
    /// began").
    WasMonarchAtTurnStart { who: PlayerRef },
    /// CR 726 — `who` currently has the initiative.
    HasInitiative { who: PlayerRef },
    /// CR 702.179 — `who`'s speed is at least `speed` (0–4). "Max speed —"
    /// abilities use `speed: 4`; "as long as your speed is N or higher" uses
    /// the listed threshold. Backed by `Player.speed`.
    SpeedAtLeast { who: PlayerRef, speed: u32 },
    /// True if any player matched by `who` has been dealt damage this turn.
    /// Backed by `Player.was_dealt_damage_this_turn`. Powers Bloodthirst
    /// (CR 702.54) — pair with `who: EachOpponent` for "if an opponent was
    /// dealt damage this turn."
    PlayerDamagedThisTurn { who: PlayerRef },
    /// True if any player matched by `who` has lost life this turn (damage or
    /// direct life loss). Backed by `Player.lost_life_this_turn`. Powers
    /// Spectacle (CR 702.111) — pair with `who: EachOpponent`.
    PlayerLostLifeThisTurn { who: PlayerRef },
    /// True if any player matched by `who` has gained life this turn. Backed by
    /// `Player.life_gained_this_turn`. Powers "if you gained or lost life this
    /// turn" end-step payoffs (Starlit Soothsayer, Star Charter).
    PlayerGainedLifeThisTurn { who: PlayerRef },
    /// True if any player matched by `who` has drawn at least `n` cards this
    /// turn. Backed by `Player.cards_drawn_this_turn`. Paired with
    /// `who: Triggerer` + `once_per_turn` to model "whenever a player draws
    /// their Nth card each turn" (Faerie Mastermind's second-draw payoff).
    PlayerDrewAtLeastThisTurn { who: PlayerRef, n: u32 },
    /// True if any player matched by `who` has an effective life total at most
    /// `life`. Powers "unless an opponent has N or less life" gates (Vampire
    /// Lacerator).
    PlayerLifeAtMost { who: PlayerRef, life: i32 },
    /// True if any player matched by `who` has an effective life total exactly
    /// `life`. Powers "if target player has exactly N life" gates (Hidetsugu's
    /// Second Rite).
    PlayerLifeExactly { who: PlayerRef, life: i32 },
    /// True if any player matched by `who` has an effective life total at least
    /// `life`. Powers "as long as you have N or more life" statics (Angel of
    /// Vitality).
    PlayerLifeAtLeast { who: PlayerRef, life: i32 },
    /// True if any player matched by `who` has an effective life total at least
    /// `delta` above the life total they started the game with (CR 103.4).
    /// Powers "as long as you have at least N life more than your starting life
    /// total" statics/activations (Righteous Valkyrie, Speaker of the Heavens).
    PlayerLifeAtLeastAboveStarting { who: PlayerRef, delta: i32 },
    /// True if any player matched by `who` has an effective life total less than
    /// or equal to half the life total they started the game with (rounded
    /// down). Powers "if your life total is less than or equal to half your
    /// starting life total" gates (Cecil, Dark Knight).
    PlayerLifeAtMostHalfStarting { who: PlayerRef },
    /// True if any player matched by `who` has the most life, or is tied for
    /// the most, among all (non-eliminated) players. Powers Dethrone (CR
    /// 702.105 — "attacks the player with the most life or tied for most
    /// life"); pair with `who: DefendingPlayer` on an `Attacks` trigger.
    PlayerHasMostLife { who: PlayerRef },
    /// `who` controls strictly more `filter` permanents than each other player
    /// (Rivalry's land leader, Damping Engine's permanent leader).
    PlayerControlsMostOf { who: PlayerRef, filter: SelectionRequirement },
    /// True if any player matched by `who` has strictly less life than at least
    /// one of their opponents. Geyadrone Dihada's "if you have less life than
    /// an opponent" loyalty-reset rider.
    PlayerHasLessLifeThanOpponent { who: PlayerRef },
    /// The trigger's subject is the permanent stamped on the ability source's
    /// `chosen_permanent` slot (Diabolic Servitude's "when the creature put
    /// onto the battlefield with this enchantment dies").
    TriggerSourceIsSourcesChosenPermanent,
    /// The triggering object *is* this ability's own source — the gate behind
    /// "a permanent **other than** this one" triggers (Last Laugh).
    TriggerSourceIsSelf,
    /// The trigger's subject is the permanent the effect's source is attached
    /// to — "whenever enchanted creature becomes tapped" (Stinging Licid).
    TriggerSourceIsSourceHost,
    /// True if the effect's source creature attacked this turn (CR 702.142
    /// Boast gate). Backed by `CardInstance.attacked_this_turn`.
    SourceAttackedThisTurn,
    /// CR 500.7 — the current turn is an extra turn. Backed by
    /// `GameState.current_turn_is_extra`. Medomai the Ageless's
    /// "can't attack during extra turns".
    IsExtraTurn,
    /// CR 701.31 — the source permanent is monstrous. Backed by
    /// `CardInstance.monstrous`. Powers "as long as this is monstrous, …"
    /// statics (Fleecemane Lion's hexproof + indestructible).
    SourceIsMonstrous,
    /// CR 702.93 — the source permanent is renowned. Backed by
    /// `CardInstance.renowned`. Powers "if it's renowned, …" attack intervening-
    /// ifs (Consul's Lieutenant) and "as long as renowned" statics.
    SourceIsRenowned,
    /// CR 301.5 — the source permanent is equipped (an Equipment is attached).
    /// Powers "as long as this creature is equipped, …" team statics (Auriok
    /// Steelshaper's Soldier/Knight anthem).
    SourceIsEquipped,
    /// The source permanent is currently a creature, reading the *computed*
    /// (layer-aware) types — true for an animated manland. Gates abilities
    /// granted by animation (Wandering Fumarole's `{0}` switch, Lavaclaw
    /// Reaches' firebreathing).
    SourceIsCreature,
    /// The ability's source permanent is still on the battlefield — the
    /// intervening-'if' half of "…if [this] is still on the battlefield"
    /// (Shirei, Shizo's Caretaker's delayed return).
    SourceOnBattlefield,
    /// The source is blocking a creature that at least one *other* blocker
    /// matching `filter` is also blocking, and every blocker on it matches
    /// `filter` (Wall of Caltrops' banding intervening-'if').
    SourceCoBlockersAllMatch { filter: crate::card::SelectionRequirement },
    /// CR 701.60 — the source permanent is suspected. Backed by
    /// `CardInstance.suspected`. Powers Repeat Offender's "if this creature
    /// is suspected, … otherwise, suspect it."
    SourceIsSuspected,
    /// CR 702.103 — the source permanent is an Aura because it was cast for
    /// its bestow cost and is still attached. Powers "if this permanent is an
    /// Aura, … instead" branches (Erebos's Emissary).
    SourceIsBestowedAura,
    /// CR 702.139 — Revolt: a permanent left the battlefield under `who`'s
    /// control this turn. Backed by
    /// `Player.permanent_left_battlefield_this_turn`.
    RevoltActive { who: PlayerRef },
    /// EOE Void — true if a nonland permanent left the battlefield this turn
    /// (any controller) or `who` cast a spell for its warp cost this turn.
    /// Backed by `GameState.nonland_permanent_left_bf_this_turn` +
    /// `Player.warped_spell_this_turn`.
    VoidActive { who: PlayerRef },
    /// True while at least one player controls no creatures (Sothera, the
    /// Supervoid's end-step cash-in).
    AnyPlayerControlsNoCreatures,
    /// True if the effect's source permanent is currently saddled (CR
    /// 702.171). Backed by `CardInstance.saddled`; gates "whenever this
    /// attacks while saddled" triggers on Mounts.
    SourceSaddled,
    /// True if the effect's source permanent was cast via Escape (CR
    /// 702.139). Backed by `CardInstance.cast_from_escape`; gates the
    /// "sacrifice it unless it escaped" ETB rider on Kroxa / Uro.
    SourceCastFromEscape,
    /// True if the source permanent reached the battlefield by being cast as
    /// a spell (any cast path), as opposed to being a token, conjured, or put
    /// onto the battlefield by another effect. Gates "if you cast it" ETB
    /// riders (Skitterbeam Battalion) — token copies and reanimated bodies
    /// don't re-fire. Backed by the persistent `CardInstance` cast flags.
    SourceWasCast,
    /// The narrower sibling: true only if the source permanent reached the
    /// battlefield by being cast **from its owner's hand** (Phage the
    /// Untouchable's "if you didn't cast it from your hand, you lose the
    /// game"). Backed by `CardInstance.cast_from_hand`.
    SourceCastFromOwnersHand,
    /// CR 702.77 — true if a card in exile is champion-linked to the
    /// effect's source (`exiled_by` points at it). Gates "when a [type] is
    /// championed with this creature" riders (Mistbind Clique).
    SourceChampionedSomething,
    /// True if the trigger's subject (a blocker) is blocking the listening
    /// source. Scopes a `Blocks`/`AnyPlayer` trigger to "whenever a creature
    /// blocks *this* creature" (Nessian Boar).
    TriggerBlocksSource,
    /// True if the trigger's object (the just-cast spell, `trigger_source`)
    /// has the same name the effect's source stamped via `Effect::NameCard`
    /// (`CardInstance.named_card`). Gates "whenever an opponent casts a spell
    /// with the chosen name" triggers — Silverquill Silencer.
    TriggerObjectNameMatchesNamedCard,
    /// True if the trigger's object (the triggering spell/permanent,
    /// `trigger_source`) has a creature type matching the effect source's
    /// ETB-chosen `chosen_creature_type` (Changeling satisfies any type).
    /// Gates chosen-type *event* triggers — "whenever you cast a spell of
    /// the chosen type" (Vanquisher's Banner) and "whenever a creature of
    /// the chosen type enters or attacks" (Kindred Discovery).
    TriggerObjectIsChosenType,
    /// True if any player `who` resolves to attacked with a creature this
    /// turn (Raid, CR 702.108 ability word). Backed by
    /// `Player.attacked_this_turn`.
    PlayerAttackedThisTurn { who: PlayerRef },
    /// True if `who` has paid or lost at least `n` {E} this turn. Backed by
    /// `Player.energy_spent_this_turn` (Izzet Generatorium's "{T}: Draw a card.
    /// Activate only if you've paid or lost four or more {E} this turn").
    EnergyPaidThisTurnAtLeast { who: PlayerRef, n: u32 },
    /// True if a creature `who` controlled dealt combat damage to a player
    /// this turn (CR 702.179 — Freerunning's alt-cost gate). Backed by
    /// `Player.dealt_combat_damage_to_player_this_turn`.
    DealtCombatDamageToPlayerThisTurn { who: PlayerRef },
    /// True if a creature other than the predicate's source entered the
    /// battlefield under `who`'s control last turn (Ephara, God of the
    /// Polis). Backed by `Player.creatures_entered_last_turn`.
    AnotherCreatureEnteredControlLastTurn { who: PlayerRef },
    /// True if any player `who` resolves to has cast a blue or black spell
    /// this turn (Veil of Summer's conditional cantrip).
    CastBlueOrBlackThisTurn { who: PlayerRef },
    /// "[who] cast a [colors] [types] spell this turn." Both lists are
    /// disjunctive and an empty list matches anything, so `{colors: [Blue],
    /// types: []}` is Ricochet Trap and `{colors: [Red], types: [Instant,
    /// Sorcery]}` is Refraction Trap. Reads `Player.spell_casts_this_turn`.
    CastSpellThisTurnWith {
        who: PlayerRef,
        colors: Vec<Color>,
        types: Vec<crate::card::CardType>,
    },
    /// "[who] has been dealt damage by `at_least` different creatures this
    /// turn" (Inferno Trap). Reads `Player.creatures_that_damaged_me_this_turn`.
    DamagedByCreaturesThisTurnAtLeast { who: PlayerRef, at_least: u32 },
    /// "[who] had `at_least` lands enter the battlefield under their control
    /// this turn" (Lavaball Trap). Reads `Player.lands_entered_this_turn`.
    LandsEnteredThisTurnAtLeast { who: PlayerRef, at_least: u32 },
    /// "[who] had a creature matching `filter` enter the battlefield under
    /// their control this turn" (Permafrost Trap). Matches over
    /// `Player.creatures_entered_this_turn`, so a creature that has since left
    /// the battlefield still counts.
    CreatureEnteredThisTurnMatching { who: PlayerRef, filter: SelectionRequirement },
    /// "A creature spell [who] cast this turn was countered by a spell or
    /// ability an opponent controlled" (Summoning Trap). Reads
    /// `Player.creature_spell_countered_by_opponent_this_turn`, stamped at the
    /// single counter funnel.
    CreatureSpellCounteredByOpponentThisTurn { who: PlayerRef },
    /// "A noncreature permanent under [who]'s control was destroyed this turn
    /// by a spell or ability an opponent controlled" (Cobra Trap). Reads
    /// `Player.noncreature_destroyed_by_opponent_this_turn`, stamped in the
    /// destroy funnel where the destroying effect's controller is known.
    NoncreaturePermanentDestroyedByOpponentThisTurn { who: PlayerRef },
    /// True if any player `who` resolves to discarded a *nonland* card within
    /// the current effect resolution. Backed by
    /// `GameState.nonland_cards_discarded_per_player_this_resolution`. Gates
    /// Kroxa's "each opponent who didn't discard a nonland card this way loses
    /// 3 life" — pair `Not(DiscardedNonlandThisEffect { Triggerer })` with a
    /// per-opponent `ForEach` so each opponent is judged independently.
    DiscardedNonlandThisEffect { who: PlayerRef },
    /// True if `who` discarded at least one card (any type) within the current
    /// effect resolution. Backed by
    /// `GameState.cards_discarded_per_player_this_resolution`. Gates "if you
    /// discarded a card this way, …" riders (Fanatic of the Harrowing).
    DiscardedThisEffect { who: PlayerRef },
    /// The most recently discarded card (cost or effect) had this creature
    /// type — Necromancer's Stockpile's "if the discarded card was a Zombie".
    /// Backed by `GameState.last_discarded_creature_types`.
    LastDiscardedHasCreatureType(crate::card::CreatureType),
    /// `who` has had at least `at_least` cards leave their graveyard
    /// this turn. Backed by `Player.cards_left_graveyard_this_turn`.
    /// Used by Lorehold "if a card left your graveyard this turn"
    /// payoffs — Living History's combat trigger, Primary Research's
    /// end-step draw rider, Wilt in the Heat's cost reduction rider.
    CardsLeftGraveyardThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// True if an opponent of `who` has cast a spell since `who`'s last turn
    /// ended (I Bask in Your Silent Awe's abandon check). Reads
    /// `Player.opponent_cast_spell_since_your_turn`, cleared at `who`'s
    /// cleanup.
    OpponentCastSpellSinceYourTurn { who: PlayerRef },
    /// CR 701.19 — true if `who` (any matching player) searched their own
    /// library this turn. `PlayerRef::EachOpponent` is satisfied by any one
    /// opponent. Archive Trap's "rather than pay" gate.
    SearchedLibraryThisTurn { who: PlayerRef },
    /// CR 702.76 — Prowl gate: true if a creature of any listed type under
    /// the resolving controller's control dealt combat damage to a player
    /// this turn (`Player.prowl_types_this_turn`; Changeling damage
    /// satisfies any type).
    ProwlTypeDealtCombatDamage { types: Vec<crate::card::CreatureType> },
    /// True if any player matching `who` had at least `at_least` cards put
    /// into their graveyard from anywhere this turn (CR 700.4 tally —
    /// `Player.cards_to_graveyard_this_turn`). Ravenous Trap's free
    /// alt-cost gate.
    CardsToGraveyardThisTurnAtLeast { who: PlayerRef, at_least: u32 },
    /// "[N] or more creature cards were put into graveyards from anywhere this
    /// turn" (Case of the Gorgon's Kiss). Counts every player's tally.
    CreatureCardsToGraveyardThisTurnAtLeast(u32),
    /// "[N] or more sources you controlled dealt damage this turn" (Case of
    /// the Burning Masks). Counts distinct sources per controller-at-the-time.
    SourcesYouControlledDealtDamageThisTurnAtLeast(u32),
    /// `who` has cast at least `at_least` spells on the current turn.
    /// Backed by `Player.spells_cast_this_turn`. Used by Burrog Barrage
    /// ("if you've cast another instant or sorcery spell this turn, …")
    /// and similar pumps that key off spell-count.
    SpellsCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// True if `who` has **not** cast any spell from their hand this turn
    /// (CR 601). Backed by `Player.spells_cast_from_hand_this_turn == 0` —
    /// casts from exile/graveyard/command zone don't count. Gates "if you
    /// haven't cast a spell from your hand this turn" (Prairie Dog,
    /// Emergent Haunting).
    NoSpellCastFromHandThisTurn { who: PlayerRef },
    /// True while the just-cast spell is the first noncreature spell cast this
    /// turn (any player). Backed by `GameState.noncreature_spells_cast_this_turn`
    /// (already incremented for the current cast at trigger time, so the first
    /// noncreature spell reads == 1). Nullstone Gargoyle. Pair with a
    /// noncreature-spell event filter so a later creature spell doesn't match.
    FirstNoncreatureSpellThisTurn,
    /// `who` has cast *exactly* `count` spells so far this turn. Backed by
    /// `Player.spells_cast_this_turn` (already incremented for the current
    /// cast at trigger time). Used by "whenever a player casts their second
    /// spell each turn" triggers (Ledger Shredder) — pair with
    /// `PlayerRef::Triggerer` + `EventScope::AnyPlayer` so it reads the
    /// caster's own count and fires exactly on the Nth spell.
    SpellsCastThisTurnEquals { who: PlayerRef, count: Value },
    /// No spells were cast by any player during the previous turn — the
    /// classic Innistrad day-side werewolf transform check. Backed by
    /// `GameState.spells_cast_last_turn == 0`.
    NoSpellsCastLastTurn,
    /// Two or more spells were cast during the previous turn — the night-side
    /// werewolf "transform back" check. Backed by
    /// `GameState.spells_cast_last_turn >= 2`.
    TwoOrMoreSpellsCastLastTurn,
    /// At least `at_least` creatures controlled by `who` died this turn.
    /// Backed by `Player.creatures_died_this_turn` (bumped from the SBA
    /// dies handler and `remove_to_graveyard_with_triggers`). Used by
    /// Witherbloom "if a creature died under your control this turn, …"
    /// end-step payoffs (Essenceknit Scholar).
    CreaturesDiedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// A creature matching `filter` died this turn, under any controller
    /// ("if a non-Zombie creature died this turn" — Undead Sprinter). Read
    /// against the death-time definitions in `creature_deaths_this_turn`.
    CreatureDiedThisTurnMatching { filter: SelectionRequirement },
    /// CR 122 — at least `at_least` *different kinds* of counters exist among
    /// the creatures `who` controls (Hundred-Battle Veteran's "three or more
    /// different kinds of counters among creatures you control"). Counts
    /// distinct `CounterType`s, not totals.
    DistinctCounterKindsAmongCreaturesAtLeast { who: PlayerRef, at_least: u32 },
    /// `who` has sacrificed at least `at_least` permanents this turn. Backed
    /// by `Player.permanents_sacrificed_this_turn`. Used by "if you
    /// sacrificed a permanent this turn" payoffs (Sawblade Skinripper).
    PermanentsSacrificedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has sacrificed at least one *artifact* this turn. Backed by
    /// `Player.artifacts_sacrificed_this_turn`. Used by "if you've sacrificed an
    /// artifact this turn" riders (Suspicious Detonation, Furtive Courier).
    SacrificedArtifactThisTurn { who: PlayerRef },
    /// At least `at_least` creatures died this turn under **any** player's
    /// control — the global "Morbid" condition (CR 700.4 "a creature died
    /// this turn"). Sums `Player.creatures_died_this_turn` across all
    /// players, so a removal spell that killed an opponent's creature
    /// earlier this turn satisfies it. Cleaner than OR-ing
    /// `CreaturesDiedThisTurnAtLeast` over each seat. Used by Tragic Slip.
    CreaturesDiedThisTurnTotalAtLeast { at_least: Value },
    /// `who` has caused at least `at_least` cards to be exiled this turn.
    /// Backed by `Player.cards_exiled_this_turn`. Used by Strixhaven
    /// "if one or more cards were put into exile this turn" payoffs
    /// (Ennis the Debate Moderator).
    CardsExiledThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// "…if there are N or more cards in exile" (Ketramose's attack / block
    /// gate). Counts every owner's exiled cards.
    CardsInExileAtLeast(u32),
    /// `who` has cast at least `at_least` instant **or** sorcery spells on
    /// the current turn. Refines `SpellsCastThisTurnAtLeast` (which
    /// counts every spell type) for cards that explicitly gate on the
    /// "instant or sorcery" subset — Potioner's Trove's "Activate only
    /// if you've cast an instant or sorcery spell this turn", future
    /// Magecraft-adjacent payoffs. Backed by
    /// `Player.instants_or_sorceries_cast_this_turn`.
    InstantsOrSorceriesCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has cast at least `at_least` creature spells on the current
    /// turn. Backed by `Player.creatures_cast_this_turn`. Reserved for
    /// future "if you've cast a creature spell this turn, …" payoffs.
    CreaturesCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has cast at least `at_least` noncreature spells on the current
    /// turn. Backed by `Player.noncreature_spells_cast_this_game_turn` — the
    /// noncreature half of "if you've cast both a creature and a noncreature
    /// spell this turn" (Eshki Dragonclaw).
    NoncreatureSpellsCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// True if the spell pointed to by `Selector::TriggerSource` (typically
    /// the just-cast spell during a `SpellCast` trigger evaluation) has at
    /// least one chosen target matching `filter`. Used by Strixhaven's
    /// **Repartee** trigger — "Whenever you cast an instant or sorcery
    /// spell that targets a creature, …" — by chaining
    /// `cast_is_instant_or_sorcery()` AND `CastSpellTargetsMatch(Creature)`.
    /// Evaluated against the topmost matching `StackItem::Spell`'s `target`
    /// slot.
    CastSpellTargetsMatch(SelectionRequirement),
    /// The just-cast spell has exactly one target in total and it matches the
    /// filter ("…that targets only a single creature you control" — Leyline
    /// of Resonance). Counts slot 0 plus every additional target.
    CastSpellTargetsOnlyOneMatching(SelectionRequirement),
    /// True if the just-cast spell (located via `ctx.trigger_source`) is
    /// itself a card matching `filter` — e.g. a noncreature spell (Sprite
    /// Dragon, Dragon's Rage Channeler) or an artifact spell. Evaluated
    /// against the topmost matching `StackItem::Spell`'s card definition.
    CastSpellMatches(SelectionRequirement),
    /// The just-cast spell matches `filter` **and** is the first such spell its
    /// caster has cast this turn ("if it's the first instant spell … you've
    /// cast this turn" — Alania, Divergent Storm). Reads
    /// `Player.spell_ids_cast_this_turn`, which already includes this cast.
    CastSpellFirstMatchingThisTurn(SelectionRequirement),
    /// On an `AuraAttachedToAny` trigger: the Aura (`ctx.trigger_source`) is
    /// attached to a nonland permanent an opponent controls whose mana value is
    /// at most the Aura's (Eriette, the Beguiler).
    AuraHostIsCheaperOpponentPermanent,
    /// True if the just-cast spell (located via `ctx.trigger_source`) was cast
    /// as an Adventure (CR 715 — its `adventuring` flag is set). Powers
    /// Chancellor of Tales' "whenever you cast an Adventure spell".
    CastSpellIsAdventure,
    /// True if the card pointed to by `ctx.trigger_source` (the just-cast
    /// spell or just-played land) shares at least one card type with the card
    /// exiled by this source (`exiled_with == ctx.source`). Drives the
    /// Innistrad "Cemetery" cycle's intervening-`if` — "Whenever a player
    /// plays a land or casts a spell, if it shares a card type with the
    /// exiled card, …" (Cemetery Gatekeeper, Cemetery Protector).
    SharesCardTypeWithExiledBySource,
    /// The just-cast spell shares a colour or mana value with the card exiled
    /// with the trigger's source (Thought Prison's imprint).
    CastSharesColorOrManaValueWithExiledBySource,
    /// The activation's chosen X equals the mana value of the card exiled with
    /// the source (Soul Foundry's "{X}, {T}" — X is that card's mana value).
    ExiledWithSourceManaValueIsX,
    /// True if the just-cast spell (via `ctx.trigger_source`) was kicked —
    /// read off the stack instance's `kicked` flag, so cast triggers
    /// ("when you cast this spell, if it was kicked" — Scourge of the
    /// Skyclaves) see the kicker state before resolution.
    CastSpellWasKicked,
    /// "If you control a commander" (Akroma's Will, Jeska's Will) — true
    /// when any battlefield permanent the effect's controller controls is
    /// one of their designated commanders (`Player.commanders`).
    YouControlACommander,
    /// True if the spell pointed to by `ctx.trigger_source` (the just-cast
    /// spell driving a `SpellCast` trigger) has at least one `{X}` symbol
    /// in its mana cost. Used by Quandrix's "whenever you cast a spell
    /// with `{X}` in its mana cost" payoffs (Geometer's Arthropod,
    /// Matterbending Mage, Paradox Surveyor's reveal filter). Evaluated
    /// against the topmost matching `StackItem::Spell`'s `card.definition.
    /// mana_cost` via `ManaCost::has_x()`.
    CastSpellHasX,
    /// True if the just-cast spell's total mana spent (the value stashed
    /// on `StackItem::Spell.mana_spent` at cast time, threaded onto the
    /// `StackItem::Trigger.mana_spent`) is at least `at_least`. Powers
    /// Opus's "if five or more mana was spent to cast that spell"
    /// branches (Deluge Virtuoso, Expressive Firedancer-style bigger-
    /// payoff modal) and Increment's "mana spent > P or T" gate (read
    /// from `ctx.mana_spent` at trigger-resolution time).
    CastSpellManaSpentAtLeast(u32),
    /// CR 702.137 (Adamant) — at least `at_least` mana of `color` was spent
    /// casting the resolving spell. Reads the per-color payment breakdown
    /// stamped at cast time (`EffectContext.mana_spent_by_color`).
    ManaSpentOfColorAtLeast { color: crate::mana::Color, at_least: u32 },
    /// "If [color] mana was spent to cast this" read from the *source
    /// permanent's* own cast provenance (`CardInstance.cast_mana_spent_by_color`),
    /// not the live spell-resolution context. Used by ETB triggers that look
    /// back at how their own creature was paid for (Gruul Scrapper's "if {R}
    /// was spent, gain haste"; Steamcore Weird's "if {R} was spent, deal 2").
    SourceCastWithColorSpent { color: crate::mana::Color, at_least: u32 },
    /// True if no *colored* mana was spent casting the just-cast spell
    /// (`ctx.trigger_source`) — Void Mirror's counter gate. Free casts
    /// (suspend, cascade) spend no mana, so they match.
    CastSpellNoColoredManaSpent,
    /// True if the just-cast spell (`ctx.trigger_source`) is one of the source
    /// permanent's chosen color (`CardInstance.chosen_color` on `ctx.source`).
    /// Diamond Mare's "whenever you cast a spell of the chosen color" payoff.
    CastSpellSharesChosenColorOfSource,
    /// Gate on whether colorless `{C}` mana was spent casting the just-cast
    /// spell (`ctx.trigger_source`). `spent: true` = "if {C} was spent" (Drowner
    /// of Truth), `false` = "if {C} wasn't spent" (Wumpus Aberration). Computed
    /// from the stack spell's total mana spent minus its colored breakdown, so
    /// {C} paid for a generic pip counts. Free casts spend nothing → not spent.
    CastSpellColorlessManaSpent { spent: bool },
    /// True if the just-cast spell's *owner* is not `ctx.controller`. A
    /// spell's owner is the player who owns the physical card (CR
    /// 108.3) — typically the same as its controller, but they diverge
    /// when one player casts another's card (Sen Triplets, Wandering
    /// Archaic, Possibility Storm, etc.). Used by Nita, Forum
    /// Conciliator's "whenever you cast a spell you don't own" trigger
    /// — fires only on the rare path where you cast a spell from an
    /// opponent's zone. Evaluated against `ctx.trigger_source`'s
    /// `StackItem::Spell.card.owner`.
    CastSpellNotOwnedByYou,
    /// The caster-relative sibling of [`Predicate::CastSpellNotOwnedByYou`]:
    /// the just-cast spell's owner isn't the player who cast it, judged off
    /// the stack item rather than the listener's controller ("whenever a
    /// player casts a spell they don't own" — Gonti, Night Minister).
    CastSpellNotOwnedByCaster,
    /// True if the just-cast spell (via `ctx.trigger_source`) was cast from
    /// exile — reads the `StackItem::Spell.card.cast_from_exile` flag.
    /// Nassari, Dean of Expression's "whenever you cast a spell from exile".
    CastSpellFromExile,
    /// True if the just-cast spell (via `ctx.trigger_source`) was cast from its
    /// owner's library — reads `StackItem::Spell.card.cast_from_library`.
    /// Melek, Izzet Paragon's "whenever you cast an instant or sorcery spell
    /// from your library, copy it".
    CastSpellFromLibrary,
    /// True if `ctx.source` (the listening permanent's id) is currently
    /// in the engine's `permanents_gained_counter_this_turn` set — i.e.
    /// the listening permanent has had one or more counters put on it
    /// during the current turn. Used by Fractal Tender's end-step rider
    /// ("if you put a counter on this creature this turn, …"). Cleared
    /// at cleanup along with the other "this turn" tallies.
    SourceGainedCounterThisTurn,
    /// True when `ctx.source` (the ability's source permanent) currently has at
    /// least `n` counters of `counter` on it. Powers "if there are N or more
    /// [kind] counters on it, …" intervening-`if` riders (Charitable Levy's
    /// three-collection-counter sacrifice threshold).
    SourceHasCountersAtLeast { counter: crate::card::CounterType, n: u32 },
    /// CR 716.2 — true when `ctx.source` (a Class enchantment) is exactly at
    /// level `n`. Gates a Class's `{cost}: Level N` activated ability (its
    /// `condition` is `SourceClassLevelIs(N-1)`) and its "when this Class
    /// becomes level N" trigger (a `ClassLevelReached` trigger filtered on
    /// `SourceClassLevelIs(N)`).
    SourceClassLevelIs(u8),
    /// CR 716.2 — true when `ctx.source` (a Class enchantment) is at level `n`
    /// or higher. Gates a Class's level-`n` triggered abilities so they only
    /// fire once the Class has reached that level.
    SourceClassLevelAtLeast(u8),
    /// True if the just-cast spell's total mana spent is **strictly
    /// greater than** the source permanent's power or toughness. Used
    /// by SOS's Increment keyword payoff: "Whenever you cast a spell,
    /// if the amount of mana you spent is greater than this creature's
    /// power or toughness, put a +1/+1 counter on this creature."
    /// Evaluated against `ctx.source` (the listening permanent) at
    /// trigger-evaluation time.
    IncrementSatisfied,
    /// True if `who`'s `zone` contains at least `at_least` cards whose
    /// `definition.name` matches the resolving spell's name. Used by
    /// Dragon's Approach's "if you have four or more cards named Dragon's
    /// Approach in your graveyard, search your library for a Dragon
    /// creature card" rider. The name is read from
    /// `EffectContext.source_name` (the resolving spell's name); when no
    /// source is available the predicate is `False`.
    SameNamedInZoneAtLeast { who: PlayerRef, zone: Zone, at_least: Value },
    /// True when the resolving spell was cast from its caster's
    /// graveyard (typically via Flashback / Aftermath / Jump-Start /
    /// Yawgmoth's Will-style "cast from graveyard" effects). Backed by
    /// `EffectContext.cast_from_hand == false`, which is stamped by
    /// `for_spell_with_source` from the resolving card's
    /// `CardInstance.cast_from_hand` flag. Used by Increasing Vengeance
    /// ("If this spell was cast from a graveyard, copy that spell twice
    /// instead") and Antiquities on the Loose's "cast from anywhere
    /// other than your hand" rider.
    CastFromGraveyard,
    /// True while the resolving spell or ability is controlled by an opponent
    /// of the evaluating controller (Pure Intentions' discard rider, Sacred
    /// Ground's land destruction). Reads `GameState.resolution_causer`.
    #[serde(alias = "DiscardCausedByOpponent")]
    CausedByOpponentSpellOrAbility,
    /// True when the resolving spell was cast from its caster's hand
    /// (the typical case). Inverse of `CastFromGraveyard`. Reserved for
    /// "if you cast this spell from your hand, …" rider patterns —
    /// Quandrix, the Proof's "instant and sorcery spells you cast from
    /// your hand have cascade" static gates against this predicate.
    /// Note: triggers / activated abilities default `cast_from_hand`
    /// to `true`, so this predicate evaluates as `True` outside of
    /// spell-resolution context too.
    CastFromHand,
    /// The two player refs resolve to the same seat — "whenever enchanted
    /// player is dealt damage" is `SamePlayer(TriggerEventPlayer,
    /// EnchantedPlayer)` (Grievous Wound).
    SamePlayer(PlayerRef, PlayerRef),
    /// True while the current turn is in (or has only ever reached) its first
    /// combat phase — i.e. no extra combat has begun yet. Gates "if it's the
    /// first combat phase of the turn, …" attack riders (Genji Glove) so the
    /// extra combat they grant doesn't re-trigger and loop.
    IsFirstCombatPhaseThisTurn,
    /// True while the current turn is in (or has only ever reached) its first
    /// end step — i.e. no extra end step has begun yet. Gates "if it's the
    /// first end step of the turn, …" riders (Y'shtola Rhul).
    IsFirstEndStepThisTurn,
    /// True while the current turn is in its first upkeep step — gates
    /// Paradox Haze's "first upkeep step of your turn" trigger so the extra
    /// upkeep it grants doesn't loop.
    IsFirstUpkeepThisTurn,
    /// True when the resolving spell was kicked (CR 702.32) — its optional
    /// kicker cost was paid at cast time. Reads `EffectContext.kicked`,
    /// stamped from the resolving `CardInstance.kicked` flag. Used by
    /// "if this spell was kicked, …" riders (Tear Asunder). Non-spell
    /// contexts default `kicked` to `false`.
    SpellWasKicked,
    /// CR 702.32b — the resolving spell was kicked with option `n` of its
    /// `kicker_options` (Anavolver's "kicked with its {1}{U} kicker").
    SpellWasKickedWith(u8),
    /// "If the sacrificed permanent was an artifact" — reads the
    /// additional-cast-cost sacrifice scratch (Foundry Helix).
    SacrificedWasArtifact,
    /// "If an outlaw was sacrificed this way" — reads the sacrifice-cost
    /// scratch stamped by the activated `sac_other_filter` path (Boneyard
    /// Desecrator).
    SacrificedWasOutlaw,
    /// "If the sacrificed permanent was a Vehicle" — reads the sacrifice
    /// scratch (Hellish Sideswipe's draw rider).
    SacrificedWasVehicle,
    /// "If the sacrificed permanent was [color]" — reads the sacrificed
    /// permanent's colors off the sacrifice scratch (Lyzolda, the Blood
    /// Witch: damage if red, draw if black).
    SacrificedWasColor(Color),
    /// "If the discarded card was multicolored" — reads the last-discarded
    /// scratch (Stormscale Anarch's doubled damage).
    LastDiscardedWasMulticolored,
    /// The last card discarded during this resolution is the given color
    /// (Chandra Ablaze's "if a red card is discarded this way").
    LastDiscardedWasColor(crate::mana::Color),
    /// The entering permanent bound to `ctx.trigger_source` arrived from a
    /// graveyard this turn, or was cast from one (escape / unearth — read
    /// via `!cast_from_hand`; exile-casts over-trigger, noted per card).
    /// Breathless Knight.
    TriggerSourceEnteredFromGraveyard,
    /// The entering permanent bound to `ctx.trigger_source` entered because its
    /// spell was cast (reads `CardInstance.entered_by_cast`). Gates "whenever a
    /// creature you control enters, if you cast it, …" (The Sibsig Ceremony).
    TriggerSourceEnteredByCast,
    /// CR 702.85 — Heroic. True when the just-cast spell (the trigger source,
    /// an `EntityRef::Card` on the stack) targets the trigger's own source
    /// permanent (`ctx.source`). Gates "Whenever you cast a spell that targets
    /// this creature, …" triggers. Reads the cast spell's `target` /
    /// `additional_targets` off the stack.
    CastSpellTargetsSource,
    /// CR 702.176 — true iff the Bargain additional cost was paid (an
    /// artifact, enchantment, or token sacrificed) at cast time. Reads
    /// `EffectContext.bargained`, stamped from `CardInstance.bargained`.
    SpellWasBargained,
    /// CR 702.187 — true iff this spell was cast from a graveyard for its
    /// Mayhem cost. Reads `EffectContext.cast_via_mayhem`, stamped from
    /// `CardInstance.cast_via_mayhem`. Gates "if this spell's mayhem cost was
    /// paid, …" riders (Sandman's Quicksand).
    SpellWasMayhem,
    /// CR 701.67 — true iff this spell's optional "you may waterbend {N}"
    /// additional cost was paid. Reads `EffectContext.cast_via_waterbend`,
    /// stamped from `CardInstance.cast_via_waterbend`. Gates "if its additional
    /// cost was paid, …" riders (Katara, Seeking Revenge; Secret of Bloodbending).
    SpellWasWaterbend,
    /// CR 701.59 — true iff this spell's "collect evidence N" additional cost
    /// was paid. Reads `EffectContext.cast_collected_evidence`, stamped from
    /// `CardInstance.cast_collected_evidence`. Gates "if evidence was collected,
    /// …" branches (Behind the Mask, Analyze the Pollen).
    SpellCollectedEvidence,
    /// CR 702.165 — true iff the effect's source permanent was cast with its
    /// Gift promised (reads `CardInstance.gift_promised`, which persists onto
    /// the battlefield). Gates a permanent-gift card's "if the gift was
    /// promised, …" ETB (Scrapshooter, Starforged Sword).
    SourceGiftPromised,
    /// True if the most recently discarded card this resolution had mana value
    /// ≤ `n`. Reads `GameState.last_discarded_mana_value` (Hollow Marauder's
    /// "draw unless they discarded a card with mana value 4 or greater").
    LastDiscardedManaValueAtMost(u32),
    /// CR 715 — true iff `ctx.controller` owns a card in exile that is on an
    /// Adventure (a creature card cast for its Adventure half, waiting in exile
    /// to be recast). Gates Howling Galefang's "has haste as long as you own a
    /// card in exile that has an Adventure".
    OwnExiledAdventureCard,
    /// True if any opponent of `ctx.controller` controls more lands
    /// than `ctx.controller` does. Backed by walking the battlefield
    /// and counting `Land` permanents per seat. Used by catch-up ramp
    /// spells like Gift of Estates ("If an opponent controls more
    /// lands than you, …"), Tithe, Knight of the White Orchid's ETB
    /// trigger, and Land Tax.
    OpponentControlsMoreLandsThanYou,
    /// True if any opponent of `ctx.controller` has a strictly higher life
    /// total. Linvala, the Preserver's first ETB ("if an opponent has more
    /// life than you, you gain 5 life").
    AnOpponentHasMoreLife,
    /// True if any opponent of `ctx.controller` controls strictly more
    /// creatures. Linvala, the Preserver's second ETB ("if an opponent
    /// controls more creatures than you, create a 3/3 Angel").
    AnOpponentControlsMoreCreatures,
    /// True if any opponent of `ctx.controller` has strictly more cards in
    /// hand. Beza, the Bounding Spring's "draw a card if an opponent has more
    /// cards in hand than you".
    AnOpponentHasMoreCardsInHand,
    /// CR (Innistrad "Coven") — `who` controls three or more creatures with
    /// different powers (every pairwise power distinct counts as ≥3 distinct
    /// power values). Gates Coven attack triggers and "activate only if …"
    /// abilities (Sigarda, Champion of Light; Dawnhart Mentor; Sungold Sentinel).
    CovenActive { who: PlayerRef },
    /// CR 702.166 — Corrupted (Phyrexia: All Will Be One ability word): an
    /// opponent of `who` has three or more poison counters. Gates Corrupted
    /// triggers / "as long as" statics.
    CorruptedActive { who: PlayerRef },
    /// Churning Reservoir — an oil counter was removed from `who`'s permanent
    /// this turn, or an oil-countered permanent of theirs hit a graveyard.
    OilActivityThisTurn { who: PlayerRef },
    /// `who` controls a creature whose power is greater than or equal to every
    /// creature's power on the battlefield — i.e. controls the creature with
    /// the greatest power, or one tied for it (Summon: Fenrir III "Ecliptic
    /// Growl"). False if `who` controls no creature.
    ControlsGreatestPowerCreature { who: PlayerRef },
    /// True when exactly one creature is attacking this combat — the
    /// CR 702.83a "attacks alone" condition that gates Exalted. Read
    /// from `GameState.attacking.len() == 1`. Outside a combat with
    /// declared attackers it evaluates `false`. Combined with an
    /// `Attacks / YourControl` trigger it implements the printed
    /// Exalted reminder ("Whenever a creature you control attacks
    /// alone, that creature gets +1/+1 until end of turn").
    AttackingAlone,
    /// True when at least `n` creatures are attacking this combat. Powers
    /// the Battalion ability word ("Whenever this creature and at least two
    /// other creatures attack" → `n == 3`). Read from
    /// `GameState.attacking.len() >= n`; `false` outside a combat with
    /// declared attackers.
    AttackingWithAtLeast(u32),
    /// **Pack tactics** (OTJ ability word) — `who` attacked this combat with
    /// creatures whose total power is at least `at_least`. Summed from the
    /// computed power of every attacking creature controlled by `who`
    /// (`GameState.attacking`); `false` outside a combat with declared
    /// attackers. Battle Cry Goblin's "if you attacked with creatures with
    /// total power 6 or greater this combat".
    AttackedWithTotalPowerAtLeast { who: PlayerRef, at_least: u32 },
    /// `who` declared at least `at_least` attackers this combat. Counts every
    /// attacking creature controlled by `who` (`GameState.attacking`); `false`
    /// outside a combat with declared attackers. Argent Dais's "whenever two
    /// or more creatures attack" (with `who: ActivePlayer`).
    AttackedWithCountAtLeast { who: PlayerRef, at_least: u32 },
    /// `who` declared one or more attackers this combat matching `filter`.
    /// Reads `GameState.attacking`; `false` outside a combat with declared
    /// attackers. Gates "Whenever you attack with one or more creatures with
    /// flying / power 4+" triggers (Teo, Spirited Glider; Bitter Work).
    AttackedWithCreatureMatching { who: PlayerRef, filter: SelectionRequirement },
    /// CR 700.13 — `who` has committed a crime this turn. Backed by
    /// `Player.committed_crime_this_turn`. Powers "as long as / if you've
    /// committed a crime this turn" riders (Nimble Brigand's evasion).
    CommittedCrimeThisTurn { who: PlayerRef },
    /// CR 708 — a permanent entered the battlefield face down under `who`'s
    /// control, or they turned a permanent face up, this turn. Backed by
    /// `Player.face_down_activity_this_turn` (Oblivious Bookworm).
    FaceDownActivityThisTurn { who: PlayerRef },
    /// CR 709.5 — `who` controls at least `count` unlocked doors among the
    /// Rooms they control (each Room can have up to two doors unlocked). Powers
    /// "as long as there are N or more unlocked doors among Rooms you control"
    /// statics (Rampaging Soulrager). Reads `CardInstance.unlocked_doors`
    /// (a per-door bitmask).
    UnlockedDoorsControlledAtLeast { who: PlayerRef, count: u32 },
    /// CR 709.5 — `who` controls at least `count` unlocked doors with
    /// **distinct names** (Promising Stairs' "eight or more different names
    /// among unlocked doors of Rooms you control" win condition).
    DistinctUnlockedDoorNamesAtLeast { who: PlayerRef, count: u32 },
    /// CR 700.12 — `who` controls an **outlaw**: a creature that is an
    /// Assassin, Mercenary, Pirate, Rogue, or Warlock. Powers "as long as you
    /// control an outlaw" / "if you control an outlaw" riders (Take the Fall).
    ControlsOutlaw { who: PlayerRef },
    /// CR 700.4-ish — **Delirium**: `who`'s graveyard holds cards of at
    /// least 4 distinct card types (the count of *types*, not cards). Backed
    /// by scanning the graveyard's `definition.card_types`. Used by Unholy
    /// Heat, Dragon's Rage Channeler, etc.
    DeliriumActive { who: PlayerRef },
    /// Descend N (LCI ability word) — `who` has `count` or more permanent
    /// cards in their graveyard. Gates "Descend 4 —" P/T pumps and ETB riders
    /// (Frilled Cave-Wurm, Basking Capybara, Coati Scavenger).
    DescendActive { who: PlayerRef, count: u32 },
    /// CR 700.11 — `who` has descended this turn (a permanent card was put into
    /// their graveyard from anywhere). Gates "if you descended this turn" riders
    /// (Deep Goblin Skulltaker, Child of the Volcano).
    DescendedThisTurn { who: PlayerRef },
    /// "If an artifact entered the battlefield under `who`'s control this turn"
    /// (Akal Pakal). Reads `Player.artifacts_entered_this_turn`.
    ArtifactEnteredThisTurn { who: PlayerRef },
    /// Hedron Alignment — "you own a card with this source's name in exile, in
    /// your hand, in your graveyard, and on the battlefield". Checks all four
    /// zones for a card sharing the source's name and owned by `who`.
    OwnsSourceNamedCardInEveryZone { who: PlayerRef },
    /// "If a planeswalker entered the battlefield under `who`'s control this
    /// turn" (Oath of Chandra). Reads `Player.planeswalkers_entered_this_turn`.
    PlaneswalkerEnteredThisTurn { who: PlayerRef },
    /// "If a creature entered the battlefield under `who`'s control this turn"
    /// (Zhalfirin Decoy's activation gate, Bellowing Elk's static). Reads
    /// `Player.creatures_entered_this_turn`.
    CreatureEnteredThisTurn { who: PlayerRef },
    /// "If another creature entered the battlefield under `who`'s control this
    /// turn" — self-excluding sibling of `CreatureEnteredThisTurn`. The source's
    /// own arrival doesn't satisfy it (Bellowing Elk: "As long as you had
    /// *another* creature enter … this turn"). Compares the entered-ids list
    /// against `ctx.source`.
    AnotherCreatureEnteredThisTurn { who: PlayerRef },
    /// **Celebration** (WOE) — two or more nonland permanents entered under
    /// `who`'s control this turn. Reads
    /// `Player.nonland_permanents_entered_this_turn` (Armory Mice, Belligerent
    /// of the Ball).
    CelebrationActive { who: PlayerRef },
    /// **Threshold** (Odyssey ability word) — `who` has seven or more cards in
    /// their graveyard. Gates "as long as / if" threshold riders (Nimble
    /// Mongoose, Werebear, Mystic Enforcer).
    ThresholdActive { who: PlayerRef },
    /// **Metalcraft** (Scars of Mirrodin) — `who` controls three or more
    /// artifacts. Gates metalcraft riders (Vault Skirge-era; Galvanic Blast).
    MetalcraftActive { who: PlayerRef },
    /// **Ferocious** (Khans of Tarkir) — `who` controls a creature with power
    /// four or greater. Gates ferocious riders (Temur Battle Rage, Savage
    /// Punch).
    FerociousActive { who: PlayerRef },
    /// **Hellbent** (Dissension) — `who` has no cards in hand. Gates hellbent
    /// riders (Anthem of Rakdos, Demonfire).
    HellbentActive { who: PlayerRef },
    /// **Formidable** (Dragons of Tarkir ability word) — the total power of
    /// creatures `who` controls is eight or greater. Gates formidable
    /// activated/triggered riders (Boltwing Marauder-era, Atarka Monument).
    FormidableActive { who: PlayerRef },
    /// "If `who` controls each creature on the battlefield with the greatest
    /// power" (Might Makes Right). True when every creature tied for the
    /// highest power is theirs; vacuously true on an empty board.
    ControlsEachGreatestPowerCreature { who: PlayerRef },
    /// CR 606.3 — "`who` activated a loyalty ability of a planeswalker this
    /// turn." The Chain Veil's end-step upkeep tax reads its negation.
    ActivatedLoyaltyThisTurn { who: PlayerRef },
}

// ── Duration ─────────────────────────────────────────────────────────────────

/// How long a temporary effect persists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Duration {
    /// Expires at the next end-of-turn Cleanup step.
    EndOfTurn,
    /// Expires when combat ends this turn.
    EndOfCombat,
    /// Until controller's next untap step.
    UntilYourNextUntap,
    /// Until controller's next upkeep — one step later than
    /// `UntilYourNextUntap`, so the affected permanent is still animated as
    /// its controller untaps (Xenic Poltergeist).
    UntilYourNextUpkeep,
    /// Until the start of the next turn.
    UntilNextTurn,
    /// CR 611.2c — "for as long as this permanent remains tapped" (Thran
    /// Weaponry). The affected set is locked in at resolution; the SBA sweep
    /// drops the effect once the source untaps or leaves.
    WhileSourceTapped,
    /// CR 611.2c — "for as long as this Equipment/Aura remains attached to it"
    /// (Assimilation Aegis). The SBA sweep drops the effect once the source
    /// stops being attached or leaves the battlefield.
    WhileSourceAttached,
    /// Indefinite (for effects like "gain control" without a clause).
    Permanent,
}

// ── Library positions, scry modes, mana ──────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryPosition {
    Top,
    Bottom,
    Shuffled,
    /// "On top or bottom — the card's owner chooses." Used by Run Behind
    /// ("Target creature's owner puts it on their choice of the top or
    /// bottom of their library."). At `place_card_in_dest` time, the
    /// owner is asked via `Decision::OptionalTrigger { description: "Put
    /// on top of library?" }`; yes = top, no = bottom. AutoDecider's
    /// default (`Bool(false)`) collapses to bottom — preserving the
    /// previous Run-Behind behavior. ScriptedDecider can flip to top
    /// for tests.
    OwnerChoice,
    /// "The owner puts it into their library second from the top or on the
    /// bottom" (Deem Inferior). The owner chooses via `Decision::
    /// OptionalTrigger`; yes = second from top (`FromTop(1)`), no = bottom.
    /// AutoDecider defaults to bottom.
    SecondFromTopOrBottom,
    /// "Put this card Nth from the top of the owner's library." Used by
    /// Approach of the Second Sun ({6}{W}{W}: "If this spell was cast
    /// from your hand and you've cast another spell named Approach of
    /// the Second Sun this game, you win the game. Otherwise, put this
    /// spell's owner gains 7 life and puts this spell into their
    /// library seventh from the top.") and similar cards. Per CR 401.7,
    /// if the library has fewer than N cards, the card goes on the
    /// bottom instead. `FromTop(0)` is equivalent to `Top`.
    FromTop(usize),
}

/// Where the non-matching revealed cards go after a
/// `RevealUntilFind` resolves. The default (`Graveyard`) matches the
/// historical behavior baked into older catalogs; SOS Strixhaven cards
/// like Geometer's Arthropod and Paradox Surveyor print "put the rest
/// on the bottom of your library in a random order" and use
/// `BottomRandom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RevealMissDest {
    /// Misses go to the controller's graveyard (legacy / Spoils-style).
    #[default]
    Graveyard,
    /// Misses go on the bottom of the controller's library, randomized.
    /// The engine inserts each miss in the order it was revealed; with
    /// no RNG hook available the order is effectively "as-revealed",
    /// which is a reasonable approximation since gameplay doesn't read
    /// the bottom of the library in any deterministic way before the
    /// next shuffle.
    BottomRandom,
    /// Misses are shuffled into the controller's library after the find
    /// resolves (Transmogrify, Indomitable Creativity).
    ShuffleIntoLibrary,
    /// Misses are exiled — "put that card into your hand and exile all other
    /// cards revealed this way" (Sacred Guide).
    Exile,
    /// Misses join the found card in its destination — "reveal cards from the
    /// top of your library until you reveal a nonland card, then put all cards
    /// revealed this way into your hand" (Treasure Hunt).
    WithFind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneDest {
    Hand(PlayerRef),
    Library { who: PlayerRef, pos: LibraryPosition },
    Graveyard,
    Exile,
    /// CR 702.170 — exile face up and mark **plotted**, so the card's owner
    /// may cast it for free as a sorcery on a later turn. The effect-granted
    /// half of plot: no plot cost is paid (Kellan Joins Up, Jace Reawakened,
    /// Make Your Own Luck, Aven Interrupter).
    ExilePlotted,
    /// Exile, stamping `exiled_with = ctx.source` so the source can reach the
    /// card later (`Selector::CardExiledWithSource`, Kaho's free cast).
    ExileWithSourceStamp,
    /// Battlefield under `controller`, optionally tapped.
    Battlefield { controller: PlayerRef, tapped: bool },
    /// CR 407.4 — the moved card's owner's ante zone.
    Ante,
}

/// Where a countered spell goes after being lifted off the stack. The
/// default (graveyard) matches CR 701.5g; Memory Lapse routes to the
/// owner's library top, Spell Crumple routes to exile, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounteredSpellZone {
    /// Top of the spell-owner's library (Memory Lapse).
    OwnerLibraryTop,
    /// Bottom of the spell-owner's library.
    OwnerLibraryBottom,
    /// Owner's choice of top or bottom of their library (Subtlety).
    OwnerLibraryTopOrBottom,
    /// Owner's hand (Remand).
    OwnerHand,
    /// Exile (Spell Crumple).
    Exile,
    /// CR 702.170 — exile and mark plotted, so its owner may cast it for free
    /// as a sorcery on a later turn (Aven Interrupter).
    ExilePlotted,
    /// Exile stamped `exiled_with` = the countering source, so the source can
    /// name what it took later (Shell of the Last Kappa).
    ExileWithSource,
}

/// What mana to add to a pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManaPayload {
    /// Add one mana of any of the colors of the card imprinted on the source
    /// (a card in exile with `exiled_with == ctx.source`). The player chooses
    /// among those colors; produces nothing if there is no imprinted card or it
    /// is colorless. Chrome Mox.
    ImprintedCardColor,
    /// Add each listed color once.
    Colors(Vec<Color>),
    /// Add `amount` colorless mana.
    Colorless(Value),
    /// Add `amount` mana of one specified color (no player choice).
    /// Used by power-scaled mana abilities like Topiary Lecturer's
    /// "{T}: Add an amount of {G} equal to this creature's power" or
    /// Cryptolith Rite-style "add 1 of color X for each Y you control".
    OfColor(Color, Value),
    /// Add `amount` mana of any one color (player chooses).
    AnyOneColor(Value),
    /// Add `amount` mana of any colors (player chooses each).
    AnyColors(Value),
    /// Add `amount` mana, each pip chosen from the given color subset
    /// (player chooses per pip). The restricted-palette sibling of
    /// `AnyColors`. Used by Culling Ritual's "Add {B} or {G} for each
    /// permanent destroyed this way". Falls back to the first listed
    /// color when the decider can't choose.
    OfColors(Vec<Color>, Value),
    /// Add one mana of any color a controller's opponent's land could
    /// produce. The pool of legal colors is the union of basic-land
    /// types under any opponent's control (`Plains` → White, `Island`
    /// → Blue, `Swamp` → Black, `Mountain` → Red, `Forest` → Green).
    /// If no opponent controls a basic-typed land, falls back to
    /// colorless (so the activation never silently no-ops). Used by
    /// Fellwar Stone — `{T}: Add one mana of any color an opponent's
    /// land could produce.`
    AnyColorOpponentCouldProduce,
    /// Add one mana of any color a basic land *you* control could produce —
    /// the controller-side mirror of `AnyColorOpponentCouldProduce`. The
    /// legal-color set is the union of basic-land types under the
    /// controller's own permanents. Falls back to colorless if none.
    /// Star Compass.
    AnyColorYouCouldProduce,
    /// Add one mana of any type the *trigger's subject* land produced
    /// (Extraplanar Lens). Falls back to colorless if it produces nothing.
    AnyTypeTriggerSourceProduces,
    /// Add one mana of any color among legendary creatures and planeswalkers
    /// you control (Mox Amber). The legal-color set is the union of those
    /// permanents' colors; produces nothing when empty.
    AnyColorAmongLegendaries,
    /// "Choose a color of a permanent you control. Add one mana of that color"
    /// (Meteor Crater). Colorless-only boards produce nothing.
    AnyColorAmongYourPermanents,
    /// Player chooses a color, then adds mana of that color equal to their
    /// devotion to it (CR 700.5). Nykthos, Shrine to Nyx's second ability.
    DevotionOfChosenColor,
    /// Resolve the inner payload normally, but tag every colored pip it
    /// produces with `restriction` ("Spend this mana only to …"). Used by
    /// the Strixhaven school mana sources (Abstract Paintmage, Tablet of
    /// Discovery, Hydro-Channeler, Great Hall of the Biblioplex,
    /// Resonating Lute). Colorless pips are restricted too
    /// (`add_restricted_colorless` — Powerstone tokens, Omen Hawker's {C}).
    Restricted(Box<ManaPayload>, SpendRestriction),
    /// Like `Restricted`, but the restriction is "spend only to cast a
    /// creature spell of the source's chosen creature type, and that spell
    /// can't be countered" (Cavern of Souls). The chosen type is read off
    /// the source permanent at resolution; with none chosen the mana is
    /// added unrestricted.
    RestrictedToChosenType(Box<ManaPayload>),
    /// Like `RestrictedToChosenType`, without Cavern's uncounterable rider
    /// (Unclaimed Territory — "creature spell of the chosen type").
    RestrictedToChosenTypePlain(Box<ManaPayload>),
    /// Add one mana of the color stamped on the source's `chosen_color`
    /// (Coldsteel Heart, choose-a-color rocks). Falls back to colorless when
    /// no color was chosen.
    ChosenColorOfSource,
    /// CR 905.2b — add one mana of any color chosen as the controller drafted
    /// cards with the source's name (Paliano, the High City). Falls back to
    /// colorless outside a drafted game.
    DraftNotedColorOfSource,
}

// ── Event specification (triggers) ───────────────────────────────────────────

/// Kinds of game events a trigger can watch for. Mirrors the `GameEvent`
/// stream in [`GameEvent`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// A permanent entered the battlefield.
    EntersBattlefield,
    /// A creature died (hit a graveyard from the battlefield).
    CreatureDied,
    /// CR 700.4 — a creature *or* artifact hit a graveyard from the
    /// battlefield. Matches `GameEvent::PermanentDied` whose `is_creature`
    /// or `is_artifact` flag is set (an artifact creature counts once).
    /// "Whenever another creature or artifact you control dies" (Judge Magister
    /// Gabranth); pair with `once_per_turn` for "one or more … die … only once
    /// each turn" (G'raha Tia). Scope by `YourControl`; exclude the source with
    /// a `Predicate` filter for the "another" rider.
    CreatureOrArtifactDied,
    /// Any permanent hit a graveyard from the battlefield (matches
    /// `GameEvent::PermanentDied`). Filter the subject via
    /// `Predicate::EntityMatches { what: TriggerSource, .. }` — Carth the
    /// Lion's "or a planeswalker you control dies".
    PermanentDied,
    /// A creature was sacrificed. Per CR 701.16, "sacrifice" is a distinct
    /// game event from "die" — Mortician Beetle / Yahenni / Bone Picker
    /// ("Whenever a player sacrifices a creature") want this specific
    /// event, not a death-of-any-cause trigger. The `Effect::Sacrifice`
    /// resolver emits both events in order (CreatureSacrificed first,
    /// then CreatureDied) so existing death-triggers still fire.
    CreatureSacrificed,
    /// Any permanent was sacrificed (creature, artifact, enchantment,
    /// land, planeswalker). Per CR 701.16, this is the broader-scope
    /// sibling of `CreatureSacrificed` — "Whenever you sacrifice a
    /// permanent" payoffs (Korvold, Fae-Cursed King; Mayhem Devil;
    /// Cruel Celebrant for permanents) want this event so they catch
    /// Treasure-sac, Clue-sac, Food-sac, and land-sacrifice resolutions
    /// alongside creature sacrifices. The `Effect::Sacrifice` resolver
    /// emits this event for every sacrificed permanent, regardless of
    /// type; for creatures it additionally emits `CreatureSacrificed`
    /// (creatures fire BOTH events, matching CR 701.16's "every
    /// sacrifice of a creature is a sacrifice of a permanent" wording).
    PermanentSacrificed,
    /// Any permanent left the battlefield.
    PermanentLeavesBattlefield,
    /// CR 701.7 — a permanent was destroyed by a spell or ability (not by
    /// combat damage or a state-based action). The event's actor is the
    /// destroying effect's controller, so `EventScope::YourControl` with
    /// `actor_is_opponent` reads "a spell or ability an opponent controls
    /// destroys a permanent you control" (Karmic Justice).
    PermanentDestroyedByEffect,
    /// CR 603.6 — a creature left the battlefield *without dying* (moved to
    /// hand / exile / library, not to a graveyard). Dour Port-Mage, Three
    /// Tree Scribe. Distinct from `CreatureDied`/`PermanentLeavesBattlefield`,
    /// which fire on the graveyard exit.
    CreatureLeavesBattlefieldNotDying,
    /// A card was drawn.
    CardDrawn,
    /// The turn's first card drawn (CR 121). Fired once per turn per player;
    /// the trigger subject is the drawn card. "The first time each turn you
    /// draw a card" (God-Eternal Kefnet).
    FirstCardDrawnThisTurn,
    /// A card was discarded.
    CardDiscarded,
    /// A spell or ability an opponent controls caused a player to discard a
    /// card (CR 701.9 + `GameState.resolution_causer`). Keyed on the *discarding*
    /// player, so `EventScope::YourControl` reads "causes you to discard"
    /// (Spiritual Focus). Emitted alongside `CardDiscarded`.
    OpponentCausedYouToDiscard,
    /// A land was played.
    LandPlayed,
    /// A spell was cast.
    SpellCast,
    /// A spell was copied (CR 707.10 — the copy is created on the stack). Scope
    /// by `YourControl` for "whenever you copy …"; filter the copied spell via
    /// `Predicate::EntityMatches { what: TriggerSource, .. }`. Ral, Storm Conduit.
    SpellCopied,
    /// A creature was declared as an attacker.
    Attacks,
    /// CR 508 — "Whenever you attack": fires **once** per combat for the
    /// attacking player when they declare one or more attackers, regardless of
    /// how many. Use a `SelfSource`/`YourControl` trigger over this instead of
    /// per-attacker `Attacks` for "whenever you attack, …" abilities (Razorkin
    /// Hordecaller). Dispatched directly from `declare_attackers`.
    YouAttack,
    /// A creature was declared as a blocker. Fired once per blocker
    /// from `declare_blockers` (CR 509.1i). Dispatched in addition to
    /// the existing `BecomesBlocked` event on the attacker side.
    Blocks,
    /// A creature became blocked.
    BecomesBlocked,
    /// CR 509.3e — "whenever this creature blocks N or more creatures". Fires
    /// once, when blockers are declared, if the source is blocking at least `n`
    /// attackers (Lairwatch Giant).
    BlocksNOrMore(u32),
    /// CR 509.3e — the attacker-side mirror: "whenever this creature becomes
    /// blocked by N or more creatures".
    BecomesBlockedByNOrMore(u32),
    /// CR 701.15 — "whenever this creature regenerates" (Skeleton
    /// Scavengers' counter rider). Fires whenever a regeneration shield is
    /// applied, whatever granted it.
    Regenerated,
    /// An attacking creature finished the declare-blockers step
    /// without any blockers assigned to it (CR 509.3g — "Whenever
    /// [creature] attacks and isn't blocked"). Fires once per
    /// unblocked attacker after `declare_blockers` completes. The
    /// `BecomesBlocked` event fires for attackers WITH blockers; this
    /// is the parallel rail for unblocked attackers.
    AttacksAndIsntBlocked,
    /// Combat damage was dealt to a player by a creature.
    DealsCombatDamageToPlayer,
    /// Combat damage was dealt to a planeswalker by a creature (keyed on the
    /// dealer). Pair with `EventScope::SelfSource`/`YourControl`; the damaged
    /// planeswalker is the default target slot (Vraska's Assassin token
    /// destroys it; Vraska, Swarm's Eminence grows the dealer).
    DealsCombatDamageToPlaneswalker,
    /// The listening permanent's controller was dealt combat damage (keyed on
    /// the *recipient*, not the dealer). Pair with `EventScope::SelfSource` for
    /// "whenever combat damage is dealt to you" downsides (Risona removes an
    /// indestructible counter). The amount rides in via `Value::TriggerEventAmount`.
    ControllerDealtCombatDamage,
    /// Combat damage was dealt to a creature by a creature.
    DealsCombatDamageToCreature,
    /// "Whenever this permanent deals damage to a creature" — the
    /// combat-agnostic sibling of `DealsCombatDamageToCreature` (Neko-Te,
    /// Kumano's Blessing). Fires from both the combat step and the
    /// non-combat damage funnel, with the damaged creature bound to slot 0.
    DealsDamageToCreature,
    /// "Whenever this permanent deals combat damage" — recipient-agnostic
    /// (player, planeswalker or creature). The union of
    /// `DealsCombatDamageToPlayer` and `DealsCombatDamageToCreature`
    /// (Descendant of Kiyomaro). The damaged entity is bound to slot 0.
    DealsCombatDamage,
    /// "Whenever this permanent deals damage" — recipient- *and*
    /// combat-agnostic (Kiyomaro, First to Stand). Fires from the combat
    /// steps and the non-combat damage funnel alike.
    DealsDamage,
    /// CR 702.130 — **Enrage**: a permanent was dealt damage (combat or
    /// non-combat). Fires the source's enrage trigger. Unlike
    /// `DealsCombatDamageToCreature` (which is keyed on the *dealer* and
    /// only on combat damage), this is keyed on the *recipient* and fires
    /// on any damage — combat, burn spells, Fight, pingers — matching the
    /// printed "Whenever this creature is dealt damage" wording. The
    /// damage amount is exposed to the trigger body via
    /// `Value::TriggerEventAmount`. Used with `EventScope::SelfSource` for
    /// enrage creatures; `AnyPlayer`/`YourControl` scopes also work for
    /// "whenever a creature you control is dealt damage" payoffs.
    DealtDamage,
    /// A permanent was dealt **combat** damage — the combat-only sibling of
    /// `DealtDamage`, keyed on the *recipient*. Pair with `EventScope::
    /// SelfSource` for "whenever this creature is dealt combat damage" (Souls
    /// of the Faultless). The amount rides in via `Value::TriggerEventAmount`.
    DealtCombatDamage,
    /// CR 510 / 119 — a **player** was dealt noncombat damage (a spell,
    /// ability, or Fight — not combat damage). Keyed on the damaged player,
    /// so `EventScope::OpponentControl` fires "whenever an opponent is dealt
    /// noncombat damage" (Chandra's Spitfire). The amount rides in via
    /// `Value::TriggerEventAmount`.
    PlayerDealtNoncombatDamage,
    /// A source the trigger's controller controls dealt noncombat damage to a
    /// creature equal to that creature's toughness (Taii Wakeen, Perfect
    /// Shot). Keyed on the damaged creature; the amount rides in via
    /// `Value::TriggerEventAmount`.
    YourSourceDealtNoncombatDamageEqualToToughness,
    /// A player was dealt damage (combat or not). Pair with
    /// `EventScope::YourSourceDamagedOpponent` for "whenever a source you
    /// control deals damage to an opponent" (Quest for Pure Flame).
    PlayerDamaged,
    /// An instant or sorcery spell its controller controls dealt damage (to
    /// any object or player). Pair with `EventScope::YourControl`. Fires once
    /// per spell resolution regardless of how many things it hit. Blaze
    /// Commando. The amount rides in via `Value::TriggerEventAmount`.
    YourInstantOrSorceryDealtDamage,
    /// An instant or sorcery spell its controller controls dealt damage to a
    /// **player**. Pair with `EventScope::YourControl`. Unlike
    /// `YourInstantOrSorceryDealtDamage` this fires once per damaged player,
    /// binding that player as the trigger subject so
    /// `SelectionRequirement::ControlledByTriggerPlayer` reads "that player
    /// controls" (Satyr Firedancer). The amount rides in via
    /// `Value::TriggerEventAmount`.
    YourInstantOrSorceryDealtDamageToPlayer,
    /// A player gained life.
    LifeGained,
    /// CR — a player paid life as a cost (Font of Agonies). The amount paid
    /// rides in via `Value::TriggerEventAmount`.
    PaidLife,
    /// CR 701.22/701.42 — a player scried or surveiled (a nonzero peek that
    /// actually happened). Fires once per scry/surveil resolution; the acting
    /// player rides in as the subject. Matoya, Archon Elder.
    ScriedOrSurveiled,
    /// CR 701.49 — a player completed a dungeon (the final room's ability
    /// resolved). "Whenever you complete a dungeon" (Dungeon Crawler).
    DungeonCompleted,
    /// CR 701.34 — a player proliferated. Fires once per proliferate
    /// instance (so a Tekuthal-doubled proliferate fires payoffs twice);
    /// the proliferating player is the event actor. "Whenever you
    /// proliferate" (Scheming Aspirant, Ezuri, Voidwing Hybrid).
    Proliferated,
    /// A player foraged (CR 701.61 — exiled three graveyard cards or sacrificed
    /// a Food; `GameEvent::Foraged`). The foraging player is the event actor.
    /// "Whenever you forage" (Corpseberry Cultivator).
    Foraged,
    /// A player collected evidence (CR 701.59 — as a cost or via
    /// `Effect::CollectEvidence`; `GameEvent::EvidenceCollected`). The
    /// collecting player is the event actor. "Whenever you collect evidence"
    /// (Surveillance Monitor, Evidence Examiner).
    EvidenceCollected,
    /// A player gave a promised gift (CR 702.165; `GameEvent::GiftGiven`). The
    /// gifting player is the event actor. "Whenever you give a gift" (Jolly
    /// Gerbils).
    GiftGiven,
    /// A player got one or more poison counters (`GameEvent::PoisonAdded`) —
    /// the player half of All Will Be One's "counters on a permanent or
    /// player". Event amount = counters added.
    PoisonAdded,
    /// CR 701.54 — the Ring tempted a player (and they chose a Ring-bearer).
    /// Matched to `GameEvent::RingTempted`; the chosen bearer rides in as the
    /// trigger subject. Powers "whenever you choose a creature as your
    /// Ring-bearer" payoffs (Call of the Ring).
    RingTempted,
    /// CR 700.14 — "Whenever you expend N." Matched to
    /// `GameEvent::Expended`; the threshold N rides the trigger's
    /// `EventSpec.filter` as `Predicate::ExpendReached(n)` so the trigger
    /// fires only when the turn's spell-mana total first reaches N.
    Expend,
    /// CR 716.2 — a Class enchantment gained a level (`GameEvent::
    /// ClassLevelReached`). Scoped to the leveling Class itself
    /// (`EventScope::This`) and filtered on `Predicate::SourceClassLevelIs(N)`
    /// so a "when this Class becomes level N" trigger fires only on that
    /// level. The Class is the trigger source.
    ClassLevelReached,
    /// A player lost life.
    LifeLost,
    /// The game entered a particular step.
    StepBegins(crate::turn_step::TurnStep),
    /// CR 904.9 — a scheme was set in motion off the top of the archenemy's
    /// scheme deck ("When you set this scheme in motion, …").
    SetInMotion,
    /// CR 901.9b — the planar die came up chaos ("Whenever chaos ensues, …").
    /// Fires on every face-up plane; the roller is the event's player.
    ChaosEnsues,
    /// CR 901.11b — the trigger's source plane/phenomenon is the one being
    /// left ("When you planeswalk away from this plane, …"). Fires on the
    /// face-up card as it goes back into the planar deck.
    PlaneswalkedAwayFrom,
    /// CR 312.5 — "When you encounter this phenomenon": the card was just
    /// turned face up off the planar deck.
    Encountered,
    /// The active player's turn just began.
    TurnBegins,
    /// A counter was added to a permanent/player.
    CounterAdded(CounterType),
    /// One or more counters of the given kind were removed from a permanent
    /// (a single event carries the total removed as its amount). Scope by
    /// `SelfSource` for "whenever counters are removed from this" — Chandra,
    /// Fire Artisan's loyalty-removal trigger.
    CounterRemoved(CounterType),
    /// A counter of *any* kind was added to a permanent (CR 122). Fires once
    /// per `CounterAdded` event regardless of counter type — "whenever one or
    /// more counters are put on a creature you control" (Stalwart Successor).
    AnyCounterAdded,
    /// An ability was activated.
    AbilityActivated,
    /// CR 702.177 — an exhaust ability was activated ("whenever you activate an
    /// exhaust ability" — Adrenaline Jockey).
    ExhaustAbilityActivated,
    /// CR 702.108 — an adapt ability was activated ("whenever you activate an
    /// adapt ability" — Gyre Engineer). Detected structurally from the
    /// activated ability's counter-check effect shape (see `Effect::is_adapt`).
    AdaptAbilityActivated,
    /// One or more cards left a player's graveyard (returned to hand /
    /// battlefield, exiled from graveyard, etc.). Used by Strixhaven
    /// "cards leave your graveyard" payoffs (Garrison Excavator, Living
    /// History, Spirit Mascot, Hardened Academic). The event fires once
    /// per card removed; the trigger handler is expected to be idempotent
    /// across batches (Strixhaven cards say "one or more cards" but the
    /// engine fires per-card and lets the trigger fire as many times).
    CardLeftGraveyard,
    /// One or more cards were put into exile (from any zone). Fires per card
    /// on the central exile-placement funnel. Pair with `once_per_turn` +
    /// `IsTurnOf(You)` for "whenever one or more cards are put into exile
    /// during your turn" (Stonebinder's Familiar).
    CardExiled,
    /// "Whenever one or more cards are put into exile from graveyards and/or
    /// the battlefield" (Ketramose, the New Dawn) — the origin-scoped sibling
    /// of `CardExiled`. Fires per card off `move_card_to`'s battlefield and
    /// graveyard branches.
    CardExiledFromPlayOrGraveyard,
    /// A permanent became the target of a spell or activated ability.
    /// Fires once per Permanent target at announce-time (when the spell
    /// hits the stack or the activated ability is pushed). Multi-target
    /// spells emit one event per target. For BecameTarget triggers the
    /// trigger source must be the targeted permanent (an implicit
    /// "target == source" check applied by `event_matches_spec`); the
    /// EventScope refines on the caster (`OpponentControl` → caster is
    /// an opponent, `YourControl` → caster is you). Used by SOS Tenured
    /// Concocter's "Whenever this creature becomes the target of a
    /// spell or ability an opponent controls, you may draw a card".
    BecameTarget,
    /// CR 601.2c — "whenever a player chooses one or more targets": one event
    /// per targeting decision, not per targeted object (Psychic Battle).
    ChoseTargets,
    /// CR 702.29c — A card was cycled (the controller paid a cycling
    /// cost to discard it from hand and draw). This event is emitted
    /// from `GameState::cycle_card` *in addition* to `CardDiscarded`,
    /// so cycle-specific triggers ("When you cycle this card",
    /// "Whenever a player cycles a card") can fire without also
    /// triggering on regular hand-discards. The triggered ability's
    /// source typically lives on the cycled card itself (CR 702.29c —
    /// "These abilities trigger from whatever zone the card winds up
    /// in after it's cycled" — the engine reads the card from its new
    /// home in the graveyard). `EventScope::SelfSource` fires when the
    /// source card was the one cycled; `EventScope::YourControl` fires
    /// when any of the controller's cards were cycled.
    CardCycled,
    /// CR 614.13ish "milled" — this card was put into a graveyard from a
    /// library (`GameEvent::CardMilled`). With `SelfSource` scope the
    /// trigger fires from the graveyard off the milled card itself
    /// (Narcomoeba, Creeping Chill).
    CardMilled,
    /// DSK — "Whenever you manifest dread, …" (`GameEvent::ManifestedDread`).
    /// The event subject is the card put into the graveyard "this way" (the
    /// non-manifested card), so a body can return it to hand (Paranormal
    /// Analyst). The manifesting player is the event actor.
    ManifestedDread,
    /// CR 702.108 — a permanent became untapped (Inspired). Fired once per
    /// permanent that flips tapped→untapped during the untap step. The
    /// triggering permanent is the event subject.
    BecomesUntapped,
    /// A permanent became tapped (Magda, Brazen Outlaw). The tapped
    /// permanent is the event subject; matched to `GameEvent::PermanentTapped`.
    Tapped,
    /// CR 702.122/702.171 — "Whenever this creature crews a Vehicle or saddles
    /// a Mount (during your main phase)." Fires from the crewing/saddling
    /// creature's side (`SelfSource` matches when the source is among the
    /// crew/riders); the crewed Vehicle / saddled Mount is the event subject,
    /// so `Selector::TriggerSource` binds to it. Matched to
    /// `GameEvent::VehicleCrewed` / `GameEvent::MountSaddled`. The "during your
    /// main phase" rider is baked into the match (both printed cards carry it).
    CrewsOrSaddles,
    /// CR 702.26 — a permanent phased in. The phasing-in permanent is the
    /// event subject; matched to `GameEvent::PermanentPhasedIn`.
    PhasesIn,
    /// CR 800.4 — "when you gain control of this permanent from another
    /// player" (Risky Move). `SelfSource`; the trigger's controller must be
    /// the *new* controller, so it fires once for whoever just took it.
    GainedControlOfThis,
    /// CR 701.40 — a permanent explored (Wildgrowth Walker, Tishana's
    /// Wayfinder payoffs). The exploring permanent is the event subject;
    /// matched to `GameEvent::Explored`.
    Explored,
    /// CR 701.57 — the controller performed a discover (Curator of Sun's
    /// Creation's "whenever you discover" payoff). The discovering player is
    /// the event subject; matched to `GameEvent::Discovered`. The discover
    /// value is exposed via `Value::TriggerEventAmount`.
    Discovered,
    /// CR 701.31 — a permanent became monstrous (Fleecemane Lion, Nessian
    /// Wilds Ravager "when this becomes monstrous" triggers). The permanent
    /// is the event subject; matched to `GameEvent::BecameMonstrous`.
    BecameMonstrous,
    /// CR 107.16 — the controller got one or more {E} (energy counters).
    /// Fires once per `AddEnergy` resolution ("Whenever you get one or more
    /// {E}"); the amount is exposed via `Value::TriggerEventAmount`. The
    /// event subject is the player; matched to `GameEvent::EnergyGained`.
    EnergyGained,
    /// CR 705.1 — the player won a coin flip ("Whenever you win a coin
    /// flip"). Fires once per won flip; the player is the event subject;
    /// matched to `GameEvent::CoinFlipWon`. Chance Encounter listens here.
    WonCoinFlip,
    /// CR 705.1 — the player lost a coin flip ("Whenever you lose a coin
    /// flip"). Fires once per lost flip; matched to `GameEvent::CoinFlipLost`.
    LostCoinFlip,
    /// CR 701.38 — "Whenever players finish voting" (Grudge Keeper). Fires
    /// once per ballot, after every vote is cast; matched to
    /// `GameEvent::VotingFinished`.
    VotingFinished,
    /// CR 706.6 — the player rolled one or more dice ("Whenever you roll one
    /// or more dice"). Fires once per roll; matched to `GameEvent::DiceRolled`.
    RolledDice,
    /// CR 701.9 batch — the player discarded one or more cards in a single
    /// effect resolution ("Whenever you discard one or more cards"). Fires once
    /// per resolution; the count is exposed via `Value::TriggerEventAmount`. The
    /// player is the event subject; matched to `GameEvent::DiscardedBatch`.
    DiscardedOneOrMore,
    /// CR 303.4 — an Aura became attached to a permanent. "Whenever an Aura you
    /// control becomes attached to a creature you control" (`EventScope::
    /// YourControl` requires the attached-to permanent to be a creature you
    /// control). The attached-to permanent is the event subject. Matched to
    /// `GameEvent::AuraAttached`. Siona, Captain of the Pyleas.
    AuraAttached,
    /// Same event, but the *Aura* is the trigger subject and the host is
    /// unrestricted — "whenever an Aura you control becomes attached to a
    /// nonland permanent an opponent controls …" (Eriette, the Beguiler), where
    /// the body has to reach both objects.
    AuraAttachedToAny,
    /// This Equipment became attached to a permanent (Blade of Shared Souls;
    /// fires off `GameEvent::AttachmentMoved` with a live host).
    BecameAttached,
    /// CR 712 — a permanent transformed. Fires once per transformed permanent;
    /// the transforming permanent is the event subject (`EventScope::SelfSource`
    /// for "when this transforms"). Matched to `GameEvent::Transformed`.
    Transformed,
    /// CR 702.159a — "Visit — [effect]": the controller rolled to visit their
    /// Attractions and the result matched one of this Attraction's lit-up
    /// numbers. Dispatched per visited Attraction with `EventScope::SelfSource`.
    VisitedAttraction,
    /// CR 702.140f — a creature mutated ("Whenever this creature mutates").
    /// Fires once per mutate onto the merged host (`EventScope::SelfSource`
    /// for the cards in the merged pile). Matched to `GameEvent::Mutated`.
    Mutated,
    /// CR 708.8 — a permanent was turned face up ("when this creature is
    /// turned face up", megamorph payoffs). The flipped permanent is the
    /// subject (`EventScope::SelfSource`). Matched to `GameEvent::TurnedFaceUp`.
    TurnedFaceUp,
    /// CR 111.10 — a token was created ("whenever you create a token" /
    /// "whenever you create a Blood token"). The new token is the event
    /// subject; pair `EventScope::YourControl` with a TriggerSource filter
    /// to narrow by token kind. Matched to `GameEvent::TokenCreated`.
    TokenCreated,
    /// CR 709.5h — a Room door was unlocked (at cast-entry or via the unlock
    /// special action). Fired with the Room permanent as the subject.
    DoorUnlocked,
    /// DSK Eerie ability word — "whenever you fully unlock a Room" (both doors
    /// unlocked). Fired with the Room permanent as the subject; pair with
    /// `EventScope::YourControl` so the unlocker's permanents trigger. Matched
    /// to `GameEvent::RoomFullyUnlocked`.
    RoomFullyUnlocked,
    /// MKM — "whenever you solve a Case" (Case File Auditor). Fired with the
    /// solved Case as the subject; pair with `EventScope::YourControl` so the
    /// solver's permanents trigger. Matched to `GameEvent::CaseSolved`.
    CaseSolved,
    /// A **land card** was put into a graveyard from anywhere (death,
    /// sacrifice, mill, discard, spell resolution). Matched to
    /// `GameEvent::CardPutIntoGraveyard { is_land: true, .. }`. Not a
    /// fan-out kind: "Whenever one or more land cards are put into your
    /// graveyard … draw a card" (The Gitrog Monster) fires once per batch
    /// of simultaneous land-to-graveyard events. Use `EventScope::YourControl`
    /// (the graveyard owner is the event player).
    LandPutIntoGraveyard,
    /// A card was put into a graveyard from anywhere. With
    /// `EventScope::SelfSource` the trigger fires off the card now sitting
    /// in the graveyard (Emrakul's "when this is put into a graveyard from
    /// anywhere, its owner shuffles their graveyard into their library").
    PutIntoGraveyard,
    /// "When this card is put into your hand from your graveyard" (Golgari
    /// Brownscale). With `EventScope::SelfSource` the trigger fires off the
    /// card now sitting in its owner's hand (dredge, or any graveyard→hand
    /// return). Matched to `GameEvent::CardPutIntoHandFromGraveyard`.
    PutIntoHandFromGraveyard,
    /// CR 502.2 / 731 — the game's day/night designation flipped ("Whenever
    /// day becomes night or night becomes day"). Fires once per transition;
    /// this is a global game event with no player subject, so pair it with
    /// `EventScope::AnyPlayer`. Matched to `GameEvent::DayNightChanged`.
    /// Brimstone Vandal listens here.
    DayNightChanged,
    /// CR 700.13 — `who` **committed a crime**: they cast a spell or activated
    /// an ability whose chosen targets include an opponent, anything an
    /// opponent controls or owns, or a spell/ability an opponent controls.
    /// Fires once per such spell/ability (not once per qualifying target). The
    /// committer is the event subject; pair with `EventScope::YourControl`
    /// ("whenever you commit a crime" — Kaervek, Gisa). Matched to
    /// `GameEvent::CommittedCrime`.
    CommittedCrime,
    /// CR 702.170 — "When this card becomes plotted, …". Fires from exile as a
    /// card is plotted; the just-plotted card is the event source (Aloe
    /// Alchemist, Longhorn Sharpshooter). Modeled as a `SelfSource` trigger
    /// dispatched directly by `plot_card`.
    BecomesPlotted,
    /// CR 701.19 — a player searched their own library. The searcher is the
    /// event subject; pair with `EventScope::OpponentControl` ("whenever an
    /// opponent searches their library" — Ob Nixilis, Unshackled). Fires once
    /// per search, whether or not a card was found. Matched to
    /// `GameEvent::PlayerSearchedLibrary`.
    PlayerSearchedLibrary,
    /// CR 103.2c — a spell or ability caused a player to shuffle their library.
    /// The shuffling player is the event subject (Psychogenic Probe).
    LibraryShuffled,
    /// A permanent left the battlefield for its owner's hand (Azorius
    /// Aethermage's "whenever a permanent is returned to your hand"). The
    /// trigger's subject is the bounced card; `EventScope::YourControl` reads
    /// the hand it landed in.
    PermanentReturnedToHand,
    /// CR 605 — a permanent was tapped to pay a mana ability's `{T}` cost.
    /// The tapped permanent is the event subject (Extraplanar Lens).
    TappedForMana,
    /// CR 701.5 — a spell on the stack was countered. The event's actor is the
    /// player who controlled the countering spell or ability (so
    /// `EventScope::YourControl` reads "a spell or ability you control counters
    /// a spell" — Lullmage Mentor); the subject is the countered spell's card.
    SpellCountered,
    /// CR 725.3 — a player became the monarch. The event's player is the
    /// new monarch, so `EventScope::OpponentControl` reads "an opponent
    /// becomes the monarch" (Knights of the Black Rose).
    BecameMonarch,
}

/// Whose events does this trigger listen for?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventScope {
    /// Only events whose subject is the source permanent itself
    /// ("When this ... enters", "When this attacks").
    SelfSource,
    /// Events caused/controlled by the source's controller ("whenever you
    /// cast a spell", "whenever you gain life").
    YourControl,
    /// Events caused/controlled by an opponent.
    OpponentControl,
    /// Any player.
    AnyPlayer,
    /// Another creature/permanent under your control (excludes `This`).
    AnotherOfYours,
    /// The active player (for step-based triggers).
    ActivePlayer,
    /// Trigger fires while the source card sits in **its owner's
    /// graveyard** (not the battlefield). Used by recursion creatures —
    /// Bloodghast's landfall, Ichorid's upkeep return, Silversmote
    /// Ghoul's lifegain return — where the ability fires off the
    /// graveyard copy and typically references it via `Selector::This`.
    /// The dispatcher walks graveyards in addition to the battlefield
    /// for triggers with this scope; the trigger's effective controller
    /// is the graveyard owner.
    FromYourGraveyard,
    /// A `PlayerDamaged` event whose source is controlled by the trigger's
    /// controller and whose damaged player is an opponent of them ("a source
    /// you control deals damage to an opponent" — Quest for Pure Flame).
    YourSourceDamagedOpponent,
    /// [`YourSourceDamagedOpponent`](Self::YourSourceDamagedOpponent) with the
    /// printed "other than this <permanent>" exclusion (Talon of Pain): the
    /// damage event's own source must not be the trigger's source.
    YourOtherSourceDamagedOpponent,
    /// The mirror of `YourSourceDamagedOpponent`: a `PlayerDamaged` event whose
    /// source is controlled by an opponent of the trigger's controller and
    /// whose damaged player *is* the trigger's controller ("whenever a source
    /// an opponent controls deals damage to you" — Michiko Konda). The damage
    /// dealer's controller is bound as the trigger player.
    OpponentSourceDamagedYou,
    /// A permanent **you control** (any, including the source) becomes the
    /// target of a spell or ability an **opponent** controls. Used with
    /// `EventKind::BecameTarget` — Battle Mammoth. Unlike SelfSource, the
    /// targeted permanent need not be the trigger source; the dispatcher
    /// checks the targeted permanent's controller == trigger controller and
    /// the caster is an opponent.
    YourPermanentTargetedByOpponent,
    /// A creature the source's controller controls (including the source)
    /// becomes the target of any spell or ability, friendly or hostile.
    /// Used with `EventKind::BecameTarget` — Nadu, Winged Wisdom.
    YourCreatureTargeted,
    /// A creature an **opponent** controls attacks the source's controller
    /// (or a planeswalker they control). Used with `EventKind::Attacks`; the
    /// dispatcher binds the attacking creature's controller into the
    /// trigger's target slot (so "that creature's controller gains control"
    /// resolves via `PlayerRef::Target(0)` — Coveted Jewel).
    ControllerAttackedByOpponent,
    /// A creature an **opponent** controls attacks a *planeswalker* the
    /// source's controller controls (only — attacks on the player don't
    /// fire it). Mila, Crafty Companion's "whenever an opponent attacks
    /// one or more planeswalkers you control". Same dispatcher as
    /// `ControllerAttackedByOpponent`, gated on the attack target.
    ControllerPlaneswalkerAttackedByOpponent,
    /// The creature the source Aura was attached to has died (left the
    /// battlefield). Matches a `CreatureDied` event whose subject is recorded
    /// in `GameState.auras_at_death` as having carried the source Aura.
    /// Powers "when enchanted creature dies" Aura triggers (Minion's Return).
    EnchantedBySource,
    /// A permanent was tapped by an effect the source's controller controls
    /// ("whenever you tap …" — Sharae, Solitary Sanctuary). Matches a
    /// `PermanentTapped` whose `actor` equals the trigger's controller; the
    /// tapped permanent is the subject, so a `.with_filter` restricts it (e.g.
    /// to an opponent's creature). Distinct from `Tapped`/`YourControl`, which
    /// key off the tapped permanent's controller rather than the tapper.
    YouTapped,
}

/// A structural filter over the unified `GameEvent` stream. The trigger fires
/// when an event of `kind` arrives, scoped per `scope`, and the optional
/// `filter` predicate holds in the post-event game state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSpec {
    pub kind: EventKind,
    pub scope: EventScope,
    /// Optional cast-time predicate (e.g. "whenever you cast a noncreature
    /// spell" is SpellCast + filter=NotCreatureSpell).
    pub filter: Option<Predicate>,
    /// CR 603.3d — "This ability triggers only once each turn." When set, the
    /// trigger fires at most once per turn (and once per batch of simultaneous
    /// events), tracked via `GameState::triggered_once_per_turn_used`. Defaults
    /// to false via `#[serde(default)]` for snapshot back-compat. Dramatic Finale.
    #[serde(default)]
    pub once_per_turn: bool,
    /// "This ability triggers only N times each turn" counted per event
    /// subject (Nadu's granted trigger is per creature). `None` = uncapped.
    #[serde(default)]
    pub per_subject_cap: Option<u8>,
    /// The event's actor (caster / controller of the triggering spell or
    /// ability) must be an opponent of the trigger's controller. Refines
    /// scopes that don't already gate on the actor — Opaline Sliver's
    /// "becomes the target of a spell an opponent controls" on SelfSource.
    #[serde(default)]
    pub actor_is_opponent: bool,
    /// "…becomes tapped, if it isn't being declared as an attacker" — excludes
    /// the CR 508.1f attacker tap from an `EventKind::Tapped` trigger (Verity
    /// Circle). Defaults to false. Only meaningful for `Tapped`.
    #[serde(default)]
    pub exclude_attacker_taps: bool,
    /// "…activates an ability without {T} in its activation cost" (Haunting
    /// Wind, Powerleech, Artifact Possession). Only meaningful for
    /// `AbilityActivated`; a tap-cost activation is filtered out so the
    /// companion `Tapped` trigger is the one that fires.
    #[serde(default)]
    pub exclude_tap_cost_abilities: bool,
    /// Restricts a damage event by the object that *dealt* it — "whenever a
    /// Goblin deals combat damage to a player" (Cabal Slaver). `Selector::
    /// TriggerSource` binds the damaged player on those events, so the dealer
    /// can't be reached through `filter`. Only meaningful for damage kinds.
    #[serde(default)]
    pub dealer_filter: Option<crate::card::SelectionRequirement>,
    /// Restricts a targeting event by the object that *declared* the target —
    /// "whenever this creature becomes the target of an Aura spell" (Fugitive
    /// Druid). `Selector::TriggerSource` binds the targeted permanent on those
    /// events, so the targeting spell/ability source can't be reached through
    /// `filter`. Only meaningful for `EventKind::BecameTarget`.
    #[serde(default)]
    pub causer_filter: Option<crate::card::SelectionRequirement>,
}

impl EventSpec {
    pub fn new(kind: EventKind, scope: EventScope) -> Self {
        Self {
            kind,
            scope,
            filter: None,
            once_per_turn: false,
            per_subject_cap: None,
            actor_is_opponent: false,
            exclude_attacker_taps: false,
            exclude_tap_cost_abilities: false,
            dealer_filter: None,
            causer_filter: None,
        }
    }
    /// "…becomes tapped, if it isn't being declared as an attacker" (Verity Circle).
    pub fn not_as_attacker(mut self) -> Self {
        self.exclude_attacker_taps = true;
        self
    }
    /// "…activates an ability without {T} in its activation cost" (Haunting Wind).
    pub fn without_tap_cost(mut self) -> Self {
        self.exclude_tap_cost_abilities = true;
        self
    }
    pub fn with_filter(mut self, p: Predicate) -> Self {
        self.filter = Some(p);
        self
    }
    /// "Whenever a [filter] deals damage …" — gate on the damage's dealer.
    pub fn dealt_by(mut self, f: crate::card::SelectionRequirement) -> Self {
        self.dealer_filter = Some(f);
        self
    }
    /// "Whenever [this] becomes the target of a [filter] spell or ability …"
    /// — gate on the object that declared the target.
    pub fn caused_by(mut self, f: crate::card::SelectionRequirement) -> Self {
        self.causer_filter = Some(f);
        self
    }
    /// Require the triggering event's actor to be an opponent.
    pub fn from_opponent(mut self) -> Self {
        self.actor_is_opponent = true;
        self
    }
    /// Mark this trigger "only once each turn" (CR 603.3d).
    pub fn once_per_turn(mut self) -> Self {
        self.once_per_turn = true;
        self
    }
    /// Cap how many times this trigger fires per distinct subject per turn
    /// ("the first time … each turn" — Stalwart Successor at cap 1; Nadu at 2).
    pub fn with_per_subject_cap(mut self, cap: u8) -> Self {
        self.per_subject_cap = Some(cap);
        self
    }
}

// ── Effect ───────────────────────────────────────────────────────────────────

/// The root instruction tree evaluated by the effect resolver.
///
/// All effects and abilities — spell effects, triggered-ability effects,
/// activated-ability effects — are `Effect` trees. Combinators let a single
/// card express modal choices, iteration, and conditionals without needing
/// engine changes per card.
//
// `large_enum_variant`: `CreateToken { definition: TokenDefinition, .. }`
// is the outlier (~368 bytes) — Boxing `TokenDefinition` is a structural
// change that touches every card factory and serde path. Tracked in
// TODO.md ("Box `TokenDefinition` in `Effect::CreateToken`") as a future
// cleanup; the stack footprint of `Effect` is fine in practice (most
// effects are deep behind `Box<Effect>` already via `Seq` / `ForEach`).
/// The toll a player pays before copying a Chain spell (CR 706 —
/// [`Effect::MayCopyThisSpell`]).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChainCopyCost {
    /// Free — Chain of Acid, Chain of Smog.
    #[default]
    Free,
    /// "may sacrifice a land of their choice" — Chain of Vapor, Chain of Silence.
    SacrificeLand,
    /// "may discard a card" — Chain of Plasma.
    DiscardCard,
    /// "may pay `cost`" — Chain Stasis.
    Mana(crate::mana::ManaCost),
}

/// What happens to tokens minted by [`Effect::CreateTokenAttacking`] when
/// the combat phase ends (CR 511.3 / the Mobilize end-of-combat sacrifice).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AttackingTokenCleanup {
    /// Tokens persist (a plain "tapped and attacking" mint).
    #[default]
    None,
    /// Sacrifice the tokens at end of combat (Mobilize).
    SacrificeAtEndOfCombat,
    /// Exile the tokens at end of combat (Myriad-style temporary copies).
    ExileAtEndOfCombat,
}

#[allow(clippy::large_enum_variant)]
/// What two milled cards must share for a `MillTwoRepeatSharing` loop to run
/// again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MillShareAxis {
    /// A colour, counting nonland cards only (Sphinx's Tutelage).
    NonlandColor,
    /// A colour, counting every milled card (Grindstone).
    AnyColor,
    /// A card type (The Tale of Tamiyo).
    CardType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    // ── Combinators ──────────────────────────────────────────────────────────
    /// Do nothing.
    Noop,
    /// Execute each inner effect in order.
    Seq(Vec<Effect>),
    /// If `cond` holds, execute `then`, else `else_`.
    If { cond: Predicate, then: Box<Effect>, else_: Box<Effect> },
    /// Execute `body` once per entity the `selector` resolves to.
    /// Inside `body`, `Selector::TriggerSource` refers to the current entity.
    ForEach { selector: Selector, body: Box<Effect> },
    /// Execute `body` `count` times.
    Repeat { count: Value, body: Box<Effect> },
    /// Run `body` once per player `who` resolves to, each time with that player
    /// as the effect's controller, in APNAP order (CR 101.4). The general form
    /// of the "each opponent does X of their choice" clauses (Rottenmouth
    /// Viper's pay-or-sacrifice-or-discard).
    EachPlayerDoes { who: PlayerRef, body: Box<Effect> },
    /// CR 705 — flip a coin `count` times. For each flip, asks the
    /// controller's decider for `Decision::CoinFlip` (heads = true,
    /// tails = false), then runs `on_heads` or `on_tails`. Used by
    /// Karplusan Minotaur, Mana Clash, Krark's Thumb, and Ral Zarek's
    /// -7 ultimate.
    FlipCoin {
        count: Value,
        on_heads: Box<Effect>,
        on_tails: Box<Effect>,
    },
    /// CR 705 — "that player flips a coin" (Planar Chaos): the flip is made by
    /// `flipper` rather than the effect's controller, so their Krark's-Thumb
    /// advantage and their win/lose triggers apply.
    FlipCoinBy { flipper: PlayerRef, on_heads: Box<Effect>, on_tails: Box<Effect> },
    /// CR 705 — "each player flips a coin" (Goblin Assassin). Every seat `who`
    /// resolves to flips its own coin in APNAP order; `on_heads` / `on_tails`
    /// then run with that seat as the controller, so a body written against
    /// `PlayerRef::You` reads "that player".
    EachPlayerFlipsCoin { who: PlayerRef, on_heads: Box<Effect>, on_tails: Box<Effect> },
    /// CR 705 — flip a coin repeatedly until the controller loses a flip or
    /// chooses to stop. Losing a flip cancels everything (zero wins). Then,
    /// in order, every `(threshold, effect)` whose threshold is at most the
    /// number of wins runs. Fiery Gambit (1+ → 3 dmg, 2+ → draw 3, 3+ →
    /// each opponent loses 5).
    FlipCoinsUntilLoseOrStop { tiers: Vec<(u32, Box<Effect>)> },
    /// CR 705.2 — Mana Clash: the controller and `opponent` each flip a coin;
    /// each player whose coin is tails takes 1 damage from `source`. Repeat
    /// until both come up heads on the same flip. `opponent` resolves to the
    /// target opponent (Target(0)).
    ManaClash { opponent: Selector },
    /// CR 706 — roll `count` N-sided dice. For each die, ask the
    /// controller's decider for `Decision::DieRoll { sides }` (which
    /// returns `DecisionAnswer::DieRoll(n)` with `1 <= n <= sides`),
    /// then walk `results` and run the **first** matching arm whose
    /// `[low, high]` range covers `n`. If no arm matches, no effect
    /// runs for that die. Mirrors `FlipCoin`'s shape for the die
    /// equivalent. Used by Goblin Goliath, Wand of the Elements, and
    /// future Krark / Aether Sphere Harvester-style "roll N dice with
    /// a results table" cards. CR 706.1 + 706.3 covered; CR 706.2 result
    /// modifiers via `modifier`; per-card 706.2b reroll via `reroll_at_most`.
    /// (An *engine-wide* reroll granted by a separate source — a global "you may
    /// reroll" replacement — is still ⏳.)
    RollDie {
        /// Number of sides on each die (e.g. 6 for d6, 20 for d20).
        /// Must be at least 2.
        sides: u8,
        /// Number of dice to roll. Each die rolls independently and
        /// runs its own results-table dispatch.
        count: Value,
        /// CR 706.2 — a flat modifier added to each natural die result
        /// before the results table is consulted (e.g. "roll a d20 and
        /// add 2"). The modified result is floored at 1 (a die's result
        /// is never reduced below 1) but may exceed `sides`, which lets
        /// an "N+" top arm catch boosted rolls. Defaults to 0 (no
        /// modifier) for snapshot back-compat. The natural roll is still
        /// what the decider returns; the modifier is applied on top.
        #[serde(default = "crate::effect::zero_value")]
        modifier: Value,
        /// CR 706.2b — reroll threshold. When greater than 0, any natural
        /// result `<= reroll_at_most` is rerolled exactly once and the new
        /// natural face is kept (even if it's also low — a single reroll
        /// per die, per the "reroll … once" pattern). The modifier is
        /// applied after the (re)roll. Models cards like "if you roll a 1,
        /// reroll" / "you may reroll any die that rolled a 1-N". Defaults
        /// to 0 (never reroll) for snapshot back-compat.
        #[serde(default)]
        reroll_at_most: u8,
        /// CR 706.3a — the results table. Each arm is `(low, high,
        /// effect)`; the first arm with `low <= rolled <= high` fires
        /// for that die. Use `(low, sides, effect)` for an "N+" arm,
        /// or `(n, n, effect)` for a single-number arm.
        results: Vec<(u8, u8, Effect)>,
        /// CR 706.5 — "if any of the dice rolled the same number, [effect]".
        /// When `count >= 2` and two or more dice share a *natural* face
        /// (doubles), this fires once after the per-die results dispatch.
        /// Defaults to `None` (no doubles check) for snapshot back-compat.
        #[serde(default)]
        on_doubles: Option<Box<Effect>>,
    },
    /// Modal — controller picks one of `modes` at cast time; the chosen index
    /// is stored in the stack item's `mode` field.
    ChooseMode(Vec<Effect>),
    /// "Choose `picks.len()` —" multi-mode pick. At resolution, runs each
    /// mode whose index appears in `picks` (in that order). Used by the
    /// Strixhaven Command cycle (Witherbloom / Lorehold / Quandrix /
    /// Silverquill / Prismari Commands), Charms, and any other "choose
    /// two of four" spell.
    ///
    /// CR 700.2d covers this: "If a player is allowed to choose more
    /// than one mode for a modal spell or ability, that player normally
    /// can't choose the same mode more than once." The `picks` field
    /// stores the controller's chosen indices; the auto-decider feeds
    /// them in deterministically (a sensible default for each card),
    /// and a later mode-pick UI can override the picks per-cast.
    ///
    /// Each target-bearing mode owns its own target slot, assigned by the
    /// mode's position among the target-bearing modes in `picks`: the first
    /// such mode reads `ctx.targets[0]`, the second `ctx.targets[1]`, and so
    /// on. This lets "choose one or both" spells whose modes target
    /// different things (Steal the Show — target player for one mode, target
    /// creature for the other) supply and resolve a target per chosen mode.
    /// Non-targeting modes run with the full context. Cast-time validation
    /// keys off the same default-`picks` ordering
    /// (`target_filter_for_slot_in_mode`), so targets line up at resolution
    /// even when a decider runs only a subset.
    ///
    /// Limitation: because `picks` is the card's default, cast-time target
    /// validation assumes the default mode set; a decider that picks a
    /// different subset still supplies targets in the default-picks slot
    /// order. Full cast-time mode selection is tracked in TODO.md.
    ChooseN { picks: Vec<u8>, modes: Vec<Effect> },
    /// CR 700.2 — "Choose up to N, where N is [a live count]." Evaluates `max`
    /// at resolution, caps it at `modes.len()`, and lets the controller pick up
    /// to that many *distinct* modes (AutoDecider takes the first `count`). The
    /// modes must be self-targeting (no chosen target slots) — Bumi, King of
    /// Three Trials (X = Lessons in your graveyard; modes pump self / scry /
    /// earthbend).
    ChooseUpToN { max: Box<Value>, modes: Vec<Effect> },
    /// CR 702.119 — Escalate. "Choose one or more. You pay the escalate cost
    /// for each mode chosen beyond the first." The cast-time `mode` is the
    /// base (always-chosen) mode; a `Decision::ChooseModes` answer escalates
    /// to additional distinct modes, running `cost` (Collective Brutality's
    /// "discard a card", capped by hand size) once per extra mode. Each
    /// chosen target-bearing mode owns a target slot in run order. AutoDecider
    /// keeps just the base mode → no escalate cost, so a plain modal cast is
    /// unaffected. Modeled at resolution (escalate cards are sorceries with
    /// no cost/effect response window).
    Escalate {
        modes: Vec<Effect>,
        cost: Box<Effect>,
    },
    /// CR 702.172 — Spree. "Choose one or more additional costs." Each mode
    /// carries its own mana cost; the chosen modes are picked at cast time
    /// (`GameAction::CastSpellSpree`), their costs folded into the total, and
    /// stamped onto the resolving `CardInstance.spree_modes`. At resolution the
    /// chosen modes run in printed order, each target-bearing mode consuming
    /// the next target slot (slot 0 = `target`, then `additional_targets`).
    /// Targets are validated at resolution, not cast time.
    Spree { modes: Vec<SpreeMode> },
    /// FIN "Tiered" (Choose one additional cost.) — a modal spell where the
    /// caster picks **exactly one** mode, pays that mode's additional mana cost
    /// at cast time, and runs its effect at resolution. Shares Spree's cast
    /// plumbing (`GameAction::CastSpellSpree` stamps the single chosen index
    /// onto `CardInstance.spree_modes`) but the cast validator enforces a
    /// one-mode selection. Fire/Ice/Thunder/Restoration Magic.
    Tiered { modes: Vec<SpreeMode> },
    /// Cast-time multi-mode selection without per-mode costs: "Choose one
    /// or both —" (Choreographed Sparks: min 1, max 2, no repeats) and
    /// "Choose up to four. You may choose the same mode more than once."
    /// (Moment of Reckoning: min 0, max 4, repeats). Shares Spree's cast
    /// plumbing — `GameAction::CastSpellSpree` validates the picks against
    /// min/max/repeats and stamps them onto `CardInstance.spree_modes`; at
    /// resolution the chosen instances run in printed order, each
    /// target-bearing instance consuming the next target slot (slot 0 =
    /// `target`, then `additional_targets`). A plain `CastSpell { mode }`
    /// falls back to running that single mode (bot / back-compat path).
    ChooseModesCast { modes: Vec<Effect>, min: u8, max: u8, allow_repeats: bool },
    /// BLB "Choose up to `budget` {P} worth of modes. You may choose the same
    /// mode more than once." — the Season cycle. Each mode has a point price
    /// in `points`; the picks must total at most `budget`. Shares
    /// `ChooseModesCast`'s cast plumbing (`GameAction::CastSpellSpree` stamps
    /// the validated picks onto `CardInstance.spree_modes`) and its
    /// resolution: chosen instances run in printed order, each target-bearing
    /// instance consuming the next target slot.
    ChooseModesByPoints { modes: Vec<Effect>, points: Vec<u8>, budget: u8 },
    /// "Choose one that hasn't been chosen —" (Captive Audience). Picks a mode
    /// at RESOLUTION from the ones the source hasn't used yet, records it on
    /// `CardInstance.modes_chosen`, and runs it. Does nothing once every mode
    /// has been chosen.
    ChooseUnchosenMode { modes: Vec<Effect> },
    /// "You may [body]" — emit a yes/no decision via
    /// `Decision::OptionalTrigger`. Run `body` only on `Bool(true)`. The
    /// `description` string is shown to the player (and serialized into
    /// the decision wire payload). The decision is asked of the
    /// effect's *controller* (`ctx.controller`).
    ///
    /// Used by SOS / STX cards that bake a "you may" into the middle of
    /// a sequence: Stadium Tidalmage's loot trigger, Pursue the Past's
    /// optional discard, Witherbloom Charm mode 0's optional sacrifice,
    /// Tenured Concocter's may-draw on becomes-target, and any future
    /// "you may pay X to do Y" rider where the cost itself is `Effect::
    /// Noop` (free) or already paid up-front. For paid optional costs
    /// (Bayou Groff's "may pay {1} to return on death") see the related
    /// `Effect::MayPay` primitive in TODO.md — `MayDo` is the no-cost
    /// variant.
    ///
    /// The `AutoDecider` answers `false` (skip) by default; tests can
    /// override via `ScriptedDecider::new([DecisionAnswer::Bool(true)])`.
    /// This matches MTG rules: any "you may" defaults to "no" unless the
    /// controller actively chooses to do it.
    ///
    /// `description` is a `String` (rather than `&'static str`) because
    /// `Effect` derives `Deserialize` and serde requires owned data when
    /// the parent enum is bound to a non-static lifetime via the rest of
    /// `GameState`. Card factories pass `"…".into()` which is a no-cost
    /// `&str → String` move at construction time.
    MayDo { description: String, body: Box<Effect> },
    /// "[Player] may [body]" — [`Effect::MayDo`] routed to another seat. The
    /// yes/no goes to `who` and `body` runs with `who` as its controller, so
    /// its "you" and its target choice belong to that player (Ley Line's
    /// "that player may put a +1/+1 counter on target creature of their
    /// choice").
    MayDoBy { who: PlayerRef, description: String, body: Box<Effect> },

    /// "[Player] may pay [cost]. If they do, `body`; otherwise `else_`" —
    /// [`Effect::MayPay`] routed to another seat (Phyrexian Tyranny's
    /// "that player loses 2 life unless they pay {2}"). The yes/no and the
    /// payment both belong to `who`, and `body`/`else_` run with `who` as
    /// their controller.
    MayPayBy {
        who: PlayerRef,
        description: String,
        mana_cost: crate::mana::ManaCost,
        body: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "You may [body]. If you don't, [else_]." The two-sided sibling of
    /// [`Effect::MayDo`] (Dakra Mystic). Shares MayPay's seat-routed yes/no
    /// suspend with an empty cost.
    MayDoElse { description: String, body: Box<Effect>, else_: Box<Effect> },
    /// "Target opponent chooses a number, then [`then`]" — the pick is routed
    /// to `who`'s seat (`ask_seat_amount`) and `then` reads it back through
    /// [`Value::ChosenNumber`]. Choice of Damnations.
    PlayerChoosesNumber {
        who: Selector,
        prompt: String,
        max: Value,
        then: Box<Effect>,
    },
    /// "Each player may bid life…the high bidder loses life equal to the high
    /// bid and [`then`]" — the controller opens, then every other player in
    /// turn order may top the standing bid; the round repeats until a full
    /// pass leaves the high bid standing. `then` runs for the high bidder.
    /// Pain's Reward (CR 104.3 — a bid above your life total is legal).
    LifeBidding { then: Box<Effect> },
    /// "You may pay {X}. When you do, [body with X]." — the controller
    /// picks X at resolution via `Decision::ChooseAmount` (0 = decline,
    /// the AutoDecider default), capped by their FLOATED mana (the MayPay
    /// convention: mana abilities aren't activatable mid-resolution), pays
    /// {X} generic from the pool, and `body` runs with `ctx.x_value = X`
    /// so `Value::XFromCost` reads the chosen amount. Tester of the
    /// Tangential's "pay {X}: move X +1/+1 counters".
    MayPayX { description: String, body: Box<Effect> },

    /// Optional **paid** branch: the controller is asked yes/no, and if
    /// they accept *and* can afford `mana_cost`, the engine deducts the
    /// mana from their pool and runs `body`. If the controller declines
    /// or can't afford the cost, the body is skipped.
    ///
    /// Sibling to `Effect::MayDo` (the no-cost variant). Powers cards
    /// like Bayou Groff ("when this dies, you may pay {1}; if you do,
    /// return it to its owner's hand"), Killian's Confidence's "may pay
    /// {W/B} to return from gy", and any other "may pay X to do Y"
    /// rider where the cost is pure mana.
    ///
    /// Cost evaluation walks the controller's *pool* (already-floated
    /// mana) — the engine doesn't tap lands automatically inside an
    /// `Effect::MayPay`, matching MTG's "you can't activate mana
    /// abilities mid-resolution unless the rules let you." Tests that
    /// want to exercise the paid path should pre-float the mana
    /// (`game.players[c].mana_pool.add_colored(...)`) and feed
    /// `DecisionAnswer::Bool(true)` to the scripted decider.
    ///
    /// X-cost variants (where the optional cost has its own X prompt)
    /// are out of scope here — those should land as a sibling
    /// `MayPayX { mana_cost, x_value, body }` if/when needed.
    MayPay {
        description: String,
        mana_cost: crate::mana::ManaCost,
        body: Box<Effect>,
        /// Runs when the cost was declined or unpayable ("if you don't, …").
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// CR 119.4 — "You may pay N life. If you do, `body`." The controller is
    /// asked yes/no; the payment (which is a life loss, CR 119.4) is only
    /// possible while their life total is at least `amount`. Declined or
    /// unpayable runs `else_`. Seymour Flux, and the broad "pay N life: draw /
    /// scry / pump" cluster. Sibling of [`Effect::MayPay`] for life costs.
    MayPayLife {
        description: String,
        amount: Value,
        body: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "You may pay {X}, where X ≤ `max`; if you do, [body reads X as
    /// `Value::TriggerEventAmount`]." The controller is prompted for a number
    /// in `0..=min(max, mana in pool)` via `Decision::ChooseAmount`; that many
    /// generic mana are spent from their floated pool and `body` runs with
    /// `event_amount` set to the paid amount. The X-cost sibling of
    /// `Effect::MayPay` — Well of Lost Dreams ("pay {X} ≤ life gained, draw X").
    /// `AutoDecider` pays 0 (nothing unprompted).
    MayPayGenericUpTo { max: Value, body: Box<Effect> },
    /// "You may pay `mana_cost` any number of times; `body` runs once per
    /// payment" (Magnetic Mountain's per-creature untap toll). Loops while the
    /// controller accepts and their floating pool can cover it.
    MayPayRepeatedly {
        who: PlayerRef,
        description: String,
        mana_cost: crate::mana::ManaCost,
        body: Box<Effect>,
    },

    /// Reflexive "when you do" payoff (CR 603.7). Wrap a *targeted* body that
    /// should choose its targets **after** the gating cost is paid — e.g. the
    /// inner half of `MayPay { body: Reflexive(<targeted bite>) }` (Itzquinth)
    /// or `MaySacrifice { then: Reflexive(<targeted support>) }` (Glorifier of
    /// Suffering). The wrapper is opaque to the cast/trigger-time target walk
    /// (it isn't listed in `target_filter_for_slot` / `requires_target`), so the
    /// outer trigger doesn't try to pre-validate the nested targets; at
    /// resolution the body is auto-targeted fresh and run.
    Reflexive { body: Box<Effect> },

    /// "You may sacrifice [count] [filter]. If you do, [then]." — the
    /// reflexive sacrifice cost (Bloodcrazed Socialite's attack +2/+2,
    /// Gut, True Soul Zealot's attack-sac → Skeleton). Asks the controller
    /// yes/no (gated on owning a legal candidate); on yes, the weakest
    /// non-source matching permanent(s) are sacrificed (the cost is almost
    /// always a token, so the auto-pick is faithful) and `then` runs.
    MaySacrifice {
        description: String,
        filter: SelectionRequirement,
        count: Value,
        then: Box<Effect>,
        /// Runs if the cost was declined or no legal sacrifice existed.
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "You may sacrifice [this / the source]. If you do, [then]." (CR 701.16 +
    /// 603.7 — the reflexive self-sacrifice cost; Eden, Seat of the Sanctum's
    /// mill-then-may-sac-then-return). Asks the controller yes/no (gated on the
    /// source still being on the battlefield); on yes, the source is sacrificed
    /// and `then` runs, else `else_`.
    MaySacrificeSource {
        description: String,
        then: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "You may tap [count] untapped [filter] you control. If you do, [then]."
    /// The reflexive tap cost (Caparocti Sunborn's attack → discover 3). Asks
    /// the controller yes/no (gated on owning `count` untapped matches); on
    /// yes, the lowest-impact matches are tapped and `then` runs.
    MayTap {
        description: String,
        filter: SelectionRequirement,
        count: Value,
        then: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "You may discard [count] card(s). If you do, [then]." The reflexive
    /// discard cost (Toph, Hardheaded Teacher — "you may discard a card; if you
    /// do, return target instant or sorcery card from your graveyard to your
    /// hand"). Asks the controller yes/no (gated on holding `count` cards); on
    /// yes the highest-mana-value cards are discarded (least castable) and
    /// `then` runs, else `else_`.
    MayDiscard {
        description: String,
        count: Value,
        then: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// Like [`Effect::MayDiscard`] but the discarded card(s) must match
    /// `filter`. The gate offers the choice only when the controller holds
    /// `count` matching cards; the auto-picker discards the highest-MV
    /// matches. "Sacrifice this unless you discard a noncreature card"
    /// (Drekavac) rides this with `then: Noop, else_: SacrificeSource`.
    MayDiscardMatching {
        description: String,
        count: Value,
        filter: SelectionRequirement,
        then: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// Reveal-from-hand gate: "you may reveal a [filter] card from your
    /// hand. If you do, run `then`; otherwise run `else_`." Used by the
    /// STX Snarl dual-land cycle (Frostboil, Furycalm, Necroblossom,
    /// Shineshadow, Vineglimmer) — the printed Oracle reads "As ~~~
    /// enters, you may reveal a [C1] or [C2] card from your hand. If you
    /// don't, ~~~ enters tapped."
    ///
    /// Asked of the effect's *controller* (`ctx.controller`). Filter is
    /// evaluated against each hand card via `evaluate_requirement_on_card`.
    /// AutoDecider auto-reveals whenever a matching card exists — the
    /// bot always wants to keep the land untapped if it can. A future
    /// UI wire could surface a `Decision::Reveal` shape so a human
    /// player can decline to reveal (a strategic bluff); not modeled
    /// here since no test exercises the decline-with-match path.
    ///
    /// If no card matches the filter, `else_` runs unconditionally
    /// (matches printed "if you don't reveal, …" — including the case
    /// where you can't).
    IfRevealFromHand {
        filter: SelectionRequirement,
        then: Box<Effect>,
        else_: Box<Effect>,
    },

    // ── Damage / life ────────────────────────────────────────────────────────
    DealDamage { to: Selector, amount: Value },
    /// CR 702 Radiance (Ravnica) — deal `amount` damage to the creature `subject`
    /// resolves to *and* each other creature that shares a color with it
    /// (Cleansing Beam, Wojek Embermage). `subject` carries the target slot
    /// (`TargetFiltered { slot: 0, filter: Creature }`), so cast/auto-target
    /// legality flows through the normal slot-0 path; the fan-out then reads the
    /// chosen creature's computed colors. A colorless subject hits only itself.
    RadianceDamage { subject: Selector, amount: Value },
    /// Deal `amount` damage to the creature `subject` resolves to *and* each
    /// other creature on the battlefield with the same name (Izzet Staticaster).
    /// `subject` carries the target slot; the fan-out reads the chosen
    /// creature's printed name. Mirrors `RadianceDamage`'s shape.
    SameNameDamage { subject: Selector, amount: Value },
    /// Each creature the controller controls deals `amount` damage to the `to`
    /// target — each damage is sourced from that creature (so its deathtouch /
    /// lifelink apply). "Each creature you control deals 1 damage to that
    /// creature" — Case of the Gateway Express.
    EachControlledCreatureDealsDamage { to: Selector, amount: Value },
    /// Damage to each player, computed *per player* from what they control:
    /// `amount` × the number of permanents matching `filter` they control
    /// (Acidic Soil — "damage to each player equal to the number of lands they
    /// control"), or a flat `amount` to each player controlling at least one
    /// match when `flat` (Disorder's player half). Players with no match take
    /// nothing.
    DealDamageToEachPlayerPerPermanent {
        filter: SelectionRequirement,
        amount: Value,
        #[serde(default)]
        flat: bool,
    },
    /// Deal `amount` damage to a target creature; any damage beyond what's
    /// lethal (its remaining toughness) is dealt to that creature's
    /// controller (CR 120.10, trample-like). Flame Spill.
    DealDamageExcessToController { to: Selector, amount: Value },
    /// "Deal N damage divided as you choose among one or more / any number
    /// of targets." Targets are chosen at cast time across slots
    /// `0..max_targets` (each filtered by `filter`); the per-target split
    /// is decided at resolution via `Decision::DivideDamage` (AutoDecider
    /// spreads as evenly as possible). Used by Forked Bolt, Pyrokinesis,
    /// Fiery Cannonade-adjacent "divide" spells, Crackle with Power.
    DealDamageDivided {
        total: Value,
        filter: SelectionRequirement,
        max_targets: u8,
        /// "Each of those creatures deals damage equal to its power to
        /// [this]" (Polukranos, World Eater) — every damaged target swings
        /// back at the source before state-based actions are checked.
        #[serde(default)]
        retaliate_to_source: bool,
    },
    /// "Deals N damage divided evenly, rounded down, among any number of
    /// targets" (Fireball). Same slot shape as `DealDamageDivided`, but each
    /// chosen target takes `total / n` — no `DivideDamage` decision; the
    /// remainder is lost per the printed rounding.
    DealDamageDividedEvenly {
        total: Value,
        filter: SelectionRequirement,
        max_targets: u8,
    },
    /// Two creatures fight: each deals damage equal to its current
    /// power to the other simultaneously. Both creatures take damage
    /// and die simultaneously to SBA. `attacker` is typically
    /// `Selector::Target(0)` or `Selector::This` (a friendly fighter);
    /// `defender` is typically `Selector::Target(1)` or an
    /// auto-selected opp creature. If either selector resolves to no
    /// permanent the effect no-ops cleanly (matches MTG's "if either
    /// is no longer a creature, no damage is dealt"). Used by SOS
    /// Chelonian Tackle, STX Decisive Denial mode 1, and similar
    /// fight-style green/quandrix removal.
    Fight { attacker: Selector, defender: Selector },
    /// One-sided fight — `source` deals damage equal to its power to `target`
    /// (no back-swing). Damage carries `source` so lifelink / deathtouch /
    /// wither apply (CR 701.12-style but unidirectional). No-ops if either
    /// selector resolves to no permanent. Stew the Coneys, Tail Swipe,
    /// Pounce, Friendly Rivalry's per-fighter swing.
    DealDamageEqualToPower { source: Selector, target: Selector },
    /// The `source` permanent deals damage equal to its power to each entity
    /// `targets` resolves to (the source is always excluded), and — when
    /// `each_opponent` — to each opponent of the source's controller. Power is
    /// read once up front; damage carries `source` so lifelink/deathtouch/
    /// wither apply. Chandra's Ignition (each opponent), Nibelheim Aflame
    /// (creatures only). No-op if `source` resolves to no permanent.
    DealDamageEqualToPowerToEach {
        source: Selector,
        targets: Selector,
        each_opponent: bool,
    },
    /// Each permanent `dealers` resolves to deals damage equal to its power
    /// to the single entity `target` resolves to (Nissa's Judgment). The
    /// many-sources mirror of `DealDamageEqualToPowerToEach`; each hit
    /// carries its own dealer so lifelink/deathtouch apply per source.
    EachDealsDamageEqualToPower { dealers: Selector, target: Selector },
    /// CR 701.12 — Exchange control of the two permanents the selectors
    /// resolve to (one each). A permanent control swap (Vedalken Plotter,
    /// Aura Thief, Switcheroo). If either selector resolves to no permanent
    /// the effect no-ops. Both permanents change controller simultaneously.
    ExchangeControl { a: Selector, b: Selector },
    /// Perplexing Chimera — exchange control of `what` and the spell whose
    /// cast fired the current trigger: the spell's controller takes `what`,
    /// and the trigger's controller takes over the spell. (The printed "you
    /// may choose new targets" rider is not modeled.)
    ExchangeControlWithTriggeringSpell { what: Selector },
    /// Twist Allegiance — you and each player `who` resolves to swap control
    /// of every creature you each control for `duration`; the swapped
    /// creatures untap and gain haste.
    ExchangeCreatureControlWith { who: Selector, duration: Duration },
    /// Overblaze — each resolved permanent's damage is doubled for the rest of
    /// the turn (CR 614.2), via `doubled_damage_sources_this_turn`.
    DoubleDamageFromSourceThisTurn { what: Selector },
    /// Whims of the Fates — starting with the controller, each player splits
    /// the permanents they control into `piles` piles and sacrifices one
    /// chosen at random. The split is round-robin over a shuffled list (no
    /// player choice).
    EachPlayerSplitsAndSacrificesRandomPile { piles: u8 },
    /// "Separate [cards] into two piles. [chooser] chooses one." `splitter`
    /// picks the first pile, `chooser` picks which pile is "chosen"; the two
    /// bodies then run against `Selector::SeparatedPile`. Invasion's
    /// Do or Die / Death or Glory / Bend or Break / Fight or Flight /
    /// Stand or Fall.
    SeparateIntoPiles {
        what: Selector,
        splitter: PlayerRef,
        chooser: PlayerRef,
        /// Runs against the pile `chooser` picked.
        chosen: Box<Effect>,
        /// Runs against the pile left over.
        other: Box<Effect>,
    },
    /// `chooser` picks exactly one of `what`; `chosen` runs against it and
    /// `other` against the rest, both via `Selector::SeparatedPile`.
    /// Barrin's Spite ("their controller chooses and sacrifices one of them").
    ChooseOneAmong {
        what: Selector,
        chooser: PlayerRef,
        chosen: Box<Effect>,
        other: Box<Effect>,
    },
    /// CR 701.12 — exchange control of a permanent you control matching
    /// `filter` (chosen at resolution: `Decision::ChooseCards` for a
    /// `wants_ui` controller, lowest CardId otherwise) and the permanent
    /// `with` resolves to. Vedalken Plotter's ETB land swap.
    ExchangeControlChoosing { filter: SelectionRequirement, with: Selector },
    GainLife  { who: Selector, amount: Value },
    LoseLife  { who: Selector, amount: Value },
    /// CR 701.10d — "double a player's life total." Each player the selector
    /// resolves to gains life equal to their current total (so 20 → 40). A
    /// player at 0 or negative is unaffected (no negative-doubling). Beacon
    /// of Immortality.
    DoubleLife { who: Selector },
    /// Each player the selector resolves to loses life equal to half their
    /// *own* current life total (rounded up when `rounded_up`, else down).
    /// Per-player evaluation — `LoseLife`'s single global amount can't scale
    /// to each target's own total. Stingerback Terror ("each opponent loses
    /// half their life, rounded up").
    LoseHalfLife { who: Selector, rounded_up: bool },
    /// "Each player loses `per` life for each [`filter`] they control"
    /// (Stronghold Discipline). Per-player evaluation, so each seat's own
    /// board sets its own loss.
    LoseLifePerControlled { who: Selector, filter: SelectionRequirement, per: Value },
    /// Deal each player the selector resolves to damage equal to half their
    /// *own* current life total (rounded up when `rounded_up`, else down).
    /// Per-player evaluation, routed through the damage funnel (so doubling /
    /// prevention / redirection apply, unlike `LoseHalfLife`). Heartless
    /// Hidetsugu ("deals damage to each player equal to half that player's
    /// life total, rounded down").
    DealHalfLifeDamage { to: Selector, rounded_up: bool },
    /// Set a player's life total to a specific value (CR 119.5).
    /// "If an effect sets a player's life total to a specific number, the
    /// player gains or loses the necessary amount of life to end up with
    /// the new total." Used by Biorhythm-style "set life to creature
    /// count", Tree of Redemption-style "exchange life with toughness",
    /// and any future effect that pins life to a specific number.
    ///
    /// Implementation note: the resolver computes `delta = new_total -
    /// current_life` and emits either a `LifeGained` event (delta > 0)
    /// or `LifeLost` event (delta < 0). Delta of 0 emits no event
    /// (matches CR 119.9 / 119.10 zero-life-change semantics).
    SetLifeTotal { who: Selector, amount: Value },
    /// CR 701.12c — "exchange life totals" between the players the two
    /// selectors resolve to (Soul Conduit, Magus of the Mirror, Mirror
    /// Universe-style swaps). Each side's previous total is captured before
    /// either changes, then each player gains/loses to reach the other's
    /// previous total; `LifeGained`/`LifeLost` events fire so lifegain-
    /// matters payoffs see the swing. A no-op when both selectors land on
    /// the same player.
    ExchangeLifeTotals { a: Selector, b: Selector },
    /// One-turn "[selected players] can't gain life this turn" lock.
    /// Sets `Player.cannot_gain_life_this_turn = true` for each player
    /// the selector resolves to. Cleared by `do_untap` at the start
    /// of any new turn. Distinct from `StaticEffect::PlayerCannotGainLife`
    /// — this is a one-shot effect with no source permanent to anchor
    /// to (Skullcrack, Sulfurous Blast's flashback rider, future
    /// one-turn lifegain locks).
    LifeGainLockThisTurn { who: Selector },
    /// One-turn "[selected players]' life total can't change this turn" lock —
    /// both gain and loss are dropped. Sets `Player.life_locked_this_turn`,
    /// cleared by `do_untap` at the turn boundary. Flare of Fortitude.
    LifeLockThisTurn { who: Selector },
    /// CR 104.3d — Angel's Grace: until end of turn the controller can't
    /// lose the game and their opponents can't win it. With `damage_floor`,
    /// damage that would drop their life below 1 drops it to 1 instead.
    /// Sets `Player.{cant_lose_this_turn, damage_floor_this_turn}`; cleared
    /// by `do_untap` at the turn boundary.
    CantLoseThisTurn { damage_floor: bool },
    /// Forbidding Spirit — "until your next turn, creatures can't attack you or
    /// planeswalkers you control unless their controller pays {amount} for
    /// each." Sets `Player.attack_tax_until_your_turn` on the controller;
    /// cleared at that player's next untap step.
    TaxAttackersUntilYourNextTurn { amount: Value },
    /// Channel — until end of turn the controller may pay 1 life per point
    /// of colorless shortfall when paying costs ("you may pay 1 life: add
    /// {C}"). Sets `Player.channel_life_for_mana`; the payment funnel
    /// converts on demand.
    ChannelLifeForMana,
    /// Permanently set `Player.cannot_gain_life` on each player the selector
    /// resolves to — "that player can't gain life for the rest of the game"
    /// (Screaming Nemesis, Everlasting Torment, Witch of the Moors). Sticks
    /// across recomputes and turns (nothing resets the flag). Non-player
    /// resolutions are ignored, so `Selector::Target(0)` over a damage target
    /// only fires when a player was hit.
    LifeGainLockGame { who: Selector },
    /// Set `Player.spells_uncounterable_this_turn` on each resolved player —
    /// "spells you control can't be countered for the rest of this turn"
    /// (Veil of Summer). Cleared at the next untap.
    GrantSpellsUncounterableThisTurn { who: Selector },
    /// Like `GrantSpellsUncounterableThisTurn` but only for *creature* spells
    /// (Domri, Anarch of Bolas's "Creature spells you cast this turn can't be
    /// countered"). Cleared at the next untap.
    GrantCreatureSpellsUncounterableThisTurn { who: Selector },
    /// Add `colors` to each resolved player's
    /// `hexproof_from_colors_this_turn` — "you and permanents you control
    /// gain hexproof from [colors] until end of turn" (Veil of Summer's
    /// rider). Cleared at the next untap.
    GrantHexproofFromColorThisTurn { who: Selector, colors: Vec<crate::mana::Color> },
    /// "You gain hexproof until your next turn" (Blossoming Calm). Sets
    /// `Player.hexproof_until_next_turn`; cleared at that player's untap.
    GainHexproofUntilYourNextTurn { who: PlayerRef },
    /// Stamp `uncounterable` on a target spell already on the stack —
    /// "Target spell can't be countered" (Vexing Shusher's activation).
    MakeSpellUncounterable { what: Selector },
    /// Set `Player.cant_cast_noncreature_this_turn` on each resolved player —
    /// "those players can't cast noncreature spells this turn"
    /// (Ranger-Captain of Eos). Cleared at the next untap.
    CantCastNoncreatureThisTurn { who: Selector },
    /// "Each player names a card" — one `NameCard` decision per resolved seat
    /// in APNAP order, stashed in `GameState.names_this_resolution` for a
    /// later step in the same resolution to read (Conundrum Sphinx).
    EachPlayerNamesCard { who: PlayerRef },
    /// "Each player reveals the top card of their library. If it's the card
    /// they named, that player puts it into their hand; if it isn't, they put
    /// it on the bottom of their library." Reads the names stashed by
    /// `EachPlayerNamesCard` (Conundrum Sphinx).
    EachPlayerRevealTopKeepIfNamed { who: PlayerRef },
    /// Controller loses `amount` life, a different selector gains it.
    Drain { from: Selector, to: Selector, amount: Value },

    /// CR 122 / 107.16 — the controller gets `amount` energy counters
    /// ({E}). Energy is a per-player resource pool (`Player.energy`), not
    /// tied to any object. "You get {E}{E}" → `AddEnergy(Const(2))`.
    AddEnergy(Value),
    /// The controller gets `amount` experience counters (`Player.experience`).
    /// A per-player resource that only accumulates; payoffs read the count.
    /// Mizzix ("get an experience counter") → `AddExperience(Const(1))`.
    AddExperience(Value),
    /// Pay `amount` energy counters as a cost at resolution; if the
    /// controller has at least that much, deduct it and resolve `then`,
    /// otherwise do nothing. Models energy-only activated/triggered
    /// payoffs ("Pay {E}{E}{E}: …") without a dedicated cost field on
    /// `ActivatedAbility` — the player commits by activating, and the
    /// energy is consumed when the ability resolves.
    PayEnergy { amount: u32, then: Box<Effect> },
    /// "You may pay an amount of {E} equal to `amount`; if you do, `then`."
    /// The `u32` sibling `PayEnergy` takes a fixed cost; this evaluates a
    /// `Value` (typically the returned card's mana value). Pays and runs
    /// `then` when the controller can afford it (bots/tests always take the
    /// upside); no-op otherwise. Jolted Awake's energy reanimation.
    PayEnergyValue { amount: Value, then: Box<Effect> },
    /// CR 701.56 — `who` time travels: for each permanent they control and each
    /// suspended card they own (in exile) with one or more time counters, they
    /// may add or remove a time counter. The bot heuristic removes one from
    /// each suspended card (so it's cast sooner) and adds one to each permanent
    /// with time counters (vanishing — so it lives longer). UI per-object
    /// choice is a follow-up.
    TimeTravel { who: PlayerRef },
    /// "You may pay any amount of {E}; deal that much damage to `to`"
    /// (Galvanic Discharge). The controller spends energy at resolution; the
    /// bot heuristic pays exactly lethal to a creature target (its remaining
    /// effective toughness, capped at available energy) or all available
    /// energy against a planeswalker / player. UI prompting is a follow-up.
    PayAnyEnergyDealDamage { to: Selector },
    /// "You may pay any amount of {E}, then `then`." The controller chooses how
    /// much energy to pay via `Decision::ChooseAmount` (capped at available
    /// energy); the amount is stashed in `state.energy_paid_this_resolution`
    /// so `then` can scale off `Value::EnergyPaidThisEffect`. Bots pay all
    /// available energy. Aether Spike's "pay any amount of {E}; counter that
    /// spell unless its controller pays {1} for each {E} paid this way."
    PayAnyEnergy { then: Box<Effect> },
    /// "Sacrifice/return this unless you pay {E}…" (CR 107.16). Pays `amount`
    /// energy if the controller can afford it; otherwise resolves `otherwise`
    /// (typically `SacrificeSource` / return-to-hand). AutoDecider pays when
    /// able. Lathnu Hellion, Greenbelt Rampager.
    PayEnergyOrElse { amount: u32, otherwise: Box<Effect> },
    /// `Value`-amount sibling of `PayEnergyOrElse`: pay {E} equal to `amount`
    /// (evaluated at resolution — usually the gained creature's mana value) or
    /// resolve `otherwise`. Volatile Stormdrake's "sacrifice that creature
    /// unless you pay {E} equal to its mana value".
    PayEnergyOrElseValue { amount: Value, otherwise: Box<Effect> },
    /// "Sacrifice this unless you pay `mana_cost`" (CR 608-style ETB
    /// tax). Pays `mana_cost` from the controller's floating pool if
    /// affordable; otherwise resolves `otherwise` (typically
    /// `SacrificeSource`). The mana sibling of `PayEnergyOrElse`. Used by
    /// Archway Commons' "When this land enters, sacrifice it unless you
    /// pay {1}."
    PayManaOrElse { mana_cost: crate::mana::ManaCost, otherwise: Box<Effect> },
    /// CR 702.29 — the resolution half of an echo trigger for a `wants_ui`
    /// controller: "sacrifice this unless you pay its echo cost". `None` =
    /// Echo—Discard a card. Prompted via the seat-routed yes/no ask; a paid
    /// mana echo auto-taps lands like the synchronous `process_echo` path.
    EchoPayOrSacrifice { mana_cost: Option<crate::mana::ManaCost> },
    /// CR 702.24 — the `wants_ui` sibling for cumulative upkeep: "sacrifice
    /// this unless you pay `cost` × its age counters". Mana/Life kinds only;
    /// the age counter was already added by `process_cumulative_upkeep`.
    CumulativeUpkeepPayOrSacrifice { cost: crate::card::CumulativeUpkeepCost },
    /// Balance (Restore Balance): each player sacrifices lands/creatures down
    /// to the fewest controlled by any player, and discards down to the
    /// smallest hand. Each player keeps their best (highest-MV, then power)
    /// permanents and highest-MV cards; a `wants_ui` picker is a follow-up.
    Balance,
    /// The generalized Balance: for each entry in `filters`, every player
    /// sacrifices down to the fewest matching permanents any player controls
    /// (keeping their highest-MV, then highest-power); with `hands`, hands then
    /// discard down to the smallest. `Balance` is
    /// `BalanceMatching { filters: [Land, Creature], hands: true }`;
    /// Balancing Act balances over `Permanent` as one group.
    BalanceMatching { filters: Vec<SelectionRequirement>, hands: bool },
    /// Genesis Wave — reveal the top X cards (X from the cast cost), put
    /// every permanent card with mana value ≤ X onto the battlefield, and
    /// the rest into the graveyard. (Printed "any number" collapses to
    /// "all matching".)
    GenesisWave,
    /// CR 107.16 — exile the top card of the controller's library, then they
    /// may pay `energy` {E}; if they do, they may cast that card without
    /// paying its mana cost (the spell stays exiled if not cast). Amped
    /// Raptor's "exile the top card, you may pay {E}{E}{E}{E} to cast it".
    ExileTopMayPayEnergyToCast { energy: u32 },

    // ── Cards / draw / discard / mill ────────────────────────────────────────
    Draw    { who: Selector, amount: Value },
    /// CR 701.45 — Learn. `who` may reveal a Lesson card they own from their
    /// sideboard ("outside the game") and put it into their hand, or discard
    /// a card to draw a card. Resolved via `Decision::Learn`. When `who`'s
    /// sideboard holds no Lesson, falls back to the legacy `Draw 1`
    /// approximation so no-sideboard games behave as before.
    Learn   { who: PlayerRef },
    /// Discard `amount` cards. If `random`, chosen randomly; else by `who`.
    Discard { who: Selector, amount: Value, random: bool },
    /// "…discards a [filter] card at random" (Rag Man). Only cards matching
    /// `filter` are eligible; nothing is discarded when none do.
    DiscardMatchingAtRandom { who: PlayerRef, filter: SelectionRequirement },
    /// "`who` exiles `amount` card(s) from their hand" — the player chooses
    /// (auto-decider exiles by hand order). Like `Discard` but routes to
    /// exile instead of the graveyard (Ashiok, Nightmare Muse −3).
    ExileFromHand { who: Selector, amount: Value },
    /// "Each resolved player discards their whole hand, then draws that many
    /// cards." Captures the hand size *before* discarding (Soratami Seer,
    /// Mind's Eye-style wheels). Distinct from `Discard` + `Draw` because the
    /// draw count must read the pre-discard hand.
    DiscardHandDrawThatMany { who: Selector },
    /// "`who` discards `count` cards unless they discard a card matching
    /// `instead`" (Wrench Mind). With a match in hand the discarder keeps
    /// the small side automatically (lowest-MV match); otherwise the full
    /// `count` is discarded via the regular Discard path.
    DiscardUnlessKind { who: PlayerRef, count: Value, instead: SelectionRequirement },
    /// Ad Nauseam — repeatedly offer "reveal the top card of your library
    /// and put it into your hand, losing life equal to its mana value?"
    /// until the controller declines or the library empties. AutoDecider
    /// declines immediately; scripted/UI deciders drive the loop.
    RevealTopToHandLoseLifeRepeat,
    /// Discard any number of cards (0 to hand-size, player's choice). Used by
    /// "discard any number of cards, then draw that many cards plus one"
    /// effects (Colossus of the Blood Age, Mind Roots-style "any number"
    /// discards). The discarded count is added to
    /// `state.cards_discarded_this_resolution`, so a follow-up `Draw` step
    /// in the same `Seq` can reference `Value::CardsDiscardedThisEffect`
    /// for the "draw equal to discarded" rider. AutoDecider picks 0 (the
    /// conservative default); ScriptedDecider supplies the exact discard
    /// list via `DecisionAnswer::Discard(_)`.
    DiscardAnyNumber { who: Selector },
    /// Set `Player.max_hand_size = None` on each resolved player, for the
    /// rest of the game. Used by Wisdom of Ages ("You have no maximum hand
    /// size for the rest of the game"), Reliquary Tower's static (which
    /// actually wires through a layer, but the simpler "for the rest of the
    /// game" cards can flip the flag directly). Skips the cleanup-step CR
    /// 514.1 discard-down step in `do_cleanup`.
    SetNoMaxHandSize { who: Selector },
    /// `who` chooses a card from their hand and puts it on top of their library
    /// (CR 701). No-op if the hand is empty. Enter the Infinite's "then put a
    /// card from your hand on top of your library."
    PutCardFromHandOnTopOfLibrary { who: Selector },
    /// "Put `count` cards from your hand on the bottom of your library"
    /// (Sawtooth Loon's "draw two cards, then put two cards from your hand on
    /// the bottom"). Chain after a `Draw` in a `Seq`. Same auto-pick residual
    /// as [`Effect::PutCardFromHandOnTopOfLibrary`] — the picker is the
    /// synchronous decider, so a UI seat isn't prompted.
    PutCardsFromHandOnBottom { who: Selector, count: Value },
    /// CR 701.19 — "Look at [who]'s hand." The resolving controller sees the
    /// hand for the rest of the game (`GameState.hands_revealed_to`), so the
    /// server view and client mirror it. Wanderguard Sentry, Thought Prison.
    LookAtHand { who: Selector },
    /// "Put the cards in your hand on the bottom of your library in any order,
    /// then draw that many cards" (Mindmoil). The hand order is the caster's,
    /// so the bottoming order is the hand order.
    BottomHandThenDrawThatMany { who: PlayerRef },
    /// "Look at the top `count` cards of `who`'s library, then exile one of
    /// them" (Thoughtpicker Witch). The rest stay on top in their original
    /// order; the auto picker exiles the highest-mana-value card.
    LookTopExileOneOfN { who: PlayerRef, count: Value },
    /// "Each player chooses `keep` permanents they control, then sacrifices the
    /// rest" (Razia's Purification). Each player's own pick — the auto picker
    /// keeps their highest-mana-value permanents.
    EachPlayerKeepsNSacrificesRest { keep: Value },
    /// "Each player returns a permanent matching `filter` they control to its
    /// owner's hand" (Curfew). APNAP order; each player picks their own, and a
    /// player controlling no match does nothing.
    EachPlayerReturnsAMatchingPermanent { filter: SelectionRequirement },
    /// CR 402.2b — set each resolved player's maximum hand size to a specific
    /// number (`Player.max_hand_size = Some(size)`). Used by "your maximum
    /// hand size is N" cards such as Null Profusion (zero) or Library of Leng
    /// adjuncts. The cleanup-step CR 514.1 enforcement then discards down to
    /// `size`.
    SetMaxHandSize { who: Selector, size: Value },
    Mill    { who: Selector, amount: Value },
    /// The controller mills `amount` cards, then puts one card matching
    /// `filter` from among those milled this way into their hand (the
    /// controller chooses; nothing happens if none qualify). Cache Grab
    /// ("mill four, then return a permanent card milled this way to hand").
    MillThenToHand {
        amount: Value,
        filter: SelectionRequirement,
        /// Runs when no milled card matched (Patient Naturalist's "if you
        /// can't, create a Treasure").
        #[serde(default)]
        otherwise: Option<Box<Effect>>,
    },
    /// Like [`MillThenToHand`] but the controller may take *up to* `take`
    /// matching cards (not just one). `take` is a [`Value`] so it can scale
    /// with game state — Gather the Pack's spell mastery ("put up to two
    /// creature cards instead of one if 2+ instant/sorcery cards are in your
    /// graveyard").
    MillThenToHandN {
        amount: Value,
        filter: SelectionRequirement,
        take: Value,
        /// Runs when no card was taken (Nashi's "if you put no cards into
        /// your hand this way, put a +1/+1 counter on Nashi").
        #[serde(default)]
        otherwise: Option<Box<Effect>>,
    },
    /// The controller mills one card, then runs a sub-effect keyed on the
    /// milled card's card type: `land` if it's a land, else `creature` if it's
    /// a creature, else `noncreature` (a nonland noncreature card). Old
    /// Rutstein — land → Treasure, creature → Insect, else → Blood. The
    /// sub-effects don't reference the milled card.
    MillThenBranchByType {
        land: Box<Effect>,
        creature: Box<Effect>,
        noncreature: Box<Effect>,
    },
    /// The resolved player mills `amount` cards; then the ability's controller
    /// draws one card for each milled card matching `filter` (Coerced
    /// Confession — "target player mills four cards; you draw a card for each
    /// creature card put into their graveyard this way").
    MillThenDrawPerType { who: Selector, amount: Value, filter: SelectionRequirement },
    /// Reveal cards from the top of each resolved player's library until
    /// `lands` land cards are revealed, then put all revealed cards into
    /// that player's graveyard (Mind Grind, Consuming Aberration's trigger).
    MillUntilLands { who: Selector, lands: Value },
    /// CR 701.13 with a repeat rider — the resolved player mills two cards; if
    /// the two milled cards share `axis`, repeat. Sphinx's Tutelage shares a
    /// colour (nonland cards only); The Tale of Tamiyo shares a card type and
    /// draws on each repeat.
    MillTwoRepeatSharing {
        who: Selector,
        axis: MillShareAxis,
        #[serde(default)]
        draw_on_repeat: bool,
    },
    /// Each player the selector resolves to exiles the top `amount` cards of
    /// their own library (CR 702.115 Ingest, processed-card exile, etc.).
    /// Mirrors `Mill` but routes to the exile zone instead of the graveyard.
    ExileTopOfLibrary {
        who: Selector,
        amount: Value,
        /// Stamp `exiled_with = source` on each exiled card (Bomat Courier's
        /// stash — recoverable via `Selector::CardExiledWithSource`).
        #[serde(default)]
        link_to_source: bool,
        /// Exile face down ("you can't look at it").
        #[serde(default)]
        face_down: bool,
    },
    /// "Exile the top `amount` cards of `who`'s library. You may cast any
    /// number of spells with mana value `max_mv` or less from among them
    /// without paying their mana costs" (Kotis, the Fangkeeper). The
    /// permission lasts for the turn and exiles the card once cast.
    ExileTopAndMayCastUpToMv { who: Selector, amount: Value, max_mv: Value },
    /// "If a source you control would deal noncombat damage to a permanent or
    /// player this turn, it deals that much damage plus `amount` instead"
    /// (Taii Wakeen, Perfect Shot). Turn-scoped, stacking across activations.
    YourNoncombatDamageBonusThisTurn { amount: Value },
    /// CR 702.170 — "exile that spell instead of putting it into your graveyard
    /// as it resolves; it becomes plotted" (Lilah, Undefeated Slickshot).
    /// Stamps the resolving spell so its post-resolution route is
    /// `ZoneDest::ExilePlotted`.
    PlotSpellOnResolve { what: Selector },
    /// "You may cast a permanent spell with mana value `max_mv` or less from
    /// your hand without paying its mana cost. If you don't, `else_`"
    /// (Kellan, the Kid).
    MayCastPermanentFromHandFree { max_mv: Value, else_: Box<Effect> },
    /// Process (Battle for Zendikar / OGW) — "you may put up to `count` cards
    /// an opponent owns from exile into that player's graveyard. If you do,
    /// [`then`]." The controller is asked yes/no (`Decision::OptionalTrigger`);
    /// on yes the oldest `count` eligible exile cards (owned by an opponent of
    /// the controller) move to their owners' graveyards and `then` runs. With
    /// no eligible cards, or on decline, `then` is skipped (the "if you do"
    /// rider). `then` reads the trigger's chosen target(s) via the shared
    /// context — Wasteland Strangler's -3/-3, Mind Raker's discard, etc.
    Process { count: u32, then: Box<Effect> },
    /// Each player the selector resolves to mills half the cards in their
    /// *own* library (rounded up when `rounded_up`, else down). Per-player —
    /// `Mill`'s global amount can't scale to each target's own library size.
    /// Lord Xander, the Collector ("target opponent mills half their library,
    /// rounded down").
    MillHalf { who: Selector, rounded_up: bool },
    /// Each player the selector resolves to discards half the cards in their
    /// *own* hand (rounded up when `rounded_up`, else down), chosen the same
    /// way as `Discard` (random pick-first for the bot harness). Lord Xander
    /// ("target opponent discards half the cards in their hand, rounded down").
    DiscardHalf { who: Selector, rounded_up: bool },
    /// Each player the selector resolves to sacrifices half the permanents
    /// they control matching `filter` (rounded up when `rounded_up`, else
    /// down). Per-player. Lord Xander ("target opponent sacrifices half the
    /// permanents they control, rounded down" — `filter` = `Permanent`).
    SacrificeHalf { who: Selector, filter: SelectionRequirement, rounded_up: bool },
    Scry    { who: PlayerRef, amount: Value },
    Surveil { who: PlayerRef, amount: Value },
    LookAtTop { who: PlayerRef, amount: Value },
    /// Look at the top `amount` cards of `who`'s library and put them back in
    /// any order — all stay on top, none bottomed (Index, Spire Owl, Sage
    /// Owl). Distinct from Scry, which may bottom cards.
    RearrangeTop { who: PlayerRef, amount: Value },
    /// "Look at the top `count` cards of your library. You may reveal up to
    /// `take` cards matching `filter` from among them, then put those on top of
    /// your library and the rest on the bottom in any order." Fertile Thicket
    /// (up to one basic land), Munda, Ambush Leader (any number of Allies).
    LookTopKeepMatchingOnTop {
        who: PlayerRef,
        count: Value,
        take: Value,
        filter: SelectionRequirement,
    },
    /// CR 701.31 — *monstrosity N*. If the source isn't already monstrous,
    /// put N +1/+1 counters on it and it becomes monstrous (emitting
    /// `GameEvent::BecameMonstrous`). Once monstrous, this is a no-op.
    Monstrosity { n: Value },
    /// CR 701.38 — *goad* each creature `what` resolves to: the resolving
    /// effect's controller is added to the creature's `goaded_by` list.
    /// Goaded creatures attack each combat if able and attack a player other
    /// than a goader if able, until that goader's next turn. Disrupt Decorum
    /// (mass goad), Bloodthirsty Blade.
    Goad { what: Selector },
    /// CR 701.60 — *suspect* each creature `what` resolves to: set its
    /// `suspected` flag so it gains menace and can't block (injected as
    /// computed keywords). Repeat Offender, Reasonable Doubt.
    Suspect { what: Selector },
    /// The inverse of [`Suspect`] — clear the `suspected` flag on every creature
    /// `what` resolves to ("~ are no longer suspected", Absolving Lammasu).
    ClearSuspected { what: Selector },
    /// CR 701.35 — *detain* each permanent `what` resolves to: stamp
    /// `detained_by = the effect's controller`. A detained permanent can't
    /// attack or block and its activated abilities can't be activated until
    /// the detainer's next turn (cleared at that turn's start). Lyev Skyknight.
    Detain { what: Selector },
    /// CR 509.4 — create `definition` under your control *blocking* the
    /// attacking creature in target slot 0 (`filter` should require an
    /// attacker — Flash Foliage). No-op if the target isn't attacking; the
    /// token joins the block map and marks the attacker blocked.
    CreateTokenBlocking { definition: crate::card::TokenDefinition, filter: SelectionRequirement },
    /// CR 701.49 — Venture into the dungeon: enter the first room of a
    /// chosen dungeon (auto: Lost Mine of Phandelver) or advance to the
    /// next room; room abilities resolve inline (`base::dungeons`).
    /// Resolving the final room completes the dungeon
    /// (`Player.dungeons_completed`, `EventKind::DungeonCompleted`).
    /// CR 309.5b/309.6 — the dungeon a player finished leaves the game. Tail
    /// of the bottommost room's ability so the removal happens as that ability
    /// finishes resolving, not while it's still on the stack.
    CompleteDungeon,
    Venture,
    /// CR 726.2 — "venture into `dungeon`": like [`Effect::Venture`] but the
    /// dungeon is fixed rather than chosen (the initiative's Undercity). A
    /// player already in a different dungeon is unaffected.
    VentureInto { dungeon: String },
    /// CR 726 — `who` takes the initiative.
    TakeInitiative { who: PlayerRef },
    /// CR 701.29 — *fateseal N*: look at the top `amount` cards of each
    /// targeted opponent's library and put any number of them on the bottom
    /// (the rest stay on top). The library-side mirror of Scry. AutoDecider
    /// bottoms nothing; a `wants_ui` controller / scripted decider chooses.
    Fateseal { who: PlayerRef, amount: Value },
    /// "Look at the top `count` cards of your library. Put any number of them
    /// into your hand and the rest into your graveyard. You lose `life_per_card`
    /// life for each card put into your hand this way." Search for Blex,
    /// Painful Truths-family digs. AutoDecider takes none (the safe default);
    /// a scripted/`wants_ui` decider picks the subset.
    DigToHandLoseLife { count: Value, life_per_card: Value },
    /// CR 701.59 — *collect evidence N*: the controller may exile cards with
    /// total mana value `amount` or greater from their graveyard; if they do,
    /// `then` resolves (the "when you do" reflexive payoff). The exile is the
    /// cost, so `then` only fires when it's paid. Sample Collector. (The
    /// engine auto-picks the cheapest qualifying set to exile; `then`'s
    /// targets are chosen when the ability goes on the stack.)
    CollectEvidence { amount: Value, then: Box<Effect> },
    /// CR 701.59 — *collect evidence X*, where the controller chooses X: they
    /// may exile any cards from their graveyard; X is the total mana value
    /// exiled. If they do, `then` resolves with `ctx.x_value = X` (read via
    /// `Value::XFromCost`). Incinerator of the Guilty. UI players pick the
    /// exact set; the bot policy exiles the whole graveyard when it opts in.
    CollectEvidenceX { then: Box<Effect> },
    /// CR 701.61 — *forage*: as an optional cost, exile three cards from your
    /// graveyard **or** sacrifice a Food. If paid, the reflexive `then` payoff
    /// resolves. Engine prefers exiling three graveyard cards; falls back to
    /// sacrificing a Food. Ships Bloomburrow forage payoffs.
    Forage { then: Box<Effect> },
    /// CR 701.57 — *discover N*: exile cards from the top of the controller's
    /// library until exiling a nonland card with mana value `n` or less, then
    /// either cast it without paying its mana cost or put it into hand; the
    /// rest go to the bottom in a random order. Geological Appraiser,
    /// Trumpeting Carnosaur.
    Discover {
        n: Value,
        /// When `Some`, the stop condition is "a card matching this filter
        /// with MV ≤ n" instead of the printed "nonland card with MV ≤ n"
        /// (Codie's "an instant or sorcery card with lesser mana value").
        #[serde(default)]
        filter: Option<crate::card::SelectionRequirement>,
    },
    /// "Exile cards from the top of `who`'s library until you exile a nonland
    /// card." The exiled cards stay in exile; the stopping card's mana value
    /// is published as `Value::LastExiledManaValue` (Undying Flames).
    ExileTopUntilNonland { who: PlayerRef },
    /// "Exile the top `batch` cards. If the last card exiled isn't a land
    /// card, repeat this process." `then` runs afterwards and reads the
    /// nonland tally through [`Value::NonlandCardsExiledThisEffect`].
    /// Rally the Horde.
    ExileTopBatchesUntilLandLast { batch: u32, then: Box<Effect> },
    /// "Reveal the top `count` cards of your library. An opponent chooses one
    /// of them. Put that card into your graveyard and the rest into your
    /// hand." Murmurs from Beyond — the pick is routed to an opponent's seat.
    RevealTopOpponentBinsOne { count: u32 },
    /// "You may cast a spell matching `filter` from among cards exiled with
    /// this source without paying its mana cost." Kaho, Minamo Historian —
    /// `filter`'s X resolves against the activation's chosen X.
    MayCastExiledWithSource { filter: crate::card::SelectionRequirement },
    /// "Exile cards from the top of `who`'s library until you exile a nonland
    /// card; you may play/cast that card [`duration`]. The other exiled cards
    /// stay in exile." The impulse-until-nonland family (Territorial Bruntar's
    /// landfall, Solstice Revelations) — unlike `Discover`, there's no MV cap
    /// on the stop and the passed-over cards aren't bottomed.
    ExileTopUntilNonlandMayPlay {
        who: PlayerRef,
        duration: crate::card::MayPlayDuration,
        /// `true` grants a free cast; otherwise the caster pays the card's
        /// normal mana cost from exile.
        #[serde(default)]
        free: bool,
        /// When `Some`, the may-play is granted only if the nonland card's mana
        /// value is *less than* this value; otherwise it's put into hand
        /// instead (Solstice Revelations — "if its mana value is less than the
        /// number of Mountains you control").
        #[serde(default)]
        hand_unless_mv_below: Option<Value>,
        /// Grant the may-cast permission to the EXILING player (the library
        /// owner) rather than the effect's controller — "ITS CONTROLLER
        /// exiles ... then may cast it" (Transforming Flourish). Defaults
        /// false (the caster gets the cast — the impulse-draw family).
        #[serde(default)]
        grant_to_exiling_player: bool,
    },
    /// "Look at the top `count` cards of your library. You may put a land card
    /// from among them onto the battlefield tapped. Put the rest on the bottom
    /// of your library in a random order." Ignis Scientia's ETB. Reuses the
    /// Impulse machinery (`to_battlefield` + `tapped`).
    DigForLandToBattlefield { count: Value },
    /// CR 702.39 — *provoke*: untap the creature `what` resolves to and force
    /// it to block this combat's source attacker if able. Sets the target's
    /// `must_block` to the effect source. Used by `shortcut::provoke`.
    Provoke { what: Selector },
    /// CR 701.40 — each permanent `who` resolves to *explores*: its
    /// controller reveals the top card of their library. If it's a land,
    /// it goes to hand; otherwise the exploring permanent gets a +1/+1
    /// counter and the revealed card stays on top (the optional
    /// "put into graveyard" choice is collapsed to keep-on-top). An empty
    /// library still counts as a (cardless) explore and grants the counter.
    /// Each explore emits `GameEvent::Explored` so payoff triggers
    /// ("whenever a creature you control explores") can fire.
    Explore { who: Selector },
    /// "Look at the top `count` cards of your library, put one of them into
    /// your hand, and the rest on the bottom of your library (or into your
    /// graveyard if `rest_to_graveyard`)." Impulse / Strategic Planning /
    /// Flow State. The controller picks via the `SearchLibrary` decision
    /// (auto-decider keeps the top card).
    /// `pick_filter` restricts which revealed cards are eligible to take
    /// (Satyr Wayfinder — "you may put a *land* card into your hand"); the
    /// rest (including non-eligible cards) follow `rest_to_graveyard`.
    /// `None` means any revealed card is eligible.
    LookPickToHand(Box<LookPick>),
    /// "Look at the top `count` cards; put one back on top and the rest into
    /// your graveyard" (Sage of Days). The controller (via the `SearchLibrary`
    /// picker) keeps one revealed card on top; the rest are milled. Shares the
    /// `ImpulsePending` machinery with `keep_on_top: true`. With
    /// `who: Some(..)` the effect reads another player's library instead and
    /// auto-picks (lowest MV kept on an opponent's — Dimir Charm mode 3).
    /// With `exile_rest` the non-kept cards are exiled instead of milled
    /// (Devourer of Destiny's opening-hand reveal), and with
    /// `rest_bottom_random` they go to the bottom in a random order
    /// (Aladdin's Lamp).
    LookTopKeepOneRestToGraveyard {
        count: Value,
        #[serde(default)]
        who: Option<PlayerRef>,
        #[serde(default)]
        exile_rest: bool,
        #[serde(default)]
        rest_bottom_random: bool,
    },
    /// Remove all counters from the selected permanent; the controller's
    /// next spell this turn costs {1} less per counter removed (Mutated
    /// Cultist's cast trigger — the "or opponent" half is dropped).
    RemoveAllCountersDiscountNextSpell { what: Selector },
    /// Exile the selected card(s), stamping `exiled_with = source` (an
    /// imprint-style permanent link with no return rider). Agatha's Soul
    /// Cauldron's {T} ability.
    ExileWithSource { what: Selector },
    /// "Exile all [filter]. For each permanent exiled this way, its controller
    /// draws a card" (Martyr's Cry). The draw is per exiled permanent and goes
    /// to the permanent's controller, not the effect's.
    ExileEachMatchingThenControllerDraws { filter: SelectionRequirement },
    /// Tempting offer (ability word, CR 207.2c): run `body` for the
    /// controller; each opponent may copy it for themselves, and the
    /// controller re-runs it once per opponent who accepted. Tempt with
    /// Bunnies.
    TemptingOffer { body: Box<Effect> },
    /// "[who] may [accept]. If a player does, run `on_accept` (that player
    /// bound to slot 0) and stop; if no one does, run `otherwise`." Asked in
    /// APNAP order via the synchronous decider (same wants_ui gap as
    /// TemptingOffer). Vexing Devil, Browbeat, Risk Factor.
    PlayersMayAccept {
        who: PlayerRef,
        description: String,
        on_accept: Box<Effect>,
        otherwise: Box<Effect>,
    },
    /// "If a creature would enter the battlefield under an opponent's
    /// control this turn, it enters under your control instead." Gather
    /// Specimens (CR 614 control-ETB replacement; expires at cleanup).
    StealCreatureEtbThisTurn,
    /// "You may put a card matching `filter` from your hand or graveyard
    /// onto the battlefield." Dakkon, Shadow Slayer −6. Auto-pick: the
    /// highest-MV match; a `wants_ui` controller picks (or declines) via
    /// the search decision.
    PutFromHandOrGraveyardOntoBattlefield { filter: crate::card::SelectionRequirement },
    /// "Reveal the top card of your library and put it into your hand. `who`
    /// loses life equal to its mana value." Sorin, Grim Nemesis +1 (`who:
    /// EachOpponent`); Caustic Bronco's saddled/unsaddled attack (each opponent
    /// / you). With `you_gain`, the controller also gains that much life
    /// (Twilight Prophet's city's-blessing drain).
    RevealTopToHandLoseMv {
        who: PlayerRef,
        #[serde(default)]
        you_gain: bool,
    },
    /// "Reveal the top card of your library. If it matches `filter`, put it
    /// onto the battlefield" — optionally with haste and a next-end-step
    /// sacrifice (Killer Instinct). A non-matching card stays on top.
    RevealTopDeployIfMatch {
        filter: SelectionRequirement,
        #[serde(default)]
        haste: bool,
        #[serde(default)]
        sacrifice_at_next_end_step: bool,
    },
    /// "Look at the top `count` cards of your library. For each, put it into
    /// your graveyard unless you pay `life` life. Then put the rest into your
    /// hand." Moonlight Bargain. The auto payer buys every card it can afford
    /// while staying above 1 life.
    LookTopEachPayLifeOrBin { count: Value, life: u32 },
    /// CR 708.2a — turn each resolved permanent face down: it becomes a 2/2
    /// creature with no name, types, or abilities. A no-op on a permanent
    /// that is already face down (CR 708.2b). Ixidron.
    TurnFaceDown { what: Selector },
    /// CR 614 — "The next time you would draw a card this turn, [`body`]
    /// instead." Queues a one-shot replacement on the controller consumed by
    /// the next draw (the Onslaught Words cycle). Charges stack and are
    /// spent front-first; unused ones evaporate at end of turn.
    ReplaceYourNextDrawThisTurn { body: Box<Effect> },
    /// CR 707.9 — turn each face-down permanent the selector picks face up
    /// *without* paying its morph cost (Break Open, Ixidor, Reality
    /// Sculptor). Non-face-down picks are skipped. A pick whose face-up side
    /// isn't a creature card (a cloaked land) *can't* be turned up; `if_cant`
    /// then runs with that permanent stamped as target 0 — Etrata, Deadly
    /// Fugitive's "exile it, then you may cast the exiled card for free".
    TurnFaceUpFree { what: Selector, if_cant: Option<Box<Effect>> },
    /// "Creatures you control gain each of `keywords` until end of turn if a
    /// creature you control already has it" (Concerted Effort's upkeep sweep).
    /// Reads the controller's *computed* keywords once, then grants the union
    /// to every creature they control.
    ShareKeywordsAmongYourCreatures { keywords: Vec<Keyword> },
    /// "Choose a card name. Target player reveals cards from the top of their
    /// library until one with that name is revealed. If it is, the rest of the
    /// revealed cards go to their graveyard and the named card goes back on
    /// top. Otherwise they shuffle." Tunnel Vision.
    NameCardRevealUntilThenBin { who: PlayerRef },
    /// "`who` chooses a card name, then reveals the top card of their
    /// library. On a match it goes to their hand, otherwise to their
    /// graveyard." Petra Sphinx.
    NameCardThenRevealTopBin { who: PlayerRef },
    /// Exile every permanent card sacrificed to pay the *current* activation's
    /// cost (Sword of the Ages' "then exile this artifact and those creature
    /// cards"). Reads the batch stashed by `activate_ability`.
    ExileCostSacrificedBatch,
    /// "Exile the top `exile_count` cards of your library, then reveal cards
    /// from the top until you reveal a card with the name a preceding
    /// `Effect::NameCard` stamped on the source. Put that card into your hand
    /// and exile all other cards revealed this way." Divining Witch.
    ExileTopThenRevealUntilNamed { exile_count: Value },
    /// "Reveal the top `count` cards of your library and put all of them with
    /// the name a preceding `Effect::NameCard` stamped into your hand. Exile
    /// the rest." Desperate Research.
    RevealTopTakeNamedExileRest { count: Value },
    /// "Each player chooses from the lands they control a land of each basic
    /// land type, then sacrifices the rest." Global Ruin. Each player keeps
    /// their first land of each basic type (a `Decision::ChooseCards` pick for
    /// a UI seat) and sacrifices every other land.
    EachPlayerKeepsOneOfEachBasicTypeSacrificesRest,
    /// "Each player chooses a card in their hand, then reveals it. The owner
    /// of each creature card revealed this way with the lowest mana value
    /// puts it onto the battlefield." Stronghold Gambit. Each player's pick
    /// routes through `Decision::ChooseCards` (auto-pick: their cheapest
    /// creature card, else their cheapest card).
    RevealChosenCardsLowestCreaturesEnter,
    /// "Until end of turn, [what] loses 'Prevent all damage that would be
    /// dealt to this'" (Glittering Lion / Lynx). Sets
    /// `CardInstance.damage_prevention_off_eot`.
    TurnOffDamagePreventionThisTurn { what: Selector },
    /// CR 611.2 — "This turn, whenever a [`filter`] …": installs a floating
    /// watcher that every matching permanent carries for the rest of the turn,
    /// including ones that enter after this resolves (Mage Hunters' Onslaught).
    /// The trigger fires with the matching permanent as its source, so
    /// `EventScope::SelfSource` and `Selector::This` read that permanent.
    GrantTriggeredAbilityThisTurnToMatching {
        filter: crate::card::SelectionRequirement,
        trigger: Box<crate::card::TriggeredAbility>,
    },
    /// CR 509 — "Attacking creatures become blocked" (Fog Patch), even ones
    /// that can't be blocked. No blocker is assigned, so nothing deals or is
    /// dealt combat damage by the block.
    AttackingCreaturesBecomeBlocked,
    /// Gonti, Lord of Luxury's ETB: look at the top `count` cards of `who`'s
    /// library, exile one face down (auto-pick: highest MV) with a
    /// while-exiled cast permission for you, and bottom the rest randomly.
    /// `who` defaults to an opponent (Gonti); The Key to the Vault points it
    /// at `PlayerRef::You`. (The any-color spend clause is dropped.)
    LookTopExileOneMayPlay {
        count: Value,
        #[serde(default = "player_ref_target_zero")]
        who: PlayerRef,
    },
    /// "Look at the top `count` cards. You may put a land card from among them
    /// onto the battlefield tapped. If you don't, put a card from among them
    /// into your hand. Put the rest on the bottom in a random order." Planar
    /// Genesis. Resolution prefers ramp: deploys a land if one is revealed,
    /// otherwise takes the highest-mana-value card to hand.
    LookTopDeployLandOrHand { count: Value },
    /// "Look at the top card of your library. If it's a land card, you may put
    /// it onto the battlefield (tapped if `tapped`)." A non-land — or a decline —
    /// stays on top (unlike the bottoming look-top variants). Mobile Homestead.
    LookTopMayDeployLand { tapped: bool },
    /// "Look at the top card of your library. If it matches `filter`, you may
    /// reveal it and put it into your hand. If you don't put it into your hand,
    /// put it on the bottom of your library." Vivien's Grizzly, Duskwatch
    /// Recruiter. The yes/no is a `Decision::OptionalTrigger` (UI players
    /// suspend/resume like `Effect::MayDo`).
    LookTopMayRevealMatchToHandElseBottom { filter: SelectionRequirement },
    /// Cabal Therapy: choose a nonland card name; target player discards
    /// every card with that name from their hand.
    NameCardTargetDiscardsMatching,
    /// "Target player reveals their hand and discards all `filter` cards"
    /// (Trapfinder's Trick). The reveal is knowledge-only; every match is
    /// discarded through the normal discard path so discard triggers fire.
    RevealHandDiscardAllMatching { who: PlayerRef, filter: SelectionRequirement },
    /// "Target player reveals their hand" (Darigaaz, the Igniter). Knowledge
    /// only: the hand becomes visible to the resolving controller for the rest
    /// of the game via `hands_revealed_to`.
    RevealHand { who: PlayerRef },
    /// "Target player reveals the top card of their library" (Aven
    /// Windreader). Knowledge only — the top card becomes visible to every
    /// player for as long as it stays on top.
    RevealTopOfLibrary { who: PlayerRef },
    /// "Reveal the top card of `who`'s library. If it matches `filter`,
    /// `on_match`. Then that player shuffles." (Prophecy.)
    RevealTopThenShuffle {
        who: PlayerRef,
        filter: SelectionRequirement,
        on_match: Box<Effect>,
    },
    /// "`who` loses all poison counters" (Leeches). Emits no `PoisonAdded`.
    RemoveAllPoison { who: PlayerRef },
    /// "Deals `amount` damage to each creature for each Aura attached to that
    /// creature" (Baki's Curse). Creatures with no Aura take nothing.
    DamageEachCreaturePerAura { amount: Value },
    /// "Sacrifice this permanent unless you tap an untapped creature you
    /// control" (Koskun Falls). The tap is a real cost paid on resolution;
    /// declining, or controlling no untapped creature, sacrifices the source.
    SacrificeSourceUnlessTapCreature,
    /// The filtered sibling — "sacrifice this unless you tap an untapped
    /// permanent you control matching `filter`" (Public Thoroughfare's
    /// artifact-or-land toll).
    SacrificeSourceUnlessTapMatching { filter: SelectionRequirement },
    /// "Each player may draw up to `max` cards. For each card less than `max`
    /// a player draws this way, that player gains `life_per_card` life."
    /// (Truce.) Asked in APNAP order.
    EachPlayerDrawsUpToElseGainsLife { max: u32, life_per_card: u32 },
    /// Liar's Pendulum: name a card, then target opponent guesses whether a
    /// card with that name is in your hand. Reveal and draw when they guess
    /// wrong. The guess rides a `Decision::OptionalTrigger` asked of the
    /// opponent.
    LiarsPendulum,
    /// Scythe of the Wretched: a creature the Equipment's host damaged this
    /// turn died — return that card to the battlefield under the Equipment
    /// controller's control and attach the Equipment to it. The check reads
    /// the death LKI snapshot's `damaged_by_this_turn`.
    ReturnVictimAndAttachSelf,
    /// Imprint — exile up to `count` cards matching `filter` from a **single**
    /// graveyard (the controller picks the graveyard, then the cards), linked
    /// to the source via `exiled_with`. Spellweaver Helix.
    ImprintFromGraveyard { filter: SelectionRequirement, count: Value },
    /// Spellweaver Helix: a card with the same name as one of the two cards
    /// exiled with this artifact was just cast — copy the *other* one and
    /// cast the copy without paying its mana cost.
    SpellweaverCopy,
    /// Slaughter Games: choose a nonland card name, then exile every card with
    /// that name from target opponent's graveyard, hand, and library; that
    /// player then shuffles.
    NameCardExileMatchingAllZones,
    /// Brain Pry: choose a nonland card name; target player reveals their hand
    /// and discards one card with that name. If they can't, you draw a card.
    NameCardTargetDiscardsOneOrYouDraw,
    /// "Choose a nonland card name, then reveal the top `count` cards of
    /// your library. Put all cards with the chosen name from among them into
    /// your hand and the rest into your graveyard." Tamiyo, Collector of
    /// Tales +1.
    NameCardRevealTop { count: Value },
    /// "Put a creature card with mana value `mv` exiled with [the source]
    /// onto the battlefield under your control. That creature is a Nightmare
    /// in addition to its other types." Ashiok, Nightmare Weaver −X.
    PutExiledCreatureOntoBattlefield { mv: Value },
    /// "You may put a creature card exiled this way onto the battlefield"
    /// (Anzrag's Rampage). Deploys one creature card among `what`, optionally
    /// with haste and a next-end-step return to its owner's hand.
    DeployExiledCreature { what: Selector, haste: bool, return_to_hand_eot: bool },
    /// "Copy it. You may cast the copy without paying its mana cost"
    /// (Reenact the Crime) — mints a token copy of the card `what` resolves to
    /// and offers a free cast of it.
    CopyCardAndCastFree { what: Selector },
    /// Imprint payoff: turn the card exiled face down with this source face up
    /// and, if it's a creature card, put it onto the battlefield under the
    /// source's controller (Summoner's Egg). Non-creature cards stay exiled
    /// face up.
    RevealImprintDeployCreature,
    /// "Target opponent reveals their hand. You may copy an instant or sorcery
    /// card in it. If you do, you may cast the copy without paying its mana
    /// cost." Reversal of Fortune.
    ReversalOfFortune,
    /// "Each player sacrifices a permanent of their choice unless they discard
    /// a card" (Possessed Portal's end step). Each player is asked in APNAP
    /// order; the auto-picker discards when it can.
    EachPlayerSacrificesUnlessDiscards,
    /// "Target opponent reveals the top `count` cards of their library. You
    /// may put a nonland permanent card with mana value `count` or less from
    /// among them onto the battlefield under your control. Then that player
    /// shuffles." (Both Xs are the same value, as printed.) Lonis, Genetics
    /// Expert.
    RevealOpponentTopPutOntoBattlefield { count: Value, filter: SelectionRequirement },
    /// "Look at the top `count` cards of your library. You may put a card
    /// matching `filter` from among them onto the battlefield tapped and
    /// attacking (joining the current combat); it gains indestructible until
    /// end of turn. Put the rest on the bottom in a random order." Winota,
    /// Joiner of Forces. No-op outside combat. Auto-picks the highest-power
    /// match; the new attacker hits the same defender the triggering creature
    /// (`trigger_source`) is attacking.
    LookTopMayDeployAttacking { count: Value, filter: SelectionRequirement },
    /// Unattach each resolved Equipment/Aura-like permanent from its host
    /// (Stolen Uniform's end-of-theft cleanup). Auras die to SBA afterward;
    /// Equipment simply sits unattached.
    Unattach { what: Selector },
    /// Run `body` at the beginning of the next end step, capturing this
    /// resolution's slot-0/1 targets (a generic `DelayedKind::NextEndStep`
    /// wrapper — Stolen Uniform's delayed unattach).
    AtNextEndStep { body: Box<Effect> },
    /// "… at end of combat" — a CR 603.7a delayed trigger that fires in the
    /// current combat's end-of-combat step (Vebulid's self-destruct, Wall of
    /// Junk's bounce). Outside combat it waits for the next one.
    AtEndOfCombat { body: Box<Effect> },
    /// "Look at the top `count` cards of your library. You may put those cards
    /// on the bottom in any order. If you do, `then`; otherwise `else_`."
    /// Petals of Insight. The controller is asked once for the whole batch.
    LookTopMayBottomAllElse { count: Value, then: Box<Effect>, else_: Box<Effect> },
    /// "For each [filter], return it to its owner's hand unless that permanent's
    /// controller pays `cost`." Cut the Tethers — one pay-or-bounce decision per
    /// permanent, asked of its controller.
    ReturnEachUnlessPays { filter: SelectionRequirement, cost: crate::mana::ManaCost },
    /// "Target opponent chooses one of the top two cards of your graveyard.
    /// Exile that card and put the other one into your hand" (Phyrexian
    /// Grimoire). `who` is the choosing player; the graveyard is the effect
    /// controller's. A one-card graveyard goes straight to hand.
    TopTwoGraveyardOpponentSplits { who: Selector },
    /// "That permanent's activated abilities can't be activated this turn"
    /// (Interdict). Records each resolved permanent in
    /// `GameState.abilities_locked_this_turn`; `activate_ability` rejects
    /// non-mana activations from a locked source until cleanup.
    LockActivatedAbilitiesThisTurn { what: Selector },
    /// "Create `definition`. Return this card to the battlefield under its
    /// owner's control when that token dies." Tatsumasa, the Dragon's Fang —
    /// pairs with an `exile_self_cost` activation, so the source is in exile
    /// while the token lives.
    CreateTokenReturnSelfWhenItDies { definition: crate::card::TokenDefinition },
    /// Firion — reduce the generic portion of each resolved permanent's
    /// printed Equip cost by `amount` (stamped onto the minted token copy).
    ReduceEquipCost { what: Selector, amount: u32 },
    /// Register a delayed trigger sacrificing each resolved permanent at the
    /// beginning of the controller's next upkeep (Firion's copy token).
    SacrificeAtNextUpkeep { what: Selector },
    /// "Sacrifice them at the beginning of the next end step" (Pull). The
    /// selector is resolved now; each hit gets its own CR 603.7a delayed
    /// trigger.
    SacrificeAtNextEndStep { what: Selector },
    /// CR 603.7a — "At the beginning of your next upkeep, `body`." A general
    /// one-shot delayed trigger bound to the resolving source (Giant Slug,
    /// Hazezon Tamar).
    AtYourNextUpkeep { body: Box<Effect> },
    /// CR 603.7a — "At the beginning of the next turn's upkeep, `body`."
    /// Fires on the very next upkeep step, whoever's turn it is; the Homelands
    /// cantrip rider (Headstone, Jinx, Prophecy, Renewal).
    AtNextTurnsUpkeep { body: Box<Effect> },
    /// CR 702.141 Encore — for each opponent, create a token copy of the
    /// source (read from exile after the encore cost exiled it) that attacks
    /// that opponent this turn if able (goad-style requirement). The tokens
    /// gain haste and are sacrificed at the beginning of the next end step.
    EncoreTokens,
    /// Memories Returning — reveal the top five cards; alternating picks
    /// (you take one to hand, an opponent bottoms one, twice each) leave you
    /// three cards in hand and two on the bottom. Auto-heuristics: you take
    /// the highest mana value, the opponent bottoms the highest remaining.
    RevealFiveDraftAgainstOpponent,
    /// Choco — look at the top `count` cards of your library; put one into
    /// your hand (auto-pick: highest-MV nonland, else highest-MV), then put
    /// every land card from among the rest onto the battlefield tapped and
    /// the remainder into your graveyard.
    LookTopTakeOneDeployLandsRestGraveyard { count: Value },
    /// Triple Triad — each player exiles their library top. Until end of
    /// turn the controller may play the card they own exiled this way for
    /// free, plus each other card exiled this way with lesser mana value.
    ExileEachTopFreePlayLesser,
    /// Random Encounter — shuffle your library, mill `amount`; put each
    /// creature card milled this way onto the battlefield with haste, and
    /// return those creatures to their owners' hands at the beginning of the
    /// next end step.
    MillDeployCreaturesUntilEndStep { amount: Value },
    /// Gilgamesh — look at the top `count` cards of your library; put every
    /// card matching `filter` onto the battlefield ("any number" resolved as
    /// take-all); bottom the rest in a random order. When one or more entered
    /// this way, `then` runs with the moved cards on `Selector::LastMoved`.
    LookTopPutMatchingOntoBattlefield {
        count: Value,
        filter: SelectionRequirement,
        then: Option<Box<Effect>>,
        /// Cap on how many matches are deployed (Expand the Sphere's "up to
        /// two"). None = all matching.
        #[serde(default)]
        max: Option<u32>,
        /// Deployed cards enter tapped.
        #[serde(default)]
        tapped: bool,
        /// The non-deployed remainder is exiled rather than left on the
        /// library bottom — "Exile the top N cards. Put all [filter] from
        /// among them onto the battlefield" (Tezzeret, Master of the Bridge).
        #[serde(default)]
        exile_rest: bool,
    },
    /// "Reveal the top `count` cards of your library. For each card type, you
    /// may put a card of that type from among them into your hand. Put the
    /// rest on the bottom of your library in a random order." Atraxa, Grand
    /// Unifier. Resolution takes one revealed card per card type present
    /// (a card satisfying multiple types is taken once); the leftovers are
    /// bottomed. Card types considered: artifact, battle, creature,
    /// enchantment, instant, land, planeswalker, sorcery.
    RevealTopTakeOnePerType { who: PlayerRef, count: Value },
    /// Reveal the top `count` cards of `who`'s library, put every card
    /// matching `filter` into their hand, and bottom the rest in a random
    /// order (CR 401.4). "Put any number" is resolved as take-all-matching
    /// (the value-maximizing default). Torsten, Founder of Benalia's
    /// "reveal seven, take creatures and/or lands."
    RevealTopTakeMatchingToHand {
        who: PlayerRef,
        count: Value,
        filter: SelectionRequirement,
        /// "…with different powers" (Rip, Spawn Hunter): at most one card of
        /// each printed power is taken, greediest-first.
        #[serde(default)]
        distinct_powers: bool,
    },
    /// Reveal the top `count` cards of `who`'s library, put every card matching
    /// `filter` into their hand, and put the rest into their graveyard (CR 701).
    /// The graveyard-partition sibling of `RevealTopTakeMatchingToHand`.
    /// Borborygmos Enraged's combat trigger ("lands to hand, the rest to gy").
    RevealTopTakeMatchingRestToGraveyard {
        who: PlayerRef,
        count: Value,
        filter: SelectionRequirement,
    },
    /// Vigean Intuition: choose a card type, then reveal the top `count` cards
    /// of your library; put every card of the chosen type into your hand and
    /// the rest into your graveyard.
    ChooseTypeRevealTopPartition { count: Value },
    /// "Reveal the top `count` cards of your library. For each card type, you
    /// may exile a card of that type from among them. Put the rest into your
    /// graveyard. You may cast a spell from among the exiled cards without
    /// paying its mana cost if you exiled `free_cast_at` or more cards this
    /// way. Then put the rest of the exiled cards into your hand."
    /// (Portent of Calamity.) Each exiled card is exiled *for* one card type.
    RevealTopExileOnePerCardType { count: Value, free_cast_at: u32 },
    /// Fertile Imagination: choose a card type; target opponent reveals their
    /// hand; create `per` 1/1 green Saproling tokens for each card of the
    /// chosen type revealed this way.
    FertileImagination { per: Value },
    /// Guild Feud: target opponent reveals the top three cards of their
    /// library, may put a creature from among them onto the battlefield, and
    /// puts the rest into their graveyard; you do the same with your top three.
    /// If two creatures are put onto the battlefield this way, they fight.
    GuildFeud,
    /// Aethermage's Touch: reveal the top `count` cards of your library; put a
    /// creature card from among them onto the battlefield with a delayed
    /// "return to owner's hand at your end step" rider, then bottom the rest.
    AethermagesTouch { count: Value },
    /// Infernal Tutor: reveal a card from your hand and search your library for
    /// a card with the same name; if you have no cards in hand (Hellbent),
    /// instead search for any card. Put it into your hand, then shuffle.
    InfernalTutor,
    /// Ignorant Bliss: exile your whole hand face down; at the beginning of the
    /// next end step return those cards to your hand, then draw a card.
    IgnorantBliss,
    /// Dovescape: counter the triggering noncreature spell; its caster creates
    /// X 1/1 white-and-blue Bird tokens with flying, where X is that spell's
    /// mana value.
    Dovescape,
    /// Isperia the Inscrutable: choose a card name; the defending player reveals
    /// their hand; if a card with that name is revealed, search your library for
    /// a creature card with flying, put it into your hand, then shuffle.
    IsperiaReveal,
    /// Kindle the Carnage: discard a card at random; if you do, deal damage
    /// equal to its mana value to each creature. You may repeat this any number
    /// of times (asked each round via `Decision::OptionalTrigger`).
    KindleTheCarnage,
    /// Grave Betrayal: on the death of a creature you don't control, schedule a
    /// next-end-step delayed reanimation of that creature under your control.
    GraveBetrayalRegister,
    /// The scheduled reanimation body: return the bound creature card (slot 0)
    /// from its graveyard to the battlefield under your control with an extra
    /// +1/+1 counter, as a black Zombie in addition to its other types.
    GraveBetrayalReanimate,
    /// Exchange `who`'s hand and graveyard (CR 701.10-style swap): every card
    /// in hand moves to the graveyard and every card in the graveyard moves to
    /// hand. Harness Infinity. (Resolved as a direct zone-vector swap; per-card
    /// enters/leaves-graveyard triggers don't fire — a faithful-enough
    /// approximation for the one card that does this.)
    ExchangeHandAndGraveyard { who: PlayerRef },
    /// Each player in `who` exiles all but the bottom `keep` cards of their
    /// library (face down — face-down exile isn't modeled, so the cards are
    /// exiled plainly). Doomsday Excruciator's "each player exiles all but the
    /// bottom six cards of their library."
    ExileLibraryExceptBottom { who: PlayerRef, keep: Value },

    // ── Zone moves ───────────────────────────────────────────────────────────
    /// Move every entity the selector resolves to into `to`.
    Move { what: Selector, to: ZoneDest },
    /// "[Return/put/exile] up to `count` cards of your choice from the set
    /// `from` resolves to" — the player-chooses sibling of
    /// `Move { what: Take {..} }` (which auto-takes in iteration order).
    /// Resolution-time `Decision::ChooseCards`; the auto decider takes the
    /// first `count` so bot play keeps maximizing. `filter` narrows the
    /// resolved set (Bind to Life's "a creature card from AMONG the milled
    /// seven" — `from: LastMoved`, filter Creature). Divergent Equation,
    /// Pull from the Grave, Return to the Ranks, Emeritus of Ideation.
    /// "Return any number of target [filter] cards with total mana value `cap`
    /// or less from `from` to `to`" (March from the Tomb). Takes the cheapest
    /// matches first until the next one would break the budget, so the count is
    /// maximized. No-op when nothing matches.
    MoveWithinTotalManaValue {
        from: Selector,
        filter: SelectionRequirement,
        cap: Value,
        to: ZoneDest,
    },
    MoveChosen {
        from: Selector,
        #[serde(default)]
        filter: Option<SelectionRequirement>,
        count: Value,
        #[serde(default)]
        up_to: bool,
        to: ZoneDest,
    },
    /// "Put `what` into its owner's library just beneath the top `count`
    /// cards of that library" (Quarry Colossus). The dynamic-`count` sibling
    /// of `LibraryPosition::FromTop`; CR 401.7 bottoms it when the library is
    /// shorter than `count`.
    PutIntoLibraryBeneathTop { what: Selector, count: Value },
    /// Search `who`'s library for a card matching `filter` and move to `to`.
    Search { who: PlayerRef, filter: SelectionRequirement, to: ZoneDest },
    /// "Search your library for any number of cards matching `filter`, put
    /// them into `to`, then shuffle" (Ugin, Eye of the Storms). The picker
    /// keeps choosing until they decline; every moved card is visible to
    /// `Selector::ExiledThisResolution` / `Selector::LastMoved` for a chained
    /// rider. CR 701.19c — the library is shuffled even on an empty pick.
    SearchAnyNumber { who: PlayerRef, filter: SelectionRequirement, to: ZoneDest },
    /// Transmute Artifact — search your library for an artifact card; if its
    /// mana value is at most the sacrificed artifact's it enters, otherwise
    /// you may pay the difference in generic mana to bring it in, and it goes
    /// to the graveyard if you don't. Reads the cast-cost sacrifice's mana
    /// value (`sacrificed_mana_value`).
    TransmuteArtifact,
    /// Search several of `who`'s zones at once for a card matching `filter`
    /// and move it to `to` — "search your graveyard, hand, and/or library"
    /// (Dark Supplicant, Delivery Moogle). Candidates pool every listed zone;
    /// the pick is taken from whichever zone holds it. Listing
    /// [`Zone::Library`] makes it a real library search (shuffle, search
    /// taxes/locks, `PlayerSearchedLibrary`); omitting it skips all of that.
    SearchZones {
        who: PlayerRef,
        zones: Vec<crate::card::Zone>,
        filter: SelectionRequirement,
        to: ZoneDest,
    },
    /// "Search your library for a land card of each basic land type, put those
    /// cards onto the battlefield, then shuffle" (Gaea's Balance). One card per
    /// basic type, each chosen from the cards carrying that type; a type with
    /// no match is simply skipped.
    SearchEachBasicLandType { who: PlayerRef, tapped: bool },
    /// CR 614 — "Until end of turn, spells and abilities you control that would
    /// add colored mana instead add that much `color` mana; you may spend that
    /// colour as though it were mana of any color" (False Dawn).
    ColoredManaBecomesThisTurn { who: PlayerRef, color: Color },
    /// CR 105 — the target instant/sorcery spell on the stack becomes a single
    /// colour of the controller's choice (Vodalian Mystic).
    SpellBecomesChosenColor { what: Selector },
    /// "Any player other than the caster may pay that spell's mana cost; if a
    /// player does, counter it" (Ice Cave). Walks the other seats in turn
    /// order and takes the first willing payer.
    OtherPlayerMayPayToCounter { what: Selector },
    /// "Search your library for up to `count` cards matching `filter` and put
    /// them into `to`." Resolves as a chain of single `Search` picks (each
    /// reuses the `SearchPending` suspend), shrinking `count` per pick.
    /// AutoDecider declines (finds none). Nylea's Intervention (lands → hand),
    /// Deathbellow War Cry (Minotaurs → battlefield). The "different names"
    /// rider is not enforced.
    SearchUpToN { who: PlayerRef, filter: SelectionRequirement, to: ZoneDest, count: Value },
    /// CR 701.19a — `picker` searches `who`'s library: the pick decision
    /// routes to `picker`'s seat, not the library's owner (Hide // Seek's
    /// "search target opponent's library ... exile that card").
    SearchPickedBy { who: PlayerRef, picker: PlayerRef, filter: SelectionRequirement, to: ZoneDest },
    /// CR 701.19 — "Search your library for any number of [`filter`] cards,
    /// exile them, then create that many tokens. Then shuffle." (Myr
    /// Incubator.) The auto-picker takes every match.
    SearchExileThenTokensPerCard {
        filter: SelectionRequirement,
        definition: crate::card::TokenDefinition,
    },
    /// CR 701.52 — `who` seeks `count` cards matching `filter`: the engine
    /// randomly chooses among the matching cards in their library (no
    /// player choice) and moves each to `to`, then no shuffle is needed
    /// (the picks are already random). Hidden-information mechanic from
    /// Tarkir: Dragonstorm (Roost Seek, Nesting Instinct, Divining Dive).
    Seek { who: PlayerRef, filter: SelectionRequirement, count: Value, to: ZoneDest },
    /// Return `count` cards matching `filter` at random from `who`'s graveyard
    /// to their hand ("return a[n] [filter] card at random from your graveyard
    /// to your hand" — Charmbreaker Devils). No player choice; stops early if
    /// the graveyard runs out of matches.
    ReturnRandomFromGraveyard { who: PlayerRef, filter: SelectionRequirement, count: Value },
    /// Pick a card at random in `who`'s graveyard; a creature card goes onto
    /// the battlefield under their control, anything else goes to `miss`.
    /// Deadbridge Chant (`miss` = hand), Search for Survivors (`miss` = exile).
    RandomGraveyardCardToBattlefieldElse { who: PlayerRef, miss: ZoneDest },
    /// Second Sunrise — each player returns to the battlefield all artifact,
    /// creature, enchantment, and land cards in their graveyard that were put
    /// there from the battlefield this turn.
    SecondSunrise,
    /// Shuffle `who`'s graveyard into their library.
    ShuffleGraveyardIntoLibrary { who: PlayerRef },
    /// Shuffle every card matching `filter` from `who`'s graveyard into their
    /// library (Repopulate) — the payoff-free sibling of
    /// `ShuffleFilteredGraveyardIntoLibraryGainLife`.
    ShuffleFilteredGraveyardIntoLibrary { who: PlayerRef, filter: SelectionRequirement },
    /// Shuffle every card matching `filter` from `who`'s graveyard into their
    /// library, then that player gains life equal to the number shuffled this
    /// way (Elixir — "shuffle all nonland cards … gain life equal to the number
    /// of cards shuffled").
    ShuffleFilteredGraveyardIntoLibraryGainLife {
        who: PlayerRef,
        filter: SelectionRequirement,
    },
    /// Shuffle `who`'s hand and graveyard into their library (Day's
    /// Undoing, Timetwister).
    ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef },
    /// Sway of the Stars — `who` shuffles their hand, graveyard, and every
    /// permanent they own into their library.
    ShuffleEverythingOwnedIntoLibrary { who: PlayerRef },
    /// Each resolved player shuffles their hand into their library, then
    /// draws that many cards (Molten Psyche, Winds of Change).
    ShuffleHandsDrawSame { who: PlayerRef },
    /// Shuffle `who`'s library (CR 103.2c). Mind's Desire's pre-exile shuffle.
    ShuffleLibrary { who: PlayerRef },
    /// CR 609.4b — "you may spend mana as though it were mana of any type"
    /// for the rest of the turn (North Star). The turn-scoped, single-seat
    /// twin of `StaticEffect::PlayersMaySpendManaAsAnyColor`.
    MaySpendManaAsAnyColorThisTurn { who: PlayerRef },
    /// Spellskite — change the primary target of the selected stack spell
    /// to this permanent, if it's a legal target for that spell (CR 115.7).
    RedirectSpellTargetToSelf { what: Selector },
    /// Gideon's Sacrifice — "All damage that would be dealt this turn to you
    /// and permanents you control is dealt to the chosen permanent instead."
    /// Registers a `(controller, chosen)` entry in `damage_redirect_this_turn`
    /// (CR 614.9), consulted by `damage_redirect_target`.
    RedirectYourDamageToChosen { what: Selector },
    /// Turn the Tables — "All combat damage that would be dealt to you this
    /// turn is dealt to `what` instead." The combat-only, player-only sibling
    /// of `RedirectYourDamageToChosen`; registers a `(controller, what)` entry
    /// in `combat_damage_redirect_this_turn`.
    RedirectYourCombatDamageToTarget { what: Selector },
    /// Shriveling Rot mode 1 — "until end of turn, whenever a creature is dealt
    /// damage, destroy it". The lethal-damage SBA treats *any* marked damage as
    /// lethal for the rest of the turn (indestructible still survives).
    DamagedCreaturesDieThisTurn,
    /// Shriveling Rot mode 2 — "until end of turn, whenever a creature dies,
    /// that creature's controller loses life equal to its toughness".
    CreatureDeathsDrainToughnessThisTurn,
    /// Deliver Unto Evil — up to `max_targets` target cards in your graveyard
    /// (slots `0..max_targets`, filter `filter`). On resolution: if you control
    /// a Bolas planeswalker, return them all to your hand; otherwise an
    /// opponent chooses two of them to leave in your graveyard and the rest go
    /// to your hand.
    DeliverUntoEvil { max_targets: u8, filter: SelectionRequirement },
    /// Nicol Bolas, Dragon-God's +1 rider — "Each opponent exiles a card from
    /// their hand or a permanent they control." Each opponent chooses one
    /// object among their hand cards and permanents to exile.
    EachOpponentExilesHandCardOrPermanent,
    /// CR 701.51 — "Open an Attraction": move the top card of your Attraction
    /// deck onto the battlefield under your control, face up. No-op when the
    /// Attraction deck is empty.
    OpenAnAttraction,
    /// "Each opponent exiles a card from their hand and may play that card for
    /// as long as it remains exiled. Each spell cast this way costs
    /// `surcharge` more" (Lightstall Inquisitor). The permission is stamped on
    /// the exiled card with the card's own cost plus the surcharge as the
    /// alternative cast cost; an `{X}` card stamps X = 0.
    EachOpponentExilesHandCardMayPlay { surcharge: u32 },
    /// "Each opponent chooses a creature they control and exiles it" — the
    /// exiles are stamped `exiled_with = source`, so a later
    /// `Selector::CardExiledWithSource` can cash them back in (Sothera, the
    /// Supervoid). Auto-picks the weakest creature for a non-UI seat.
    EachOpponentExilesOwnCreature,
    /// Nicol Bolas, Dragon-God's −8 — "Each opponent who doesn't control a
    /// legendary creature or planeswalker loses the game" (CR 104.3a).
    EachOpponentWithoutLegendaryLoses,
    /// Finale of Promise — slot 0 is up to one target instant card, slot 1 up
    /// to one target sorcery card, each in your graveyard with mana value X or
    /// less. Free-cast each (exiled on resolve); if X ≥ 10, copy each twice
    /// with new targets.
    FinaleOfPromise,
    /// Feather, the Redeemed — mark the selected spell on the stack so that,
    /// as it resolves, it's exiled instead of going to the graveyard and
    /// returns to its controller's hand at the next end step. Applied by
    /// Feather's cast trigger to `Selector::TriggerSource`.
    MarkExileReturnOnResolve { what: Selector },
    /// Niv-Mizzet Reborn — reveal the top ten cards of your library; for each
    /// of the ten guild color pairs, take one revealed card whose printed
    /// colors are exactly that pair into your hand; put the rest on the bottom
    /// of your library in a random order. (When a pair has several matches the
    /// first revealed is taken.)
    NivMizzetReveal,
    /// Gifts Ungiven — search up to `count` library cards with different
    /// names and reveal them; the targeted opponent chooses `opponent_picks`
    /// of them, which go to `chosen_to`; the rest go to `rest_to`; shuffle.
    SearchSplitOpponentChooses {
        opponent: Selector,
        count: u32,
        opponent_picks: u32,
        chosen_to: ZoneDest,
        rest_to: ZoneDest,
    },
    /// Move the resolving spell (`ctx.source`) from the stack into its
    /// owner's library, then shuffle. The Beacon cycle's "Shuffle this card
    /// into its owner's library" recursion rider (Beacon of Immortality,
    /// Beacon of Destruction). Runs at resolution before the spell would go
    /// to the graveyard, so the card never lands in the graveyard.
    ShuffleSelfIntoLibrary,
    /// "Return [this spell] to its owner's hand" as part of its own
    /// resolution (Journey to the Oracle's discard rider). Flags the
    /// post-resolution routing the same way `ShuffleSelfIntoLibrary` does.
    ReturnResolvingSpellToHand,
    /// "Distribute `total` counters of `kind` among the permanents created
    /// earlier in THIS resolution" (`GameState.last_created_tokens`) — the
    /// resolution-time sibling of the cast-time `DistributeCounters`
    /// (fresh tokens can't be cast-time targets, CR 601.2d). A UI
    /// controller picks each token's share via `ChooseAmount` (the last
    /// token takes the remainder); non-UI seats split as evenly as
    /// possible. Fractal Bloom.
    DistributeCountersAmongLastCreated {
        total: Value,
        kind: crate::card::CounterType,
    },
    /// "Put this card into its owner's library `from_top` cards from the
    /// top" for the RESOLVING spell (Approach of the Second Sun's
    /// seventh-from-the-top). Sets a transient consumed by the
    /// post-resolution routing instead of the graveyard trip; `from_top`
    /// is clamped to the library size.
    PutResolvingSpellInLibraryFromTop(u32),
    /// "Exile this spell, then put it onto the battlefield transformed under
    /// its owner's control" for the RESOLVING spell — the front face isn't a
    /// permanent, so the card is claimed by the post-resolution routing rather
    /// than by an effect reaching for a battlefield object (Esper Origins).
    /// `counter` is placed on the permanent as it arrives.
    PutResolvingSpellOnBattlefieldTransformed {
        #[serde(default)]
        counter: Option<crate::card::CounterType>,
    },
    /// Revel in Silence: each resolved player can't cast spells or activate
    /// loyalty abilities for the rest of the turn
    /// (`Player.silenced_this_turn`).
    /// CR 614 — "if target land is tapped for mana, it produces colorless
    /// mana instead of its usual type", indefinitely (Quarum Trench Gnomes).
    ReplaceTargetLandManaWithColorless { what: Selector },
    /// CR 615 — "if a spell or ability that targets this creature would cause
    /// a source to deal damage to it this turn, prevent that damage"
    /// (Silhouette). Turn-scoped; read at the damage funnel against the
    /// resolution's own target list.
    PreventTargetingDamageThisTurn { what: Selector },
    /// "Choose a card name. Target opponent reveals `count` cards at random
    /// from their hand, then discards all cards with that name revealed this
    /// way." Nebuchadnezzar.
    NameCardRevealRandomDiscardNamed { who: PlayerRef, count: Value },
    SilencePlayersThisTurn { who: PlayerRef },
    /// "Exile [this spell]" as part of its own resolution (Revel in
    /// Silence). Same flag pattern as `ShuffleSelfIntoLibrary`.
    ExileResolvingSpell,

    // ── Mana ─────────────────────────────────────────────────────────────────
    AddMana { who: PlayerRef, pool: ManaPayload },
    /// "Add mana equal to [permanent]'s mana cost" (Elemental Resonance).
    /// Reads the resolved permanent's printed cost pip-by-pip: colored →
    /// that color, generic/{C} → colorless, hybrid → its first color, X →
    /// skipped. Mana goes to the resolving controller's pool.
    AddManaEqualToPermanentCost { permanent: Selector },
    /// CR 500.4 exception — add these fixed color pips to the resolving
    /// player's pool and mark them "you don't lose this mana as steps and
    /// phases end" (Savage Ventmaw's attack trigger). The mana survives every
    /// step/phase empty this turn and clears at cleanup.
    AddManaKeptThisTurn { who: PlayerRef, colors: Vec<Color> },
    /// Like `AddManaKeptThisTurn` but adds `amount` mana of a single `color`
    /// (Neheb, Dreadhorde Champion — "add that much {R}", where the amount is
    /// the number of cards discarded this resolution).
    AddManaKeptThisTurnCount { who: PlayerRef, color: Color, amount: Value },

    // ── Permanent mutations ──────────────────────────────────────────────────
    Destroy { what: Selector },
    /// CR 701.15g — "Destroy ... It can't be regenerated." Behaves like
    /// `Destroy` but bypasses regeneration shields (Terminate, Putrefy,
    /// Day of Judgment, Vindicate, ...). Indestructible and Shield-counter
    /// replacements still apply — only regeneration is denied.
    DestroyNoRegen { what: Selector },
    /// "Destroy each nonland permanent with mana value equal to `value`" —
    /// resolves `value` at resolution (typically `CountersOn { This, Charge }`)
    /// and destroys every nonland permanent whose mana value matches. Ratchet
    /// Bomb, Engineered Explosives, Blast Zone.
    DestroyEachNonlandWithManaValue { value: Value },
    /// Creature-scoped sibling of `DestroyEachNonlandWithManaValue`: destroys
    /// every *creature* whose mana value matches `value` (Sanguine Praetor's
    /// "destroy each creature with the same mana value as the sacrificed
    /// creature" via `Value::SacrificedManaValue`).
    DestroyEachCreatureWithManaValue { value: Value },
    /// The filtered form of `DestroyEachNonlandWithManaValue`: destroys every
    /// permanent matching `filter` whose mana value equals `value` (Powder
    /// Keg's "each artifact and creature").
    DestroyEachMatchingWithManaValue { filter: SelectionRequirement, value: Value },
    /// "Destroy `what` and all other permanents with the same name" (Wake of
    /// Destruction).
    DestroyAllSharingNameWith { what: Selector },
    /// "Choose a number between 0 and `max`. Destroy all creatures with power
    /// greater than or equal to the chosen number." The controller picks the
    /// number at resolution (`Decision::ChooseAmount`); a bot/AutoDecider picks
    /// 0 (destroy everything). Expel the Interlopers.
    ChooseNumberDestroyByPower { max: u32 },
    /// CR 701.15 — add a regeneration shield to each resolved permanent.
    /// The shield is a one-shot replacement that fires the next time the
    /// permanent would be destroyed this turn (tap + remove from combat +
    /// heal damage instead of dying). Powers "{cost}: Regenerate this
    /// creature" activated abilities (Drudge Skeletons, River Boa, Korlash).
    Regenerate { what: Selector },
    /// CR 701.15g — "it can't be regenerated this turn": existing shields stop
    /// applying and new ones do nothing for the rest of the turn. Rage of
    /// Purphoros, the Terror-style removal riders.
    CantBeRegeneratedThisTurn { what: Selector },
    /// "If [each resolved permanent] would die this turn, exile it instead."
    /// Installs an until-end-of-turn death replacement (CR 614, same shape
    /// as a finality counter) on every permanent the selector resolves to.
    /// Because the redirect lasts the whole turn, it catches deaths from
    /// later combat / removal too, not just the spell's own damage. Used by
    /// Wilt in the Heat (paired with a `DealDamage`).
    ExileIfWouldDieThisTurn { what: Selector },
    /// "Target instant/sorcery card in your graveyard gains flashback until
    /// end of turn; its flashback cost equals its mana cost." Installs an
    /// until-end-of-turn `granted_flashback_eot` (= the card's own mana
    /// cost) on each resolved graveyard card, making it castable via the
    /// normal flashback path (pay the cost, exile on resolve). Used by the
    /// SOS "Flashback" instant.
    GrantFlashbackThisTurn { what: Selector },
    /// "Target creature card in your graveyard gains embalm until end of turn.
    /// The embalm cost is equal to its mana cost." (CR 702.88 — Cursecloth
    /// Wrappings.) Pushes a real embalm activation onto the card's
    /// `granted_activated_eot`, so it activates through the normal
    /// graveyard-ability path and is cleared at cleanup.
    GrantEmbalmThisTurn { what: Selector },
    /// "Target instant/sorcery card in your graveyard gains harmonize until
    /// end of turn; its harmonize cost equals its mana cost" (Songcrafter
    /// Mage). Stamps an until-end-of-turn `granted_harmonize_eot` (= the
    /// card's own mana cost), castable via the normal Harmonize path (CR
    /// 702.180 — pay the cost, optionally tap a creature to reduce it, exile
    /// on resolve). Cleared at cleanup.
    GrantHarmonizeThisTurn { what: Selector },
    /// "[Each resolved card] gains miracle `cost` until end of turn." Stamps
    /// an until-end-of-turn `may_play_until` permission **plus** a
    /// `granted_alt_cast_cost_eot` of `cost`, so the controller may cast the
    /// card this turn by paying `cost` (rather than its full mana cost or
    /// for free). Used by Lorehold, the Historian's "instant and sorcery
    /// cards in your hand have miracle {2}" grant.
    GrantMiracle { what: Selector, cost: crate::mana::ManaCost },
    Exile   { what: Selector },
    /// The "Enduring" cycle (Bloomburrow): "When this dies, if it was a
    /// creature, return it to the battlefield. It's an enchantment." Returns
    /// the source from its owner's graveyard to the battlefield under their
    /// control, then strips the Creature card type from the returned object
    /// so it comes back as a noncreature enchantment (and the gate self-
    /// limits — a noncreature can't satisfy "if it was a creature", so it
    /// won't loop). No-op if the source isn't a creature card in a graveyard.
    ReturnSelfAsEnchantment,
    /// "When this creature dies, return it to the battlefield tapped and with
    /// `amount` `kind` counters under its owner's control." Returns the source
    /// from its owner's graveyard (Unstoppable Slasher — two stun counters).
    /// No-op if the source isn't in a graveyard.
    ReturnSelfTappedWithCounters { kind: crate::card::CounterType, amount: u32 },
    /// "Return it to the battlefield tapped under its owner's control." The
    /// plain, counter-less sibling of `ReturnSelfTappedWithCounters` — used by
    /// a granted "when this dies, return it tapped" rider (Fake Your Own Death).
    /// No-op if the source isn't in a graveyard.
    ReturnSelfTapped,
    /// "Return it to the battlefield under its owner's control" (untapped) —
    /// the untapped sibling of `ReturnSelfTapped` (Presumed Dead's granted
    /// die-trigger). No-op if the source isn't in a graveyard.
    ReturnSelf,
    /// CR 702.171 — "Exile the source and up to one creature that saddled it
    /// this turn (`CardInstance.saddled_by`), then return those cards to the
    /// battlefield under their owners' control." A same-resolution flicker
    /// (Fortune, Loyal Steed). Bots/tests take one saddler; the source is
    /// always included. No-op for anything not on the battlefield.
    ExileAndReturnSelfWithSaddler,
    /// Bronzehide Lion — "when this creature dies, return it to the
    /// battlefield [as its `back_face` Aura] attached to a creature you
    /// control" (auto-pick: greatest power). No-op if the source isn't in a
    /// graveyard, has no back face, or its owner controls no creature.
    ReturnSelfTransformedAttached,
    /// "Return this card to the battlefield attached to [slot-0 target]"
    /// (Gift of Immortality's end-step re-attach). No-op if the source isn't
    /// in a graveyard or the target isn't a battlefield permanent.
    ReturnSelfAttachedToTarget,
    /// "Return this card from your graveyard to the battlefield attached to
    /// that creature" — the trigger-source sibling of
    /// [`Effect::ReturnSelfAttachedToTarget`] (the Scourge Dragon Auras).
    ReturnSelfAttachedToTrigger,
    /// "Its controller chooses a creature they don't control. Return this card
    /// from its owner's graveyard to the battlefield attached to that
    /// creature" (Necrotic Plague). `chooser` picks; the pool is every creature
    /// that player doesn't control. No-op if the source isn't in a graveyard or
    /// the pool is empty.
    ReturnSelfAttachedToChoiceOf { chooser: PlayerRef },
    /// "Return the top creature card of `who`'s graveyard to the battlefield."
    /// Top = most recently put into the graveyard (Mistmoon Griffin). No-op if
    /// the graveyard holds no creature card.
    ReturnTopCreatureFromGraveyard { who: PlayerRef },
    /// CR 712 — Transform the targeted double-faced permanent(s): swap each to
    /// its other face in place (same object, keeping counters / tapped state /
    /// attachments per CR 712.9). Toggles front↔back; a `Transforms` event
    /// fires per permanent so "when this transforms" triggers can react.
    Transform { what: Selector },
    /// CR 709.5 — "unlock a locked door of up to one target Room." Unlocks one
    /// still-locked door of each Room the selector resolves to (the left door
    /// first, else the right), firing that door's unlock triggers and the
    /// Eerie `RoomFullyUnlocked` event. No-op for fully-unlocked Rooms.
    /// Ghostly Keybearer's combat-damage trigger.
    UnlockRoomDoor { what: Selector },
    /// CR 709.5c — "Lock or unlock a door of target Room you control" (Marina
    /// Vendrell). Unlocks a still-locked door when there is one; otherwise
    /// re-locks the right door so it can be unlocked (and re-triggered) later.
    LockOrUnlockRoomDoor { what: Selector },
    /// CR 702.93 — mark each matching permanent as renowned (sets
    /// `CardInstance.renowned`). Paired with the Renown counter add so the
    /// once-only gate keys off the real flag, not a counter heuristic.
    BecomeRenowned { what: Selector },
    /// CR 710.2 — Flip the matching flip-card permanent(s) to their flipped
    /// (bottom) face in place (same object, keeping counters / tapped state /
    /// attachments). One-way; no-op on a permanent already flipped or without a
    /// flip face. Emits `Flipped` so "when this flips" triggers can react.
    Flip { what: Selector },
    /// CR 701.37 — Meld. If the source's controller both owns and controls
    /// the source and a permanent named `partner`, exile both, then put the
    /// melded card (resolved from the registry by `into`) onto the
    /// battlefield under their control. The two component cards ride in
    /// `CardInstance.meld_parts`, so the melded permanent leaves the
    /// battlefield as both cards (CR 712.16/712.17). No-op otherwise
    /// (701.37b).
    Meld { partner: String, into: String },
    /// "[Filter] spells you cast this turn cost {amount} less to cast" —
    /// turn-scoped generic cost reduction (Urza, Planeswalker's +2).
    /// Pushed onto `Player.turn_spell_discounts`, consulted by
    /// `cost_reduction_for_spell`, cleared at cleanup (CR 514.2).
    SpellsCostLessThisTurn { filter: SelectionRequirement, amount: u32 },
    /// "Face-down spells you cast this turn cost {amount} less to cast"
    /// (Goblin Maskmaker). Bumps `Player.face_down_discount_this_turn`, read
    /// by `face_down_cast_cost` and cleared at cleanup (CR 514.2).
    FaceDownSpellsCostLessThisTurn { amount: u32 },
    /// "That spell gains [keywords]" — grants keywords to a spell on the
    /// stack for as long as it's there (Judith, Carnage Connoisseur).
    /// Recorded in `GameState.spell_keyword_grants`.
    GrantKeywordsToSpell { what: Selector, keywords: Vec<Keyword> },
    /// "Exile target [permanent], then search its owner's graveyard, hand,
    /// and library for any number of cards with the same name as that
    /// [permanent] and exile them. Then that player shuffles." Crumble to
    /// Dust, Spreading Plague-style name sweeps. `what` resolves the
    /// anchor permanent (slot 0); its printed name keys the sweep.
    ExileSameNameAsTarget { what: Selector },
    /// Exile target card(s) and stamp each with `exiled_with = source`, the
    /// permanent association read by counting effects like
    /// `Value::DistinctCardTypesExiledWith`. Keen-Eyed Curator's
    /// "{1}: Exile target card from a graveyard."
    ExileTaggedWithSource { what: Selector },
    /// CR 702.76 — Hideaway N. Look at the top `count` cards of the controller's
    /// library, exile one face down stamped `exiled_with = source`, then put the
    /// rest on the bottom in a random order. The hidden card is later played via
    /// `CastWithoutPayingImmediate { what: Selector::CardExiledWithSource }`.
    /// The controller's pick is auto-resolved to the highest-mana-value card.
    Hideaway { count: Value },
    /// "Exile any number of target cards from graveyards." The controller
    /// picks a subset (via `Decision::ChooseCards`) of every graveyard card
    /// matching `filter`; chosen cards move to exile. AutoDecider exiles
    /// nothing (the conservative "up to" default); the bot exiles opponents'
    /// cards. Devious Cover-Up's graveyard-strip rider.
    ExileAnyNumberFromGraveyards { filter: crate::card::SelectionRequirement },
    /// "You may exile one or more cards matching `filter` from *your*
    /// graveyard. When you do, `then` runs" (CR 603.7 reflexive). The exiled
    /// cards are placed on `Selector::LastMoved`, so `then` can read their
    /// count via `Value::CountOf(Selector::LastMoved)` — Specter of Mortality's
    /// team `-X/-X`. Declining (or an empty graveyard) skips `then`.
    MayExileFromYourGraveyard {
        filter: crate::card::SelectionRequirement,
        then: Box<Effect>,
    },
    /// "Exile all cards from all graveyards." (Rest in Peace's ETB — a
    /// non-optional graveyard wipe across every player.) `filter` restricts
    /// the wipe to matching cards (Sanctifier en-Vec's black/red sweep).
    ExileAllGraveyards {
        #[serde(default)]
        filter: Option<crate::card::SelectionRequirement>,
        /// Skip the controller's own graveyard (Phyrexian Scriptures III —
        /// "exile all cards from all opponents' graveyards").
        #[serde(default)]
        opponents_only: bool,
    },
    /// Living End / Living Death: each player exiles all creature cards
    /// from their graveyard, sacrifices all creatures they control, then
    /// puts the exiled cards onto the battlefield under their owner's
    /// control.
    LivingEnd,
    /// "Exile that player's graveyard" — graveyard hate scoped to a single
    /// player (Go Blank). `who` resolves to the affected player; `filter`
    /// narrows it to a subset (Tombfire's "all cards with flashback").
    ExilePlayerGraveyard {
        who: PlayerRef,
        #[serde(default)]
        filter: Option<crate::card::SelectionRequirement>,
    },
    /// Exile all cards from `who`'s hand (each resolved player). Ashiok's
    /// −10 pairs this with `ExilePlayerGraveyard`.
    ExileHand { who: PlayerRef },
    /// Exile the controller's whole hand face down, stamped
    /// `exiled_with = source` (Bottled Cloister, Moonring Mirror).
    ExileHandLinked { who: PlayerRef },
    /// Return every card the controller owns stamped `exiled_with = source`
    /// from exile to their hand (Bottled Cloister's upkeep half).
    ReturnLinkedExilesToHand { who: PlayerRef },
    /// "Look at the top `count` cards of `who`'s library. Exile any number of
    /// them, then put the rest back in any order." Dimir Machinations. The
    /// headless pick exiles the priciest nonland cards.
    LookExileAnyNumberRestBack { who: Selector, count: Value },
    /// "Exile target creature card from a graveyard. This permanent becomes a
    /// copy of it, except it keeps the ability that did this." Dimir
    /// Doppelganger — the granting ability is re-appended after the copy.
    ExileFromGraveyardBecomeCopy { what: Selector },
    /// "Each player returns all cards with the same name as `what` from their
    /// graveyard to the battlefield" (Bloodbond March). `what` resolves the
    /// name-bearing object (the triggering creature spell).
    ReturnSameNameFromAllGraveyards { what: Selector },
    /// "Put the top card of your library on the bottom of your library."
    /// Crown of Convergence's {G}{W}.
    PutTopOnBottom { who: Selector },
    /// Cloudstone Curio — "you may return another permanent you control that
    /// shares a permanent type with `with` to its owner's hand." `with` is the
    /// permanent that just entered.
    MayReturnSharingPermanentType { with: Selector },
    /// Mindleech Mass — look at the resolved player's hand and cast one of the
    /// cards from it without paying its mana cost.
    LookAtHandCastFree { who: Selector },
    /// Reroute — change the target of the resolved activated ability on the
    /// stack to a new legal one (CR 115.7b). Single-target abilities only.
    ChangeTargetOfAbility { what: Selector },
    /// Aetherplasm — return the source to its owner's hand, then put a
    /// creature card from the controller's hand onto the battlefield blocking
    /// the creature the source was blocking (CR 509.1 — a late blocker).
    ReturnSelfDeployBlocker,
    /// Ink-Treader Nephilim — copy the resolved spell once for each other
    /// creature it could legally target, each copy aimed at a different one.
    CopySpellForEachOtherLegalCreature { what: Selector },
    /// Mimeofacture — search the resolved permanent's controller's library for
    /// a card with that permanent's name and put it onto the battlefield under
    /// *your* control; then that player shuffles.
    SearchOpponentLibraryForSameName { what: Selector },
    /// "Search `who`'s library for a card with the same name as `what`, put it
    /// onto the battlefield under `who`'s control, then shuffle" (Verdant
    /// Succession). Unlike `SearchOpponentLibraryForSameName`, `what` may be a
    /// card that has already left the battlefield (a death trigger's subject).
    SearchSameNameToBattlefield { who: PlayerRef, what: Selector },
    /// "For each card exiled this way, search that player's library for all
    /// cards with the same name and exile them. Then that player shuffles"
    /// (Haunting Echoes). Names are read off the cards exiled earlier in this
    /// resolution, so it pairs with a preceding graveyard exile.
    ExileLibraryCardsNamedLikeExiledThisResolution { who: PlayerRef },
    /// Development — "create a 3/1 red Elemental token unless any opponent has
    /// you draw a card", `times` times. Each iteration asks an opponent.
    TokenUnlessOpponentLetsYouDraw { token: TokenDefinition, times: u32 },
    /// Sunforger — search the controller's library for a card matching
    /// `filter`, cast it without paying its mana cost, then shuffle.
    /// With `include_hand`, the hand is searched too and only a library
    /// find shuffles (Aether Searcher).
    SearchAndCastFree {
        filter: SelectionRequirement,
        #[serde(default)]
        include_hand: bool,
    },
    /// Tawnos's Coffin — exile [what] and every Aura attached to it, stamped
    /// `exiled_with = source`, keeping the creature's counters noted on the
    /// exiled object (CR 122.2 otherwise drops them).
    CoffinExile { what: Selector },
    /// The return half: put the noted creature back tapped with its noted
    /// counters and re-attach the Auras exiled with it. Fires off the Coffin's
    /// leaves-the-battlefield *and* becomes-untapped triggers.
    CoffinReturn,
    /// Flickerform — exile the source's host and every Aura attached to it,
    /// then at the next end step return the host and re-attach the Auras.
    FlickerHostWithAuras,
    /// The delayed half of `FlickerHostWithAuras`: return every card stamped
    /// `exiled_with = source` to the battlefield, re-attaching the Auras to
    /// `host`. A no-op if the host itself isn't in the pile any more.
    ReturnLinkedExilesToBattlefieldAttached { host: crate::card::CardId },
    /// Breath of Fury — sacrifice the enchanted creature, attach the source to
    /// another creature you control, and if that happened untap your creatures
    /// and append an extra combat phase after this one.
    SacrificeEnchantedForExtraCombat,
    /// Eye of the Storm — exile the triggering instant/sorcery, then its
    /// controller casts a free copy of every instant/sorcery exiled with the
    /// source.
    EyeOfTheStorm { what: Selector },
    /// Warp World — each player shuffles all permanents they own into their
    /// library, reveals that many cards, battlefields every artifact, creature
    /// and land, then every enchantment, and bottoms the rest.
    WarpWorld,
    /// "You choose a card matching `filter` from `who`'s graveyard or hand
    /// and exile it." A single cross-zone choice (Memory Leak). Auto-picks
    /// the highest-mana-value match (a `wants_ui` chooser is a follow-up).
    ExileChosenFromHandOrGraveyard { who: PlayerRef, filter: SelectionRequirement },
    /// "Target player reveals the top `reveal` cards of their library. You
    /// choose `pick` of those cards and put them into that player's graveyard.
    /// Put the rest on top in any order." Bamboozle; Balshan Beguiler at
    /// (2, 1). The chooser is the effect's controller (auto-picks the
    /// highest-mana-value cards).
    RevealTopChooseToGraveyard { who: PlayerRef, reveal: Value, pick: Value },
    /// "Gain control of target Aura that's attached to a permanent. Attach it
    /// to another permanent it can enchant." Aura Graft — the new host is
    /// chosen by the effect's controller (auto-picks their own cheapest legal
    /// permanent, else any legal one).
    GainControlAndReattachAura { what: Selector },
    /// "Mill a card. For each coloured mana symbol in the milled card's mana
    /// cost, add one mana of that colour." Charmed Pendant.
    MillAddManaForColoredSymbols { who: PlayerRef },
    /// "Exile `count` cards matching `filter` from `who`'s graveyard" — the
    /// mandatory, graveyard-only sibling of `ExileChosenFromHandOrGraveyard`
    /// (Decaying Soil's upkeep). Auto-picks the cheapest matches; fewer
    /// matches than `count` exiles what there is.
    ExileFromGraveyard { who: PlayerRef, count: Value, filter: SelectionRequirement },
    /// CR 701.38 — an option vote. Starting with the controller and proceeding
    /// in turn order, each player votes for one of `options`; how the tally is
    /// spent is `tally`'s business. Untargeted, so it ignores hexproof/shroud.
    Vote { options: Vec<VoteOption>, tally: VoteTally },
    /// CR 701.38 — "you choose how each player votes this turn" (Illusion of
    /// Choice). Sets `GameState.vote_controller_this_turn` to the resolved
    /// player; each vote this turn is answered by them on the voter's behalf.
    ControlVotesThisTurn { who: PlayerRef },
    /// CR 701.31 — "Will of the council." Starting with the controller, each
    /// player votes for one permanent matching `filter` (evaluated relative to
    /// the controller, so Council's Judgment's "a nonland permanent you don't
    /// control" reads `Nonland.and(ControlledByOpponent)`). Every permanent
    /// with the most votes (or tied for most) is exiled. Resolved inline via
    /// the per-seat decider; AutoDecider's first-legal pick makes the vote
    /// deterministic for bots/tests. No targeting (CR 701.31c), so it ignores
    /// hexproof/shroud.
    WillOfTheCouncilExile { filter: SelectionRequirement },
    /// The general form of [`Effect::WillOfTheCouncilExile`]: each player votes
    /// for one of the cards `candidates` resolves to (which may live in any
    /// zone), and every card tied for most votes is moved to `to` (Custodi
    /// Squire's graveyard ballot).
    WillOfTheCouncilOnCards { candidates: Selector, to: ZoneDest },
    /// "Put the bottom card of your library into your graveyard. If it's a
    /// creature card with power less than or equal to `max_power`, put it onto
    /// the battlefield" (Grenzo, Dungeon Warden).
    BottomCardToGraveyardThenDeploy { max_power: Value },
    /// "Starting with you, each player chooses one permanent matching each of
    /// `filters` from among those controlled by the player to their left.
    /// Destroy each permanent chosen this way." Grenzo's Rebuttal.
    EachPlayerDestroysChosenFromLeftNeighbor { filters: Vec<SelectionRequirement> },
    /// CR 603.6e — "Exile [what] until [this] leaves the battlefield."
    /// Moves the resolved card(s) to exile, linking each to the source
    /// permanent (the ability's source). When that source leaves play the
    /// engine returns the exiled card(s) to `return_to`. Powers Banisher
    /// Priest / Fiend Hunter / Oblivion Ring (return to battlefield) and
    /// Brain Maggot / Tidehollow Sculler (return to hand).
    /// Exile each resolved permanent/card stamped `exiled_with = source`
    /// (recoverable via `Selector::CardExiledWithSource`) but with no
    /// return-when-the-source-leaves link — the effect that scheduled it owns
    /// the return (Legion's Initiative).
    ExileLinked { what: Selector },
    /// Search the City — the trigger source is a card its controller just
    /// played. If a card exiled with this source shares its name, return one
    /// to its owner's hand; when that empties the pile, sacrifice this source
    /// and its controller takes an extra turn.
    SearchTheCityReturn,
    ExileUntilSourceLeaves {
        what: Selector,
        return_to: crate::card::ExileReturnZone,
    },
    /// CR 603.6e — "Exile any number of other permanents you control matching
    /// `filter` until this leaves the battlefield" (Lumbering Battlement). The
    /// controller picks the subset (`Decision::ChooseCards`, min 0); each pick
    /// is linked to the source exactly like `ExileUntilSourceLeaves`.
    ExileAnyNumberUntilSourceLeaves { filter: SelectionRequirement },
    /// CR 725 — Palace Jailer's "exile [what] until an opponent becomes the
    /// monarch". Exiles the resolved permanent(s) with a `monarch_guard` set to
    /// the controller (who has just become the monarch); the card returns to the
    /// battlefield the moment the monarchy leaves that player, not when the
    /// source leaves play.
    ExileUntilOpponentMonarch { what: Selector },
    /// Mirror March — flip a coin until you lose a flip, then create that many
    /// token copies of `what`. The copies gain haste and are exiled at the
    /// beginning of the next end step.
    FlipUntilLossThenTokenCopies { what: Selector },
    /// CR 705.1 — "Flip a coin until you lose a flip", running `per_win` once
    /// for each flip won (Crazed Firecat).
    FlipUntilLoss { per_win: Box<Effect> },
    /// Amplifire — reveal from the top of your library until you reveal a
    /// creature card; until your next turn the source's base P/T become twice
    /// that card's. The reveal is bottomed in a random order.
    RevealUntilCreatureDoubleBasePt,
    /// Illusionist's Bracers — copy the activated ability that just triggered
    /// this. Mana abilities never reach the stack, so the topmost `activated`
    /// stack item from the source is always a legal copy target (CR 706.10).
    /// The copy may choose new targets in every declared slot; the original
    /// is offered first, so a conservative decider keeps it.
    CopyActivatedAbilityMayChooseTargets,
    /// Exile each resolved permanent, then return it to the battlefield under
    /// its owner's control at the beginning of the next end step, entering with
    /// an extra +1/+1 counter (creatures) or loyalty counter (planeswalkers).
    /// Registers a per-card `DelayedKind::NextEndStep` trigger. Semester's End.
    ExileReturnNextEndStep { what: Selector },
    /// Exile each resolved permanent, then return it to the battlefield under
    /// its **owner's** control at the beginning of the next end step, with no
    /// extra counter. The plain-flicker sibling of `ExileReturnNextEndStep`
    /// (Voidwalk, Voyager Staff — "return it under its owner's control").
    /// Exile each resolved permanent, returning it under its owner's control
    /// at the beginning of the controller's next upkeep (Kaya, Ghost
    /// Assassin's 0). The upkeep-timed sibling of
    /// `ExileReturnToOwnerNextEndStep`.
    ExileReturnAtYourNextUpkeep { what: Selector },
    ExileReturnToOwnerNextEndStep {
        what: Selector,
        /// The permanent returns tapped (Mystifying Maze).
        #[serde(default)]
        tapped: bool,
    },
    /// "You may exile this. If you do, return it to the battlefield under its
    /// owner's control at the beginning of your next upkeep. It gains haste."
    /// An optional self-flicker whose return is deferred to the controller's
    /// next upkeep (CR 603.4). Obzedat, Ghost Council's end-step ability.
    /// "You may exile this card. If you do, [body]." The source is moved from
    /// wherever it now sits (a death trigger sees it in the graveyard) into
    /// exile, and only then does `body` run — Academy Rector, Gamekeeper.
    MayExileSelfThen { body: Box<Effect> },
    /// Scrying Glass — "choose a number greater than 0 and a color; `who`
    /// reveals their hand; if they reveal exactly that many cards of that
    /// colour, draw a card." The guess is made by the effect's controller.
    GuessColorCountInHand { who: PlayerRef, max: u32 },
    /// "Attach target Aura card from a graveyard to this creature" — the Aura
    /// returns to the battlefield already attached (Iridescent Drake).
    AttachAuraFromGraveyardTo { aura: Selector, host: Selector },
    /// CR 701.3 — "Attach [this] to `host`" (The Aetherspark's +1). Moves the
    /// source's attachment without paying an equip cost; a `host` that resolves
    /// to nothing leaves it where it is.
    AttachSourceTo { host: Selector },
    /// "Until end of turn, whenever a player taps a `land` for mana, that
    /// player adds an additional `extra`" (Bubbling Muck). The turn-scoped
    /// sibling of `StaticEffect::ExtraManaOnLandTap`.
    ExtraManaOnLandTapThisTurn { land: crate::card::LandType, extra: crate::mana::Color },
    MayExileSelfReturnNextUpkeepHaste,
    /// "Return this card to the battlefield tapped under its owner's control
    /// at the beginning of their next upkeep" (Phytotitan). Fired from a
    /// dies-trigger; registers a `DelayedKind::YourNextUpkeep` return.
    ReturnSelfAtNextUpkeepTapped,
    /// CR 702.55 — Haunt. Exile the source card (the dying creature, or the
    /// resolving instant/sorcery) "haunting" a creature, then register a
    /// `DelayedKind::WhenHauntedCreatureDies` delayed trigger that runs `body`
    /// when that creature dies. The haunted creature is auto-picked (preferring
    /// an opponent's) — the controller's choice is a deferred UI follow-up.
    HauntCreature { body: Box<Effect> },
    Tap     { what: Selector },
    /// CR 702.171 — "Target Mount you control becomes saddled" (Guidelight
    /// Matrix). Sets `CardInstance.saddled` on each `what` and fires
    /// `GameEvent::MountSaddled` (no riders), so saddled-attack triggers see it.
    SetSaddled { what: Selector },
    /// "That player taps `amount` untapped permanents matching `filter` they
    /// control" (Tangle Wire). Auto-pick: lands, then other noncreatures,
    /// then creatures by ascending power. Taps as many as exist when short.
    PlayerTapsUntapped { who: PlayerRef, filter: SelectionRequirement, amount: Value },
    /// "Until end of turn, if you would put one or more +1/+1 counters on a
    /// creature you control, put that many plus one instead" (Prairie Dog).
    /// Bumps `Player.extra_plus_one_counters_this_turn`, a transient
    /// Hardened-Scales bonus cleared at cleanup.
    GrantExtraPlusOneCountersThisTurn { who: PlayerRef },
    /// Combine Guildmage — "this turn, each creature you control enters with an
    /// additional +1/+1 counter." Bumps `Player.extra_etb_p1p1_counters_this_turn`
    /// (stacking across activations); cleared at cleanup. Distinct from
    /// `GrantExtraPlusOneCountersThisTurn`, which amplifies counters *placed*.
    CreaturesEnterWithExtraCounterThisTurn { who: PlayerRef },
    /// CR 509.1h — "target unblocked attacking creature becomes blocked."
    /// Marks the attacker blocked with no blockers assigned, so (absent
    /// trample) it deals no combat damage. Curtain of Light.
    BecomeBlocked { what: Selector },
    /// Sweep (CR 207.2c ability word) — "Return any number of `filter` you
    /// control to their owner's hand." The controller picks the subset (min 0);
    /// the count is published as `Value::PermanentsReturnedThisEffect` for a
    /// follow-up step in the same resolution to scale off (Barrel Down
    /// Sokenzan, Sink into Takenuma).
    ReturnAnyNumberToHand { filter: SelectionRequirement },
    /// "Tap any number of untapped permanents matching `filter` you control;
    /// this source gets +`power`/+`toughness` until end of turn for each one
    /// tapped this way" (Orphans of the Wheat). The controller chooses which
    /// to tap (min 0); the pump scales with the count.
    TapAnyNumberThenPumpPerTapped {
        filter: SelectionRequirement,
        power: i32,
        toughness: i32,
    },
    /// "You may tap any number of untapped `filter` you control. If you do, put
    /// a `counter` counter on each of those" (Urge to Feed). The counter-payout
    /// sibling of `TapAnyNumberThenPumpPerTapped`.
    TapAnyNumberThenCounters { filter: SelectionRequirement, counter: crate::card::CounterType },
    /// Entrancing Lyre — tap `what` and lock it from untapping for as long as
    /// the source permanent stays tapped (`CardInstance.untap_locked_by`).
    TapAndUntapLock { what: Selector },
    /// Shipbreaker Kraken — tap `what` and lock it from untapping for as long
    /// as the source permanent stays on the battlefield
    /// (`CardInstance.untap_locked_while_present`).
    TapAndLockWhileSourcePresent { what: Selector },
    /// CR 702.158d — "choose a sector, then [body] each creature in it."
    /// The controller picks alpha/beta/gamma; `body` reads the picks via
    /// `Selector::CreaturesInChosenSector` (Space Beleren's −1 and −5).
    ChooseSector { body: Box<Effect> },
    /// CR 702.158d — creatures can be blocked this turn only by creatures in
    /// the same sector (Space Beleren's +1).
    SectorBlockLockThisTurn,
    /// "Tap each creature that was blocked by [what] this turn; those
    /// creatures don't untap during their controllers' next untap steps"
    /// (Triton Tactics), reading `CardInstance.blocked_attackers_this_turn`.
    TapBlockedByAndSkipUntap { what: Selector },
    /// CR 506.4 — remove every targeted creature from combat: an attacker is
    /// pulled from the attack (its blocks released), a blocker is unassigned.
    /// It stops being an attacking/blocking creature but stays on the
    /// battlefield (Labyrinth of Skophos, Falter-style effects).
    RemoveFromCombat { what: Selector },
    /// CR 508.1a — "that creature can't attack during its controller's next
    /// turn" (Wall of Dust). Arms `CardInstance.attack_ban`, which the
    /// controller's untap step promotes and that turn's cleanup clears.
    CantAttackNextTurn { what: Selector },
    /// CR 508.1a — "[creatures] can't attack this turn" (Festival). Arms the
    /// ban live for the current turn rather than the next one.
    CantAttackThisTurn { what: Selector },
    /// "Until end of turn, if you tap a land you control for mana, it produces
    /// [color] instead of any other type" (Deep Water). The turn-scoped,
    /// controller-scoped sibling of `StaticEffect::LandsProduceColorInstead`.
    YourLandsProduceColorThisTurn(crate::mana::Color),
    /// "Each player may discard up to `max` cards. This deals damage to each
    /// player equal to `max` minus the number they discarded" (Mind Bomb).
    EachPlayerMayDiscardUpToThenDamage { max: u32 },
    /// "Each player may discard a card. Each player who discarded a card this
    /// way may search their library for a basic land card, reveal it, put it
    /// into their hand, then shuffle" (Borderland Explorer). APNAP order; a
    /// seat that declines the discard is skipped.
    EachPlayerMayDiscardThenTutorBasic,
    /// CR 701.3c — "attach target Aura attached to a [type] to another
    /// permanent of that type" (Enchantment Alteration). Moves the *targeted*
    /// Aura rather than the source; the new host must satisfy the Aura's own
    /// enchant filter, else the move is illegal and nothing happens.
    ReattachTargetAura { aura: Selector, to: Selector },
    /// CR 702.26 — phase out every permanent the selector resolves to. The
    /// permanent moves to `GameState.phased_out` (treated as nonexistent) and
    /// phases back in during its controller's next untap step. Vodalian
    /// Illusionist. With `until_source_leaves` the permanents skip the
    /// untap-step phase-in and only return when the source leaves the
    /// battlefield (Out of Time); the source gains one time counter per
    /// permanent phased out this way.
    PhaseOut {
        what: Selector,
        #[serde(default)]
        until_source_leaves: bool,
    },
    /// Untap every permanent the selector resolves to. The optional
    /// `up_to` cap limits the count to "up to N" — used by Frantic
    /// Search ("untap up to three lands"), Cryptolith Rite-style
    /// abilities, etc. `None` means "untap all matching" (the
    /// pre-cap default behavior). When the selector resolves to more
    /// than `up_to` matches, the picker takes the first `up_to`
    /// in resolution order; auto-resolution favors highest-CMC lands
    /// for max mana refund.
    Untap   { what: Selector, #[serde(default)] up_to: Option<Value> },
    /// "Simultaneously untap all tapped [what] and tap all untapped [what]"
    /// (Breaking Wave). A sequenced Untap-then-Tap would re-tap everything it
    /// just untapped, so the swap is read once and applied in one pass.
    SwapTappedState { what: Selector },
    /// Give a temporary +P/+T bonus.
    PumpPT  { what: Selector, power: Value, toughness: Value, duration: Duration },
    /// Double the resolved creature's power `times` times for `duration`
    /// (CR — Exponential Growth: "double target creature's power X times").
    /// Computes the creature's current power and adds `power * (2^times - 1)`
    /// as a pump bonus, so power ends at `power * 2^times`. `times` ≤ 0 is a
    /// no-op. Reusable for any "double power N times" card.
    DoublePower { what: Selector, times: Value, duration: Duration },
    /// CR 701.10 — "double the number of `kind` counters on each [selector]."
    /// For every resolved permanent, add counters of `kind` equal to the number
    /// it currently has (so N → 2N). Honors counter-doubling replacements
    /// (Doubling Season) on the added counters. Kalonian Hydra's on-attack
    /// trigger over the controller's creatures.
    DoubleCountersOnEach { what: Selector, kind: crate::card::CounterType },
    /// CR 701.10 — "double the number of each kind of counter on the resolved
    /// permanent(s)." For every counter kind present, add that many more (N →
    /// 2N per kind). Honors counter-doubling replacements. Vorel of the Hull
    /// Clade, Gilder Bairn.
    DoubleAllCountersOn { what: Selector },
    /// Override the resolved permanent's base power and toughness via a
    /// layer-7b continuous effect. Unlike `PumpPT` (which adds to the
    /// existing P/T via direct bonus fields), `SetBasePT` installs a
    /// proper `Modification::SetPowerToughness(p, t)` continuous effect
    /// that participates in the layer system. Used by Strixhaven's
    /// **Square Up** ({U}{R}: "Until end of turn, target creature has
    /// base power and toughness 0/4") and any future "base P/T
    /// becomes" effect. Counters and +P/+T modifications still stack
    /// on top per CR 613.7f / 613.7c — so a +1/+1 counter on a Square-
    /// Upped creature makes it 1/5, not 1/1.
    SetBasePT { what: Selector, power: Value, toughness: Value, duration: Duration },
    /// CR 613.7d — switch each resolved permanent's power and toughness for
    /// `duration` (layer-7d, applied after all other P/T changes). Twisted
    /// Image, Wandering Fumarole's animated `{0}` ability.
    SwitchPT { what: Selector, duration: Duration },
    /// Animate each permanent picked by `what` into a creature for
    /// `duration` (the canonical "manland" effect — Celestial Colonnade,
    /// Creeping Tar Pit, Mutavault, …). Installs a stack of continuous
    /// effects: layer-4 `AddCardType(Creature)` + each creature subtype,
    /// layer-7b `SetPowerToughness`, and layer-6 keyword grants. The
    /// permanent keeps its other types (a land stays a land — it becomes a
    /// "land creature"). Typically targets `Selector::This` from an
    /// activated ability, but works on any resolved permanent.
    BecomeCreature {
        what: Selector,
        power: Value,
        toughness: Value,
        creature_types: Vec<crate::card::CreatureType>,
        keywords: Vec<Keyword>,
        duration: Duration,
    },
    /// CR 205.1b — "[what] becomes a P/T [types] creature", *replacing* its
    /// other card types (the Urza's Saga Opal / Veiled / Hidden enchantments
    /// stand up as creatures and stop being enchantments). Indefinite;
    /// `SetCardTypesTo` reverts it by a later layer-4 timestamp.
    BecomeCreatureLosingTypes {
        what: Selector,
        power: Value,
        toughness: Value,
        creature_types: Vec<crate::card::CreatureType>,
        keywords: Vec<Keyword>,
    },
    /// CR 205.1b layer-4 type replacement — "[what] becomes an enchantment"
    /// (Opal Acrolith's `{0}`, Hidden Stag's land trigger). Indefinite; the
    /// fresh timestamp beats any earlier `BecomeCreatureLosingTypes`.
    SetCardTypesTo { what: Selector, card_types: Vec<crate::card::CardType> },
    /// "[what] becomes an artifact creature for `duration`" — adds
    /// `CardType::Creature` (layer 4) only, keeping the permanent's printed
    /// power/toughness (correct for Vehicles, which carry P/T even when not a
    /// creature). Unlike `BecomeCreature` this never sets P/T. Guidelight
    /// Matrix's "target Vehicle you control becomes an artifact creature until
    /// end of turn".
    AnimateAsCreature { what: Selector, duration: Duration },
    /// CR 611.2c — grant `keyword` to every permanent matching `filter` for
    /// the rest of the turn. Unlike `GrantKeyword` over `EachPermanent` the
    /// affected set is *not* locked in at resolution, so permanents that
    /// arrive later are covered too (Falter — "creatures without flying can't
    /// block this turn"). `filter` is matched card-locally, so a keyword
    /// granted by another continuous effect isn't seen.
    GrantKeywordToMatchingThisTurn { filter: SelectionRequirement, keyword: Keyword },
    /// CR 613 layer 7b — "[what]'s base power becomes `power`" for `duration`,
    /// leaving base toughness intact (Belligerent Yearling: base power becomes
    /// equal to the entering Dinosaur's power until end of turn).
    SetBasePower { what: Selector, power: Value, duration: Duration },
    GrantKeyword { what: Selector, keyword: Keyword, duration: Duration },
    /// Grant several keywords at once to a single `what` (one target slot).
    /// "Target creature gains flying, double strike, and vigilance until end of
    /// turn" (Case of the Shattered Pact) — cleaner than a `Seq` of separate
    /// `GrantKeyword`s, which would each declare their own target.
    GrantKeywords { what: Selector, keywords: Vec<Keyword>, duration: Duration },
    /// Each permanent picked by `what` loses `keyword` for `duration`
    /// (CR 613.7 layer 6 — the removal outranks any earlier grant).
    /// Shadowspear's "lose hexproof and indestructible until end of turn";
    /// `Duration::Permanent` is the indefinite loss (Ageless Sentinels).
    LoseKeyword { what: Selector, keyword: Keyword, duration: Duration },
    /// CR 701.33 — abandon `this` scheme: turn it face down and put it on the
    /// bottom of its owner's scheme deck. The ongoing-scheme escape hatch
    /// ("abandon this scheme" — Perhaps You've Met My Cohort).
    AbandonThisScheme,
    /// CR 702.15 — "Target creature loses all landwalk abilities" (Hammerheim).
    /// Layer-6 removal of every landwalk variant at once, which
    /// `LoseKeyword`'s exact match can't express.
    LoseAllLandwalk { what: Selector, duration: Duration },
    /// Each permanent picked by `what` doesn't untap during its
    /// controller's next untap step (Vorinclex's land lock, Exert-style
    /// `skip_next_untap` flag).
    SkipNextUntap { what: Selector },
    /// CR 502.3 — the target player skips their next untap step (Yosei, the
    /// Morning Star). Adds one charge to `Player.skip_next_untap_step`.
    SkipPlayerUntapStep { player: PlayerRef },
    /// "Target player skips their next draw step" (Fatigue). Adds one charge to
    /// `Player.skip_next_draw_step`.
    SkipPlayerDrawStep { player: PlayerRef },
    /// CR 727 — "Restart the game." The current game ends with no winner and a
    /// fresh one begins from every card involved, with the effect's controller
    /// as starting player (727.1a). Cards exiled with the source are exempt
    /// from the reshuffle (727.5) and, when `deploy_source_exiles`, enter the
    /// battlefield under the controller's control — Karn Liberated's −14.
    RestartGame { deploy_source_exiles: bool },
    /// CR 305.1 — "Target player can't play lands this turn" (Turf Wound).
    /// Sets `Player.cant_play_lands_this_turn`, cleared at the turn boundary.
    PlayerCantPlayLandsThisTurn { player: PlayerRef },
    /// "Target player can't cast [filter] spells this turn" (Cease-Fire).
    /// Checked at the cast gate; cleared at cleanup.
    PlayerCantCastMatchingThisTurn { who: PlayerRef, filter: SelectionRequirement },
    /// "The next [filter] spell you cast this turn can't be countered"
    /// (Insist, Overmaster). Arms a one-shot grant consumed by the next
    /// matching cast.
    NextSpellCantBeCountered { filter: SelectionRequirement },
    /// "Lands `who` controls don't untap during their next untap step"
    /// (Bontu's Last Reckoning). Adds one charge to
    /// `Player.lands_dont_untap_next_untap`; non-land permanents untap normally.
    LandsDontUntapNextUntapStep { who: Selector },
    /// CR 502.3 — "Creatures don't untap during `who`'s next untap step"
    /// (Blinding Beam). The creature-side sibling of
    /// `LandsDontUntapNextUntapStep`; consumed in `do_untap`.
    CreaturesDontUntapNextUntapStep { who: Selector },
    /// Each permanent picked by `what` becomes a single color of the
    /// controller's choice for `duration` (CR 105 / layer 5 SetColors).
    /// Wild Mongrel ("becomes the color of your choice until end of turn").
    BecomeChosenColor { what: Selector, duration: Duration },
    /// CR 205.1b — the creature-type sibling of `BecomeChosenColor`: the
    /// controller names a creature type and every permanent picked by `what`
    /// becomes exactly that type for `duration` (Unnatural Selection). One
    /// choice covers the whole effect. `excluded` carries the printed
    /// restriction ("a creature type other than Wall" — Imagecrafter,
    /// Standardize, Mistform Mutant); the decision offers no excluded type
    /// and a decider that names one anyway is overruled.
    BecomeChosenCreatureType {
        what: Selector,
        duration: Duration,
        #[serde(default)]
        excluded: Vec<crate::card::CreatureType>,
    },
    /// Each permanent picked by `what` becomes exactly `colors` (replacing
    /// its colors) for `duration` (CR 105 / layer-5 `SetColors`). The
    /// fixed-color sibling of `BecomeChosenColor`. Crimson Wisps ("becomes
    /// red"), Crimson Wisps-style color set without a player choice.
    BecomeColor {
        what: Selector,
        colors: Vec<crate::mana::Color>,
        duration: Duration,
        /// When true, `colors` are *added* to the permanent's own colors
        /// (layer-5 `AddColor`) rather than replacing them — "becomes black in
        /// addition to its other colors" (Possessed Goat). Defaults to false.
        #[serde(default)]
        additive: bool,
    },
    /// Each permanent picked by `what` has its creature types set to exactly
    /// `creature_types` for `duration` (CR 305.7 / layer-4 `SetCreatureTypes`).
    /// The type-line half of "becomes a [color] [type]" cards — pair with
    /// `BecomeColor` / `SetBasePT` / `LoseAllAbilities` for the full rewrite
    /// (Kasmina's Transmutation → blue Frog, Kenrith's Transformation → Elk,
    /// Turn to Frog, Lignify → Treefolk).
    /// CR 613 layer 7d — "switch target creature's power and toughness" for
    /// `duration` (Transmutation).
    SwitchPowerToughness { what: Selector, duration: Duration },
    BecomeCreatureType { what: Selector, creature_types: Vec<crate::card::CreatureType>, duration: Duration },
    /// CR 205.1b / 613.4 — each permanent picked by `what` gains
    /// `creature_types` *in addition* to its own for `duration` (layer-4
    /// additive `AddCreatureType`, unlike `BecomeCreatureType`'s full set).
    /// "That creature becomes a Mutant in addition to its other types"
    /// (Jenova, Ancient Calamity).
    AddCreatureTypes { what: Selector, creature_types: Vec<crate::card::CreatureType>, duration: Duration },
    /// CR 612 — change the target's text by replacing all instances of one
    /// color word with another, both chosen by the controller (layer 3;
    /// rewrites Protection-from-color). Trait Doctoring, Mind Bend.
    ReplaceColorWord { what: Selector, duration: Duration },
    /// CR 612 / 305.7 — change the target's text by replacing all instances
    /// of one basic land type with another, both chosen by the controller
    /// (layer 3; rewrites the type line + landwalk, so a swapped basic taps
    /// for the new color). Trait Doctoring, Mind Bend.
    ReplaceBasicLandType { what: Selector, duration: Duration },
    /// CR 612.1 — change target spell or permanent's text by replacing all
    /// instances of one creature type with another, indefinitely; the new
    /// type can't be Wall (CR 205.3m). Unlike its color/land-type siblings
    /// this rewrites the object's *definition* rather than emitting a layer-3
    /// effect, so it survives a zone change into the battlefield and reaches
    /// creature-type words inside ability text. Artificial Evolution.
    ReplaceCreatureTypeText { what: Selector },
    /// The controller chooses a color as the source enters; stamp it onto the
    /// source's `chosen_color` (CR 614 — Coldsteel Heart, choose-a-color mana
    /// rocks). Read later by `ManaPayload::ChosenColorOfSource`.
    ChooseColorForSelf,
    /// Tablet of the Guilds: as this enters, choose two colors (stamped on
    /// `CardInstance.chosen_colors`).
    ChooseTwoColorsForSource,
    /// Tablet of the Guilds: whenever you cast a spell that is at least one of
    /// the source's chosen colors, gain 1 life for each chosen color it is.
    GainLifePerChosenColorOfCast,
    /// Each permanent picked by `what` gains protection from a color of the
    /// controller's choice for `duration` (`Decision::ChooseColor` →
    /// `Keyword::Protection(color)`). Mother of Runes, Giver of Runes, Gods
    /// Willing, Apostle's Blessing.
    GrantProtectionFromChosenColor { what: Selector, duration: Duration },
    /// Grant a transient triggered ability to each permanent picked by
    /// `what`, for `duration`. Stashed in `GameState.
    /// granted_triggers_eot`; `Duration::Permanent` bakes the trigger onto the
    /// permanent's own definition instead (the Volver kickers). The dispatcher
    /// walks both printed `triggered_abilities` and granted ones,
    /// firing matching events from either source. Used by Root
    /// Manipulation ("creatures you control gain 'whenever this
    /// creature attacks, you gain 1 life'") and Rabid Attack
    /// ("creatures gain 'when this creature dies, draw a card'" — die
    /// half requires LTB-trigger-snapshot follow-up).
    GrantTriggeredAbility {
        what: Selector,
        trigger: Box<crate::card::TriggeredAbility>,
        duration: Duration,
    },
    /// "Target creature loses all abilities until end of turn." Installs a
    /// `Modification::RemoveAllAbilities` continuous effect against each
    /// resolved permanent at layer 6. While in scope, the layer system
    /// clears keywords on the computed permanent AND flips its
    /// `lost_all_abilities` flag — the trigger dispatcher and activated-
    /// ability resolver consult that flag to skip the printed
    /// triggered/activated abilities (CR 113.10b). Used by Turn to Frog,
    /// Mercurial Transformation, Lignify (the "loses all abilities" half
    /// of these "creature becomes X" effects).
    LoseAllAbilities { what: Selector, duration: Duration },
    AddCounter    { what: Selector, kind: CounterType, amount: Value },
    /// "Put up to `amount` `kind` counters on [what]. This ability can't cause
    /// the total number of `kind` counters on it to be greater than `cap`."
    /// Clamps per target against its current pool (Clockwork Avian).
    AddCounterCapped { what: Selector, kind: CounterType, amount: Value, cap: Value },
    /// "Put up to `max` `kind` counters on [what]" — the controller picks the
    /// count once and it applies to every resolved permanent (Esper Terra's
    /// "if it's a Saga, put up to three lore counters on it"). `filter`, when
    /// set, narrows the resolved permanents (the Saga check).
    AddCountersUpTo {
        what: Selector,
        kind: CounterType,
        max: Value,
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },
    RemoveCounter { what: Selector, kind: CounterType, amount: Value },
    /// CR 603.7e — "Until the end of your next turn, whenever you cast a
    /// spell, [body]." A repeating cast watcher whose window outlives the
    /// installing turn (Season of the Bold's three-point mode); the cast
    /// spell is bound as the body's `Selector::TriggerSource`.
    OnEachSpellYouCastUntilEndOfYourNextTurn { body: Box<Effect> },
    /// "For each color, put a `kind` counter on a permanent matching `filter`
    /// of that color. If you put counters on `win_at` permanents this way,
    /// you win the game" (Call the Spirit Dragons). One pick per color in
    /// WUBRG order; a multicolored permanent may be picked more than once,
    /// so the win check counts *distinct* recipients.
    CounterOnMatchingOfEachColor {
        filter: SelectionRequirement,
        kind: CounterType,
        win_at: u32,
    },
    /// CR 603.7e — "When you next cast a creature spell this turn, that
    /// creature enters with N additional counters of `kind`." Registers a
    /// one-shot rider on the controller (`Player.pending_creature_etb_counters`)
    /// drained onto the next creature spell they cast this turn; unused riders
    /// expire at cleanup. The FIN "Summon" saga chapters (Fenrir "Heavenward
    /// Howl", Brynhildr "Gestalt Mode"-adjacent growth).
    GrantNextCreatureSpellCounters { kind: CounterType, amount: Value },
    /// CR 603.7e — "When you next cast a creature spell this turn, that creature
    /// gains `keyword`." One-shot rider (`Player.pending_creature_etb_keywords`)
    /// applied to the next creature spell's permanent as it enters; expires at
    /// cleanup (Summon: Brynhildr's "Gestalt Mode" haste).
    GrantNextCreatureSpellKeyword { keyword: Keyword },
    /// For each permanent matching `filter` whose current power exceeds its
    /// base power, put that many +1/+1 counters on it (the per-permanent
    /// difference). Sovereign Okinec Ahau's attack trigger (CR 122).
    AddCountersForPowerOverBase { filter: SelectionRequirement },
    /// CR 701.63 — *endure N*: the controller of `target` either puts N
    /// +1/+1 counters on it, or creates an N/N white Spirit creature token.
    /// The choice is the controller's (AutoDecider keeps the counters so the
    /// enduring permanent grows). N=0 does nothing (CR 701.63b).
    Endure { target: Selector, n: Value },
    /// CR 701.68 — *blight N*: the controller puts N -1/-1 counters on a
    /// creature they control (their choice). With no creature they can't
    /// blight (701.68b) so it's a no-op. N=0 does nothing.
    Blight { n: Value },
    /// CR 701.66 — *earthbend N*: target land you control becomes a 0/0 land
    /// creature with haste (in addition to its other types) and gets N +1/+1
    /// counters; a delayed trigger returns it tapped when it dies or is exiled.
    Earthbend { n: Value },
    /// CR 701.65 — *airbend* the object(s) `what` resolves to: exile each,
    /// and for as long as it stays exiled its owner may cast it for {2}
    /// rather than its mana cost. Wrap in `ApplyToTargets` for the common
    /// "airbend up to one target [filter]" shape.
    Airbend { what: Selector },
    /// Remove every counter of every kind from `what` (CR 122.6 — Vampire
    /// Hexmage's "remove all counters from target permanent").
    RemoveAllCounters { what: Selector },
    /// Remove a single counter of any one kind from `what` (the controller's
    /// choice; auto-picks the first present kind). Thrull Parasite's "remove a
    /// counter from target nonland permanent".
    RemoveAnyCounter { what: Selector },
    /// Remove up to `amount` counters (any kinds, controller's choice; the
    /// auto-picker drains present kinds greedily) from `what`. A permanent
    /// target drains its counters; a player target drains poison counters
    /// (CR 122.6 — Price of Betrayal's "artifact, creature, planeswalker, or
    /// opponent"). Fewer than `amount` present removes all of them.
    RemoveCountersUpTo { what: Selector, amount: Value },
    /// Set the loyalty (CR 606) of `what` to `value` — a loyalty-set effect
    /// ("its loyalty becomes …" / "reset to its starting loyalty"). Overwrites
    /// the `Loyalty` counter count outright rather than adding/removing.
    /// Geyadrone Dihada's +1 loyalty-reset rider.
    SetLoyalty { what: Selector, value: Value },
    /// "You may activate loyalty abilities of ~ twice this turn rather than
    /// only once" — Kaito, Dancing Shadow's combat-damage rider. Sets the
    /// resolved planeswalkers' `loyalty_twice_this_turn` (cleared at cleanup).
    GrantLoyaltyTwiceThisTurn { what: Selector },
    /// CR 702.65 — Aura swap: "Exchange this Aura with an Aura card in your
    /// hand." The source Aura returns to hand and the chosen hand Aura enters
    /// attached to the same permanent (Arcanum Wings). Auto-picks the
    /// highest-MV hand Aura; a `wants_ui` controller gets a ChooseCards pick.
    AuraSwapFromHand,
    /// Ria Ivor — "the next time target creature would deal combat damage to
    /// one or more players this combat, prevent that damage; create that many
    /// 1/1 Phyrexian Mite tokens" (shield keyed on the creature as source).
    PreventNextDamageByTargetMintMites,
    /// Ichormoon Gauntlet — "choose a counter on target permanent; put an
    /// additional counter of that kind on it." Auto-picks +1/+1 when present,
    /// else the most numerous kind.
    AddCounterOfPresentKind { what: Selector },
    /// "Move `amount` [counter] counters from [from] onto [to]" (Steel
    /// Dromedary). Capped by the source's live count; no-ops when `from`
    /// and `to` resolve to the same permanent.
    MoveCounters { from: Selector, to: Selector, counter: crate::card::CounterType, amount: Value },
    /// Rakdos, the Showstopper — "flip a coin for each creature that isn't one
    /// of `exclude_types`; destroy each creature whose coin comes up tails."
    /// The controller flips (honoring coin-flip advantage).
    CoinFlipEachCreatureDestroyOnTails { exclude_types: Vec<crate::card::CreatureType> },
    /// Awaken the Erstwhile — "each player discards all the cards in their
    /// hand, then creates that many `token` tokens." Resolved in turn order.
    EachPlayerDiscardsHandMakeTokens { token: crate::card::TokenDefinition },
    /// Memory Jar — "each player exiles all cards from their hand face down and
    /// draws seven cards. At the beginning of the next end step, each player
    /// discards their hand and returns to their hand each card they exiled this
    /// way." Stamps the exiles with the source and registers the end-step
    /// delayed trigger itself, so the whole card is one effect.
    EachPlayerExilesHandDrawsSeven,
    /// The end-step half of `EachPlayerExilesHandDrawsSeven`, registered as a
    /// delayed trigger by that effect (not printed on any card directly).
    EachPlayerDiscardsHandReturnsExiledWithSource,
    /// "[Player] ignores this permanent's static effect until end of turn"
    /// (Damping Engine). Records `(source, controller)` on the player for the
    /// turn; the static's gates skip anyone holding the pass.
    IgnoreStaticFromSourceThisTurn,
    /// Rumbling Ruin — "count the +1/+1 counters on creatures you control;
    /// creatures your opponents control with power ≤ that number can't block
    /// this turn." The affected set is locked in at resolution (computed power).
    OpponentWeakCreaturesCantBlockByYourCounters,
    /// Galloping Lizrog — "remove any number of +1/+1 counters from among
    /// creatures you control; put twice that many +1/+1 counters on this."
    /// The choose-any-number is collapsed to all other creatures (the
    /// optimal play), then doubled onto the source (honoring CR 614.16
    /// counter doublers).
    DoubleP1P1CountersFromYourCreatures,
    /// CR 702.43c — the Modular death trigger's counter move: put the dying
    /// source's last-known +1/+1 counters on the targeted artifact creature.
    /// A `StaticEffect::ModularBonusCounters` (Zabaz) controlled by the
    /// recipient's controller adds its bonus.
    ModularCounters { what: Selector },
    /// CR 702.62e-f — exile the selected creature with `time_counters` time
    /// counters; if it doesn't have suspend, it gains suspend (the card
    /// "Suspend"). Its owner's `process_suspend` ticks it down and free-casts
    /// it when the last counter is removed.
    GrantSuspend { what: Selector, time_counters: u32 },
    /// "You may cast spells from your hand this turn without paying their
    /// mana costs" (Yusri's five-win jackpot). Sets the controller's
    /// `free_spells_from_hand_this_turn` flag, cleared at end-of-turn.
    FreeSpellsFromHandThisTurn,
    /// "As this enters, choose a card type" (Serra's Emissary). Asks the
    /// controller via `ChooseMode` over the permanent+spell card types and
    /// stamps `CardInstance.chosen_card_type` on the source.
    ChooseCardTypeForSource,
    /// Lonis's steal: target opponent reveals the top `count` cards of their
    /// library; you may put a nonland permanent card with mana value at most
    /// `max_mv` from among them onto the battlefield under your control; the
    /// rest go to the bottom in random order. Auto-picks the highest-MV
    /// legal card.
    OpponentRevealsPickToBattlefield { count: Value, max_mv: Value },
    /// Gaea's Will — until end of turn you may play lands and cast spells
    /// from your graveyard: every card currently there gets a pay-own-cost
    /// `may_play_until` permission and lands ride the graveyard land-play
    /// gate. Cards reaching the graveyard later this turn aren't granted.
    PlayFromGraveyardThisTurn,
    /// Gaea's Will — "If a card would be put into your graveyard from
    /// anywhere this turn, exile it instead" (turn-scoped Rest in Peace,
    /// own cards only).
    ExileYourGraveyardBoundThisTurn,
    /// Glimpse of Tomorrow — shuffle all permanents you own into your
    /// library, reveal that many cards; non-Aura permanents enter the
    /// battlefield, then Auras (auto-attached), the rest to the bottom in
    /// random order.
    GlimpseOfTomorrow,
    /// Garth One-Eye — choose a not-yet-chosen name among `names`, create a
    /// copy of that card and you may cast it (approximated: the copy is put
    /// into your hand as a real card; used names tracked in
    /// `CardInstance.name_choices_used`).
    GarthOneEye { names: Vec<String> },
    /// Chef's Kiss — gain control of target single-target spell, copy it,
    /// and reselect each target at random among legal targets that aren't
    /// you or your permanents (keeps the old target when none exist).
    ChefsKiss,
    /// Grist's +1 — create a 1/1 black-green Insect, then mill a card; if an
    /// Insect card was milled, add a loyalty counter and repeat (loop-capped).
    GristPlusOne,
    /// Portent Tracker — choose target battle: if an opponent protects it,
    /// remove a defense counter; otherwise put one on it (CR 310.7).
    AdjustBattleDefense { what: Selector },
    /// Yusri, Fortune's Flame — choose a number 1..=`max`, flip that many
    /// coins; run `per_win` per won flip and `per_loss` per lost flip, then
    /// `all_won` if the chosen number was `all_won_min`+ and every flip won.
    /// With `stop_on_loss` the flipping stops at the first lost flip ("flip a
    /// coin that many times or until you lose a flip" — Squee's Revenge).
    /// `all_won` reads the chosen number as `Value::XFromCost`.
    FlipCoinsChooseCount {
        max: u32,
        per_win: Box<Effect>,
        per_loss: Box<Effect>,
        all_won: Box<Effect>,
        all_won_min: u32,
        #[serde(default)]
        stop_on_loss: bool,
    },
    /// Vraska, Betrayal's Sting's −2 — "[what] becomes a Treasure artifact
    /// with '{T}, Sacrifice: add one mana of any color' and loses all other
    /// card types and abilities" (layer 4 type swap + layer 6 ability wipe +
    /// a granted sac-for-mana ability, all while it stays on the battlefield).
    BecomeTreasure { what: Selector },
    /// CR 122.1b — Add a keyword counter to `what`. The host gains the
    /// named keyword while at least one counter of this kind is present
    /// (applied as a layer-6 grant in `compute_battlefield`). Removed
    /// independently of the keyword; the host loses the keyword
    /// (assuming no other source) when the last keyword counter is
    /// removed. Push (modern_decks batch 183): added per CR 122.1b.
    AddKeywordCounter { what: Selector, keyword: crate::card::Keyword, amount: Value },
    /// CR 122.1b / "choose a kind of counter at random" — pick uniformly at
    /// random one option `what` doesn't already have (a keyword counter from
    /// `keyword_options`, or a +1/+1 counter when `plus_one_plus_one`), and put
    /// one of that kind on it. Crystalline Giant. If every option is already
    /// present, nothing happens.
    AddRandomMissingCounter {
        what: Selector,
        keyword_options: Vec<crate::card::Keyword>,
        #[serde(default)]
        plus_one_plus_one: bool,
    },
    /// CR 122.1b — Remove up to `amount` keyword counters of `keyword`
    /// from `what`. Clamped at the source's actual count; the host loses
    /// the keyword (assuming no other source) when the last counter of
    /// this kind is removed. Counterpart to `AddKeywordCounter`. Push
    /// (claude/modern_decks, batches 192-193): added — closes the loop
    /// for "strip flight" / "remove a vigilance counter" style effects.
    RemoveKeywordCounter { what: Selector, keyword: crate::card::Keyword, amount: Value },
    /// CR 122.5 — move `amount` counters of `kind` from `from` to `to`.
    /// Clamped at the source's actual counter count; emits a single
    /// `CounterRemoved` for the source and a single `CounterAdded` for
    /// each target. The doubling-counter replacement (CR 614.16) does
    /// NOT apply to the destination — moves are explicitly NOT counter
    /// creation under CR 122.5 (the counters already exist; they're
    /// being relocated). Powers Tester of the Tangential's "pay {X}.
    /// When you do, move X +1/+1 counters from this creature onto
    /// another target creature" combat trigger.
    MoveCounter   { from: Selector, to: Selector, kind: CounterType, amount: Value },
    /// Move every counter (all kinds) from `from` onto `to` (The Ozolith's
    /// begin-combat transfer). Relocation, not creation — no doublers.
    MoveAllCounters { from: Selector, to: Selector },
    /// CR 603.7 — a reflexive "when you do, …" sub-trigger: push `body`
    /// onto the stack as a triggered ability of the same source and
    /// controller instead of resolving it inline. The containing
    /// effect/ability finishes resolving first (mana abilities resolve
    /// immediately), then the body waits for priority like any trigger —
    /// so opponents get a response window the inline fold denied them.
    /// The body's targets are auto-picked at push time (a reflexive
    /// trigger targets when it triggers, CR 603.7d). Rubble Rouser's
    /// "{T}, Exile a card from your graveyard: Add {R}. When you do,
    /// this creature deals 1 damage to each opponent."
    ReflexiveTrigger { body: Box<Effect> },
    /// CR 701.34a — Proliferate. "Choose any number of permanents and/or
    /// players that have a counter, then give each another counter of a
    /// kind already there." The auto-decider implements a strategic
    /// baseline: grow good counters (+1/+1, Loyalty, Charge, Page) on the
    /// controller's permanents, grow bad counters (-1/-1, Stun) on enemy
    /// permanents, any other kind by default, and add one poison to each
    /// opponent already poisoned. (No multi-select UI yet.)
    Proliferate,
    /// Gain control of `what`. `to` names the new controller (`None` = the
    /// effect's controller, the common case — Threaten, Act of Treason). A
    /// `Some(pref)` hands control to another player — Wishclaw Talisman's
    /// "target opponent gains control" downside. `#[serde(default)]` keeps
    /// pre-field snapshots deserializing as `None`.
    GainControl {
        what: Selector,
        #[serde(default)]
        to: Option<PlayerRef>,
        duration: Duration,
    },
    /// Gain control of the resolved permanents for as long as the effect's
    /// source remains on the battlefield (Sower of Temptation).
    GainControlWhileSourceRemains { what: Selector },
    /// CR 611.2c — "gain control of `what` for as long as this permanent
    /// remains tapped" (Vedalken Shackles). The steal unwinds in the SBA
    /// sweep once the source untaps or leaves; while it holds something the
    /// source skips its own untap step ("you may choose not to untap").
    GainControlWhileSourceTapped { what: Selector },
    /// CR 611.2c — "gain control of that permanent for as long as that Aura is
    /// attached to it" (Eriette, the Beguiler). The Aura is
    /// `ctx.trigger_source`; the stolen permanent is whatever it's attached to.
    GainControlWhileTriggerAuraAttached,
    /// CR 611.2c sibling — the resolved permanents gain `keyword` for as long
    /// as the effect's source stays tapped on the battlefield (Hisoka's Guard's
    /// shroud grant). Unwound by the same SBA sweep.
    GrantKeywordWhileSourceTapped {
        what: Selector,
        keyword: crate::card::Keyword,
    },
    /// Mindblaze's back half — the controller picks a number greater than 0,
    /// the resolved player reveals their library, and if it holds *exactly*
    /// that many cards with the name a preceding `Effect::NameCard` stamped,
    /// the source deals `damage` to them. That player shuffles either way.
    RevealLibraryNamedCountPunish { who: Selector, damage: Value },
    /// Moonring Mirror's upkeep — the controller may exile their whole hand
    /// face down stamped `exiled_with = source`; if they do, every *other*
    /// card they own already stamped that way returns to their hand.
    ExileHandThenReclaimLinked,
    /// Reweave — the resolved permanent's controller sacrifices it, then
    /// reveals from the top of their library until a permanent card sharing a
    /// card type with it turns up, puts that card onto the battlefield, and
    /// shuffles.
    SacrificeThenRevealUntilSharedType { what: Selector },
    /// Struggle for Sanity — the resolved opponent reveals their hand, then
    /// they and the controller alternate exiling one card each (they go first)
    /// until the hand is empty. Their picks return to hand; the controller's
    /// go to the graveyard.
    AlternatingExileFromHand { who: Selector },
    /// "Double the amount of each type of unspent mana you have" (Doubling
    /// Cube) — every color and colorless pip in the controller's pool is
    /// duplicated, restrictions included.
    DoubleUnspentMana,
    /// CR 614.5 — "If any source you control would deal damage … this turn,
    /// it deals double that damage instead." Sets the controller's
    /// turn-scoped flag read by `scale_damage_to` (Quest for Pure Flame).
    DoubleYourSourcesDamageThisTurn,
    /// Goblin Welder: target artifact's controller simultaneously sacrifices
    /// it and returns an artifact card from their graveyard to the
    /// battlefield. The graveyard half is auto-picked (highest mana value).
    WeldArtifacts { what: Selector },
    /// Create `count` copies of the given token under `who`'s control.
    CreateToken { who: PlayerRef, count: Value, definition: TokenDefinition },
    /// "You may remove any number of `kind` counters from this. If you do,
    /// create that many `definition` tokens." The tokens are stamped
    /// `created_by = source`, so `ExileTokensCreatedBySourceForCounters` can
    /// trade them back (Tetravus).
    RemoveCountersToCreateTokens { kind: CounterType, definition: TokenDefinition },
    /// The mirror: "you may exile any number of tokens created with this. If
    /// you do, put that many `kind` counters on it" (Tetravus).
    ExileTokensCreatedBySourceForCounters { kind: CounterType },
    /// Register a `DelayedKind::NextEndStep` exile for every token minted
    /// earlier in this resolution (reads `last_created_tokens`). Chain after
    /// a `CreateToken` inside a `Seq` for "create N transient tokens, exile
    /// them at the next end step" (Valduk, Keeper of the Flame).
    ExileLastCreatedTokensAtNextEndStep,
    /// The cleanup-step sibling: "Exile them at the beginning of the next
    /// cleanup step" (Waylay), so the tokens survive the end step.
    ExileLastCreatedTokensAtNextCleanup,
    /// Sacrifice-flavored sibling: "Sacrifice that token at the beginning of
    /// the next end step" (Urabrask's Forge) — dies-triggers fire, unlike the
    /// exile variant.
    SacrificeLastCreatedTokensAtNextEndStep,
    /// Incubate N (CR 701.53): create an Incubator double-faced token under
    /// `who`'s control with `amount` +1/+1 counters on it.
    Incubate { who: PlayerRef, amount: Value },
    /// Amass N (CR 701.43): put `count` +1/+1 counters on an Army `who`
    /// controls, creating a 0/0 black Army creature token first if they
    /// control none. `extra_type` is added to the Army (Amass Zombies /
    /// Amass Orcs mint a token that's also that subtype).
    Amass { who: PlayerRef, count: Value, extra_type: Option<crate::card::CreatureType> },
    /// Create `count` tokens already tapped and attacking (CR 508.3a). The
    /// new tokens join the current combat attacking the same defender the
    /// effect's source is attacking (falling back to the controller's first
    /// opponent when the source isn't itself an attacker). Powers "create N
    /// tokens tapped and attacking" riders and Mobilize (CR 702.169).
    /// `cleanup` registers the tokens to leave at end of combat. No-op
    /// outside the combat phase.
    CreateTokenAttacking {
        who: PlayerRef,
        count: Value,
        definition: TokenDefinition,
        #[serde(default)]
        cleanup: AttackingTokenCleanup,
    },
    /// CR 508.3a — put the resolved permanent(s) into the current combat
    /// tapped and attacking, bypassing the declare-attackers timing/sickness
    /// gates. Each joins attacking the same defender the effect's source is
    /// attacking (else the controller's first opponent). No-op outside combat.
    /// Composes with a preceding `Move … → Battlefield { tapped: true }` to
    /// reanimate a creature "tapped and attacking" (Alesha, Who Smiles at Death,
    /// via `Selector::LastMoved`).
    JoinCombatAttacking { what: Selector },
    /// Exile the top card of the source controller's library; if it's a
    /// creature card, the source gets +power/+toughness until end of turn equal
    /// to that card's power and toughness. Bioplasm's attack trigger.
    ExileTopSelfPumpIfCreature,
    /// Myriad (CR 702.115): for each opponent of the source's controller
    /// other than the player the source is attacking, create a token that's
    /// a copy of the source, tapped and attacking that opponent. The copies
    /// are exiled at end of combat. No-op outside combat / when the source
    /// isn't attacking a player.
    Myriad,
    /// "The next instant or sorcery spell you cast this turn costs {amount}
    /// less to cast" (Thundertrap Trainer, Maelstrom Muse). `amount` is
    /// evaluated at resolution, so `Value::PowerOf(TriggerSource)` reads the
    /// source's power as the ability resolves. Pushes a one-shot discount
    /// onto `Player.pending_is_discounts` that lapses after the next such
    /// spell.
    GrantNextInstantOrSorceryDiscountThisTurn { amount: Value },
    /// Support N (CR 701.32): "Put a +1/+1 counter on each of up to N target
    /// creatures." Each of slots `0..max_targets` is an optional creature
    /// target (filtered by `filter`); every supplied target gains one +1/+1
    /// counter at resolution.
    SupportCounters { max_targets: u8, filter: SelectionRequirement },
    /// "Distribute N `counter` counters among any number of target creatures"
    /// (CR 601.2d divided choice — Jugan, the Rising Star; Hatchery Spider).
    /// Each of slots `0..max_targets` is an optional target (filtered by
    /// `filter`); the per-target split is decided at resolution via
    /// `Decision::DivideDamage` (AutoDecider spreads as evenly as possible).
    DistributeCounters { total: Value, counter: CounterType, filter: SelectionRequirement, max_targets: u8 },
    /// "Do X to each of up to N target permanents" (CR 115 — the generic
    /// multi-target rider). Slots `0..max_targets` are targets filtered by
    /// `filter`; at resolution `effect` runs once per supplied target with
    /// `Selector::Target(0)` bound to that target. Powers "return up to
    /// three target creatures" (Sea God's Scorn), "deal 1 damage to each
    /// of up to three target creatures" (Wrap in Flames), "tap up to N
    /// target permanents", etc. The inner effect must address its operand
    /// via `Selector::Target(0)`.
    ///
    /// `min_targets` is how many targets the printed text *requires*:
    /// 0 = "up to N / any number" (every slot optional, including slot 0 —
    /// a `wants_ui` caster may decline each pick); 1 = "one or two
    /// targets" (Prismari Charm). Targets fill left-to-right; declining
    /// ends selection.
    ApplyToTargets {
        max_targets: u8,
        #[serde(default)]
        min_targets: u8,
        filter: SelectionRequirement,
        effect: Box<Effect>,
    },
    /// "Up to X targets" wrapper (Crackle with Power): truncates the
    /// supplied cast-time target list to the cast's `x_value` before
    /// running `body` (usually an `ApplyToTargets` whose `max_targets` is
    /// the static slot ceiling). Cast-time slot enumeration still offers
    /// the ceiling; an over-select at small X is dropped at resolution.
    CapTargetsAtX { body: Box<Effect> },
    /// "N target …" where N is the paid {X} (CR 601.2c — Synod Artificer's
    /// "Tap X target noncreature artifacts"). Like `CapTargetsAtX`, but slots
    /// `0..X` are *required* rather than optional, so an activation that supplies
    /// too few targets is rejected instead of half-firing. `body` is normally an
    /// `ApplyToTargets` whose `max_targets` is the static slot ceiling.
    TargetsExactlyX { body: Box<Effect> },
    /// The `Value`-driven sibling of `CapTargetsAtX`: "up to [amount] target
    /// creatures", where the cap is computed at resolution rather than paid as
    /// {X} (Mogis's Marauder's devotion-to-black cap).
    CapTargetsAt { amount: Value, body: Box<Effect> },
    /// Transparent wrapper declaring that target slots `>= min` are optional
    /// ("up to one target …") for an otherwise-conventional `body` whose slots
    /// come from *distinct* effects — the case `ApplyToTargets` can't express
    /// because it rebinds every supplied target to `Target(0)`. Primal Might is
    /// `OptionalTargets { min: 1, body: Seq[counters on Target(0), Fight{
    /// attacker: Target(0), defender: Target(1) }] }`: slot 0 (the friendly
    /// creature) is required, slot 1 (the enemy it fights) may be declined, and
    /// the Fight no-ops when slot 1 resolves to nothing. Evaluation just runs
    /// `body`; the wrapper only feeds the targeting walk (`min_targets_in_mode`
    /// / `target_slot_optional`).
    OptionalTargets { min: u8, body: Box<Effect> },
    /// Eerie Ultimatum — return any number of permanent cards with different
    /// names from the controller's graveyard to the battlefield. The controller
    /// picks at resolution (`Decision::ChooseCards`); duplicate names are
    /// dropped so each returned card has a distinct name.
    ReturnGraveyardPermanentsDifferentNames,
    /// "Return all [filter] cards from `who`'s graveyard to the battlefield"
    /// (Push the Limit). No pick — every match comes back under the effect's
    /// controller. With `sacrifice_eot` each returns is sacrificed at the
    /// beginning of the next end step.
    ReturnAllMatchingFromGraveyardToBattlefield {
        who: PlayerRef,
        filter: SelectionRequirement,
        #[serde(default)]
        sacrifice_eot: bool,
    },
    /// "Return up to `max` `filter` cards from your graveyard to your hand"
    /// (Mythos of Brokkos). Resolution-time `Decision::ChooseCards` pick (no
    /// targeting); reusable for any choose-as-resolves graveyard recursion.
    ReturnGraveyardCardsToHand { filter: SelectionRequirement, max: Value },
    /// "Until end of turn, you may cast creature spells from your graveyard by
    /// foraging in addition to paying their other costs; a creature cast this
    /// way enters with a finality counter" (Osteomancer Adept, CR 701.61).
    GrantForageGraveyardCreatureCastsThisTurn,
    /// "Put a `filter` card from a graveyard onto the battlefield under your
    /// control" (Victor, Valgavoth's Seneschal). Untargeted: a resolution-time
    /// `Decision::ChooseCards` over *every* graveyard, auto-picking the highest
    /// mana value for a non-UI seat.
    PutGraveyardCardOntoBattlefield { filter: SelectionRequirement },
    /// "`who` shuffles up to `max` `filter` cards from their graveyard into
    /// their library." Resolution-time `Decision::ChooseCards` by the affected
    /// player (no targeting) — a graveyard-recursion / anti-mill rider
    /// (Cathartic Parting, Rite of Renewal). Mirror of
    /// `ReturnGraveyardCardsToHand` for the library destination.
    ShuffleGraveyardCardsIntoLibrary {
        who: PlayerRef,
        filter: SelectionRequirement,
        max: Value,
        /// Put the chosen cards on top of the library in any order instead of
        /// shuffling them in (Stillness in Motion).
        #[serde(default)]
        to_top: bool,
    },
    /// Genesis Ultimatum — look at the top `count` cards of the controller's
    /// library; put any number of permanent cards among them onto the
    /// battlefield and the rest into hand. The controller picks the permanents
    /// to deploy at resolution (`Decision::ChooseCards`).
    LookTopNDeployPermanentsRestToHand { count: Value },
    /// Yidaro, Wandering Monster — "When you cycle this card, shuffle it into
    /// your library from your graveyard. If you've cycled a card with this
    /// name `threshold` or more times this game, put it onto the battlefield
    /// from your graveyard instead." Reads `GameState.cycled_count_by_name`.
    CycleRecurFromGraveyard { threshold: u32 },
    /// Illuna, Apex of Wishes — exile cards from the top of the controller's
    /// library until a nonland *permanent* card is exiled, then put that card
    /// onto the battlefield or into hand (controller's choice via
    /// `Decision::OptionalTrigger`). The other exiled cards stay in exile.
    ExileTopUntilPermanentToBattlefieldOrHand,
    /// Nethroi, Apex of Death — return any number of creature cards from the
    /// controller's graveyard with total power `max_total` or less to the
    /// battlefield. The controller picks the set at resolution
    /// (`Decision::ChooseCards`); picks are accepted greedily until the next
    /// would exceed the cap.
    ReturnGraveyardCreaturesUpToTotalPower { max_total: Value },
    /// Scout for Survivors — return up to `max_count` creature cards from the
    /// controller's graveyard with total **mana value** `max_total` or less to
    /// the battlefield, each with `counters` +1/+1 counters. The controller
    /// picks the set at resolution; picks are accepted greedily until the next
    /// would exceed either cap.
    ReturnGraveyardCreaturesUpToTotalManaValue {
        max_total: Value,
        max_count: Value,
        counters: u32,
    },
    /// Protean Hulk — search the controller's *library* for any number of
    /// creature cards with total mana value `max_total` or less, put them
    /// onto the battlefield, then shuffle. The controller picks the set at
    /// resolution; picks are accepted greedily until the next would exceed
    /// the cap. The library variant of
    /// [`ReturnGraveyardCreaturesUpToTotalManaValue`].
    SearchLibraryCreaturesUpToTotalManaValue { max_total: Value },
    /// Command the Dreadhorde — choose any number of creature and/or
    /// planeswalker cards in *any* graveyard, deal damage to the controller
    /// equal to their total mana value, then put them onto the battlefield
    /// under the controller's control. The set is chosen at resolution
    /// (`Decision::ChooseCards`).
    CommandTheDreadhorde,
    /// Swift Silence — counter every other spell on the stack, then draw a
    /// card for each spell countered this way (CR 701.5).
    CounterAllOtherSpellsDrawPer,
    /// Fall (Rise // Fall) — `who` reveals `count` cards at random from their
    /// hand, then discards each nonland card revealed this way. Lands revealed
    /// this way stay in hand.
    RevealRandomDiscardNonland { who: Selector, count: Value },
    /// "Target opponent reveals a card at random from their hand" (the
    /// Planeswalker's cycle). The card stays in hand; its mana value is
    /// stamped for `Value::LastRevealedManaValue` and the card itself for
    /// `Selector::LastRevealedCard`, so a follow-up clause in the same `Seq`
    /// can scale off it or exile it.
    RevealRandomFromHand { who: Selector },
    /// Mass Polymorph — "Exile all creatures you control, then reveal cards
    /// from the top of your library until you reveal that many creature cards.
    /// Put all creature cards revealed this way onto the battlefield, then
    /// shuffle the rest of the revealed cards into your library."
    MassPolymorph,
    /// Wild Evocation — "that player reveals a card at random from their hand.
    /// If it's a land card, the player puts it onto the battlefield. Otherwise,
    /// the player casts it without paying its mana cost if able." The free cast
    /// uses auto-targets; an uncastable pick simply stays in hand.
    RandomHandCardDeployOrCastFree { who: Selector },
    /// "Tap up to N target permanents; they don't untap during their
    /// controller's next untap step" where N is a runtime `Value`
    /// (Archipelagore — N = `Value::MutateCount`). The controller chooses up
    /// to the evaluated count of matching permanents at resolution
    /// (`Decision::ChooseCards`), so the dynamic count sidesteps the
    /// fixed-slot declared-target model. `skip_untap` adds the
    /// `skip_next_untap` flag to each tapped permanent.
    /// `exact: true` turns the pick into an all-or-nothing cost: the chooser
    /// must tap exactly `count` (min = max in the `ChooseCards` decision;
    /// AutoDecider fills top-down). Aziza's "tap three untapped creatures
    /// you control" — the surrounding `If` guarantees enough candidates.
    TapUpToValue { count: Value, filter: SelectionRequirement, skip_untap: bool, exact: bool },
    /// Enlist (CR 702.151): "As this attacks, you may tap a nonattacking
    /// creature you control without summoning sickness. When you do, add its
    /// power to this creature's power until end of turn." The "you may" /
    /// "which creature" collapses to auto-tapping the highest-power eligible
    /// creature (only when its power is positive, so it's never a downgrade).
    Enlist,
    /// Create `count` token copies of the permanent resolved by `source`,
    /// controlled by `who`. The copy inherits the source's printed
    /// CardDefinition (name, P/T, types, keywords, activated/triggered
    /// abilities, static abilities). `extra_creature_types` are added on
    /// top of the source's printed creature subtypes (Applied Geometry,
    /// Echocasting Symposium: "create a token that's a copy of target X,
    /// except it's a 0/0 Fractal creature in addition to its other
    /// types" — pass `vec![CreatureType::Fractal]` to honor the printed
    /// "in addition to" rider). The token is stamped with `is_token =
    /// true` so token-cleanup SBA removes it when it leaves the
    /// battlefield. Power/toughness override is honored when both
    /// `override_pt: Some((p, t))` is set (Applied Geometry overrides
    /// the source's printed P/T to 0/0). The override applies *before*
    /// any +1/+1 counter pile.
    CreateTokenCopyOf {
        who: PlayerRef,
        count: Value,
        source: Selector,
        #[serde(default)]
        extra_creature_types: Vec<crate::card::CreatureType>,
        /// Card types added on top of the source's printed types (Vaultborn
        /// Tyrant: "that token is an artifact in addition to its other types").
        #[serde(default)]
        extra_card_types: Vec<crate::card::CardType>,
        #[serde(default)]
        override_pt: Option<(i32, i32)>,
        /// CR 707.2 rider — the copy's color is set to these colors ("except
        /// it's a 5/5 black Demon" — Ardyn, the Usurper). Applied as a color
        /// indicator (CR 105.2c) plus stripping the copied cost's colored pips
        /// (generic count preserved) so the copy is exactly these colors.
        /// `None` keeps the source's own colors.
        #[serde(default)]
        override_colors: Option<Vec<crate::mana::Color>>,
        /// The token copy enters tapped (Sin, Spira's Punishment — "create a
        /// tapped token that's a copy"). `false` = enters untapped (default).
        #[serde(default)]
        enters_tapped: bool,
        /// CR 707.2e rider — the token copy isn't legendary (Helm of the
        /// Host). Strips supertypes from the copy so the legend rule doesn't
        /// destroy it alongside a legendary host.
        #[serde(default)]
        non_legendary: bool,
        /// Add the Legendary supertype to the copy even if the source isn't
        /// legendary (Adagia, Windswept Bastion's "except it's legendary").
        #[serde(default)]
        legendary: bool,
        /// Keywords layered on top of the copy ("with flying" — Kaya's Spirit
        /// copy, Kinzu's toxic 1 copy).
        #[serde(default)]
        extra_keywords: Vec<crate::card::Keyword>,
    },
    /// Create `count` token copies of the permanent resolved by `source`
    /// (controlled by `who`), each gaining haste until end of turn and
    /// sacrificed at the beginning of the next end step. Devastating Onslaught
    /// ("create X tokens that are copies of target artifact or creature you
    /// control").
    CreateTokenCopiesHasteSac {
        who: PlayerRef,
        count: Value,
        source: Selector,
    },
    /// Sin, Spira's Punishment — exile a permanent card from `who`'s graveyard
    /// at random, then create a tapped token that's a copy of it. If the
    /// exiled card was a land, repeat (bounded by the graveyard size).
    ExileRandomGraveyardCopyTapped { who: PlayerRef },
    /// CR 701.32 — Populate: `who` creates a token that's a copy of a creature
    /// token they control (their choice; AutoDecider keeps the highest-power
    /// one). No-op if they control no creature token.
    Populate { who: PlayerRef },
    /// CR 716.2 — advance the source Class enchantment by one level, emitting a
    /// `GameEvent::ClassLevelReached` (so "when this Class becomes level N"
    /// triggers fire). The resolution effect of a `{cost}: Level N` activated
    /// ability. No-op if the source isn't a Class on the battlefield.
    AdvanceClassLevel,
    /// CR 707.2 — `what` becomes a copy of the permanent resolved by
    /// `source`: its copiable characteristics (name, mana cost, card
    /// types, subtypes, abilities, P/T, loyalty) are overwritten with a
    /// clone of the source's current definition. The copy is locked in at
    /// resolution time — later changes to the source don't propagate (a
    /// one-shot definition rewrite, the same mechanism the engine uses for
    /// MDFC face-swap, rather than a layer-1 continuous effect). Instance
    /// state (id, owner, controller, counters, tapped, summoning sickness,
    /// token-ness) is preserved. `extra_creature_types` are added on top
    /// of the copied subtypes (Phantasmal Image's "it's an Illusion in
    /// addition to its other types"). Powers Clone, Phantasmal Image,
    /// Mockingbird (enter-as-a-copy). If `source` resolves to nothing the
    /// effect is a no-op (the copier stays itself — usually a 0/0 that
    /// dies to SBA, matching the printed "you may" decline).
    BecomeCopyOf {
        what: Selector,
        source: Selector,
        #[serde(default)]
        extra_creature_types: Vec<crate::card::CreatureType>,
        /// CR 707.2 "except it has this ability" — the copier keeps its own
        /// triggered abilities on top of the copied ones, so the copy can
        /// copy again (Artisan of Forms).
        #[serde(default)]
        keep_own_triggered: bool,
        /// The activated-ability twin of `keep_own_triggered` — Volatile
        /// Chimera's "except it has this ability", so it can shift again.
        #[serde(default)]
        keep_own_activated: bool,
    },
    /// CR 707.2 — continuous (layer-1) sibling of `BecomeCopyOf`: each
    /// `what` becomes a copy of `source` for `duration`, via a
    /// `Modification::CopyCardDefinition` continuous effect (the snapshot is
    /// locked in at resolution). `non_legendary` strips the Legendary
    /// supertype from the copy (CR 707.2e — Echoing Equation's "except
    /// they aren't legendary"). Computed characteristics (types, colors,
    /// keywords, P/T) and printed-ability dispatch both honor the copy
    /// while it lasts.
    BecomeCopyOfFor {
        what: Selector,
        source: Selector,
        duration: Duration,
        #[serde(default)]
        non_legendary: bool,
    },
    /// Target becomes a basic land of `land_type` (losing other types/abilities).
    BecomeBasicLand { what: Selector, land_type: LandType, duration: Duration },
    /// The controller chooses one basic land type, then every land picked by
    /// `what` becomes that type for `duration` (losing other types/abilities).
    /// Terraformer — "{1}: Choose a basic land type. Each land you control
    /// becomes that type until end of turn."
    LandsBecomeChosenBasicType { what: Selector, duration: Duration },
    /// "As [this] enters, choose a basic land type." Asks the controller (via
    /// the `ChooseColor` decision, basics mapping 1:1 onto colors) and stamps
    /// the choice on the source's `chosen_land_type`. Paired with
    /// `StaticEffect::LandsYouControlAreChosenType` (Realmwright).
    ChooseBasicLandTypeForSource,
    /// Grant `what` landwalk of the source's `chosen_land_type` (stamped by a
    /// preceding `ChooseBasicLandTypeForSource`), permanently. Traveler's
    /// Cloak. No-op when no type was chosen.
    GrantChosenTypeLandwalk { what: Selector },
    /// CR 601.2b — "You and target spell's controller bid life. You start at
    /// 1; in turn order each may top the high bid. The high bidder loses that
    /// much life; if you win, counter that spell." Mages' Contest.
    BidLifeToCounterTargetSpell { what: Selector },
    /// CR 305 — each resolved permanent *gains* all five basic land types for
    /// `duration` (additive, keeping existing types and abilities). Installs a
    /// layer-4 `AddLandType` continuous effect per basic type so the lands tap
    /// for any color (Energybending, Prismatic Omen-style fixers).
    GainAllBasicLandTypes { what: Selector, duration: Duration },
    /// CR 305 — each resolved permanent *gains* one land type in addition to
    /// its other types (layer 4, additive). Sealock Monster's monstrosity
    /// trigger ("target land becomes an Island in addition to its other types").
    GainLandType { what: Selector, land_type: crate::card::LandType, duration: Duration },
    /// Target becomes a creature with the given P/T and creature types,
    /// losing all other card types, abilities, and creature subtypes
    /// (CR 613 layers 4/6/7). Oko's "becomes a 3/3 Elk", Turn to Frog's
    /// "0/1 blue Frog with no abilities", etc. Defaults to a vanilla 1/1.
    ResetCreature {
        what: Selector,
        #[serde(default = "value_one")]
        power: Value,
        #[serde(default = "value_one")]
        toughness: Value,
        #[serde(default)]
        creature_types: Vec<crate::card::CreatureType>,
        duration: Duration,
    },
    /// "Choose target player matching `filter`, then `then`." A player target
    /// slot 0 with an explicit restriction, for bodies that don't reference
    /// the target themselves — the EXO Keeper cycle's "choose target opponent
    /// who <trails you> as you activate this ability".
    TargetPlayerThen { filter: crate::card::SelectionRequirement, then: Box<Effect> },
    /// Attach `what` (Aura/Equipment) to `to`.
    Attach { what: Selector, to: Selector },
    /// Licid (Stronghold) — "This creature loses this ability and becomes an
    /// Aura enchantment with enchant creature. Attach it to `host`. You may
    /// pay `end_cost` to end this effect." The source's creature definition
    /// is stashed on the instance and its live one is rewritten to
    /// Enchantment — Aura with a single `LicidDetach` ability; the aura
    /// riders ride the printed `equipped_bonus`, inert until attached.
    LicidAttach { host: Selector, end_cost: crate::mana::ManaCost },
    /// The `end_cost` half of [`Effect::LicidAttach`] — the source unattaches
    /// and goes back to being a creature.
    LicidDetach,

    // ── Stack interaction ────────────────────────────────────────────────────
    /// Counter target spell (removes from stack; sends to owner's graveyard).
    CounterSpell { what: Selector },
    /// CR 115.7 — "Change the target of target spell with a single target."
    /// Re-enumerates the spell's legal targets and lets this effect's
    /// controller repoint it (the current target is offered first, so an
    /// auto-decider is a no-op). A spell with zero or several targets is
    /// left alone. Shunt, Deflection.
    ChangeSpellTarget { what: Selector },
    /// Test of Talents — "Counter target instant or sorcery spell. Search
    /// its controller's graveyard, hand, and library for any number of
    /// cards with the same name as that spell and exile them. That player
    /// shuffles, then draws a card for each card exiled from their hand
    /// this way." The exile sweep is exhaustive (every same-named card),
    /// keyed on the countered spell's printed name and owner.
    CounterSpellExileSameNamed { what: Selector },
    /// Counter target spell; if the mana spent to cast it was less than its
    /// mana value, the controller draws a card (Unravel).
    CounterSpellDrawIfUnderpaid { what: Selector },
    /// Counter target spell and route it to a specific zone instead of the
    /// owner's graveyard. CR 701.6a's default is "a countered spell is put
    /// into its owner's graveyard"; cards like Memory Lapse / Remand /
    /// Spell Crumple print an "instead" clause that overrides this. The
    /// on-stack card is removed from the stack and placed into `zone`
    /// (top of library for Memory Lapse; exile for Spell Crumple; owner's
    /// hand for Remand).
    CounterSpellToZone {
        what: Selector,
        zone: CounteredSpellZone,
    },
    /// Ashiok's Erasure — counter target spell, exile it linked to the source
    /// (returns to its owner's hand when the source leaves), and stamp the
    /// source's `named_card` so its `OpponentsCantCastNamed` static locks that
    /// name while the source stays on the battlefield.
    CounterSpellExileNameLock { what: Selector },
    /// Counter target activated/triggered ability. The selector resolves
    /// to a permanent (the ability's source), and the engine removes the
    /// topmost `StackItem::Trigger` whose `source` matches. Used by
    /// Consign to Memory.
    CounterAbility { what: Selector },
    /// "Counter target activated ability from an artifact source and destroy
    /// that artifact if it's on the battlefield" (Ouphe Vandals) — the
    /// `CounterAbility` sibling that also kills the source.
    CounterAbilityAndDestroySource { what: Selector },
    /// Counter target spell, activated ability, or triggered ability (Voidslime).
    /// The one selector may resolve to a stack spell (matched by card id) or an
    /// ability's source (matched like `CounterAbility`); whichever kind the
    /// target names is removed. Uncounterable spells are skipped.
    CounterSpellOrAbility { what: Selector },
    /// Counter target spell **unless** its controller pays `mana_cost`.
    /// At resolution, the engine attempts to auto-pay on behalf of the
    /// targeted spell's controller — if affordable, the spell stays;
    /// otherwise it's countered. Used by Mystical Dispute (counter unless
    /// controller pays {3}). Spells flagged `uncounterable` are skipped.
    CounterUnlessPaid {
        what: Selector,
        mana_cost: crate::mana::ManaCost,
        /// CR — "If that spell is countered this way, exile it instead of
        /// putting it into its owner's graveyard." When true, a successful
        /// counter routes the spell to exile. Reject. Defaults to false.
        #[serde(default)]
        exile: bool,
        /// Extra generic pips, evaluated at resolution — "pays {X}, where X
        /// is its power" (Mausoleum Wanderer rides `SacrificedPower`).
        #[serde(default)]
        extra_generic: Option<Value>,
    },
    /// CR 702.21 — Ward's "counter that spell or ability unless its
    /// controller pays [cost]" trigger body. Walks the stack for the
    /// topmost `Spell` with `card.id == target` or `Trigger` with
    /// `source == target`, then tries to auto-pay the `cost` on behalf
    /// of that item's controller. If unpaid, the item is removed
    /// (spells go to graveyard; abilities just vanish off the stack).
    ///
    /// Distinct from `CounterUnlessPaid` because (a) it also counters
    /// activated/triggered abilities (for the "or ability" half of CR
    /// 702.21a), and (b) the cost menu is the broader
    /// `WardCost` (mana / life / discard / sacrifice creature).
    CounterUnless {
        what: Selector,
        cost: crate::card::WardCost,
    },
    /// "Unless [who] pays [cost], [then]." The generic Rhystic-tax rider:
    /// resolve `who` to a player (typically `PlayerRef::Triggerer` — the
    /// opponent who cast the spell / drew the card), ask them yes/no whether to
    /// pay `cost`, and if they decline or can't afford it, resolve `then` (in
    /// the trigger's own context, so `PlayerRef::You` in `then` is the rider's
    /// controller). Powers Rhystic Study, Mystic Remora, Esper Sentinel
    /// (draw), Smothering Tithe (Treasure), Indulgent Tormentor, etc. The
    /// AutoDecider declines to pay by default (the conservative line — let the
    /// effect happen). When `who` resolves to no one, `then` resolves outright.
    UnlessPlayerPays {
        who: PlayerRef,
        cost: crate::card::WardCost,
        then: Box<Effect>,
        /// The mirror branch — "if any player pays {2}, discard three cards"
        /// (Rhystic Scrying). Runs only when the tax was actually paid.
        #[serde(default)]
        if_paid: Option<Box<Effect>>,
    },
    /// Copy target spell/ability `count` times.
    CopySpell    { what: Selector, count: Value },
    /// As `CopySpell`, but when the copied spell is a permanent spell the
    /// resulting token permanent carries riders: `grant_haste` gives it
    /// haste until end of turn, and `sacrifice_eot` schedules a
    /// next-end-step sacrifice. Choreographed Sparks — "Copy target
    /// creature spell you control. The copy gains haste and 'At the
    /// beginning of the end step, sacrifice this token.'" The riders are
    /// stamped on the copy's `CardInstance.resolve_riders` and applied by
    /// the permanent-spell resolution path in `stack.rs`; they no-op for
    /// instant/sorcery copies.
    CopySpellWithRiders {
        what: Selector,
        count: Value,
        grant_haste: bool,
        sacrifice_eot: bool,
    },
    /// Gogo — copy target activated or triggered ability on the stack
    /// `times` times (the selector resolves to the ability's *source*
    /// permanent, mirroring `CounterAbility`). Copies keep the original's
    /// targets; the printed "you may choose new targets" is auto-kept.
    CopyAbility { what: Selector, times: Value },
    /// Copy target spell **unless** its caster pays `mana_cost`. Used by
    /// Wandering Archaic ("Whenever an opponent casts an instant or sorcery
    /// spell, that player may pay {2}. If they don't, you may copy the
    /// spell."). The resolver: (a) walks the stack for the spell whose
    /// `card.id` matches `what`; (b) asks the spell's caster yes/no via
    /// `Decision::OptionalTrigger`; (c) if they accept *and* can afford
    /// `mana_cost` from their pool, deducts it and skips the copy; (d) if
    /// they decline or can't afford, copies the spell `count` times above
    /// it on the stack.
    ///
    /// AutoDecider's default answer is `false` (decline to pay) — the
    /// printed Oracle implies most casters won't have an extra {2}
    /// floating, so the conservative default is "let the copy happen."
    /// ScriptedDecider can override via `DecisionAnswer::Bool(true)` for
    /// tests that want to exercise the pay path.
    CopySpellUnlessPaid {
        what: Selector,
        mana_cost: crate::mana::ManaCost,
        count: Value,
    },
    /// Copy target spell `count` times, then "you may choose new targets
    /// for the copy/copies" (CR 707.12 + 115.7). Same resolver as
    /// `CopySpell`, but after each copy is pushed the controller's decider
    /// is consulted (`Decision::ChooseTarget`, original target offered
    /// first) to re-point the copy's primary target at any legal object.
    /// Reverberate / Fork / Twincast. AutoDecider keeps the original
    /// target (first legal); a scripted/UI decider can repoint it.
    CopySpellMayChooseTargets { what: Selector, count: Value },
    /// CR 706 — the Onslaught Chain cycle's rider: "Then `who` may copy this
    /// spell and may choose a new target for that copy." The copy is controlled
    /// by `who`, who retargets it, so a chain can bounce around the table.
    /// `cost` is the printed toll paid first ("may sacrifice a land",
    /// "may discard a card"); a declined or unpayable toll ends the chain.
    /// Asked through the installed decider — the target seat's own UI prompt is
    /// a follow-up (see TODO.md's decision-plumbing audit).
    MayCopyThisSpell { who: PlayerRef, cost: ChainCopyCost },
    /// CR 702.18 — "until end of turn, this permanent can be the target of
    /// spells and abilities controlled by `player` as though it didn't have
    /// shroud" (Autumn Willow). Waives shroud on `ctx.source` for that seat
    /// only; cleared at cleanup.
    WaiveShroudForPlayerThisTurn { player: PlayerRef },
    /// "Destroy each [filter] unless its controller pays `life` life" — one
    /// pay-or-die decision per permanent, asked of its controller (Giant
    /// Albatross). `no_regen` denies regeneration (CR 701.15g).
    DestroyEachUnlessPaysLife {
        filter: SelectionRequirement,
        life: u32,
        #[serde(default)]
        no_regen: bool,
    },
    /// Zada, Hedron Grinder — the triggering spell targets only this source, so
    /// copy it once per *other* creature its controller controls that the spell
    /// could legally target, each copy aimed at a different one. No-op when the
    /// spell targets anything besides the source.
    CopyForEachOtherTargetableCreature,
    /// CR 115.7c — "You may choose new targets for target spell." Repoints
    /// every declared slot of the targeted spell in place (Redirect), each
    /// against its own printed filter. This effect's controller (the
    /// redirector) chooses via `Decision::ChooseTarget`. Unlike
    /// `CopySpellMayChooseTargets` this mutates the original spell.
    ChooseNewTargetsForSpell { what: Selector },
    /// Psychic Battle — each player reveals the top card of their library; the
    /// player who revealed the greatest mana value (uniquely) may repoint the
    /// targets of the spell that just chose them. A tie changes nothing.
    RevealTopGreatestMayChangeTargets,
    /// Demonstrate (CR 702.150) — copy this spell for its caster, then pick an
    /// opponent who also copies it; every copy may choose new targets. Modeled
    /// as a non-optional "always demonstrate" (the printed "you may" collapses
    /// since copying a beneficial spell is virtually always correct). Driven
    /// off a `SpellCast`/`SelfSource` self-cast trigger so both copies land
    /// above the original.
    Demonstrate,

    // ── Cast-without-paying / may-play ───────────────────────────────────────
    /// "Until [duration], you may cast/play that card [from where it is]."
    /// Stamps `CardInstance.may_play_until` on every card matched by `what`
    /// (typically a single target in graveyard or exile). The granted
    /// player then invokes `GameAction::CastFromZoneWithoutPaying` during
    /// a sorcery-speed (or instant-speed if the card is an instant) window
    /// to actually cast it.
    ///
    /// `to_owner` flips the recipient from "this effect's controller" to
    /// "the matched card's owner" — used by Suspend Aggression's "its
    /// owner may play it until the end of their next turn."
    ///
    /// `exile_after` propagates to the permission so casts pay this off
    /// land the resolved instant/sorcery in exile (Nita, The Dawning
    /// Archaic). For permanent spells the flag is ignored — they enter
    /// the battlefield normally.
    GrantMayPlay {
        what: Selector,
        duration: crate::card::MayPlayDuration,
        #[serde(default)]
        to_owner: bool,
        #[serde(default)]
        exile_after: bool,
        /// "You may cast that card" *paying its cost* (Hostage Taker) —
        /// stamps the card's own mana cost as the granted alt-cast cost so
        /// the may-play cast isn't free.
        #[serde(default)]
        pay_own_cost: bool,
        /// "…and you may spend mana as though it were mana of any type to
        /// cast it" (Gonti, Hostage Taker): the stamped alt-cast cost is
        /// the card's mana value as generic, payable by any colors.
        #[serde(default)]
        any_color: bool,
    },
    /// Stamp a conditional surcharge onto cards just granted a may-play
    /// permission (chain after `GrantMayPlay` in a `Seq`): "it costs
    /// [cost] more to cast this way unless the spell targets a permanent
    /// matching [filter]" — Mavinda, Students' Advocate's {8} rider.
    /// Writes `CardInstance.granted_cast_surcharge_eot`; consumed by the
    /// permission-cast path at target-validation time.
    StampMayPlaySurcharge {
        what: Selector,
        cost: crate::mana::ManaCost,
        filter: crate::card::SelectionRequirement,
    },
    /// Grant a one-shot permission to cast `what`'s MDFC **back face from the
    /// graveyard**, paying the back's cost (Pestilent Cauldron — "sacrifice
    /// this, then you may cast Restorative Burst transformed"). Sets the
    /// `may_cast_back_from_graveyard` flag on the resolved card; the controller
    /// then casts the back via `GameAction::CastSpellBack`, which hops the card
    /// out of the graveyard and consumes the permission. No-op if `what` has no
    /// back face.
    GrantCastBackFromGraveyard { what: Selector },
    /// Resolve-now equivalent of `GrantMayPlay`: at effect resolution
    /// time, ask the controller "cast `what` without paying its mana
    /// cost?" via `Decision::OptionalTrigger`. On yes, the card is
    /// pushed through the free-cast helper from `source_zone` with
    /// auto-targets / auto-decisions; on no (or no match), nothing
    /// happens.
    ///
    /// Used by Improvisation Capstone (each exiled non-land card),
    /// The Dawning Archaic (attack trigger), Nita Forum Conciliator
    /// (could use either model; we use this for the trigger half).
    /// `source_zone` is `Graveyard` for "cast it from your graveyard"
    /// and `Exile` for "cast it from exile."
    CastWithoutPayingImmediate {
        what: Selector,
        source_zone: crate::card::Zone,
        #[serde(default)]
        exile_after: bool,
        /// Cast a *copy* of the card instead — the original stays put
        /// (Capricious Hellraiser, CR 707.12).
        #[serde(default)]
        copy: bool,
        /// When non-zero the cast isn't free: the controller pays the card's
        /// own mana cost reduced by this much generic ("that copy costs {2}
        /// less to cast" — God-Eternal Kefnet). Declining, or being unable to
        /// pay, skips the cast.
        #[serde(default)]
        reduce_generic: u32,
        /// The cast isn't free at all — the controller pays the card's own
        /// cost (minus `reduce_generic`). "Copy them. You may cast any number
        /// of the copies" (The Tale of Tamiyo IV).
        #[serde(default)]
        pay_own_cost: bool,
    },
    /// "You may cast any number of spells from among them without paying
    /// their mana costs" — repeatedly offers the remaining castable cards
    /// (a declined card is re-offered after each accepted cast), so the
    /// controller effectively picks the CAST ORDER; the loop ends after a
    /// full pass with no accepts. Lands are skipped (played, not cast).
    /// Improvisation Capstone.
    /// `filter` narrows what may be cast this way — Epic Experiment's
    /// "instant and sorcery spells with mana value X or less". `None` offers
    /// every nonland card.
    CastAnyOrderWithoutPaying {
        what: Selector,
        source_zone: crate::card::Zone,
        #[serde(default)]
        filter: Option<SelectionRequirement>,
        /// Cap on how many may be cast ("a spell from each opponent's
        /// graveyard" — Jetsam). `None` is the printed "any number".
        #[serde(default)]
        cap: Option<Value>,
    },
    /// "You may cast a [filter] spell from your hand without paying its
    /// mana cost" (Maelstrom Archangel; Oracle of Bones restricts to
    /// instants/sorceries). The controller picks one matching nonland hand
    /// card via `Decision::ChooseCards` (min 0 — may decline; AutoDecider
    /// declines) and it's free-cast with auto-targets.
    CastFromHandWithoutPaying {
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },
    /// CR 702.104 — Tribute N. An opponent may put N +1/+1 counters on the
    /// source as it enters; if they decline, `otherwise` runs. The opponent
    /// answers via `Decision::OptionalTrigger` (synchronous decider, like
    /// TemptingOffer; AutoDecider declines, so the trigger half fires).
    /// Run as a SelfSource ETB trigger — `shortcut::tribute`.
    Tribute { n: u32, otherwise: Box<Effect> },
    /// Paradigm (SOS supplemental keyword) — registers a non-one-shot
    /// `DelayedKind::YourNextMainPhase` trigger that on each of the
    /// controller's pre-combat main phases offers them "cast a copy of
    /// this from exile without paying its mana cost?" via
    /// `Effect::CastFreeParadigmCopy`. Used as the trailing effect of
    /// the Paradigm Lesson cycle (Restoration Seminar, Decorum
    /// Dissertation, Germination Practicum, Echocasting Symposium,
    /// Improvisation Capstone). Pairs with `exile_on_resolve = true` so
    /// the card lands in exile and stays reachable for the recurring
    /// copy trigger.
    RegisterParadigm,
    /// Paradigm body. At trigger-fire time, locates the trigger's
    /// `source` (the Paradigm-exiled card) in exile, asks the controller
    /// "cast a copy?" via OptionalTrigger, and on yes mints a tokenized
    /// copy of the card's definition + free-casts it with auto-targets.
    /// The original exiled card is left untouched so the recurrence
    /// continues each main phase.
    CastFreeParadigmCopy,

    /// Cipher (CR 702.46). Trailing effect of a Cipher spell: the controller
    /// may exile this spell card "encoded" on a creature they control (stamping
    /// `CardInstance.encoded_on`). The combat-damage-to-player dispatch then
    /// offers a free copy whenever that creature connects. Sets a pending flag
    /// consumed by `continue_spell_resolution` so the card routes to exile
    /// instead of the graveyard.
    Cipher,

    /// Cascade (CR 702.85). Triggered "when you cast this spell": exile
    /// cards from the top of the controller's library until a nonland
    /// card with mana value strictly less than `max_mv` is exiled. The
    /// controller may cast that card without paying its mana cost. The
    /// remaining exiled cards go to the bottom of the library (random
    /// order — approximated as bottom, the same indistinguishable model
    /// `RevealMissDest::BottomRandom` uses, since the bottom ordering is
    /// hidden until the next shuffle/reveal).
    ///
    /// `max_mv` is the cascading spell's mana value. Card factories pass
    /// `Value::Const(printed_mv)` (cascade's MV gate is the printed cost,
    /// unaffected by cost reduction per CR 702.85b). The shortcut
    /// [`cascade`] wires the standard SpellCast/SelfSource trigger.
    Cascade { max_mv: Value },
    /// CR 702.20 — Ripple N. The source spell's cast trigger: reveal the top N
    /// cards of your library; you may cast any with the same name as the source
    /// for free; put the rest on the bottom. Cast-from-library recursion (a
    /// rippled copy ripples again) falls out of the cast path naturally — the
    /// shortcut [`ripple`] wires the SpellCast/SelfSource trigger.
    Ripple { n: Value },

    /// Exile the top card of `who`'s library and stamp a may-play
    /// permission on it for `duration`. Used by Conspiracy Theorist,
    /// Elemental Mascot, Ark of Hunger, Archaic's Agony and similar
    /// "exile top of library; until [end of next turn / end of turn] you
    /// may play that card" effects. A single-shot composite that
    /// combines `Move(TopOfLibrary → Exile)` + `MayPlayPermission`
    /// stamp atomically so the may-play targets the just-moved card.
    /// `count` exiles that many cards off the top (Fallen Shinobi peels
    /// two), stamping each with the may-play permission.
    ExileTopAndGrantMayPlay {
        who: PlayerRef,
        count: Value,
        duration: crate::card::MayPlayDuration,
        /// "You may cast" *paying* the card's cost as generic — the
        /// any-type-mana pay-to-cast rider (Nassari, Dean of Expression).
        /// `false` keeps the free-cast grant (Robber of the Rich).
        #[serde(default)]
        pay_any_color: bool,
        /// Only cards at or under this mana value get the permission — the
        /// rest stay exiled with no grant (Kotis, the Fangkeeper's "spells
        /// with mana value X or less"). `None` grants on every exiled card.
        #[serde(default)]
        max_mana_value: Option<Value>,
        /// "You may play them" *paying their own mana cost* — the plain
        /// impulse-draw grant (Light Up the Stage, Reckless Impulse,
        /// Wrenn's Resolve). Stamps the card's actual cost as its alt-cast
        /// cost so the may-play isn't a free cast. Mutually exclusive with
        /// `pay_any_color`; `false` keeps the free-cast default.
        #[serde(default)]
        pay_own_cost: bool,
        /// "If you don't [cast it], …" fallback (Chandra, Torch of
        /// Defiance's +1). Registers a next-end-step delayed trigger that
        /// runs this body if the exiled card is still in exile (uncast).
        #[serde(default)]
        uncast_penalty: Option<Box<Effect>>,
    },

    /// "Exile the top card of your library. If it's a land, create `token`.
    /// Otherwise, you may play it until the end of your next turn (paying its
    /// own cost)." The exiled land stays exiled. Bruse Tarl, Roving Rancher.
    ExileTopLandTokenElseMayPlay {
        token: TokenDefinition,
    },

    /// "Look at the top card of `library`'s library and exile it face down.
    /// `grantee` may play it for as long as it remains exiled, and mana of
    /// any type can be spent to cast it" (Gonti, Night Minister). The
    /// any-type spend is the card's mana value as generic (CR 609.4b), the
    /// same equivalence Gonti, Lord of Luxury uses.
    ExileTopFaceDownGrantPlay { library: PlayerRef, grantee: PlayerRef },

    /// CR 614 — "As this enters, exile up to `count` cards matching `filter`
    /// from your graveyard." Each exiled card is stamped `exiled_with =
    /// source`, so `Selector::CardExiledWithSource` and
    /// `Value::CardsExiledWithSourceCount` see them (Mimeoplasm, Revered One).
    AsEntersExileFromYourGraveyard { count: Value, filter: SelectionRequirement },

    /// CR 707.2 — "this permanent becomes a copy of `what`, except it's
    /// `base_pt` and has this ability." `what` picks a card exiled with the
    /// source; the granting activated ability is re-appended so it can copy
    /// again (Mimeoplasm, Revered One).
    BecomeCopyOfExiledCard { what: Selector, base_pt: Option<(i32, i32)> },

    // ── Sacrifice ────────────────────────────────────────────────────────────
    Sacrifice { who: Selector, count: Value, filter: SelectionRequirement },
    /// Each player picked by `who` sacrifices **all** permanents they control
    /// matching `filter` — no choice involved (CR 701.16). All Is Dust's
    /// "each player sacrifices all colored permanents they control".
    SacrificeAllMatching { who: Selector, filter: SelectionRequirement },
    /// Sacrifice each permanent `what` resolves to, by its own controller
    /// (CR 701.16). The targeted sibling of `SacrificeAllMatching`.
    SacrificeSelected { what: Selector },
    /// CR 701.16 — "Each player sacrifices all [`filter`] they control except
    /// for `keep`" (Keldon Firebombers). Each player keeps their `keep`
    /// highest-mana-value matches; the rest go.
    EachPlayerSacrificesDownTo { filter: SelectionRequirement, keep: Value },
    /// Living Death (CR — mass reanimation swap): each player exiles all
    /// creature cards from their graveyard, then sacrifices all creatures they
    /// control, then puts the cards they exiled this way onto the battlefield
    /// under their own control.
    LivingDeath,
    /// "If this is the first time this ability has resolved this turn, [mode 0].
    /// If the second time, [mode 1]. …" (CR 603-style escalation — Vito,
    /// Fanatic of Aclazotz). Runs `modes[min(n, len-1)]` where `n` is the count
    /// of prior resolutions this turn keyed by the source, then increments.
    EscalatingThisTurn { modes: Vec<Effect> },
    /// Bringer of the Last Gift's ETB: each player sacrifices all creatures
    /// they control *except the source*, then each player returns all creature
    /// cards that were already in their graveyard (not put there this way) to
    /// the battlefield under their control.
    SacrificeOthersThenReanimate,
    /// Each player, in APNAP order, may put one permanent card matching
    /// `filter` from their hand onto the battlefield (Show and Tell); with
    /// `others_only` the source's controller is skipped (Hunted Wumpus,
    /// Charmed Griffin). Every seat auto-picks its highest-mana match —
    /// the "may" isn't a real prompt yet (tracked in TODO.md).
    EachPlayerMayPutPermanentFromHand {
        filter: SelectionRequirement,
        #[serde(default)]
        others_only: bool,
        /// Eureka — "repeat this process until no one puts a card onto the
        /// battlefield". Loops the whole APNAP pass while any seat played.
        #[serde(default)]
        repeat: bool,
    },
    /// "`who` exiles `count` permanents they control of their choice" — the
    /// exile analogue of Annihilator's forced sacrifice (Bane of Bala Ged).
    /// The affected player chooses which permanents; a `wants_ui` player with
    /// a genuine choice is prompted, bots/no-choice auto-pick the weakest.
    PlayerExilesPermanents { who: PlayerRef, count: Value, filter: SelectionRequirement },
    /// "`who` returns `count` permanents they control of their choice to
    /// their owner's hand" — the bounce analogue of
    /// `PlayerExilesPermanents` (Devastating Mastery's alt-cost rider,
    /// Multiple Choice's X=2 bullet). `up_to` makes the count optional
    /// ("up to two"); the auto path returns the weakest matches.
    PlayerReturnsPermanentsToHand {
        who: PlayerRef,
        count: Value,
        filter: SelectionRequirement,
        up_to: bool,
    },
    /// Sacrifice this effect's source permanent (CR 701.16), firing proper
    /// death triggers. Used by end-of-turn self-sacrifice (Blitz, Ball
    /// Lightning) where `Effect::Move { This → Graveyard }` would skip the
    /// `CreatureDied` event.
    SacrificeSource,
    /// Exile this effect's source permanent (no death trigger). Used by
    /// temporary tokens exiled at the next end step (Manaform Hellkite's
    /// Dragon Illusion, Kari Zev's Ragavan-style temps).
    ExileSource,
    /// Sacrifice the specific permanent(s) named by `what` (CR 701.16), firing
    /// proper sacrifice + death triggers. Unlike `Effect::Sacrifice` (which
    /// makes a player choose `count` matching permanents) this sacrifices an
    /// already-chosen object — e.g. a creature reanimated this turn that a
    /// delayed trigger must sacrifice at the next end step (Footsteps of the
    /// Goryo, Apprentice Necromancer). The sacrificing player is each target's
    /// own controller.
    SacrificePermanent { what: Selector },
    /// "Sacrifice this permanent unless you sacrifice a [filter]." (The Gitrog
    /// Monster's upkeep cost.) The controller may sacrifice one permanent
    /// matching `filter` they control to spare the source; if they have none —
    /// or a UI seat declines — the source is sacrificed instead. AutoDecider
    /// keeps the source by sacrificing the weakest matching permanent when one
    /// is available.
    SacrificeSourceUnlessSacrifice { filter: SelectionRequirement },
    /// "Sacrifice this permanent unless you return a [`filter`] you control to
    /// its owner's hand" (the Invasion-block Lair lands). The bounce sibling of
    /// `SacrificeSourceUnlessSacrifice`.
    SacrificeSourceUnlessReturn { filter: SelectionRequirement },
    /// "Sacrifice this permanent unless you pay [cost]" over the full
    /// [`crate::card::WardCost`] menu — discard, exile-from-graveyard, life,
    /// mana. Rotting Giant, Cursed Monstrosity. A UI seat gets a yes/no;
    /// the AutoDecider pays whenever the cost is payable.
    SacrificeSourceUnlessCost { cost: crate::card::WardCost },
    /// "Sacrifice any number of [filter]. [payoff] for each one." The
    /// controller chooses how many to sacrifice via `Decision::ChooseAmount`
    /// (AutoDecider sacrifices none). For each sacrifice, `per_each` runs
    /// once — so a `GainLife 3` body pays 3 × count. Plunge into Darkness.
    SacrificeAnyNumber {
        who: PlayerRef,
        filter: SelectionRequirement,
        per_each: Box<Effect>,
    },
    /// "Pay any amount of life. Look at that many cards from the top of your
    /// library, put one into your hand, and exile the rest." The controller
    /// chooses the amount via `Decision::ChooseAmount` (capped at current
    /// life; AutoDecider pays 0). Plunge into Darkness mode 1.
    PayLifeLookTake { who: PlayerRef },
    /// Vizkopa Confessor — "Pay any amount of life. Target opponent reveals
    /// that many cards from their hand. You choose one of them and exile it."
    /// The controller pays N via `Decision::ChooseAmount` (AutoDecider pays 0),
    /// then `opp` reveals their cheapest N cards (modeling their free choice of
    /// which to reveal) and the controller exiles one. `opp` is `EachOpponent`
    /// (exact in 1v1; the "target" nuance is dropped in multiplayer).
    PayLifeRevealExileFromHand { opp: PlayerRef },
    /// "You may pay any amount of life. If you do, draw that many cards."
    /// Amount via `Decision::ChooseAmount`, capped at current life
    /// (AutoDecider pays 0). Necrodominance's end step.
    PayLifeDraw { who: PlayerRef },
    /// CR 701.30 — "Clash with an opponent": you and the lowest-seat
    /// opponent each reveal your library's top card and may bottom it
    /// (synchronous decider); if yours had the higher mana value, `on_win`
    /// runs. Recross the Paths.
    ClashWithOpponent { on_win: Box<Effect> },
    /// Goblin Charbelcher: reveal from the top until a land; deal damage
    /// equal to the nonland reveals to `to` (doubled when the land has
    /// `double_if`'s subtype); all reveals go to the bottom.
    RevealUntilLandDamage { to: Selector, double_if: Option<crate::card::LandType> },
    /// Calibrated Blast: reveal from the top until a nonland card, deal
    /// damage equal to that card's mana value to `to`, and bottom the
    /// reveals in a random order.
    RevealUntilNonlandDamage { to: Selector },

    /// Head Games — `who` puts their hand on top of their library, then this
    /// effect's controller searches that library for that many cards and puts
    /// them into `who`'s hand. The library is shuffled afterwards.
    HeadGames { who: PlayerRef },

    /// Trade Secrets — `who` draws two, then you draw up to four; `who` may
    /// repeat the loop as many times as they choose (capped so a bot loop
    /// terminates on an empty library).
    TradeSecrets { who: PlayerRef },

    /// Strongarm Tactics — each player discards a card, then each player who
    /// did not discard a creature card loses `life` life.
    EachPlayerDiscardsElseLosesLife { life: u32 },

    /// Kamahl's Summons — each player may reveal any number of creature cards
    /// from their hand, then creates `token` once per card revealed this way.
    EachPlayerRevealsCreaturesForTokens { token: Box<crate::card::TokenDefinition> },

    /// False Cure — "until end of turn, whenever a player gains life, that
    /// player loses `per` life for each 1 life gained." A turn-scoped watcher
    /// over *every* seat, unlike the controller-scoped
    /// [`Effect::WheneverYouGainLifeThisTurn`].
    AnyLifeGainPunishedThisTurn { per: u32 },

    /// The general shape of [`Effect::RevealUntilNonlandDamage`]: reveal from
    /// the top until a nonland card, bottom the reveals in a random order,
    /// then run `then` with the revealed card's mana value published as
    /// [`Value::LastRevealedManaValue`]. Goblin Machinist, Kaboom!.
    RevealUntilNonlandThen { then: Box<Effect> },

    /// "Choose a creature type, then [`then`]" — the resolution-time sibling
    /// of [`Effect::NameCreatureType`]. Stamps the pick on the source's
    /// `chosen_creature_type` (so `SelectionRequirement::
    /// IsSourceChosenCreatureType` and friends resolve inside `then`) and
    /// runs the body. Riptide Chronologist, Riptide Shapeshifter, Walking
    /// Desecration, Peer Pressure.
    ChooseCreatureTypeThen { who: PlayerRef, then: Box<Effect> },

    /// "Each player chooses a creature type. [`then`]" — every seat picks in
    /// APNAP order and the union is published for the body through
    /// `SelectionRequirement::IsTypeChosenThisWay`. Harsh Mercy,
    /// Patriarch's Bidding.
    EachPlayerChoosesCreatureTypeThen { then: Box<Effect> },
    /// Skyserpent Seeker-style ramp: reveal from the top of your library until
    /// you reveal `count` land cards; put those lands onto the battlefield
    /// (`tapped`), and put the rest on the bottom of your library in a random
    /// order. Stops early if the library runs out.
    RevealUntilLandsToBattlefield { count: Value, tapped: bool },
    /// "Until your next turn, whenever a creature attacks you or a
    /// planeswalker you control, [body]" — registers a floating trigger;
    /// the attacker is bound as `Selector::TriggerSource`. Tamiyo +2.
    OnAttackedUntilYourNextTurn { body: Box<Effect> },
    /// "Until end of turn, whenever a creature matching `filter` attacks,
    /// [body]" — registers a turn-scoped floating trigger fired by any
    /// player's qualifying attacker; the attacker is bound as
    /// `Selector::TriggerSource`. Summon: Leviathan II/III.
    OnMatchingAttacksThisTurn { filter: SelectionRequirement, body: Box<Effect> },
    /// "Whenever a creature blocks this turn, its controller gets `amount`
    /// poison counters" (Noxious Assault). Turn-scoped flag consumed at
    /// blocker declaration.
    BlockersPoisonedThisTurn { amount: u32 },
    /// Stagger (Lightning, Army of One) — until your next turn, if a source
    /// would deal damage to `who` or a permanent they control, it deals
    /// double that damage instead (CR 614.5-style replacement).
    StaggerPlayerUntilYourNextTurn { who: PlayerRef },
    /// "Sacrifice a [filter] with the greatest mana value" picker.
    /// Mirrors `Sacrifice` but the candidate sort prefers maximum CMC.
    /// Used by Soul Shatter ("Each opponent sacrifices a creature or
    /// planeswalker with the greatest mana value among permanents
    /// that player controls"). Auto-decider picks the highest-CMC
    /// matching permanent per player. When `by_power` is set the sort
    /// key is greatest *power* instead (Crackling Doom, Tribute to Hunger).
    SacrificeGreatestMV {
        who: Selector,
        count: Value,
        filter: SelectionRequirement,
        #[serde(default)]
        by_power: bool,
    },

    /// "Punisher" choice (CR 601-style "unless"). Each player `chooser`
    /// resolves to may avoid `otherwise` by performing one of `options`.
    /// The engine resolves heuristically: that player performs the first
    /// option they can afford (LoseLife within their life total, Sacrifice
    /// with a legal permanent); if none is affordable, `otherwise` runs
    /// for the ability's controller. Options run with the chooser as the
    /// effect controller, so they use `Selector::Player(PlayerRef::You)`.
    /// Indulgent Tormentor: each opponent pays 3 life or sacrifices a
    /// creature, otherwise the controller draws a card.
    Punisher {
        chooser: Selector,
        options: Vec<Effect>,
        otherwise: Box<Effect>,
    },

    /// CR 701.55 — "[player] faces a villainous choice — [A], or [B]." Each
    /// resolved `who` (in APNAP order) chooses one option, then that option's
    /// actions are performed with the chooser as effect controller (so
    /// `PlayerRef::You` = the chooser). Unlike `Punisher` there is no fallback
    /// payoff: both options are genuine choices. The bot heuristic picks the
    /// lower-`self`-harm option; UI-player prompting is a follow-up (TODO.md).
    VillainousChoice {
        who: Selector,
        option_a: Box<Effect>,
        option_b: Box<Effect>,
    },

    // ── Counters on players ──────────────────────────────────────────────────
    AddPoison { who: Selector, amount: Value },
    /// CR 122.1i / 728 — give each resolved player `amount` rad counters.
    AddRadCounters { who: Selector, amount: Value },

    // ── Misc atomic operations needed by existing cards ──────────────────────
    /// Reveal the top card of `who`'s library; if `reveal_filter` matches, draw it.
    RevealTopAndDrawIf {
        who: PlayerRef,
        reveal_filter: SelectionRequirement,
        /// On a miss, the player may put the revealed card into their
        /// graveyard instead of leaving it on top (Nylea, Keen-Eyed).
        #[serde(default)]
        may_graveyard_miss: bool,
    },

    /// Reveal the top card of `who`'s library (fires `TopCardRevealed` event for
    /// the animation) without moving it. Used by Chaos Warp's "reveal top card"
    /// step where the put-onto-battlefield clause is handled separately.
    RevealTopCard { who: PlayerRef },

    /// Reveal the top card of `who`'s library; if it matches `filter`, run
    /// `then`. The card stays on top either way. The CHK "Deceiver" cycle
    /// (reveal top, if a land then pump/grant the source — Brutal/Cruel/Feral/
    /// Callous Deceiver).
    RevealTopThenIf {
        who: PlayerRef,
        filter: SelectionRequirement,
        then: Box<Effect>,
    },

    /// Reveal the top card of `who`'s library; if it's a permanent card, put it
    /// onto the battlefield under its owner's control (firing ETB). Otherwise
    /// it stays on top. Chaos Warp.
    RevealTopPutPermanentOntoBattlefield { who: PlayerRef },
    /// Reveal the top card of `who`'s library. If it matches `filter`, they may
    /// put it onto the battlefield with a `counter` counter on it and
    /// `extra_types` added to its types; a miss or a decline leaves it on top.
    /// Arbiter of the Ideal.
    RevealTopMayPutOntoBattlefield {
        who: PlayerRef,
        filter: SelectionRequirement,
        #[serde(default)]
        counter: Option<crate::card::CounterType>,
        #[serde(default)]
        extra_types: Vec<crate::card::CardType>,
    },
    /// Reveal the top `count` cards; put every card matching `filter` onto the
    /// battlefield under `who`'s control, the rest on the bottom of the library.
    /// Gishath, Sun's Avatar's combat-damage trigger (reveal = damage dealt,
    /// filter = Dinosaur creature card). "Any number" is auto-take-all.
    RevealTopNPutMatchingToBattlefield { who: PlayerRef, count: Value, filter: SelectionRequirement },

    /// Reveal the top card of `who`'s library; if it's a land, put it onto the
    /// battlefield (untapped). Otherwise put it into their hand. Coiling
    /// Oracle, Growth Spiral, Llanowar Loamspeaker-style ramp.
    RevealTopLandToBattlefieldElseHand { who: PlayerRef },

    /// Look at the top card of `who`'s library; if it's a land, put it into
    /// their hand, otherwise mill it (surveil-flavored dig). Both printed
    /// "may" choices are auto-taken (the player-favorable line). Traveling
    /// Botanist's "becomes tapped" trigger.
    LookTopLandToHandElseBin { who: PlayerRef },

    /// Reveal the top card of `who`'s library; if it's a permanent card with
    /// mana value ≤ `max_mv`, put it onto the battlefield; otherwise put it
    /// into their hand. Matter Reshaper's death trigger.
    RevealTopPutPermanentMvElseHand { who: PlayerRef, max_mv: Value },

    /// "You may put a [filter] card from your hand onto the battlefield." The
    /// controller picks up to `count` matching cards from `who`'s hand (via
    /// `ChooseCards`, min 0 — always optional) and they enter under the
    /// controller's control, `tapped` if set. With `haste`, each entrant gains
    /// haste until end of turn; with `sacrifice_eot`, each is sacrificed at the
    /// beginning of the next end step (Sneak Attack, Through the Breach). Plain
    /// form (no haste / no sac) ships Elvish Piper / Quicksilver Amulet.
    PutFromHandOntoBattlefield {
        who: PlayerRef,
        filter: SelectionRequirement,
        count: Value,
        tapped: bool,
        haste: bool,
        sacrifice_eot: bool,
        /// "At the beginning of the next end step, return that permanent to
        /// its owner's hand" (Surprise Deployment). The gentler sibling of
        /// `sacrifice_eot`; both may be set, but the bounce wins.
        #[serde(default)]
        return_eot: bool,
        /// "If you do, …" — runs only when at least one card was actually put
        /// onto the battlefield (Dermoplasm's bounce-itself rider).
        #[serde(default)]
        then: Option<Box<Effect>>,
    },

    /// CR 614.9 — "All damage that would be dealt to this creature this turn
    /// is dealt to `to` instead" (Karona's Zealot). Registers a turn-scoped
    /// redirect on the source; cleared at cleanup.
    RedirectDamageToThisThisTurn { to: Selector },

    /// CR 614 — "The next time `what` would deal combat damage this turn, it
    /// deals that damage to its controller instead" (Goblin Psychopath).
    RedirectNextCombatDamageToController { what: Selector },

    /// CR 614 — "The next time `what` would deal combat damage to an opponent
    /// this turn, it deals that damage to `to` instead" (Soltari Guerrillas).
    RedirectNextCombatDamageTo { what: Selector, to: Selector },

    /// CR 702.15 — "Target creature gains landwalk of each of the land types
    /// of the land sacrificed to activate this ability" (Excavator). Reads the
    /// activation's sacrificed permanent; a no-op when none was paid.
    GrantSacrificedLandTypesLandwalk { what: Selector, duration: Duration },

    /// No Quarter — on a `BecomesBlocked` / `Blocks` trigger, destroy the
    /// weaker half of the pair. `attacker_side` picks which one dies: `false`
    /// destroys the blocker when the attacker outclasses it, `true` destroys
    /// the attacker when the blocker does.
    DestroyBlockPairWeakerSide { attacker_side: bool },

    /// CR 614.9 — "All damage that would be dealt this turn by target spell is
    /// dealt to that spell's controller instead" (Reverberation). Keyed on the
    /// spell's card id, so it keeps redirecting after the spell has resolved.
    RedirectSpellDamageToItsController { what: Selector },

    /// Tap `what` and hold it down for as long as the source stays tapped
    /// (Phyrexian Gremlins). Registers a `PreventUntap` tied to the source.
    TapAndHoldWhileSourceTapped { what: Selector },

    /// Exile the resolving source with its `CardDefinition.exile_countdown`
    /// counters on it (All Hallow's Eve). The upkeep tick and the payoff live
    /// on the definition; this only starts the fuse.
    ExileSelfWithCountdown,

    /// CR 702.18 — "You gain shroud until end of turn" (Gilded Light). No
    /// player, `who` included, may target them for the rest of the turn.
    PlayerGainsShroudThisTurn { who: PlayerRef },

    /// Dimensional Breach — exile all permanents; for as long as any remain
    /// exiled this way, each player returns one they own to the battlefield
    /// at the beginning of their upkeep.
    DimensionalBreach,

    /// The per-upkeep half of [`Effect::DimensionalBreach`]: the active player
    /// returns one of the cards exiled by the source that they own.
    DimensionalBreachReturn,

    /// Day of the Dragons — exile all creatures you control, then create that
    /// many `token` tokens. The exiled cards come back when the source leaves.
    ExileYourCreaturesForDragons { token: crate::card::TokenDefinition },

    /// Parallel Thoughts — search your library for `count` cards, exile them
    /// in a face-down pile stamped to the source, and shuffle both piles.
    ExileFaceDownDrawPile { count: Value },

    /// "Lands you control gain '{T}: Add one mana of any color' until end of
    /// turn" (Divergent Growth) — a duration-scoped `GrantActivatedAbility`.
    GrantActivatedAbilityToMatching {
        filter: SelectionRequirement,
        ability: Box<crate::effect::ActivatedAbility>,
        duration: Duration,
    },

    /// "You may move any number of `kind` counters from this creature onto
    /// other creatures" (Forgotten Ancient). The controller distributes; the
    /// auto-decider spreads them evenly over the matching creatures.
    DistributeCountersFromSource { kind: crate::card::CounterType, filter: SelectionRequirement },

    /// "Reveal the card you drew. If it matches `filter`, `then`" (Primitive
    /// Etchings). Reads the draw that fired this trigger.
    RevealDrawnCardThenIf { filter: SelectionRequirement, then: Box<Effect> },

    /// "Players can't cast creature or planeswalker spells until the end of
    /// your next turn" (Single Combat). Registers a game-wide lock keyed to the
    /// controller, lifted at the end of their first turn after this resolves.
    LockCreatureAndPlaneswalkerCasts,

    /// Ugin, the Ineffable's +1: exile the top card of your library face down,
    /// create `token`, and when that token leaves the battlefield put the
    /// exiled card into its owner's hand.
    ExileTopFaceDownTokenReturns { token: crate::card::TokenDefinition },

    /// "You may put a creature card from your hand onto the battlefield tapped
    /// and attacking [the defender the source is attacking]. Return that
    /// creature to its owner's hand at the beginning of the next end step"
    /// (Ilharg, the Raze-Boar; Kaalia-style deploy). CR 508.4 — the creature
    /// enters attacking without being declared, so no attack triggers fire.
    DeployCreatureFromHandAttacking {
        filter: SelectionRequirement,
        /// `true` returns the creature to hand at the next end step; `false`
        /// leaves it (a permanent deploy).
        return_to_hand_eot: bool,
    },

    /// "Put up to `count` land cards from your hand and/or graveyard onto the
    /// battlefield tapped." Deploys as many lands as available (up to `count`),
    /// preferring the graveyard so hand lands stay playable. Worldsoul's Rage.
    DeployLandsFromHandAndGraveyard { count: Value },
    /// "Put up to `count` land cards from your hand onto the battlefield
    /// tapped" (The Gitrog, Ravenous Ride).
    PutLandsFromHandOntoBattlefieldTapped { count: Value },

    /// CR 701.34 — Manifest: put the top `amount` cards of `who`'s library
    /// onto the battlefield face down as 2/2 creatures (the real card is
    /// stashed and can be turned face up for its mana cost if it's a creature).
    Manifest { who: PlayerRef, amount: Value },
    /// CR 701.34 from the hand — the resolved player manifests `count`
    /// cards from their hand; if `controller_draws`, the effect's
    /// controller draws one card per card manifested (Kozilek, the Broken
    /// Reality).
    ManifestFromHand { who: Selector, count: Value, controller_draws: bool },
    /// Create a token and attach it to the resolved permanent (Role tokens
    /// — Wicked Role; CR 111.10). The token must be an Aura-style
    /// attachment; it enters attached.
    CreateTokenAttachedTo { target: Selector, definition: crate::card::TokenDefinition },
    /// Like `CreateTokenAttachedTo`, but mints one token per permanent the
    /// selector resolves to (CR 111.10) — "for each creature your opponents
    /// control, create a Cursed Role token attached to that creature"
    /// (Asinine Antics).
    CreateTokenAttachedToEach { target: Selector, definition: crate::card::TokenDefinition },
    /// Indomitable Creativity: destroy up to X chosen permanent targets
    /// matching `filter` (slots `0..X` from the cast's target list); for
    /// each destroyed this way its controller reveals from the top until an
    /// artifact or creature card, puts it onto the battlefield, and
    /// shuffles the rest in.
    DestroyTargetsPolymorph { filter: SelectionRequirement },
    /// Destroy X chosen permanent targets matching `filter` (slots `0..X`
    /// from the cast's target list) — the plain sibling of
    /// `DestroyTargetsPolymorph`. Heliod's Intervention mode 0.
    DestroyTargets { filter: SelectionRequirement },
    /// "Choose a land of each basic land type, then destroy those lands."
    /// (Sundering Titan.) The source's controller chooses; the engine
    /// auto-picks one land per basic type (Plains/Island/Swamp/Mountain/
    /// Forest), preferring an opponent's land, and destroys the union.
    DestroyLandOfEachBasicType,
    /// CR 702.77 — Champion a [filter]: exile another matching permanent you
    /// control linked to the source (returns when the source leaves), or
    /// sacrifice the source if you exile nothing. Mistbind Clique,
    /// Changeling Hero.
    Champion { filter: SelectionRequirement },
    /// Exile up to `count` cards from graveyards, chosen by the controller
    /// (Faerie Macabre). `of` restricts candidates to one player's graveyard
    /// ("that player's graveyard" — Skullsnatcher); `single` restricts the
    /// picks to a single graveyard ("from a single graveyard" — Rag Dealer:
    /// picks after the first must share the first pick's owner). A
    /// `wants_ui` controller picks via `ChooseCards`; the auto path takes
    /// the highest-MV opponent cards.
    ExileUpToNFromGraveyards { count: Value, of: Option<PlayerRef>, single: bool },
    /// Choose a color, exile the top `amount` cards of `who`'s library, and
    /// create one `token` per exiled card of the chosen color (Oona, Queen
    /// of the Fae).
    ExileTopMintPerChosenColor { who: Selector, amount: Value, token: crate::card::TokenDefinition },
    /// Spells cast by opponents of the effect's controller that match
    /// `filter` cost `{amount}` more until the controller's next turn
    /// (Elspeth Conquers Death chapter II). Cleared at the controller's
    /// untap step.
    SpellTaxUntilYourNextTurn { amount: u32, filter: SelectionRequirement },
    /// Cataclysm-family: each resolved player keeps one artifact, one
    /// creature, one enchantment, and one planeswalker from among the
    /// nonland permanents they control (auto-pick keeps the highest mana
    /// value of each) and sacrifices the rest (Ajani, Nacatl Avenger's -4).
    SacrificeAllButOnePerType { who: Selector },
    /// "Each [resolved] player chooses a [`filter`] permanent they control,
    /// then sacrifices the rest [of their `filter` permanents]" — Deadly
    /// Vanity (keep one creature or planeswalker). Like
    /// `SacrificeAllButOnePerType` but scoped to a single filter; the keeper
    /// is auto-picked as the highest-mana-value match (the same approximation
    /// the Cataclysm family uses for the "each player chooses" clause).
    EachPlayerKeepsOneSacrificeRest { who: Selector, filter: SelectionRequirement },
    /// "Wish" — put a card you own matching `filter` from your sideboard
    /// ("outside the game") or from exile into your hand (Karn, the Great
    /// Creator's -2). Chosen via `Decision::ChooseCards` for a `wants_ui`
    /// controller; auto-picks the first sideboard match otherwise.
    WishToHand { filter: SelectionRequirement },
    /// "Shuffle up to `max` cards you own matching `filter` from outside the
    /// game into your library" (Research // Development's left half).
    WishToLibrary { filter: SelectionRequirement, max: Value },
    /// "Exile another target nonland permanent. If you controlled it, return it
    /// to the battlefield tapped. Otherwise, its controller [does something]"
    /// (Unyielding Gatekeeper). The branch reads the exiled permanent's
    /// controller at the moment it left.
    ExileThenBranchByController { what: Selector, theirs: Box<Effect> },
    /// CR 702.106b — search your library for a creature card with the OTHER of
    /// the source's two chosen names (the one the trigger's spell isn't),
    /// reveal it, put it into your hand, then shuffle (Summoner's Bond).
    SearchForOtherChosenName,
    /// "You may play basic lands from outside the game" (Sovereign's Realm),
    /// modeled as fetching one basic of a chosen color into your hand — the
    /// land drop itself then runs the normal path.
    BasicLandFromOutsideGameToHand,

    /// CR 702.166 — Manifest dread: look at the top two cards of `who`'s
    /// library, put one onto the battlefield face down as a 2/2 creature, and
    /// the other into their graveyard.
    ManifestDread { who: PlayerRef },

    /// Manifest dread `count` times, then put `counters` +1/+1 counters on each
    /// of the creatures manifested this way (Valgavoth's Onslaught, where both
    /// are the cast's X). Self-contained so the manifested set is tracked across
    /// the repeats without leaking into `last_moved_cards`.
    ManifestDreadRepeatThenCounters { count: Value, counters: Value },

    /// CR 702.182 — Cloak: put the top `amount` cards of `who`'s library onto
    /// the battlefield face down as 2/2 creatures with ward {2}. Each can be
    /// turned face up for its mana cost if it's a creature card.
    Cloak {
        who: PlayerRef,
        amount: Value,
        /// Cloak cards from `who`'s hand instead of off the top of their
        /// library (Vannifar, Evolved Enigma).
        #[serde(default)]
        from_hand: bool,
    },

    /// Reveal the top `count` cards of the controller's library; an opponent
    /// chooses one of them, which goes to the controller's hand. Each
    /// remaining revealed card is exiled, gaining `counter` if `Some`.
    /// Karn, Scion of Urza's +1 (reveal two, opponent chooses, exile the
    /// other with a silver counter). A UI opponent is prompted; a bot chooser
    /// gives the controller the lowest-mana-value card.
    RevealTopOpponentChoosesToHand {
        count: Value,
        counter: Option<crate::card::CounterType>,
        /// Only cards matching this may be chosen (Animal Magnetism's
        /// "chooses a creature card"). `None` = anything revealed.
        #[serde(default)]
        pick_filter: Option<SelectionRequirement>,
        /// The pick goes onto the battlefield instead of to hand, and the
        /// rest to the graveyard instead of exile (Animal Magnetism).
        #[serde(default)]
        pick_to_battlefield: bool,
    },

    /// "Return a card of an opponent's choice matching `filter` from your
    /// graveyard to your hand" — untargeted (Tasigur, the Golden Fang). The
    /// choosing opponent is prompted when they want a UI; a bot chooser hands
    /// back the lowest-mana-value match.
    ReturnFromGraveyardOpponentChooses { filter: SelectionRequirement },

    /// Menacing Ogre — every player secretly picks a number up to `max`, the
    /// picks are revealed at once, and each player who named the highest loses
    /// that much life. `on_you_win` runs when the source's controller is among
    /// them. The picks go through `ask_seat_amount`, so a UI seat is prompted
    /// on its own turn through the suspend machinery.
    EachPlayerChoosesNumberHighestLoses { max: u32, on_you_win: Box<Effect> },

    /// CR 708.2 — "Look at target face-down creature." The resolving
    /// controller sees the real card for as long as it stays face down
    /// (`GameState.face_down_revealed_to`). Aven Soulgazer, Spy Network.
    LookAtFaceDown { what: Selector },

    /// "Create an X/X creature token of the chosen color and type" — reads the
    /// source's `chosen_color` + `chosen_creature_type` (Riptide Replicator).
    CreateTokenOfChosenColorAndType { pt: Value },

    /// Return one card the controller owns with a `counter` counter on it
    /// from exile to their hand (removing the counter). Karn's −1. When more
    /// than one qualifies the controller takes the highest-value one.
    ReturnFromExileWithCounter { counter: crate::card::CounterType },

    /// Put every creature card stamped `exiled_with == ctx.source` (exiled by
    /// this permanent — e.g. via `StaticEffect::ExileDyingOpponentCreatures`)
    /// onto the battlefield under the controller, optionally granting each
    /// `Keyword::Decayed`. Gisa, Glorious Resurrector's upkeep.
    ReturnExiledBySourceToBattlefield {
        #[serde(default)]
        decayed: bool,
    },

    /// Kianne, Dean of Substance — exile the top card of the controller's
    /// library; if it's a land card, put it into their hand, otherwise leave it
    /// exiled with one `counter` counter on it.
    StudyTopCard { counter: crate::card::CounterType },

    /// Imbraham, Dean of Theory — exile the top `count` cards of the
    /// controller's library, each with one `counter` counter on it.
    ExileTopWithCounters { count: Value, counter: crate::card::CounterType },

    /// CR 401.6 — "Until end of turn, you may look at the top card of your
    /// library any time, and you may play lands and cast spells from the top
    /// of your library." Sets `Player.play_from_top_this_turn` for the
    /// effect's controller (The Belligerent's attack trigger).
    GrantPlayFromTopThisTurn,

    /// "You may cast up to `count` spells from among face-up cards your
    /// opponents own from exile this turn without paying their mana costs."
    /// Grants the controller a free `may_play_until` end-of-turn permission
    /// on up to `count` opponent-owned cards in exile (Ashiok, Nightmare
    /// Muse −7). Approximation: the per-spell cast cap isn't enforced beyond
    /// the number of cards granted.
    CastUpToNFromOpponentsExile { count: Value },

    /// Omnath, Locus of Creation — run `branches[n]` where `n` is the number
    /// of times *this ability* has already resolved this turn (0-indexed), then
    /// bump that count. Past the last branch it does nothing. The sibling of
    /// `EscalatingThisTurn`, which repeats its last branch instead.
    NthResolutionThisTurn { branches: Vec<Effect> },

    /// Scholarship Sponsor — each player controlling fewer lands than the
    /// player with the most searches their library for up to (difference)
    /// basic land cards, puts them onto the battlefield tapped, then shuffles.
    CatchUpBasicLands,

    /// Uvilda, Dean of Perfection — the controller may exile an instant or
    /// sorcery card from their hand with `count` hone counters on it. The
    /// per-upkeep tick-down + reduced cast-from-exile is handled engine-side
    /// (`GameState::process_hone`).
    HoneFromHand { count: Value },

    /// Controller chooses `count` cards from their hand and puts them on top of
    /// their library in a chosen order (first chosen = topmost).
    PutOnLibraryFromHand { who: PlayerRef, count: Value },

    /// Each player in `who` puts one card from their hand on top of their
    /// library (Sadistic Augermage). Suspends per-seat for UI players via the
    /// same continuation machinery as a symmetric discard.
    EachPlayerPutsHandCardOnTop { who: Selector },

    /// Sacrifice one creature `who` controls matching `filter` and store its
    /// power in the resolution context for later `Value::SacrificedPower`
    /// references. Used by Thud (sacrifice creature, deal damage equal to
    /// its power) and similar spells.
    SacrificeAndRemember { who: PlayerRef, filter: SelectionRequirement },

    /// Destroy the resolved permanent and record its power/toughness/mana value
    /// on the resolution context (like [`Effect::SacrificeAndRemember`]) so a
    /// following `Value::SacrificedToughness`/`SacrificedPower` reads it. Orzhov
    /// Charm's "destroy target creature and you lose life equal to its
    /// toughness".
    DestroyAndRemember { what: Selector },

    /// Internal plumbing: re-stamp the P/T (and mana value) of a creature
    /// sacrificed as an *activation cost* before running `body`, so
    /// `Value::SacrificedPower/Toughness` and the
    /// `ManaValueEqualsSacrificedPlus` search filter read them at resolution
    /// even after intervening resolutions reset the scratch (Witch's Oven,
    /// Transfigure). Wrapped around the queued ability effect by
    /// `activate_ability`; not meant for card definitions.
    WithSacrificedPt {
        power: i32,
        /// Summed power of every permanent sacrificed to the cost (Soulblast).
        #[serde(default)]
        total_power: i32,
        toughness: i32,
        /// How many permanents the cost sacrificed (Vicious Betrayal's "for
        /// each creature sacrificed this way" — `Value::SacrificedCount`).
        #[serde(default)]
        count: u32,
        #[serde(default)]
        mana_value: u32,
        /// The sacrificed permanent itself, for `Selector::SacrificedCard`.
        #[serde(default)]
        card: Option<crate::card::CardId>,
        body: Box<Effect>,
    },

    /// Internal plumbing: re-stamp the power of the creature tapped to pay a
    /// Station ability's cost (CR 702.184a) before running `body`, so
    /// `Value::TappedForCostPower` reads it at resolution. Wrapped around the
    /// queued Station effect by `activate_ability`; not for card definitions.
    WithTappedPower { power: i32, body: Box<Effect> },

    /// "Target opponent reveals their hand. You choose a card from it
    /// matching `filter`. They discard it." Inquisition of Kozilek,
    /// Thoughtseize, etc. Currently the **caster** auto-picks the first
    /// matching card via `AutoDecider`; an interactive picker UI is a
    /// future improvement.
    DiscardChosen {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
    },
    /// "Target player reveals `reveal` cards from their hand. You choose one of
    /// them; that player discards it." The reveal-capped sibling of
    /// [`Effect::DiscardChosen`] — the caster only sees the revealed subset
    /// (Disciple of Phenax). Which cards get revealed is the revealing
    /// player's choice; the engine takes them in hand order.
    DiscardChosenFromRevealed { from: Selector, reveal: Value },
    /// "Look at `from`'s hand; you may choose `count` card(s) matching
    /// `filter`. That player puts the chosen card(s) on the bottom of their
    /// library, then draws that many cards." Vendilion Clique. Same
    /// caster-picks-from-hand shape as [`Effect::DiscardChosen`], but the
    /// chosen card is bottomed and replaced rather than discarded.
    ///
    /// Approximations: (1) the printed "may" is modeled as forced-if-able —
    /// the caster auto-picks the first matching card (declining is rarely
    /// desired), same as `DiscardChosen`; (2) `from` follows the engine's
    /// ETB-trigger convention of `EachOpponent` (player-targeting on triggers
    /// is an engine-wide gap, see Archon of Cruelty) — faithful in 1v1, so
    /// the printed "target player" self-cast mode is not yet available.
    BottomChosenFromHandAndDraw {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
    },
    /// "Look at `from`'s hand. Exile a card matching `filter`; its owner may
    /// play it, and it costs `extra_cost` more for as long as it remains
    /// exiled." Elite Spellbinder. The owner keeps a may-play grant (with an
    /// added generic tax) through their next turn — an approximation of the
    /// printed "for as long as it remains exiled".
    ExileFromHandTaxed {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
        extra_cost: u32,
    },
    /// CR 603.6e — "Target player reveals their hand; you choose [count]
    /// card(s) matching [filter]. Exile [them] until [this] leaves the
    /// battlefield." The caster picks from `from`'s hand; the chosen
    /// card(s) are exiled and linked to the ability's source. Powers Brain
    /// Maggot / Tidehollow Sculler / Kitesail Freebooter (`return_to`
    /// = Hand).
    ExileChosenUntilSourceLeaves {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
        return_to: crate::card::ExileReturnZone,
    },

    /// "Target player reveals their hand; you choose [count] card(s) matching
    /// [filter] and exile [them]." Same caster-picks-from-hand shape as
    /// `DiscardChosen`, but the chosen cards are exiled permanently (not
    /// linked to a source, unlike `ExileChosenUntilSourceLeaves`). Thought-Knot
    /// Seer. With `link_to_source` the exiled cards are stamped
    /// `exiled_with = source` (recoverable via `Selector::CardExiledWithSource`
    /// — Bane Alley Broker's face-down stash); `face_down` hides them.
    ExileChosenFromHand {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
        #[serde(default)]
        link_to_source: bool,
        #[serde(default)]
        face_down: bool,
    },

    /// `what` becomes the given card type *in addition* to its other types,
    /// indefinitely (anchored to the permanent — expires when it leaves).
    /// Phyrexian Scriptures chapter I's "becomes an artifact". With
    /// `until_eot` the grant is a CR 611.2 end-of-turn effect instead
    /// (Liquimetal Torque, Myr Landshaper).
    AddCardTypeIndefinitely {
        what: Selector,
        card_type: crate::card::CardType,
        #[serde(default)]
        until_eot: bool,
    },

    /// `what` LOSES the given card type until end of turn — the layer-4
    /// subtractive sibling of `AddCardTypeIndefinitely` (Neurok Transmuter's
    /// "isn't an artifact").
    LoseCardTypeUntilEot { what: Selector, card_type: crate::card::CardType },

    /// Grant `what` an activated ability. `Duration::Permanent` anchors the
    /// grant to the permanent (`CardInstance.granted_activated_abilities`,
    /// cleared when it leaves the battlefield — Urza's Saga chapters I/II);
    /// `EndOfTurn`/`EndOfCombat` ride `granted_activated_eot`, cleared at
    /// cleanup (Lightning Volley, Retraction Helix).
    GainActivatedAbility { what: Selector, ability: Box<ActivatedAbility>, duration: Duration },

    // ── Delayed triggers and pact costs ──────────────────────────────────────
    /// Register a delayed triggered ability that fires later. `kind` selects
    /// the future event (your next upkeep, next end step, …); `body` is the
    /// effect that resolves when the trigger fires. Captures the current
    /// `ctx.targets[0]` so the body can reference it via `Selector::Target(0)`.
    DelayUntil {
        kind: DelayedTriggerKind,
        body: Box<Effect>,
    },
    /// `DelayUntil` with an explicit capture: `capture` is resolved NOW and
    /// becomes the delayed body's slot-0 target, so "that creature" survives
    /// the wait (Gift of Immortality's re-attach, Rescue from the Underworld's
    /// sacrificed card).
    DelayUntilWithCapture {
        kind: DelayedTriggerKind,
        capture: Selector,
        body: Box<Effect>,
    },
    /// "Add an amount of {C} equal to [amount] at the beginning of your
    /// next main phase" — `amount` is evaluated NOW (so resolution-scoped
    /// scratch values like `CounteredSpellManaSpent` are read while still
    /// live) and baked as a constant into the registered
    /// `YourNextMainPhase` delayed trigger. Mana Sculpt.
    AddManaAtNextMainPhase {
        amount: Value,
        /// When true the mana is added as `AnyColors` (the player picks each
        /// pip) rather than {C} — Plasm Capture's "in any combination of
        /// colors". Defaults to false (Mana Sculpt's colorless bank).
        #[serde(default)]
        any_color: bool,
    },

    /// "When [target creature] dies this turn, [body]." Registers an
    /// event-keyed delayed trigger watching `ctx.targets[slot]`'s death. The
    /// targeted creature's controller is captured as `Target::Player` so the
    /// body can reference it via `Selector::Target(0)` even after the
    /// creature has left the battlefield. Expires at cleanup. Used by
    /// Searing Blood ("deals 3 damage to its controller", slot 0) and
    /// Devouring Tendrils ("gain 2 life", the damaged permanent in slot 1).
    WhenTargetDiesThisTurn {
        body: Box<Effect>,
        /// Which target slot to watch (default 0 via `#[serde(default)]`).
        #[serde(default)]
        slot: usize,
        /// Cast-time filter for the watched target when this effect is the
        /// slot's *only* mention (Melira's "another target creature or
        /// artifact"); `None` when a preceding effect already declared it
        /// (Searing Blood's damage). Surfaced by the target walkers.
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },

    /// "Whenever a creature you control enters this turn, [body]." Registers
    /// a turn-scoped delayed trigger (CR 603.4) that fires once per creature
    /// the controller controls that enters for the rest of the turn; the
    /// entering creature is exposed to `body` as `Selector::TriggerSource`.
    /// Expires at cleanup. Used by First Day of Class.
    CreaturesYouControlEnteringThisTurn {
        body: Box<Effect>,
    },

    /// "Each player puts a creature card with mana value `max_mv` or less from
    /// their graveyard onto the battlefield" (Crypt Champion). Each player's
    /// highest-mana-value eligible creature is auto-chosen (UI players don't
    /// pick which — a documented approximation), moved under that player's
    /// control in APNAP order.
    EachPlayerReanimateCreatureMaxMv {
        max_mv: u32,
    },

    /// "Whenever target creature deals damage this turn, you gain that much
    /// life" (CR 603.4). Watches the creature in target slot `slot`; each time
    /// it deals damage (combat or noncombat) the controller gains that much
    /// (the amount rides in via `Value::TriggerEventAmount`). Expires at
    /// cleanup. Paladin of Prahv's Forecast rider.
    GainLifeWhenTargetDealsDamageThisTurn {
        #[serde(default)]
        slot: usize,
    },

    /// CR 603.4 — "Whenever [the creature in target slot `slot`] deals combat
    /// damage to a player this turn, [body]" (Captain Howler). Expires at
    /// cleanup; the amount rides in via `Value::TriggerEventAmount`.
    WhenTargetDealsCombatDamageToPlayerThisTurn {
        #[serde(default)]
        slot: usize,
        body: Box<Effect>,
    },

    /// "Until end of turn, whenever a creature you control dies, [body]."
    /// Registers a turn-scoped delayed trigger (CR 603.4) that fires once per
    /// creature the controller controlled (read from death LKI) that dies for
    /// the rest of the turn; the dead creature is exposed to `body` as
    /// `Selector::TriggerSource`. Expires at cleanup. Used by Waltz of Rage.
    CreaturesYouControlDyingThisTurn {
        body: Box<Effect>,
    },

    /// "Until end of turn, whenever a creature [matching `filter`] dies, [body]"
    /// — any player's creature (CR 603.4). Registers a turn-scoped delayed
    /// trigger firing once per dead creature whose death LKI matches `filter`;
    /// the dead creature is `Selector::TriggerSource`. Massacre Girl's chain.
    WheneverCreatureDiesThisTurn {
        filter: SelectionRequirement,
        body: Box<Effect>,
    },

    /// "Until end of turn, whenever a creature you control deals combat damage
    /// to a player, [body]." Registers a turn-scoped delayed trigger (CR 603.4)
    /// that fires per qualifying combat-damage event; the dealing creature is
    /// exposed to `body` as `Selector::TriggerSource`. Expires at cleanup.
    /// Used by Mistway Spy's turn-face-up grant.
    CreaturesYouControlDealingCombatDamageThisTurn {
        body: Box<Effect>,
    },

    /// "Until end of turn, whenever you gain life, [body]." Registers a
    /// turn-scoped delayed trigger (CR 603.4) firing per `LifeGained` event for
    /// the controller; the amount is bound via `Value::TriggerEventAmount`.
    /// Expires at cleanup. Vizkopa Guildmage's second activated ability
    /// ("each opponent loses that much life").
    WheneverYouGainLifeThisTurn { body: Box<Effect> },

    /// "Whenever a spell or ability an opponent controls causes you to discard
    /// cards this turn, [body]." Registers a turn-scoped delayed trigger
    /// (CR 603.4) firing once per such discard with the discarded card bound
    /// as the trigger source. Expires at cleanup. Pure Intentions.
    WheneverOpponentMakesYouDiscardThisTurn { body: Box<Effect> },

    /// "Until end of turn, whenever a card is put into an opponent's graveyard
    /// from anywhere, [body]." Registers a turn-scoped delayed trigger
    /// (CR 603.4); the graveyard's owner is bound as the body's `Target(0)`
    /// (Duskmantle Guildmage — "that player loses 1 life"). Expires at cleanup.
    WheneverCardEntersOpponentGraveyardThisTurn { body: Box<Effect> },

    /// "Whenever you cast a spell this turn, [body]" — like
    /// `OnYourNextSpellCastThisTurn` but repeating until cleanup
    /// (Rediscover the Way chapter III; gate the body with an
    /// `Effect::If` over `Selector::TriggerSource` for spell-type filters).
    OnEachSpellCastThisTurn { body: Box<Effect> },
    /// "When you cast your next spell this turn, [body]." Registers a
    /// one-shot turn-scoped delayed trigger (CR 603.7e); the cast spell is
    /// exposed to `body` as `Selector::TriggerSource`. Expires at cleanup.
    /// Codie, Vociferous Codex.
    OnYourNextSpellCastThisTurn {
        body: Box<Effect>,
    },
    /// CR 603.4 — one-shot "when you next activate an exhaust ability that
    /// isn't a mana ability this turn, [body]" (Pit Automaton). Expires at
    /// cleanup.
    OnYourNextExhaustActivationThisTurn {
        body: Box<Effect>,
    },
    /// One-shot "when you next cast an instant or sorcery spell this turn,
    /// [body]" — other casts leave it armed (Mercurial Spelldancer's copy).
    OnYourNextInstantSorceryThisTurn {
        body: Box<Effect>,
    },
    /// "When you cast a spell with the chosen name for the first time this
    /// turn, [body]." Like `OnYourNextSpellCastThisTurn` but only fires for a
    /// cast whose name matches the source's `named_card`; other casts don't
    /// consume it. One-shot, expires at cleanup. Medomai's Prophecy III.
    OnYourNextNamedSpellThisTurn {
        body: Box<Effect>,
    },

    /// Signal the Clans — "Search your library for three creature cards and
    /// reveal them. If you reveal three cards with different names, choose one
    /// of them at random and put that card into your hand. Shuffle the rest
    /// into your library." Auto-search picks the three highest-mana-value
    /// distinct-named creature cards (there's never a reason to reveal a
    /// duplicate name or fewer than three); if three distinct names are found
    /// one is taken uniformly at random. The library is always shuffled.
    SignalTheClans,
    /// Unexpected Results — "Shuffle your library, then reveal the top card. If
    /// it's a nonland card, you may cast it without paying its mana cost. If
    /// it's a land card, you may put it onto the battlefield and return
    /// Unexpected Results to its owner's hand." The nonland free-cast goes
    /// through `cast_card_for_free` from the library (declining leaves it on
    /// top); the land branch puts it onto the battlefield and returns this
    /// spell to hand via `return_resolving_spell_to_hand`.
    UnexpectedResults,
    /// Ecological Appreciation: search your library and graveyard for up to
    /// `count` creature cards with different names and mana value ≤ X
    /// (`Value::XFromCost`); an opponent chooses two to shuffle into your
    /// library, the rest enter the battlefield. Auto-pickers: caster takes
    /// the highest-MV candidates, the opponent denies the two biggest.
    SearchSplitWithOpponent { count: u32 },
    /// Fact or Fiction / Atris — reveal the top `count` cards of your library;
    /// an opponent separates them into two piles; put one pile into your hand
    /// and the other into your graveyard. The split + pick are modeled by a
    /// value heuristic (opponent isolates the single highest-mana-value card;
    /// you keep the pile with the greater total mana value), so it resolves
    /// without an interactive `Decision::SplitPiles`.
    /// `to_bottom` sends the unchosen pile to the bottom of the library
    /// instead of the graveyard (Jace, Architect of Thought's −2).
    FactOrFiction {
        count: Value,
        #[serde(default)]
        to_bottom: bool,
    },
    /// Storm Herald — return all Aura cards from your graveyard to the
    /// battlefield, each attached to a legal creature (auras with no legal
    /// creature stay in the graveyard); exile them at the next end step.
    ReanimateAurasExileEot,
    /// Allure of the Unknown — reveal the top six cards of your library; an
    /// opponent exiles a nonland card from among them; the rest go to your
    /// hand; you may cast the exiled card without paying its mana cost.
    AllureOfTheUnknown,

    /// Possibility Storm — fires when a player casts a spell from their hand
    /// (the cast spell is the trigger source). That player exiles the spell,
    /// then exiles cards from the top of their library until they exile one
    /// sharing a card type with it; they may cast that card without paying
    /// its mana cost. All cards exiled this way then go to the bottom of
    /// their library in a random order.
    PossibilityStorm,

    /// CR 614 — "Permanents enter tapped this turn" (Due Respect). A
    /// turn-scoped blanket replacement over every entry path.
    PermanentsEnterTappedThisTurn,

    /// Knowledge Pool's cast replacement — the just-cast spell (the trigger
    /// source) is exiled stamped `exiled_with = source`, and its caster may
    /// then free-cast one of the *other* cards exiled with the source.
    KnowledgePool,

    /// "Pay {cost} or you lose the game." Used for pact upkeep payments
    /// (Pact of Negation, Summoner's Pact). Auto-pays when the controller
    /// can afford; eliminates the controller otherwise. (No interactive
    /// "do I want to pay?" prompt yet — pact costs are virtually always
    /// paid, and skipping the prompt avoids another suspend path.)
    PayOrLoseGame {
        mana_cost: crate::mana::ManaCost,
        life_cost: u32,
    },

    /// Add `count` "first-spell tax" charges against each player resolved
    /// by the selector. Each charge taxes that player's next spell {1}
    /// more (consumed at cast time via `consume_first_spell_tax`). Used by
    /// Chancellor of the Annex's opening-hand reveal — `who: EachOpponent`.
    AddFirstSpellTax {
        who: PlayerRef,
        count: Value,
    },

    /// Set `Player.sorceries_as_flash` on each resolved player so they may
    /// cast sorcery spells at instant speed until their next turn.
    /// Cleared in `do_untap`. Used by Teferi, Time Raveler's +1.
    GrantSorceriesAsFlash { who: PlayerRef },

    /// "Reveal cards from the top of `who`'s library until you reveal a
    /// card matching `find`, or `cap` cards have been revealed. Put the
    /// found card (if any) into `to`; mill the rest, lose 1 life per
    /// card revealed." Used by Spoils of the Vault.
    ///
    /// The auto-decider picks the **first** matching card (so the search
    /// resolves deterministically in tests). Real Oracle has the player
    /// name a card up-front; we bypass that, instead matching anything
    /// passing `find`. The "lose 1 per revealed" rider is wired to
    /// `life_per_revealed` so callers can disable it (Spoils → 1; future
    /// "search until type, no life cost" cards → 0).
    ///
    /// `miss_dest` controls where the non-matching revealed cards end
    /// up. Defaults to `RevealMissDest::Graveyard` for snapshot
    /// back-compat — the previous behavior. Several Strixhaven cards
    /// (Geometer's Arthropod, Paradox Surveyor, Follow the Lumarets)
    /// printed-want misses placed on the bottom of the library in
    /// random order; pass `RevealMissDest::BottomRandom` to honor that.
    RevealUntilFind {
        who: PlayerRef,
        find: SelectionRequirement,
        to: ZoneDest,
        cap: Value,
        life_per_revealed: u32,
        #[serde(default)]
        miss_dest: RevealMissDest,
    },

    /// CR-style "impulse exile until a unique name" — Tainted Pact. Exile
    /// the top card of `who`'s library; they may put it into their hand
    /// unless it shares a name with a card already exiled this way, in which
    /// case the process ends with that card exiled. Repeat until a card is
    /// taken or a duplicate name is exiled (or the library empties). The
    /// "you may keep digging" choice is asked of the controller via
    /// `Decision::OptionalTrigger`; the default (AutoDecider) takes the first
    /// uniquely-named card, and a `Bool(true)` answer means "decline, keep
    /// digging."
    ExileUntilDuplicateName { who: PlayerRef },

    /// Grant the controller one additional land play this turn. Used by
    /// Explore, Dryad of the Ilysian Grove, Oracle of Mul Daya, and similar
    /// "you may play an additional land" effects. Bumps
    /// `Player.extra_land_plays` by `count`.
    GrantExtraLandPlay { who: PlayerRef, count: Value },

    /// "As [this] enters, choose a creature type." Used by Cavern of Souls.
    /// Asks the controller via the `ChooseCreatureType` decision and stores
    /// the chosen type on the source permanent's `chosen_creature_type`
    /// field. Subsequent cast paths consult that field via
    /// `caster_grants_uncounterable` to gate which creature spells the
    /// Cavern protects (only those that share the named type).
    NameCreatureType { what: Selector },

    /// As [`Effect::NameCreatureType`], but a named player makes the choice
    /// (Callous Oppressor — "an opponent chooses a creature type").
    NameCreatureTypeBy { what: Selector, who: PlayerRef },

    /// As [`Effect::NameCreatureType`], but the choice is limited to a closed
    /// list — A Killer Among Us's "secretly choose Human, Merfolk, or
    /// Goblin". Only `options` are offered, and an answer outside them is
    /// clamped to the first.
    NameCreatureTypeAmong { what: Selector, options: Vec<crate::card::CreatureType> },

    /// CR 201.3 — "As [this] enters, choose a card name." Pithing Needle,
    /// Phyrexian Revoker. Asks the controller via the `NameCard` decision and
    /// stores the chosen name on the source permanent's `named_card` field;
    /// `activate_ability` then suppresses non-mana activated abilities of
    /// sources with that name. `what` selects the permanent to stamp
    /// (typically `Selector::This`).
    ///
    /// CR 201.4a — `restrict_to` narrows the namespace to card names whose
    /// printed characteristics match ("choose a LAND card name" — Petrified
    /// Hamlet). The suggestion feed is filtered and an off-namespace answer is
    /// rejected. `None` allows any card name.
    NameCard {
        what: Selector,
        #[serde(default)]
        restrict_to: Option<SelectionRequirement>,
    },

    /// "As [this] enters, choose a number." Stores the chosen number on the
    /// source permanent's `chosen_number` field (Sanctum Prelate — read by the
    /// chosen-MV noncreature lock). `max` bounds the choice.
    ChooseNumberForSource { max: u32 },

    /// CR 729 — "Players play a Magic subgame, using their libraries as their
    /// decks. Each player who doesn't win the subgame loses half their life,
    /// rounded up" (Shahrazad). The nest is bot-piloted and bounded; a stalled
    /// or drawn subgame has no winner, so everyone pays.
    PlaySubgame,

    /// CR 509.1 — "If each of those creatures could block all creatures the
    /// other is blocking, remove both from combat; each then blocks all
    /// creatures the other was blocking" (Sorrow's Path). A no-op when either
    /// swap would be illegal.
    SwapBlockAssignments { a: Selector, b: Selector },

    /// CR 614 — grant Kumano's rider until end of turn: a creature `what`
    /// damages this turn is exiled instead of dying (Runesword).
    GrantDamageExilesVictimThisTurn { what: Selector },

    /// CR 701.15g — "if `what` deals damage to a creature this turn, that
    /// creature can't be regenerated this turn" (Runesword).
    GrantDamageDeniesRegenerationThisTurn { what: Selector },

    /// CR 603.4 — "When `what` leaves the battlefield this turn, [body]"
    /// (Runesword's sacrifice rider). Expires at cleanup.
    WhenTargetLeavesBattlefieldThisTurn { what: Selector, body: Box<Effect> },

    /// CR 614 — "As this enters, exile `count` creature cards from your
    /// graveyard. If you can't, put this into its owner's graveyard instead.
    /// For each card exiled this way, this enters with a +2/+0, +1/+1, or
    /// +0/+2 counter on it" (Frankenstein's Monster). The kind is chosen per
    /// exiled card.
    EnterExilingGraveyardCreaturesForCounters { count: Value },

    /// CR 614 — "As this enters, pay any amount of life", capped by `max`
    /// (Nameless Race). The paid amount is stamped on the source's
    /// `chosen_number` for `DynamicPt::ChosenNumberAsEntered` to read.
    PayAnyAmountOfLifeCapped { max: Value },

    /// "As [this] enters, choose a permanent matching `filter`." Stores the
    /// chosen permanent's id on the source's `chosen_permanent` field
    /// (Dauntless Bodyguard). `Selector::ChosenPermanentOfSource` reads it
    /// back. No-op (leaves `None`) when no legal choice exists.
    ChoosePermanentForSource { filter: SelectionRequirement },

    /// CR 201.3 — "Choose a nonland card name. Opponents can't cast spells
    /// with the chosen name until your next turn" (Academic Probation). Asks
    /// the resolving controller via the `NameCard` decision and records the
    /// name in their `opponents_cant_cast_named` lock; the cast-legality gate
    /// rejects opponents' spells with a matching name until the controller's
    /// next turn clears it.
    NameOpponentCastLock,

    /// "[That card]'s owner can't cast spells with that name until your next
    /// turn" (Reflector Mage). Reads the name of the card resolved by `what`
    /// (typically the just-bounced `Selector::Target(0)`) and records it in
    /// the resolving controller's `opponents_cant_cast_named` lock — the same
    /// lock Academic Probation uses, but keyed off a *targeted* card's name
    /// instead of a chosen one. The cast-legality gate then rejects that
    /// owner's (an opponent's) spells of that name until the controller's
    /// next turn clears it.
    LockTargetNameUntilYourNextTurn { what: Selector },

    /// "[Player] skips their next `count` turns." Bumps the affected
    /// player's `skip_turns` counter; the turn-advance logic in
    /// `do_cleanup` decrements and bypasses each scheduled-skip turn.
    /// Used by Ral Zarek, Guest Lecturer's -7 ult ("Flip five coins.
    /// Target opponent skips their next X turns, where X is the number
    /// of coins that came up heads.") via a `FlipCoin` + `SkipTurns`
    /// chain.
    SkipTurns { who: PlayerRef, count: Value },
    /// CR 506 — "[Player] skips their next combat phase" (Stonehorn
    /// Dignitary, Fog Bank-adjacent tempo). Bumps `who`'s `skip_next_combat`
    /// counter; `advance_step` consumes it when their turn would enter Begin
    /// Combat, jumping to the postcombat main.
    SkipNextCombatPhase { who: PlayerRef },
    /// CR 725 — `who` becomes the monarch. "You become the monarch."
    BecomeMonarch { who: PlayerRef },
    /// CR 701.54 — "the Ring tempts you." Increments `who`'s ring-temptation
    /// count (capped at 4) and lets them designate a creature they control as
    /// their Ring-bearer. The bearer-specific abilities (can't-be-blocked-by-
    /// greater-power at 1+, attack loot at 2+, blocked-creature sacrifice at
    /// 3+, combat-damage drain at 4+) are applied directly off the player's
    /// temptation level rather than synthesized as a literal emblem.
    RingTempts { who: PlayerRef },
    /// CR 701.54c (level 3+) — register every creature in `what` to be
    /// sacrificed by its controller at end of combat (reuses the
    /// `attacking_token_cleanup` funnel). Used for The Ring-bearer's
    /// "blocking creature's controller sacrifices it at end of combat."
    SacrificeAtEndOfCombat { what: Selector },
    /// CR 702.131 — Ascend. If `who` controls ten or more permanents, they
    /// get the city's blessing (a permanent player designation). A no-op
    /// otherwise. "Ascend" on a sorcery/instant resolves once; the
    /// permanent-static variant re-checks each time it's seen.
    Ascend { who: PlayerRef },
    /// CR 731 — "it becomes day." Sets the game's day designation.
    BecomeDay,
    /// CR 731 — "it becomes night." Sets the game's night designation.
    BecomeNight,
    /// CR 500.7 — "[Player] takes [count] extra turn(s) after this one."
    /// Banks `count` onto each resolved player's `extra_turns`; consumed
    /// by `advance_turn`. Time Walk, Temporal Manipulation, Ral Zarek's
    /// -7 coin-flip emblem.
    TakeExtraTurn { who: PlayerRef, count: Value },
    /// CR 701.35 — "Put target creature on the bottom of its owner's library.
    /// That creature's controller reveals cards from the top of their library
    /// until they reveal a creature card, puts it onto the battlefield, and
    /// the rest on the bottom in any order." Proteus Staff.
    BottomThenRevealUntilCreature { what: Selector },
    /// "Each player exiles the top card of their library. The player who
    /// exiled the card with the greatest mana value takes an extra turn after
    /// this one." Ties re-run over the tied players (Timesifter).
    ExileTopGreatestManaValueTakesExtraTurn,
    /// "This creature gains all activated abilities of target creature until
    /// end of turn" (Quicksilver Elemental) — the grants are stamped onto the
    /// source's `granted_activated_eot`.
    GainAllActivatedAbilitiesOf { what: Selector, duration: Duration },
    /// CR 701.12 — "Its controller chooses target permanent another player
    /// controls that shares a card type with it. Exchange control of those
    /// permanents." (Confusion in the Ranks.) The partner is picked as the
    /// effect resolves; the auto-picker takes the highest mana value.
    ExchangeControlWithSharedType { what: Selector },
    /// "Search your library for a nonland card and reveal it. Each opponent who
    /// cast a spell this turn with the same name as that card loses `amount`
    /// life. Then shuffle." (Grim Reminder.)
    SearchRevealPunishSameNameCasters { amount: Value },
    /// CR 723.1 — "You control target player during that player's next turn."
    /// Registers a pending control entry consumed when that player actually
    /// takes a turn (Mindslaver).
    ControlPlayerNextTurn { who: PlayerRef },
    /// CR 500.8 — "That player skips each instance of the chosen step or phase
    /// this turn." The affected player picks draw step / main phase / combat
    /// phase (`Decision::ChooseModes`); the pick is turn-scoped. Fatespinner.
    ChooseStepToSkipThisTurn { who: PlayerRef },
    /// CR 724 — "End the turn." Exiles all spells and abilities from the
    /// stack (including the resolving card), removes everything from combat,
    /// and skips straight to the cleanup step. Sundial of the Infinite,
    /// Day's Undoing.
    EndTheTurn,
    /// CR 614 — "Until end of turn, if a player taps a land for mana, it
    /// produces [something else] instead of any other type and amount."
    /// `mine_only` scopes it to the controller's own land taps (Harvest
    /// Mage); `nonbasic_only` to nonbasic lands (Pale Moon).
    ReplaceLandManaThisTurn {
        #[serde(default)]
        mine_only: bool,
        #[serde(default)]
        nonbasic_only: bool,
        /// The tapping player picks a color instead of getting {C}.
        #[serde(default)]
        color_of_choice: bool,
    },
    /// CR 615.7 — "Prevent all damage a [filter] source of your choice would
    /// deal this turn." The source is chosen as the effect resolves among
    /// battlefield permanents and stack spells matching `filter` (AutoDecider
    /// picks a stack spell first, else the highest-power permanent).
    /// Burrenton Forge-Tender. `gain_life_from_colors` adds Samite
    /// Ministration's "whenever damage from a black or red source is prevented
    /// this way, you gain that much life" rider (empty = no refund).
    PreventAllDamageFromChosenSourceThisTurn {
        filter: crate::card::SelectionRequirement,
        gain_life_from_colors: Vec<crate::mana::Color>,
    },
    /// CR 615.8 — "The next time a source of your choice would deal damage to
    /// you this turn, prevent that damage. When damage is prevented this way,
    /// [rider]" (New Way Forward). The shield is scoped to the controller and
    /// expires after one instance; `rider` runs for the controller with the
    /// prevented amount bound to `Value::TriggerEventAmount` and the shielded
    /// source bound as `Selector::Target(0)`.
    PreventNextDamageToYouFromChosenSourceWithRider {
        filter: crate::card::SelectionRequirement,
        rider: Box<Effect>,
    },
    /// "Put `amount` +1/+1 counters *or* charge counters on a permanent
    /// matching `onto` you control" (Dismantle) — both the counter kind and the
    /// recipient are picked as the effect resolves. A `wants_ui` controller
    /// gets a `ChooseModes` kind pick and a `ChooseTarget` recipient pick;
    /// other seats take the first listed kind and the highest-mana-value
    /// recipient. A zero `amount` or an empty candidate set is a no-op.
    AddCountersOfChosenKind {
        onto: crate::card::SelectionRequirement,
        kinds: Vec<CounterType>,
        amount: Value,
    },
    /// CR 615.7 — "Prevent all damage `what` would deal this turn." The
    /// targeted sibling of `PreventAllDamageFromChosenSourceThisTurn`; `what`
    /// may name a permanent or a spell on the stack. With `gain_life`, the
    /// effect's controller gains life equal to the damage prevented this way,
    /// event by event (Hallow).
    PreventAllDamageFromTargetThisTurn {
        what: Selector,
        #[serde(default)]
        gain_life: bool,
        /// CR 615.8 — "the **next** time `what` would deal damage this turn":
        /// the shield soaks one whole damage instance and then expires (Awe
        /// Strike). `false` is the turn-long blanket (Hallow).
        #[serde(default)]
        next_instance_only: bool,
    },
    /// CR 615.7 — "The next time a [filter] source of your choice would deal
    /// damage to you this turn, prevent that damage." A one-event,
    /// source-restricted shield around the controller. Circle of Protection
    /// cycle.
    PreventNextDamageFromChosenSource {
        filter: crate::card::SelectionRequirement,
        /// Deflecting Palm — damage prevented by this shield is dealt to
        /// the chosen source's controller.
        #[serde(default)]
        reflect: bool,
        /// Who the shield protects. `None` = the controller (the Circle of
        /// Protection default); a selector shields whatever it resolves to —
        /// "damage to target creature" (Charm Peddler) or another seat.
        #[serde(default)]
        to: Option<Selector>,
        /// "You gain life equal to the damage prevented this way"
        /// (Cho-Arrim Alchemist).
        #[serde(default)]
        gain_life: bool,
        /// CR 614.9 — "that damage is dealt to `redirect_to` instead"
        /// (General's Regalia).
        #[serde(default)]
        redirect_to: Option<Selector>,
        /// "ALL damage … this turn by a source of your choice" — the shield
        /// stays up for the whole turn instead of soaking one event
        /// (Oracle's Attendants).
        #[serde(default)]
        whole_turn: bool,
    },
    /// CR 615.7 — "Prevent the next `amount` damage that a source of your
    /// choice would deal to you and/or permanents you control this turn. If
    /// damage is prevented this way, deal that much damage to `to`."
    /// One shared pool across the whole team (Refraction Trap).
    PreventNextFromChosenSourceToTeam {
        amount: Value,
        to: Selector,
        /// "The next time [it] would deal damage" — soak one whole damage
        /// event rather than a point budget (Opal-Eye, Konda's Yojimbo).
        #[serde(default)]
        one_event: bool,
    },
    /// CR 615.7 — "The next time a source of your choice would deal damage to
    /// any target this turn, prevent that damage." Unlike
    /// `PreventNextFromChosenSourceToTeam` the shield floats over every
    /// recipient, so it soaks whichever damage event the chosen source deals
    /// first (Martyr's Cause). `what` names the source outright instead of
    /// asking (Impulsive Maneuvers' losing flip aims at the attacker).
    PreventNextEventFromChosenSourceAnywhere {
        #[serde(default)]
        what: Option<Selector>,
    },
    /// "Creatures `what` gain protection from the colors of `of` until
    /// `duration`" (Samite Elder). Reads the live colors of the permanent(s)
    /// `of` resolves to and grants one `Keyword::Protection` per color.
    GrantProtectionFromColorsOf { what: Selector, of: Selector, duration: Duration },
    /// "Search your library for any number of cards matching `filter`, exile
    /// them stamped with this source, then shuffle" (Skyship Weatherlight).
    /// The linked exile is readable by `Selector::ExiledWithSource` and
    /// [`Effect::ReturnRandomExiledWithSource`].
    SearchExileLinked { who: PlayerRef, filter: SelectionRequirement, count: Value },
    /// "Choose a card at random that was exiled with this source. Put it into
    /// its owner's hand" (Skyship Weatherlight's `{4}, {T}`).
    ReturnRandomExiledWithSource,
    /// CR 706.8a — "roll `count` `sides`-sided dice and store those results on
    /// `what`" (Centaur of Attention's ETB).
    RollAndStoreDice { what: Selector, count: Value, sides: u8 },
    /// CR 706.8b — "you may reroll any number of `what`'s stored results."
    /// The chooser is asked yes/no per result; an auto seat rerolls anything
    /// below the current most-common value.
    RerollStoredResults { what: Selector },
    /// CR 614.9 — "The next time damage would be dealt to `what` this turn,
    /// that damage is dealt to `to` instead" (Mirrorwood Treefolk). A one-shot
    /// per-permanent redirect consumed by the first damage event.
    RedirectNextDamageTo { what: Selector, to: Selector },
    /// Goblin Game (CR 720-adjacent silliness) — each player secretly hides at
    /// least one item, all are revealed, each player loses life equal to their
    /// own count, and the player(s) with the fewest then lose half their life
    /// rounded up. Counts are asked per seat (`ask_seat_amount`); an auto seat
    /// hides one.
    GoblinGame,
    /// "Each player reveals their hand, chooses one card of each color from
    /// it, then discards all other nonland cards" (Noxious Vapors). The keep
    /// set is one card per color present; multicolored cards count for every
    /// colour they carry, so a gold card can be the only keep.
    EachPlayerKeepsOneOfEachColorDiscardsRest,
    /// "Each player chooses a land they control of each basic land type.
    /// Return those lands to their owners' hands" (Planar Overlay).
    EachPlayerReturnsALandOfEachBasicType,
    /// Parley — "each player reveals the top card of their library. For each
    /// nonland card revealed this way, [`then`]. Then each player draws a
    /// card." The nonland count is published as
    /// `Value::CardsRevealedThisEffect` for the body to read, and the revealed
    /// cards stay on top (the draw takes them).
    Parley { then: Box<Effect> },
    /// "Reveal any number of `filter` cards in your hand, then [`then`]" — the
    /// Urza's Destiny Scent / Seer cycle and Metalworker. The revealed count is
    /// published as `Value::CardsRevealedThisEffect` for the body to read.
    RevealAnyNumberFromHand { filter: crate::card::SelectionRequirement, then: Box<Effect> },
    /// "Exile `what`, then search its controller's graveyard, hand, and library
    /// for all cards with the same name and exile them; that player shuffles."
    /// The Urza's Destiny name-hate cycle (Eradicate, Quash, Scour, Sowing Salt,
    /// Splinter). `what` may be a stack object (Quash counters it).
    ExileAllCopiesOfTargetName { what: Selector },
    /// "Choose a card name. Search `who`'s graveyard, hand, and library for up
    /// to `count` cards with that name and exile them. Then that player
    /// shuffles." (Ancient Vendetta.) The name is chosen by the effect's
    /// controller at resolution.
    NameCardThenExileFromZones { who: PlayerRef, count: u32 },
    /// "Exile all tokens with the same name as `what`" (Dual Nature's leave
    /// trigger). Nontoken permanents sharing the name are untouched; `what`
    /// resolves off the death/leave LKI snapshot when it has already gone.
    ExileTokensSharingNameWith { what: Selector },
    /// "Exile `what`, then return it to the battlefield under its owner's
    /// control" — an immediate blink (Flicker). Tokens cease to exist.
    ExileAndReturnToOwner { what: Selector },
    /// CR 120.4a — "deal `amount` damage to `to`; excess damage is dealt to
    /// `excess_to` instead." The split happens before the damage event, so the
    /// creature takes exactly lethal (marked damage and the source's deathtouch
    /// counted). `condition` gates the rider (Ram Through's "if the creature
    /// you control has trample"); `None` always redirects.
    DealDamageExcessTo {
        to: Selector,
        amount: Value,
        excess_to: Selector,
        condition: Option<crate::card::Predicate>,
    },
    /// Aura Barbs — each enchantment deals `amount` damage to its controller,
    /// then each Aura attached to a creature deals `amount` damage to that
    /// creature.
    EnchantmentsBiteControllersAndHosts { amount: Value },
    /// Minamo's Meddling — counter target spell, then its controller reveals
    /// their hand and discards each card sharing a name with a card spliced
    /// onto that spell (CR 702.47).
    CounterSpellDiscardSplicedNames { what: Selector },
    /// "Reveal the top `count` cards of your library. For each of those
    /// cards, put that card into your hand unless any opponent pays
    /// `life` life. Then exile the rest." (Sword-Point Diplomacy.)
    /// Opponents are asked in turn order per card via `ask_seat_bool`.
    RevealTopPayOrTake { count: Value, life: Value },
    /// CR 714.4 (DFC sagas) — "Exile this Saga, then return it to the
    /// battlefield transformed under your control." The return is a new
    /// object: lore counters clear and the back face's ETB fires. Fable of
    /// the Mirror-Breaker chapter III. A `from_graveyard` ability uses the
    /// same effect to return the card transformed from a graveyard (Garland,
    /// Knight of Cornelia).
    ExileSelfReturnTransformed,
    /// The mirror of `ExileSelfReturnTransformed` — "exile this, then return it
    /// to the battlefield (front face up)". Used by the FIN Dominants' Saga
    /// chapter III to reset the flip cycle.
    ExileSelfReturnFrontFace,
    /// CR 505.1b — "there is an additional combat phase after this one."
    /// Banks `count` onto `GameState.additional_combat_phases`; when the
    /// active player leaves the End of Combat step with the counter set, the
    /// turn loops back to Begin Combat (a fresh combat phase) instead of
    /// advancing to the postcombat main. Built for combat-phase activated
    /// extra-combat effects (Hellkite Charger, Aggravated Assault while in
    /// combat), usually paired with an `Untap` so creatures can attack again.
    /// Main-phase-cast "after this main phase, an additional combat + main"
    /// sorceries (Relentless Assault) use `AdditionalCombatPhaseAfterMain`.
    AdditionalCombatPhase { count: Value },
    /// "You choose which creatures attack this turn, and which creatures
    /// block and how." Master Warcraft — sets `GameState.combat_chooser` to
    /// the resolving controller; both declaration steps then hand priority to
    /// that seat instead of the active/defending player. Clears at cleanup.
    ChooseCombatThisTurn,
    /// CR 505.1b — "After this main phase, there is an additional combat
    /// phase followed by an additional main phase." Banks a combat phase
    /// that begins when the active player leaves their current main phase;
    /// the following main phase comes from the normal EndCombat → PostMain
    /// flow. Relentless Assault.
    AdditionalCombatPhaseAfterMain { count: Value },
    /// CR 500.7 — "there is an additional end step after this step." Banks
    /// `count` extra end steps; when the active player leaves the End step with
    /// one banked, the turn loops back to another End step (Y'shtola Rhul).
    AdditionalEndStep { count: Value },
    /// CR 500.9 — "you get an additional upkeep step after this one." Banks
    /// `count` extra upkeep steps; when the active player leaves the Upkeep
    /// with one banked, the turn loops back to another Upkeep (Paradox Haze).
    AdditionalUpkeepStep { count: Value },
    /// "At the beginning of each combat this turn, [body]." Registers a
    /// turn-scoped `DelayedKind::EachCombatThisTurn` delayed trigger that
    /// runs `body` at the start of every Begin-Combat step for the rest of
    /// the controller's turn, then expires at cleanup. Full Throttle pairs
    /// this with `AdditionalCombatPhaseAfterMain` to untap attackers between
    /// its extra combats.
    AtEachCombatThisTurn { body: Box<Effect> },
    /// CR 114 — "[Player] gets an emblem with '[triggered abilities]'."
    /// Appends an `Emblem` (named after its source) to the player's
    /// emblem zone. Emblems never leave; their triggered abilities fire
    /// from the command zone alongside battlefield permanents (the
    /// dispatcher walks each player's emblems). Used by planeswalker
    /// ultimates — Professor Dellian Fel's -6, the upkeep-draw / end-step
    /// emblems, etc.
    CreateEmblem {
        who: PlayerRef,
        name: String,
        #[serde(default)]
        triggered: Vec<TriggeredAbility>,
        /// Static (anthem) abilities the emblem grants — Vivien Reid's −8.
        #[serde(default)]
        statics: Vec<crate::card::StaticAbility>,
    },

    /// "[Player] wins the game." Used by Approach of the Second Sun's
    /// second-cast win condition, Coalition Victory, Test of Endurance,
    /// Felidar Sovereign, and similar alt-win effects. The engine
    /// eliminates every other player so the standard
    /// `check_state_based_actions` win-detection path (≤ 1 alive player
    /// → `game_over = Some(winner)`) promotes the named player to the
    /// winner on the next SBA pass. No CR violation: the state-based
    /// action approach matches CR 104.2a's "you win the game" wording.
    WinGame { who: PlayerRef },

    /// CR 104.2a/104.4b — "the player with the highest life total wins the
    /// game. If two or more players are tied for highest life total, the game
    /// is a draw." Celestial Convergence.
    HighestLifeWinsElseDraw,

    /// CR 104.4 — "the game is a draw" (Divine Intervention). Ends the game
    /// with no winner.
    GameIsADraw,

    /// "[Player] loses the game" (CR 104.3a). Eliminates the named player;
    /// the SBA pass promotes the last player standing to the winner.
    /// Strixhaven Stadium's ten-point payoff.
    LoseGame { who: PlayerRef },

    /// "Prevent all combat damage that would be dealt this turn." Sets
    /// `GameState.prevent_combat_damage_this_turn = true`; combat
    /// damage resolution (`resolve_combat_damage_with_filter`) reads
    /// the flag and zeroes every assigned damage value (CR 615.1
    /// replacement-effect emulation — see the note on the field). The
    /// flag clears in `do_cleanup` alongside other until-end-of-turn
    /// state. Used by Owlin Shieldmage's ETB and the Holy Day / fog
    /// family of effects.
    PreventAllCombatDamageThisTurn,
    /// CR 615 — "Prevent all combat damage that would be dealt by [filter]
    /// creatures this turn" (Hunter's Ambush's nongreen fog). The filtered
    /// sibling of `PreventAllCombatDamageThisTurn`.
    PreventAllCombatDamageByMatchingThisTurn { filter: crate::card::SelectionRequirement },

    /// CR 615.1 fog with an exception — "Prevent all combat damage this turn
    /// except combat damage dealt by [filter] creatures." Inspire Awe ("by
    /// enchanted creatures and enchantment creatures"). Sets the global fog
    /// flag plus the per-dealer exception filter.
    PreventCombatDamageExceptDealtBy { except: SelectionRequirement },

    /// CR 615 — "Prevent all combat damage that would be dealt to `who` this
    /// turn" (Druid's Deliverance's player-scoped fog). Records the resolved
    /// player(s) in `GameState.combat_damage_prevented_to_players_this_turn`;
    /// the combat resolver zeroes any combat hit aimed at them.
    PreventAllCombatDamageToPlayerThisTurn { who: PlayerRef },

    /// CR 701.16 — "[source's controller] sacrifices [the source] unless they
    /// pay {X}, where X is its mana value." The pay-or-sacrifice threat used by
    /// Soul Tithe (granted to the enchanted permanent via its Aura). Reads the
    /// source permanent's live mana value; the controller keeps it by paying
    /// that much generic mana (auto-tapping), otherwise it is sacrificed.
    SacrificeSourceUnlessPayManaValue,

    /// "When this enters, sacrifice it unless you pay [cost]." The fixed-cost
    /// sibling of `SacrificeSourceUnlessPayManaValue` — the printed cost is a
    /// literal `ManaCost`, not the source's mana value (which is 0 for a land).
    /// The source's controller keeps it by paying (auto-tap), else it is
    /// sacrificed. Gateway Plaza, Transguild Promenade ("pay {1}").
    SacrificeSourceUnlessPay { cost: crate::mana::ManaCost },

    /// "Shuffle any number of cards from your hand into your library, then
    /// draw that many cards" (Credit Voucher). The controller picks the subset
    /// via `Decision::ChooseCards`; AutoDecider shuffles none.
    ShuffleAnyNumberFromHandThenDraw { who: PlayerRef },

    /// "Each player reveals the top card of their library. If all cards
    /// revealed this way are creature cards, put those cards onto the
    /// battlefield under their owners' control" (Game Preserve). Nothing moves
    /// unless every revealed card is a creature.
    EachPlayerRevealTopAllEnterIfAllCreatures,

    /// "Each player reveals the top `count` cards of their library, puts all
    /// land cards revealed this way onto the battlefield tapped, and exiles the
    /// rest" (Clear the Land).
    EachPlayerRevealTopNKeepLandsExileRest { count: Value },

    /// CR 508.1 / 509.1d — "This turn, creatures can't attack (or block)
    /// unless their controller pays {X} for each attacking (blocking) creature
    /// they control" (War Tax, War Cadence). Symmetric and turn-scoped.
    AddAttackTaxThisTurn { amount: Value },
    AddBlockTaxThisTurn { amount: Value },

    /// "Sacrifice this unless you pay {1} for each [thing]" — the dynamic
    /// sibling of `SacrificeSourceUnlessPay`, where the generic amount is a
    /// `Value` read at resolution (Megatherium, Extravagant Spirit).
    SacrificeSourceUnlessPayValue { generic: Value },
    /// "Sacrifice this unless you pay `per` for each `kind` counter on it. If
    /// you pay, `then`." Cyclone's escalating upkeep.
    PayPerCounterOrSacrifice {
        kind: crate::card::CounterType,
        per: crate::mana::ManaCost,
        then: Box<Effect>,
    },
    /// "…and `amount` damage to any target of an opponent's choice"
    /// (Cuombajj Witches). The opponent to the controller's left picks;
    /// CR 801.5a keeps the pick inside both players' ranges of influence.
    OpponentChoosesTargetForDamage { amount: Value },

    /// CR 614.9 — "Prevent all combat damage that would be dealt to and dealt
    /// by `target` this turn." Adds the target creature to
    /// `GameState.combat_damage_prevented_creatures`; the combat resolver
    /// then skips that creature in both directions. Maze of Ith.
    PreventAllCombatDamageInvolving { target: Selector },

    /// CR 615 — "Prevent all combat damage that would be dealt to `target` this
    /// turn." Incoming-only (the creature still deals its own combat damage) —
    /// the turn-scoped sibling of the `PreventAllCombatDamageToThis` static.
    /// Adds the target to `GameState.combat_damage_prevented_to_this_turn`,
    /// which the resolver consults via `combat_damage_prevented_to_self`.
    /// Fleeting Flight.
    PreventCombatDamageToTargetThisTurn { target: Selector },

    /// "Prevent all combat damage that would be dealt by target creature this
    /// turn" — the deal-side mirror of `PreventCombatDamageToTargetThisTurn`.
    /// Adds the target to `GameState.combat_damage_prevented_by_this_turn`.
    /// Azorius Ploy.
    PreventCombatDamageByTargetThisTurn { target: Selector },
    /// CR 615 — "Prevent all damage `target` would deal this turn", combat and
    /// noncombat alike (Chain of Silence). The all-damage superset of
    /// `PreventCombatDamageByTargetThisTurn`.
    PreventAllDamageByTargetThisTurn { target: Selector },

    /// "You may have `dealer` deal damage equal to its power to `to`. If you
    /// do, `dealer` assigns no combat damage this turn." The Laccolith cycle's
    /// becomes-blocked trigger; declining leaves combat untouched.
    MayDealPowerThenNoCombatDamage { dealer: Selector, to: Selector },

    /// "Target creature can't block `source` this turn." Records a
    /// `(target, source)` pair in `GameState.cant_block_pairs`; the
    /// declare-blockers validator rejects that specific block. Kozilek's
    /// Pathfinder ({C}: target creature can't block this creature this turn).
    CantBlockSourceThisTurn { target: Selector },
    /// CR 509.1b — "creatures matching `filter` can't block this turn"
    /// (Concussive Bolt's metalcraft rider). Snapshots the matching creatures
    /// at resolution; cleared at cleanup.
    MatchingCantBlockThisTurn { filter: SelectionRequirement },

    /// CR 508.1a — "[creatures] can attack this turn as though they didn't
    /// have defender." Records the resolved permanents in
    /// `GameState.attack_despite_defender_this_turn`; cleared at cleanup
    /// (Krotiq Nestguard's activated ability).
    AttackDespiteDefenderThisTurn { what: Selector },

    /// CR 509.1c — "Target creature blocks `source` this turn if able."
    /// Sets the target's `must_block` to the ability source (like Provoke
    /// but without untapping the target). Matsu-Tribe Decoy.
    MustBlockSource { what: Selector },

    /// CR 509.1c — the two-slot sibling: "`blocker` blocks `attacker` this
    /// turn if able", where the attacker is itself chosen (Feral Contest).
    MustBlockTarget { blocker: Selector, attacker: Selector },

    /// "Destroy `what`. For each permanent put into a graveyard this way, its
    /// controller creates a token" (Terastodon). Victims that survive the
    /// destroy (indestructible, a replacement) pay nothing.
    DestroyThenVictimControllersMakeToken {
        what: Selector,
        definition: crate::card::TokenDefinition,
        /// "They can't be regenerated" (March of Souls). Defaults to false
        /// (Terastodon lets its victims regenerate).
        #[serde(default)]
        no_regen: bool,
    },

    /// "Prevent the next N damage that would be dealt to `target` this
    /// turn." (CR 615.7) Pushes a per-target prevention shield consumed
    /// by the non-combat damage path; the shield expires at cleanup.
    /// Samite Healer, Healing Salve, Awe Strike-style effects.
    PreventNextDamage { target: Selector, amount: Value },
    /// "Prevent the next N damage that would be dealt to `target` this turn.
    /// For each 1 damage prevented this way, put a +1/+1 counter on it."
    /// Test of Faith — the counter rider rides the shield
    /// (`counters_on_target`).
    PreventNextDamageWithCounters { target: Selector, amount: Value },
    /// CR 615.7 — "Prevent the next N damage that would be dealt *by* this
    /// permanent this turn" (Barbed Wire). The deal-side mirror of
    /// `PreventNextDamage`: a floating shield keyed to the source, soaking the
    /// next N damage it deals to anything.
    PreventNextDamageFromSourceThisTurn { amount: Value },
    /// CR 615 — "The next time a source of your choice would deal damage to
    /// you this turn, prevent half that damage, rounded down" (Dark Sphere).
    PreventNextHalfDamageToYouThisTurn,
    /// CR 615 — "The next time a source of your choice would deal damage to
    /// you this turn, that source deals it and this also deals that much to
    /// the source's controller" (Eye for an Eye). The damage still lands; the
    /// mirror rides alongside it.
    MirrorNextDamageToYouThisTurn,
    /// CR 614.9 — "Until end of turn, if damage would be dealt to any
    /// creature, you may have that damage dealt to you instead" (Blood of the
    /// Martyr). Set on the effect's controller; cleared at end of turn.
    RedirectCreatureDamageToYouThisTurn,

    /// "The next `amount` damage that would be dealt to `target` this turn is
    /// dealt to `to` instead." (CR 614.9 — Carom, Razia's redirect.) Pushes a
    /// per-target prevention shield flagged with `redirect_to`; when it soaks
    /// damage, that damage is re-dealt to the chosen permanent.
    RedirectNextDamage { target: Selector, to: Selector, amount: Value },

    /// CR 614.9 — "The next time `what` would deal combat damage to `to` this
    /// turn, `what` deals that damage to itself instead" (Shield Dancer). A
    /// one-event shield on `to`, scoped to `what` and redirecting back onto it.
    RedirectNextDamageBackAtSource { what: Selector, to: Selector },

    /// "Prevent all damage that would be dealt to `target` this turn."
    /// (CR 615) A fog scoped to one player/permanent — Pradesh Gypsies,
    /// "you don't lose / prevent all damage to you". Non-combat path.
    /// With `redirect_to` set the prevented damage is dealt to that
    /// entity instead (CR 614.9 — Sivvi's Valor).
    PreventAllDamageThisTurn {
        target: Selector,
        #[serde(default)]
        redirect_to: Option<Selector>,
    },
    /// "Prevent all damage that would be dealt to `target` this turn. For each
    /// 1 damage prevented this way, put a +1/+1 counter on it." Brace for
    /// Impact — the counter rider rides the shield (`counters_on_target`).
    PreventAllDamageThisTurnWithCounters { target: Selector },

    /// CR 615 — "Prevent all damage that `from` would deal to `to` this turn."
    /// A source-restricted fog (Stonewise Fortifier's "prevent all damage that
    /// would be dealt to this creature by target creature this turn").
    PreventAllDamageBetweenThisTurn { from: Selector, to: Selector },

    /// "You may tap or untap `what`" — the controller picks which per resolved
    /// permanent (Thassa's Ire, Puppeteer). Tapped permanents untap and
    /// untapped ones tap when the controller declines to choose.
    TapOrUntap { what: Selector },

    /// CR 615 — "Until your next turn, prevent all damage that would be dealt
    /// to and dealt by `target`" (Kiora, the Crashing Wave's +1). Registers the
    /// permanent in `damage_locked_until_turn_of`, checked on both ends of
    /// every damage event and dropped at the controller's next untap step.
    PreventDamageToAndByUntilYourNextTurn { target: Selector },

    /// CR 701.5 — counter target spell only if a card exiled with the source
    /// shares its name (Mindreaver). A no-op when nothing matches.
    CounterSpellIfNameExiledWithSource { what: Selector },

    /// "The next time damage would be dealt to `target` creature this turn,
    /// destroy that creature instead." (Kill-Suit Cultist.) A one-event
    /// prevent-all shield flagged `destroy`: it soaks the next damage event
    /// and destroys the protected permanent.
    ReplaceNextDamageWithDestroy { target: Selector },

    /// "The next time a source would deal damage to `target` this turn,
    /// prevent that damage; `target` gains life equal to the damage
    /// prevented this way." (CR 615.1 prevention + life gain.) Pushes a
    /// per-target prevention shield flagged `gain_life`; when it soaks
    /// damage the protected player gains that much life. Reverse Damage.
    PreventNextDamageAndGainLife { target: Selector, amount: Value },

    /// "Damage can't be prevented this turn." (CR 615.12) Sets a global
    /// flag that suppresses every prevention shield for the rest of the
    /// turn. Skullcrack, Heated Debate, Impractical Joke's rider.
    DamageCantBePreventedThisTurn,

    /// "Players can't search libraries this turn." (Shadow of Doubt.) Sets a
    /// global flag; library searches that turn find nothing.
    PreventSearchesThisTurn,

    /// "`who` gains protection from everything until their next turn"
    /// (The One Ring): can't be targeted, all damage prevented. Cleared
    /// when that player's turn begins.
    PlayerProtectionUntilNextTurn { who: PlayerRef },

    /// Register "when [the token just created in this resolution] leaves the
    /// battlefield, run `body`" as a delayed trigger. The current trigger
    /// source (e.g. a card this resolution exiled) is captured as the body's
    /// `Target(0)`. Hofri Ghostforge's "when that token leaves, return the
    /// exiled card to its owner's graveyard".
    WhenLastCreatedTokenLeaves { body: Box<Effect> },

    /// "Choose a creature type. Creatures other than creatures of the
    /// chosen type get -P/-T until end of turn." Crippling Fear-style
    /// choose-and-sweep primitive. Synchronously surfaces a
    /// `ChooseCreatureType` decision (caster's seat) and then applies
    /// `PumpPT(power, toughness, EOT)` to every battlefield creature
    /// whose `definition.subtypes.creature_types` does NOT contain the
    /// answered type. The decision is resolved synchronously off
    /// `self.decider`, so AutoDecider (which picks `Demon`) and
    /// ScriptedDecider both work; UI players don't get a separate
    /// prompt today (degraded to the auto-decider choice — same as
    /// other implicit-choice cards).
    DiminishCreaturesExceptChosenType { power: Value, toughness: Value },

    /// CR 615 — "Prevent all damage that would be dealt to `target` this turn
    /// by sources of the color of your choice." Avacyn, Guardian Angel. The
    /// color is picked as the effect resolves (`Decision::ChooseColor`).
    PreventAllDamageFromChosenColorThisTurn { target: Selector },

    /// "Search your graveyard, hand, and/or library for an Aura card and put
    /// it onto the battlefield attached to [the source]. If you search your
    /// library this way, shuffle." Boonweaver Giant. Candidates pool all three
    /// zones; the Aura's own enchant filter is not re-checked.
    SearchAuraAttachToSource,

    /// "For each planeswalker you control, you may activate one of its loyalty
    /// abilities this turn as though none of its loyalty abilities have been
    /// activated this turn." The Chain Veil. Bumps the controller's
    /// `extra_loyalty_activations` budget.
    GrantExtraLoyaltyActivations,

    /// "Choose a card in your hand. `who` guesses whether its mana value is
    /// greater than `threshold`. If they guessed wrong, you may cast that card
    /// without paying its mana cost." Master of Predicaments. The guess is a
    /// `Decision::OptionalTrigger` on the guesser's seat (true = "greater").
    GuessManaValueAboveElseCastFree { who: PlayerRef, threshold: u32 },

    /// "`who` may pay `life`. If they don't, they return a permanent they
    /// control to its owner's hand." Umbilicus — one pay-or-bounce decision per
    /// affected player, asked of that player.
    PlayerReturnsPermanentUnlessPaysLife { who: PlayerRef, life: u32 },

    /// "Return to its owner's hand each creature `who` controls with power
    /// greater than the number of cards in their hand" (Noetic Scales). The
    /// comparison is re-read per player, which a single `Value` filter can't
    /// express.
    ReturnCreaturesWithPowerGreaterThanHand { who: PlayerRef },

    /// "Search `who`'s library for a card with the same name as `subject`,
    /// reveal it, put it into `to`, then shuffle" (Remembrance). The name is
    /// read at resolution from the (possibly already-dead) subject's LKI.
    SearchSameNameAs {
        who: PlayerRef,
        subject: Selector,
        to: ZoneDest,
        /// "Search for **any number** of cards with that name" (Secret
        /// Summoning). `None` searches out a single copy.
        #[serde(default)]
        count: Option<Value>,
    },

    /// CR 615 — "Prevent the next `total` damage that would be dealt this turn
    /// to any number of targets, divided as you choose." The prevention-side
    /// sibling of `DealDamageDivided`; shares `Decision::DivideDamage`.
    /// Serra's Hymn.
    PreventNextDamageDivided { total: Value, filter: SelectionRequirement, max_targets: u8 },

    /// Stamp `what`'s first resolved permanent on the source's
    /// `chosen_permanent` slot, so a later `Selector::ChosenPermanentOfSource`
    /// can name it. The resolution-time sibling of
    /// `Effect::ChoosePermanentForSource` (which picks by filter at ETB) —
    /// Diabolic Servitude remembers the creature it reanimated.
    RememberPermanentOnSource { what: Selector },

    /// "You may put an Aura card from your hand onto the battlefield attached
    /// to `host`" (Academy Researchers). The Aura's own enchant filter is
    /// re-checked against the host; nothing happens when no Aura fits.
    PutAuraFromHandAttachedTo { host: Selector },

    /// "Choose a color. `who` reveals their hand and discards all cards of
    /// that color" (Persecute). The color is chosen by the resolving
    /// controller; the reveal publishes the hand to them.
    ChooseColorThenDiscardMatching { who: PlayerRef },

    /// "Choose a card type. `who` reveals their hand. Deal `per` damage to
    /// that player for each card of the chosen type revealed this way"
    /// (Blood Oath). The type is chosen by the resolving controller.
    ChooseCardTypeRevealHandDamage { who: PlayerRef, per: Value },

    /// Crooked Scales — "Flip a coin. If you win, destroy `win`. If you lose,
    /// destroy `lose` unless you pay `repeat_cost` and repeat this process."
    /// Loops until a win, a decline, or the loser is destroyed.
    CoinFlipDestroyLoop { win: Selector, lose: Selector, repeat_cost: crate::mana::ManaCost },

    /// Thieves' Auction — exile all nontoken permanents, then, starting with
    /// the controller, each player in turn order picks one of the exiled
    /// cards and puts it onto the battlefield tapped under their control,
    /// repeating until every exiled card is claimed.
    ThievesAuction,

    /// Stamp the resolved player on the source's `chosen_player` slot so a
    /// later `PlayerRef::ChosenPlayerOfSource` can name them — the player
    /// twin of [`Effect::RememberPermanentOnSource`]. Backs the Torment
    /// Nightmare Horrors' "that player gains N life" leave trigger.
    RememberPlayerOnSource { who: PlayerRef },

    /// CR 614 — "As this enters, sacrifice any number of creatures. This
    /// creature's power becomes their total power and its toughness their
    /// total toughness" (Dracoplasm). The totals are stamped on the source's
    /// `chosen_number` / `remembered_amount`, read back by
    /// `DynamicPt::EnteredTotals`.
    AsEntersSacrificeForTotalPt,

    /// "Any player may exile `count` cards from their graveyard. If a player
    /// does, `then`." Walks the seats in turn order starting with the
    /// source's controller and takes the first willing payer; `then` runs
    /// once. Carrion Rats, Carrion Wurm.
    AnyPlayerMayExileFromGraveyard { count: Value, then: Box<Effect> },

    /// CR 614 — "Until end of turn, if `from` would draw a card, instead that
    /// player skips that draw and you draw a card" (Plagiarize). Registered on
    /// `GameState.draws_redirected_this_turn` and consumed in `draw_one`.
    RedirectDrawsThisTurn { from: PlayerRef },

    /// CR 615 — "If any source would deal `at_least` or more damage to a
    /// permanent or player this turn, it deals `becomes` damage instead"
    /// (Equal Treatment). A turn-scoped global damage rewrite applied in
    /// `scale_damage_to`.
    DamageBecomesThisTurn { at_least: u32, becomes: u32 },

    /// "This deals `amount` damage to target player or planeswalker. That
    /// player (or that planeswalker's controller) may instead have the damage
    /// dealt to a creature they control" (Flaming Gambit). The redirect
    /// choice belongs to the victim.
    DamageTargetPlayerMayRedirect { amount: Value },

    /// CR 707 — "Copy the target spell for each other permanent or player it
    /// could target; each copy targets a different one of those" (Radiate).
    /// The target spell must have exactly one target.
    CopySpellForEachOtherTarget { what: Selector },

    /// "Reveal a card in your hand, then put that card onto the battlefield if
    /// it has the same name as a permanent" (Retraced Image). The reveal is
    /// the controller's pick; nothing happens without a name match.
    RevealAndReplayNamedPermanent,

    /// "For each creature token on the battlefield, its controller creates a
    /// token that's a copy of that creature" (Parallel Evolution).
    CopyEachCreatureToken,

    /// The punisher / "any opponent may" shape — each seat `who` resolves to is
    /// asked `prompt` in turn order from the controller. The first yes runs
    /// `accepted` (with that seat readable as [`PlayerRef::AcceptingPlayer`])
    /// and stops; `otherwise` runs only when every seat declines. Book Burning,
    /// Breaking Point, Dwarven Driller, Dwarven Scorcher, Distant Memories.
    AnyPlayerMayAccept {
        who: PlayerRef,
        prompt: String,
        accepted: Box<Effect>,
        otherwise: Box<Effect>,
    },

    /// "Exchange your graveyard and your library, then shuffle" (Morality
    /// Shift). A whole-zone swap; per-card leaves-graveyard triggers don't
    /// fire, matching `ExchangeHandAndGraveyard`.
    ExchangeGraveyardAndLibrary { who: PlayerRef },

    /// CR 407.4 — "[who] antes the top card of their library." With `optional`
    /// each resolved player is asked first; `then` runs once per player who
    /// actually anted (bound as that effect's controller — Rebirth's "that
    /// player's life total becomes 20") and `else_` once per player who
    /// declined (Amulet of Quoz's coin flip).
    AnteTopOfLibrary {
        who: PlayerRef,
        #[serde(default)]
        optional: bool,
        #[serde(default)]
        then: Option<Box<Effect>>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },
    /// CR 407.4 — put `what` into its owner's ante zone from wherever it is.
    Ante { what: Selector },
    /// "Put all other cards you own from the ante into your graveyard"
    /// (Jeweled Bird). Skips the effect's own source.
    AnteToGraveyard { who: PlayerRef },
    /// CR 407.3 — "You own target card in the ante. Exchange that card with
    /// the top card of your library" (Darkpact). Ownership of the picked ante
    /// card moves to `who` before the swap.
    TakeAnteCardForLibraryTop { who: PlayerRef },
    /// CR 407.3 — permanently exchange ownership of `a` and `b`, then send
    /// each to `a_to` / `b_to` under its new owner (Tempest Efreet,
    /// Timmerian Fiends). The only rules-legal ownership change.
    ExchangeOwnership { a: Selector, b: Selector, a_to: ZoneDest, b_to: ZoneDest },
    /// CR 119.4 — "[who] may pay N life. If they don't, `else_`." The asked
    /// player isn't the effect's controller (Tempest Efreet, Bronze Tablet);
    /// a seat that can't afford the life is never asked.
    PlayerMayPayLifeElse { who: PlayerRef, life: Value, else_: Box<Effect> },

    /// "Each player may exile any number of cards from their graveyard,"
    /// then `then` runs once (Grave Consequences). Each seat picks its own
    /// batch via `Decision::ChooseCards`; unlike
    /// [`Effect::AnyPlayerMayExileFromGraveyard`] every seat is asked and
    /// `then` runs regardless.
    EachPlayerMayExileAnyNumberFromGraveyard { then: Box<Effect> },

    /// "Each player loses `per` life for each `filter` card in their
    /// graveyard" — the graveyard twin of [`Effect::LoseLifePerControlled`],
    /// so each seat's own graveyard sets its own loss (Grave Consequences).
    LoseLifePerCardInGraveyard { who: Selector, filter: SelectionRequirement, per: Value },

    /// "That player exiles the top `count` cards of their library. If two or
    /// more of those cards have the same name, repeat this process"
    /// (Scalpelexis). Capped at 20 iterations against a stacked library.
    ExileTopRepeatOnDuplicateNames { who: PlayerRef, count: Value },

    /// "You lose all but `keep` life," remembering how much was lost on the
    /// source (`CardInstance.remembered_amount`) so a later trigger can pay it
    /// back via [`Value::RememberedAmountOfSource`]. Soulgorger Orgg.
    LoseAllButLifeRemembered { who: PlayerRef, keep: Value },

    /// "Counter target spell. If that spell is countered this way, exile it
    /// instead… You may play it without paying its mana cost for as long as it
    /// remains exiled" (Spelljack). The exiled card gets a may-play grant for
    /// the countering player with no duration.
    CounterSpellExileMayPlayFree { what: Selector },

    /// "Exile any number of `filter` cards from your graveyard," recording them
    /// on the source's exile link so a CDA can read the pile (Sutured Ghoul's
    /// as-enters clause). The controller picks; the headless default takes all.
    ExileAnyNumberFromGraveyardOnSource { filter: SelectionRequirement },

    /// "That player chooses a permanent for each card in their graveyard, then
    /// untaps those permanents" (Mist of Stagnation). Auto-picks the player's
    /// own tapped permanents when they aren't a UI seat.
    UntapChosenPerCardInGraveyard { who: PlayerRef },

    /// "`who` may exile a card from their graveyard. If they don't, `otherwise`"
    /// (Web of Inertia). One seat, one card, one yes/no.
    MayExileFromGraveyardElse { who: PlayerRef, otherwise: Box<Effect> },

    /// "Creatures `who` controls can't attack `defender` this turn"
    /// (Web of Inertia's punishment half). Cleared at cleanup.
    CantAttackPlayerThisTurn { who: PlayerRef, defender: PlayerRef },

    /// "Prevent all damage that sources of the color of your choice would deal
    /// this turn" (Prismatic Strands) — the recipient-less sibling of
    /// [`Effect::PreventAllDamageFromChosenColorThisTurn`].
    PreventAllDamageFromChosenColorGlobally,

    /// "Other players can't play lands or cast spells from their graveyards
    /// this turn. You may play lands and cast spells from other players'
    /// graveyards this turn as though those cards were in your graveyard"
    /// (Shaman's Trance).
    ShamansTrance,
}

/// CR 702.172 — one Spree mode: an additional mana cost paired with the
/// effect it buys. Chosen at cast time; run at resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreeMode {
    pub cost: crate::mana::ManaCost,
    pub effect: Effect,
}

/// Lightweight mirror of `DelayedKind` for use inside
/// `Effect`. Kept separate so `effect.rs` doesn't need to import from
/// `game::`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelayedTriggerKind {
    /// "At the beginning of the next combat" — the next Begin Combat step on
    /// any player's turn (Legion's Initiative).
    NextCombat,
    /// "Until your next turn, whenever a creature attacks you or a
    /// planeswalker you control, …" (Jace, Architect of Thought's +1). The
    /// attacker is the trigger source.
    CreatureAttacksYouUntilYourNextTurn,
    YourNextUpkeep,
    NextEndStep,
    /// "At the beginning of the end step of target player's next turn"
    /// (Suppress). `Effect::DelayUntil` reads the player from target slot 0.
    TargetsNextEndStep,
    /// "At the beginning of [the damaged player]'s next draw step" (Nafs Asp).
    /// Reads the player from the trigger's event subject, falling back to
    /// target slot 0.
    TargetsNextDrawStep,
    /// "At end of combat, …" — fires once at the current turn's end-of-combat
    /// step (Fortune, Loyal Steed's saddle blink).
    EndOfCombat,
    /// "At the beginning of your next pre-combat main phase, …" Used by
    /// Chancellor of the Tangle's opening-hand reveal — the mana ritual
    /// fires on main rather than upkeep so the {G} doesn't empty out of
    /// the pool before the player can spend it (mana pools clear on
    /// step transition, MTG rule 500.4).
    YourNextMainPhase,
    /// "At the beginning of the next cleanup step" (Waylay). Fires in the
    /// cleanup step of the turn it was registered in, so the objects it
    /// touches survive the end step.
    NextCleanupStep,
}

/// Opening-hand ("if this is in your opening hand, you may ...") effect.
/// Resolved by `GameState::apply_opening_hand_effects` after all players
/// finish mulligans and before the first turn begins.
///
/// Each variant covers one of the canonical Magic shapes:
/// * **Leyline / Gemstone Caverns** — the card begins the game on the
///   battlefield instead of in hand.
/// * **Chancellor of the Tangle / of the Annex** — the card stays in hand,
///   but reveals at game start to register a one-shot trigger that fires
///   later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpeningHandEffect {
    /// "If [card] is in your opening hand, you may begin the game with it
    /// on the battlefield." After moving to play, run `extra` so cards like
    /// Gemstone Caverns can stamp themselves with a luck counter (or any
    /// other one-shot ETB-style follow-up).
    StartInPlay {
        tapped: bool,
        extra: Effect,
    },
    /// "You may reveal [card] from your opening hand. If you do, [body]."
    /// The card stays in hand; we register a `DelayedTrigger` of `kind`
    /// whose effect is `body`. Used by the Chancellors.
    RevealForDelayedTrigger {
        kind: DelayedTriggerKind,
        body: Effect,
    },
    /// "Any time you could mulligan and [card] is in your hand, you may
    /// exile all the cards from your hand, then draw that many cards."
    /// Surfaces as an additional answer in the mulligan decision; not run
    /// post-mulligan. The variant exists so the catalog can declaratively
    /// flag the card and `apply_opening_hand_effects` skips it.
    MulliganHelper,
}

/// One choice on a CR 701.38 ballot: the word players vote for, and what that
/// word does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VoteOption {
    pub label: String,
    pub effect: Effect,
}

impl VoteOption {
    pub fn new(label: &str, effect: Effect) -> Self {
        Self { label: label.to_string(), effect }
    }
}

/// How a CR 701.38 vote is spent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteTally {
    /// Will of the council — only the option with the most votes happens. The
    /// printed tie-breaker is always the later option ("…or the vote is tied"),
    /// so a tie resolves to the highest-indexed tied option.
    Majority,
    /// Council's dilemma — every option's effect runs once per vote it drew.
    PerVote,
    /// "…each choice with the most votes or tied for most votes" (Council
    /// Guardian): every winning option's effect runs once.
    AllTied,
}

impl Default for Effect {
    /// Default `Effect` is `Noop` — a permanent with no spell effect
    /// (creature, enchantment, etc.) leaves this slot at its default.
    /// Lets `CardDefinition` derive `Default`, so card constructors can
    /// use `..Default::default()` and skip boilerplate.
    fn default() -> Self { Effect::Noop }
}

mod query;

/// Serde default for `Effect::RollDie.modifier` — a zero (no-op) die-roll
/// modifier, so snapshots written before CR 706.2 modifiers existed still
/// deserialize cleanly.
pub fn zero_value() -> Value {
    Value::Const(0)
}

/// Serde default for `LookTopExileOneMayPlay.who` (Gonti's target opponent).
pub fn player_ref_target_zero() -> PlayerRef {
    PlayerRef::Target(0)
}

/// Serde default for `ResetCreature` P/T (vanilla 1/1).
pub fn value_one() -> Value {
    Value::Const(1)
}

fn zonedest_has_target(z: &ZoneDest) -> bool {
    match z {
        ZoneDest::Hand(p) | ZoneDest::Library { who: p, .. } => matches!(p, PlayerRef::Target(_)),
        ZoneDest::Battlefield { controller, .. } => matches!(controller, PlayerRef::Target(_)),
        ZoneDest::Graveyard
        | ZoneDest::Exile
        | ZoneDest::ExilePlotted
        | ZoneDest::ExileWithSourceStamp
        | ZoneDest::Ante => false,
    }
}

// ── Static abilities / ability shells (see effect/abilities.rs) ─────────────
mod abilities;
pub use abilities::*;

// ── Helpers / shortcut constructors ──────────────────────────────────────────

pub mod shortcut;

#[cfg(test)]
#[path = "tests/effect_query.rs"]
mod tests;

/// Payload for [`Effect::LookPickToHand`] — boxed so the ~160 card call sites
/// set only the fields that differ from [`LookPick::default`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct LookPick {
    pub who: PlayerRef,
    pub count: Value,
    #[serde(default)]
    pub rest_to_graveyard: bool,
    #[serde(default)]
    pub pick_filter: Option<SelectionRequirement>,
    /// How many cards to put into hand (default 1). When >1 the controller
    /// picks the first via the decision and the rest auto-fill from the
    /// remaining eligible revealed cards. Consult the Star Charts kicked.
    #[serde(default)]
    pub take: Option<Value>,
    /// Picks go onto the battlefield instead of to hand (Collected
    /// Company's "put them onto the battlefield").
    #[serde(default)]
    pub to_battlefield: bool,
    /// "If you put a [filter] card into your hand this way, you gain N
    /// life" (Chrome Courier's artifact rider).
    #[serde(default)]
    pub gain_life_if_pick: Option<(SelectionRequirement, u32)>,
    /// "You gain life equal to the greatest power among creature cards
    /// put into your graveyard this way" (Discerning Taste). Only
    /// meaningful with `rest_to_graveyard`.
    #[serde(default)]
    pub gain_life_greatest_power_rest: bool,
    /// Printed "you MAY put ... into your hand": an explicit empty pick
    /// from a UI player is honored as a decline (the whole revealed set
    /// follows the rest-routing), and a partial pick (fewer than `take`)
    /// is respected rather than topped up. Mandatory picks (`false`)
    /// auto-fill top-down as before, and the AutoDecider harness keeps
    /// the fill either way so bot play is unchanged.
    #[serde(default)]
    pub optional: bool,
    /// "If you put a card into your hand this way, [effect]" — runs once
    /// after the picks land (Sidequest: Catch a Fish's Food + transform).
    #[serde(default)]
    pub then_if_picked: Option<Box<Effect>>,
    /// Typed routing (Zimone's Experiment): picked LAND cards go onto
    /// the battlefield tapped while other picks go to hand.
    #[serde(default)]
    pub picked_lands_to_battlefield: bool,
    /// The non-picked rest is bottomed in a genuinely RANDOM order
    /// (printed "in a random order") instead of revealed order.
    #[serde(default)]
    pub rest_bottom_random: bool,
    /// "Put one into your hand and exile the rest" (Eye of Yawgmoth).
    /// Takes precedence over `rest_to_graveyard`.
    #[serde(default)]
    pub rest_to_exile: bool,

    /// Picks matching this filter go onto the battlefield instead of to hand
    /// (Break Out's mana-value-2-or-less creature). Applied per pick, after
    /// `to_battlefield` and `picked_lands_to_battlefield`.
    #[serde(default)]
    pub picked_matching_to_battlefield: Option<SelectionRequirement>,
    /// Picks routed to the battlefield gain haste until end of turn
    /// (Break Out).
    #[serde(default)]
    pub battlefield_haste: bool,
}

impl Default for LookPick {
    fn default() -> Self {
        Self {
            who: PlayerRef::You,
            count: Value::ONE,
            rest_to_graveyard: false,
            pick_filter: None,
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
            then_if_picked: None,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
            rest_to_exile: false,
            picked_matching_to_battlefield: None,
            battlefield_haste: false,
        }
    }
}
